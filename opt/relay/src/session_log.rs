use std::net::IpAddr;

use tracing::{error, info, warn};
use uuid::Uuid;

pub fn log_session_start(server_id: Uuid, agent_ip: IpAddr) {
    info!(
        target: "relay::session",
        "[SESSION] start: server_id={}, agent_ip={}",
        server_id, agent_ip
    );
}

pub fn log_session_end(server_id: Uuid, bytes_tx: u64, bytes_rx: u64) {
    info!(
        target: "relay::session",
        "[SESSION] end: server_id={}, bytes_tx={}, bytes_rx={}",
        server_id, bytes_tx, bytes_rx
    );
}

pub fn log_session_error(server_id: Uuid, error_msg: &str) {
    error!(
        target: "relay::session",
        "[SESSION] error: server_id={}, error={}",
        server_id, error_msg
    );
}

#[allow(dead_code)]
pub fn log_session_unused_warn(server_id: Uuid, detail: &str) {
    warn!(
        target: "relay::session",
        "[SESSION] note: server_id={}, detail={}",
        server_id, detail
    );
}
