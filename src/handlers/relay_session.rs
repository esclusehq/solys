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
//! Phase 69: Each [`run_relay_session`] call now receives the per-server
//! `bytes_counter` from the [`PerServerRuntime`](crate::handlers::relay_client::PerServerRuntime)
//! that spawned it. The yamux stream comes from the per-server session
//! (one Session per server, not one global session). The streaming logic
//! itself is unchanged — the function is already fully generic over
//! `S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static`.
//!
//! Byte accounting accumulates toward that server's 100 GB rekey threshold.
//! The counter is fetched from the per-server `PerServerRuntime` by the
//! caller in `relay_client::drive_inbound_streams`.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tracing::{debug, info, warn};

/// Run one inbound yamux stream's lifecycle.
///
/// `yamux_stream` is the gateway-opened stream; `local_addr` is the
/// `host:port` to forward bytes to (defaults to `127.0.0.1:25565` from
/// [`crate::state::RelayConfig::local_mc_addr`]).
///
/// `bytes_counter` is the per-server [`Arc<AtomicU64>`] from the
/// [`PerServerRuntime`](crate::handlers::relay_client::PerServerRuntime)
/// that owns this session. It accumulates bytes toward that server's
/// rekey threshold.
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

/// TLV type byte for datagram frames (D-08). Type byte `0x01` means
/// the payload is a raw datagram. Reserve `0xFF` for future control.
const TLV_TYPE_DATAGRAM: u8 = 0x01;

/// Run one UDP relay session's lifecycle for a single long-lived
/// yamux stream (D-07).
///
/// The yamux stream carries TLV-framed datagrams: [1-byte type]
/// [4-byte big-endian length][payload]. Both directions use the
/// same framing.
///
/// `yamux_stream` is the gateway-opened yamux stream. `local_addr`
/// is the `host:port` of the local Bedrock container (e.g.
/// "127.0.0.1:19132") from `RelayServerConfig.local_mc_addr` (D-11).
///
/// `bytes_counter` accumulates total bytes toward the rekey threshold.
pub async fn run_udp_relay_session<S>(
    yamux_stream: S,
    local_addr: String,
    bytes_counter: Arc<AtomicU64>,
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    // Bind to ephemeral port for outgoing UDP (D-10)
    let udp = UdpSocket::bind("0.0.0.0:0")
        .await
        .context("Failed to bind local UDP socket")?;

    // Parse local container address (D-11 — use local_mc_addr directly)
    let remote_addr: std::net::SocketAddr = local_addr
        .parse()
        .context(format!("Invalid local_mc_addr for UDP: {}", local_addr))?;

    // Split yamux stream for concurrent read/write
    let (mut yamux_read, mut yamux_write) = tokio::io::split(yamux_stream);
    let udp = Arc::new(udp);
    let total_bytes = Arc::new(AtomicU64::new(0));

    // Task: yamux → UdpSocket (reads TLV frames from gateway, sends UDP to container)
    let yamux_to_udp = {
        let udp = udp.clone();
        let remote = remote_addr;
        let total = total_bytes.clone();
        tokio::spawn(async move {
            let mut len_buf = [0u8; 4];
            let mut payload = vec![0u8; 65535];
            loop {
                // Read TLV type byte
                let mut type_byte = [0u8; 1];
                if yamux_read.read_exact(&mut type_byte).await.is_err() { break; }
                if type_byte[0] != TLV_TYPE_DATAGRAM { continue; } // skip control frames

                // Read 4-byte big-endian length
                if yamux_read.read_exact(&mut len_buf).await.is_err() { break; }
                let len = u32::from_be_bytes(len_buf) as usize;
                if len > payload.len() { break; }

                // Read payload
                if yamux_read.read_exact(&mut payload[..len]).await.is_err() { break; }

                // Send to container via UDP
                if let Err(e) = udp.send_to(&payload[..len], remote).await {
                    warn!("UDP relay: failed to send to container {}: {}", remote, e);
                    break;
                }
                total.fetch_add(1 + 4 + len as u64, Ordering::Relaxed);
            }
            Ok::<_, anyhow::Error>(())
        })
    };

    // Task: UdpSocket → yamux (reads UDP from container, writes TLV frames to gateway)
    let udp_to_yamux = {
        let total = total_bytes.clone();
        tokio::spawn(async move {
            let mut buf = vec![0u8; 65535];
            loop {
                let (n, src) = match udp.recv_from(&mut buf).await {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("UDP relay: recv_from failed: {}", e);
                        break;
                    }
                };
                // Write TLV: [0x01 type][4-byte big-endian n][datagram bytes]
                if let Err(e) = yamux_write.write_all(&[TLV_TYPE_DATAGRAM]).await {
                    warn!("UDP relay: failed to write TLV type: {}", e);
                    break;
                }
                if let Err(e) = yamux_write.write_all(&(n as u32).to_be_bytes()).await {
                    warn!("UDP relay: failed to write TLV length: {}", e);
                    break;
                }
                if let Err(e) = yamux_write.write_all(&buf[..n]).await {
                    warn!("UDP relay: failed to write TLV payload: {}", e);
                    break;
                }
                if let Err(e) = yamux_write.flush().await {
                    warn!("UDP relay: flush failed: {}", e);
                    break;
                }
                total.fetch_add(1 + 4 + n as u64, Ordering::Relaxed);
                let _ = src; // source address available if needed
            }
            Ok::<_, anyhow::Error>(())
        })
    };

    // Wait for either task to finish
    let result = tokio::select! {
        r = yamux_to_udp => { r }
        r = udp_to_yamux => { r }
    };

    match result {
        Ok(inner) => {
            if let Err(e) = inner {
                warn!("UDP relay session task error: {}", e);
            }
        }
        Err(join) => {
            warn!("UDP relay session task join error: {}", join);
        }
    }

    let total = total_bytes.load(Ordering::Relaxed);
    bytes_counter.fetch_add(total, Ordering::Relaxed);
    info!("UDP relay session ended, forwarded {} bytes", total);
    Ok(())
}
