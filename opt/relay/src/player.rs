use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{copy_bidirectional, AsyncReadExt};
use tokio::net::TcpStream;
use tracing::{debug, error, info, warn};
use crate::state::AppState;

/// The suffix every relay subdomain has on `*.play.esluce.com`.
const RELAY_SUFFIX: &str = ".play.esluce.com";
/// Max length of a Minecraft Java String per protocol spec (VarInt-prefixed, 32767 chars max, but DNS labels cap at 253).
const MAX_MC_STRING_BYTES: usize = 255;
/// How long to wait for the Handshake packet to arrive before giving up.
const HANDSHAKE_READ_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn run_player_listener(state: Arc<AppState>) -> anyhow::Result<()> {
    let player_bind = state.config.server.player_bind.clone();
    let listener = tokio::net::TcpListener::bind(player_bind.clone()).await?;
    info!("[PLAYER] Listening on {}", player_bind);
    loop {
        let (stream, peer) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_player_connection(state, stream, peer).await {
                debug!("[PLAYER] Connection ended: {}", e);
            }
        });
    }
}

async fn handle_player_connection(
    state: Arc<AppState>,
    mut tcp: TcpStream,
    peer: SocketAddr,
) -> anyhow::Result<()> {
    // Rate-limit per source IP (D-20)
    if !state.rate_limiter.check(peer.ip()) {
        warn!("[PLAYER] Rate-limited: peer={}", peer);
        // Clean close (D-18)
        drop(tcp);
        return Ok(());
    }

    // Read the MC Java Handshake packet (BLOCKER 1 fix).
    // The Handshake is the FIRST packet the client sends on a new TCP connection.
    // Format: [VarInt packet length][VarInt packet ID = 0x00][VarInt protocol version][String server address][ushort server port][VarInt next state]
    //
    // We only need the String server address (the rest is forwarded to the agent verbatim).
    let (subdomain, prefix_bytes) = match read_mc_handshake_subdomain(&mut tcp, peer).await {
        Ok(s) => s,
        Err(e) => {
            warn!("[PLAYER] Failed to parse Handshake from {}: {}; closing", peer, e);
            drop(tcp);  // D-18: clean close
            return Ok(());
        }
    };
    debug!("[PLAYER] Handshake from {}: subdomain={}", peer, subdomain);

    // Look up the server by subdomain (NOT by source IP — see BLOCKER 1).
    let server_id = match state.registry.lookup_by_subdomain(&subdomain) {
        Some(id) => id,
        None => {
            warn!("[PLAYER] No active tunnel for subdomain={}; closing", subdomain);
            drop(tcp);  // D-18: clean close
            return Ok(());
        }
    };

    let handle = match state.registry.get(&server_id) {
        Some(h) => h,
        None => {
            warn!("[PLAYER] Registry race: subdomain={} → server_id={} but no handle; closing", subdomain, server_id);
            drop(tcp);
            return Ok(());
        }
    };

    // Open a new yamux stream on the agent's tunnel
    let control_lock = handle.yamux_control.lock().await;
    let mut control = match control_lock.as_ref() {
        Some(c) => c.clone(),
        None => {
            warn!("[PLAYER] Tunnel handle has no yamux control (stale); closing");
            drop(tcp);
            return Ok(());
        }
    };
    let mut yamux_stream = match control.open_stream().await {
        Ok(s) => s,
        Err(e) => {
            error!("[PLAYER] Failed to open yamux stream for server={}: {}", server_id, e);
            drop(tcp);
            return Ok(());
        }
    };

    // Forward the Handshake prefix bytes we already buffered, then bidirectionally copy.
    tokio::io::AsyncWriteExt::write_all(&mut yamux_stream, &prefix_bytes).await?;

    // Bidi copy with 5-min idle timeout (D-19)
    let copy_result = tokio::time::timeout(
        Duration::from_secs(300),
        copy_bidirectional(&mut tcp, &mut yamux_stream),
    ).await;

    match copy_result {
        Ok(Ok((in_bytes, out_bytes))) => {
            handle.bytes_in.fetch_add(in_bytes as u64, std::sync::atomic::Ordering::Relaxed);
            handle.bytes_out.fetch_add(out_bytes as u64, std::sync::atomic::Ordering::Relaxed);
            crate::metrics::PLAYER_BYTES_IN.inc_by(in_bytes as f64);
            crate::metrics::PLAYER_BYTES_OUT.inc_by(out_bytes as f64);
            crate::session_log::log_session_end(server_id, in_bytes as u64, out_bytes as u64);
        }
        Ok(Err(e)) => {
            debug!("[PLAYER] Bidi copy error: {}", e);
            crate::session_log::log_session_error(server_id, &e.to_string());
        }
        Err(_) => {
            debug!("[PLAYER] Idle timeout (5m); closing");
        }
    }
    Ok(())
}

/// Read the Minecraft Java Handshake packet and extract the subdomain from
/// the `server address` field. Returns (subdomain, buffered_bytes_to_forward).
///
/// The packet structure is:
///   [VarInt packet length][VarInt packet ID = 0x00][VarInt protocol version]
///   [String server address (VarInt length + UTF-8 bytes)][ushort server port]
///   [VarInt next state]
///
/// We read at most `HANDSHAKE_READ_TIMEOUT` worth of bytes; if the Handshake
/// doesn't arrive, we close (D-18). We buffer the bytes we read (the full
/// Handshake packet) and return them in `prefix_bytes` so the caller can
/// forward them to the agent (the agent expects to receive the entire
/// Handshake packet on a fresh yamux stream, not just the server address).
pub async fn read_mc_handshake_subdomain(
    tcp: &mut TcpStream,
    _peer: SocketAddr,
) -> anyhow::Result<(String, Vec<u8>)> {
    // Read up to 1 KiB — the Handshake packet for typical MC clients is
    // ~30-100 bytes (server address is short: "abc12345.play.esluce.com").
    let mut buf = vec![0u8; 1024];
    let mut total_read = 0usize;
    let read_fut = async {
        loop {
            let n = tcp.read(&mut buf[total_read..]).await?;
            if n == 0 { break; }  // EOF
            total_read += n;
            // If we've read at least the packet length + packet ID + the start
            // of the server address's VarInt length, try to parse.
            if total_read >= 2 && try_parse_handshake(&buf[..total_read]).is_some() {
                break;
            }
            if total_read >= buf.len() {
                anyhow::bail!("Handshake packet > 1 KiB (unusual)");
            }
        }
        Ok::<(), anyhow::Error>(())
    };
    tokio::time::timeout(HANDSHAKE_READ_TIMEOUT, read_fut).await??;

    let (subdomain, consumed) = try_parse_handshake(&buf[..total_read])
        .ok_or_else(|| anyhow::anyhow!("Incomplete or invalid Handshake"))?;

    // Sanity check: the address must end in `.play.esluce.com`
    if !subdomain.ends_with(RELAY_SUFFIX) {
        anyhow::bail!("Server address does not end in {}: {}", RELAY_SUFFIX, subdomain);
    }
    // Extract the subdomain (everything before `.play.esluce.com`)
    let subdomain = subdomain[..subdomain.len() - RELAY_SUFFIX.len()].to_string();
    if subdomain.is_empty() || subdomain.len() > 63 {
        anyhow::bail!("Subdomain length out of range: '{}'", subdomain);
    }
    // Validate charset (lowercase alphanumeric + hyphens)
    if !subdomain.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        anyhow::bail!("Subdomain has invalid characters: '{}'", subdomain);
    }
    // Return the FULL buffered Handshake bytes so the caller can forward them
    let prefix_bytes = buf[..consumed].to_vec();
    Ok((subdomain, prefix_bytes))
}

/// Try to parse a Minecraft Handshake packet from `buf`. Returns
/// `Some((server_address, bytes_consumed))` on success, `None` if the buffer
/// is incomplete (caller should read more). See VarInt encoding in the plan's
/// `<interfaces>` block.
fn try_parse_handshake(buf: &[u8]) -> Option<(String, usize)> {
    let mut p = 0usize;
    // [VarInt packet length]
    let (packet_length, n) = read_varint(buf, p)?;
    p += n;
    let packet_end = p.checked_add(packet_length as usize)?;
    if buf.len() < packet_end { return None; }  // need more bytes
    // [VarInt packet ID] — must be 0x00 for Handshake
    let (packet_id, n) = read_varint(buf, p)?;
    p += n;
    if packet_id != 0x00 { return None; }  // not a Handshake
    // [VarInt protocol version] — read but don't validate value
    let (_proto, n) = read_varint(buf, p)?;
    p += n;
    // [String server address]
    let (server_address, n) = read_mc_string(buf, p)?;
    let _ = n;
    // [ushort server port] and [VarInt next state] follow; we don't need them
    Some((server_address, packet_end))
}

fn read_varint(buf: &[u8], start: usize) -> Option<(u32, usize)> {
    let mut value: u32 = 0;
    let mut shift = 0u32;
    let mut i = start;
    loop {
        if i >= buf.len() { return None; }
        let b = buf[i];
        value |= ((b & 0x7F) as u32) << shift;
        i += 1;
        if b & 0x80 == 0 { return Some((value, i - start)); }
        shift += 7;
        if shift >= 35 { return None; }  // VarInt overflow
    }
}

fn read_mc_string(buf: &[u8], start: usize) -> Option<(String, usize)> {
    let (byte_len, n) = read_varint(buf, start)?;
    let byte_len = byte_len as usize;
    if byte_len > MAX_MC_STRING_BYTES { return None; }
    let str_start = start + n;
    let str_end = str_start.checked_add(byte_len)?;
    if buf.len() < str_end { return None; }
    let s = std::str::from_utf8(&buf[str_start..str_end]).ok()?;
    Some((s.to_string(), n + byte_len))
}
