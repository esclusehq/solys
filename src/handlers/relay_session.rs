//! Per-stream yamux ↔ local TCP forwarder.
//!
//! Phase 68 (Plan 02) / Phase 69 (Plan 02). The gateway opens one yamux
//! stream per inbound Minecraft Java client connection. For each accepted
//! stream, the agent spawns a [`run_relay_session`] task that:
//!
//! 1. Opens a `TcpStream` to the configured local MC address (typically
//!    `127.0.0.1:25565`).
//! 2. Copies bytes bidirectionally between the yamux `StreamHandle` and
//!    the local TCP stream.
//! 3. On either half closing, drops both — the yamux stream's `Drop` impl
//!    signals the gateway (T-68-08: only local 127.0.0.1 is ever dialed;
//!    user-supplied WS payloads can never redirect this target).
//!
//! Byte accounting is shared with the per-server heartbeat task via
//! [`Arc<AtomicU64>`] (D-18). Each inbound stream in
//! [`relay_client::drive_inbound_streams`] looks up the per-server
//! `bytes_transferred` counter from the
//! `RwLock<HashMap<ServerId, PerServerRuntime>>` so the 100 GB rekey
//! threshold (D-25 / RESOLVED Q4) is correctly summed per server.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

/// Run one inbound yamux stream's lifecycle.
///
/// `yamux_stream` is the gateway-opened stream; `local_addr` is the
/// `host:port` to forward bytes to (defaults to `127.0.0.1:25565` from
/// [`crate::state::RelayConfig::local_mc_addr`]).
///
/// `bytes_counter` is incremented by the byte counts returned from
/// `copy_bidirectional` so the heartbeat task can sum toward the 100 GB
/// rekey threshold (D-25).
pub async fn run_relay_session<S>(
    yamux_stream: S,
    local_addr: String,
    bytes_counter: Arc<AtomicU64>,
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    debug!(local = %local_addr, "Relay session: dialing local MC address");
    let tcp = TcpStream::connect(&local_addr)
        .await
        .with_context(|| format!("Relay session: failed to connect to local MC at {}", local_addr))?;

    // We split the TCP stream into owned halves so the bidirectional copy
    // can drive both directions concurrently. This matches the pattern
    // used elsewhere in the agent's networking code.
    let (tcp_read, tcp_write) = tcp.into_split();
    let (yamux_read, yamux_write) = tokio::io::split(yamux_stream);

    // copy_bidirectional returns (bytes_from_yamux_to_tcp, bytes_from_tcp_to_yamux).
    let copy_result = copy_bidirectional_with_count(yamux_read, tcp_write, tcp_read, yamux_write).await;

    match copy_result {
        Ok((down_bytes, up_bytes)) => {
            let total = down_bytes.saturating_add(up_bytes);
            bytes_counter.fetch_add(total, Ordering::Relaxed);
            debug!(
                down = down_bytes,
                up = up_bytes,
                total = total,
                "Relay session closed cleanly"
            );
        }
        Err(e) => {
            // We still attempt to account whatever partial bytes may have
            // flowed before the error. We don't have a reliable count
            // after a mid-stream error, so we just log and move on.
            warn!(error = %e, "Relay session closed with error");
        }
    }

    info!(local = %local_addr, "Relay session ended");
    Ok(())
}

/// Manual implementation of bidirectional copy that returns per-direction
/// byte counts. We can't use `tokio::io::copy_bidirectional` directly
/// because it doesn't return the counts we need for the rekey threshold.
/// We drive the two halves in parallel and return when either side EOFs.
async fn copy_bidirectional_with_count<R1, W1, R2, W2>(
    r1: R1,
    w1: W1,
    r2: R2,
    w2: W2,
) -> Result<(u64, u64)>
where
    R1: AsyncRead + Unpin,
    W1: AsyncWrite + Unpin,
    R2: AsyncRead + Unpin,
    W2: AsyncWrite + Unpin,
{
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut r1 = r1;
    let mut w1 = w1;
    let mut r2 = r2;
    let mut w2 = w2;

    let mut buf1 = vec![0u8; 16 * 1024];
    let mut buf2 = vec![0u8; 16 * 1024];
    let mut count1: u64 = 0;
    let mut count2: u64 = 0;

    // Phase 1: read from r1, write to w1 (yamux → local MC)
    // Phase 2: read from r2, write to w2 (local MC → yamux)
    // We do one phase at a time per loop iteration and check for EOF on
    // either side; on EOF we shut down the other side's write half.
    loop {
        tokio::select! {
            biased;
            res = r1.read(&mut buf1) => {
                match res? {
                    0 => {
                        let _ = w1.shutdown().await;
                        // Drain r2 → w2 until EOF.
                        loop {
                            match r2.read(&mut buf2).await? {
                                0 => break,
                                n => {
                                    count2 += n as u64;
                                    w2.write_all(&buf2[..n]).await?;
                                }
                            }
                        }
                        return Ok((count1, count2));
                    }
                    n => {
                        count1 += n as u64;
                        w1.write_all(&buf1[..n]).await?;
                    }
                }
            }
            res = r2.read(&mut buf2) => {
                match res? {
                    0 => {
                        let _ = w2.shutdown().await;
                        // Drain r1 → w1 until EOF.
                        loop {
                            match r1.read(&mut buf1).await? {
                                0 => break,
                                n => {
                                    count1 += n as u64;
                                    w1.write_all(&buf1[..n]).await?;
                                }
                            }
                        }
                        return Ok((count1, count2));
                    }
                    n => {
                        count2 += n as u64;
                        w2.write_all(&buf2[..n]).await?;
                    }
                }
            }
        }
    }
}
