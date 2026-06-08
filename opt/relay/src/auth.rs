use uuid::Uuid;

use crate::error::GatewayError;
use crate::state::AppState;
use crate::types::ServerMapping;

/// Call the backend's `/internal/relay/authorize` endpoint with HMAC-SHA256
/// signature. Returns `Ok(Vec<ServerMapping>)` — all servers authorized by
/// this relay token (Phase 69: 1:N token→server mapping). Backward-compatible:
/// the backend may return a single object or an array; both are handled.
/// Returns `Err(GatewayError::...)` on infrastructure failure.
pub async fn authorize(
    state: &AppState,
    token: &Uuid,
) -> Result<Vec<ServerMapping>, GatewayError> {
    state.backend.authorize(*token).await
}
