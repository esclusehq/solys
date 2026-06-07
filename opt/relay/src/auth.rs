use serde::Serialize;
use uuid::Uuid;

use crate::error::GatewayError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct Authorization {
    pub node_id: Uuid,
    pub user_id: Uuid,
}

/// Call the backend's `/internal/relay/authorize` endpoint with HMAC-SHA256
/// signature. Returns `Ok(Authorization)` on 200, `Ok(Authorization { user_id: Uuid::nil() })`
/// on 403 (or 401) to avoid leaking the difference between "unknown token" and
/// "token doesn't own server", and `Err(GatewayError::...)` on infrastructure failure.
pub async fn authorize(
    state: &AppState,
    token: &Uuid,
    server_id: &Uuid,
) -> Result<Authorization, GatewayError> {
    state.backend.authorize(*token, *server_id).await
}
