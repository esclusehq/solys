//! Bearer token authentication middleware
//!
//! Validates `Authorization: Bearer <token>` header against a server-configured token.
//! Unauthenticated requests receive HTTP 401 Unauthorized.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

// API token loaded from environment variable at startup
lazy_static::lazy_static! {
    static ref API_TOKEN: String = {
        std::env::var("ESCLUSE_API_TOKEN").unwrap_or_else(|_| {
            tracing::warn!(
                "ESCLUSE_API_TOKEN not set, using default 'change-me' token — \
                 this is INSECURE and should only be used for development"
            );
            "change-me".to_string()
        })
    };
}

/// Authenticate incoming requests via Bearer token
///
/// Checks the `Authorization` header for a valid `Bearer <token>` value.
/// Returns `Ok(Response)` on match or `Err(StatusCode::UNAUTHORIZED)` on mismatch.
pub async fn auth_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match auth_header {
        Some(token) if token == API_TOKEN.as_str() => {
            Ok(next.run(request).await)
        }
        _ => {
            tracing::warn!(
                path = %request.uri().path(),
                "Unauthorized request — invalid or missing Bearer token"
            );
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::HeaderValue,
        routing::get,
        Router,
        middleware,
        response::IntoResponse,
    };

    async fn dummy_handler() -> impl IntoResponse {
        "OK"
    }

    fn test_app() -> Router {
        Router::new()
            .route("/test", get(dummy_handler))
            .layer(middleware::from_fn(auth_middleware))
    }

    #[tokio::test]
    async fn test_valid_token() {
        let app = test_app();
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/test")
                    .header("Authorization", "Bearer change-me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_missing_token() {
        let app = test_app();
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_token() {
        let app = test_app();
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/test")
                    .header("Authorization", "Bearer wrong-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_missing_bearer_prefix() {
        let app = test_app();
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/test")
                    .header("Authorization", "Basic change-me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
