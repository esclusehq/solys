//! Trace ID middleware for distributed tracing
//!
//! Extracts or generates X-Trace-ID header and propagates through the request chain.

use axum::{
    extract::Request,
    http::{
        header::HeaderName,
        header::HeaderValue,
    },
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Header name for trace ID
pub const TRACE_ID_HEADER: &str = "x-trace-id";

/// Trace ID extension key - store trace ID in request extensions
#[derive(Clone, Debug)]
pub struct TraceId(pub String);

/// Get or create trace ID from request headers, or generate new UUID v4
pub async fn trace_id_middleware(mut request: Request, next: Next) -> Response {
    // Get or create trace ID
    let trace_id = request
        .headers()
        .get(TRACE_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Inject into extensions for use by handlers
    request.extensions_mut().insert(TraceId(trace_id.clone()));

    // Call next handler
    let mut response = next.run(request).await;

    // Add trace ID to response headers
    let header_name = HeaderName::from_static(TRACE_ID_HEADER);
    match HeaderValue::from_str(&trace_id) {
        Ok(header_value) => {
            response.headers_mut().insert(header_name, header_value);
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to set trace ID header");
        }
    }

    response
}

/// Extract trace ID from request extensions
pub fn get_trace_id(request: &axum::extract::Request) -> Option<String> {
    request.extensions().get::<TraceId>().map(|t| t.0.clone())
}