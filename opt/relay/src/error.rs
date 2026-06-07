use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("Authentication failed")]
    Auth,
    #[error("Rate limited")]
    RateLimited,
    #[error("Backend unreachable: {0}")]
    BackendUnreachable(String),
    #[error("Tunnel limit reached")]
    TunnelLimit,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, kind) = match &self {
            GatewayError::Auth => (StatusCode::UNAUTHORIZED, "auth_failed"),
            GatewayError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
            GatewayError::BackendUnreachable(_) => (StatusCode::BAD_GATEWAY, "backend_unreachable"),
            GatewayError::TunnelLimit => (StatusCode::TOO_MANY_REQUESTS, "tunnel_limit"),
            GatewayError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            GatewayError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };
        let body = Json(json!({
            "error": kind,
            "message": self.to_string(),
        }));
        (status, body).into_response()
    }
}
