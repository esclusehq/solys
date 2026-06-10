use uuid::Uuid;

use crate::error::GatewayError;
use crate::state::AppState;
use crate::types::ServerMapping;

const REDIS_CACHE_PREFIX: &str = "gateway_auth_token_v1";
const REDIS_CACHE_TTL_SECS: usize = 300; // 5 minutes

/// Retrieve the cached authorization for `token` from Redis, if any.
async fn get_cached(
    redis: &mut redis::aio::ConnectionManager,
    token: &Uuid,
) -> Result<Option<Vec<ServerMapping>>, redis::RedisError> {
    let key = format!("{}:{}", REDIS_CACHE_PREFIX, token);
    let raw: Option<String> = redis::cmd("GET")
        .arg(&key)
        .query_async(redis)
        .await?;
    match raw {
        Some(json) => serde_json::from_str(&json)
            .map(Some)
            .or_else(|_| Ok(None)), // stale JSON → treat as miss
        None => Ok(None),
    }
}

/// Store the authorization result in Redis with a TTL.
async fn set_cached(
    redis: &mut redis::aio::ConnectionManager,
    token: &Uuid,
    mappings: &[ServerMapping],
) {
    let key = format!("{}:{}", REDIS_CACHE_PREFIX, token);
    let json = match serde_json::to_string(mappings) {
        Ok(j) => j,
        Err(_) => return, // serialisation failure is non-fatal
    };
    let _ = redis::cmd("SETEX")
        .arg(&key)
        .arg(REDIS_CACHE_TTL_SECS)
        .arg(&json)
        .query_async::<_, ()>(redis)
        .await;
}

/// Authorize a relay token with backend fallback + Redis caching + retry.
///
/// 1. Check Redis cache (hit → return).
/// 2. Call backend with up to 3 retries (exponential backoff: 1s, 2s, 4s).
/// 3. On success → populate Redis cache.
/// 4. On failure → try Redis cache again (stale-while-revalidate).
///
/// This is production-ready for transient backend outages (e.g. Cloudflare 525)
/// and high-frequency reconnect storms.
pub async fn authorize(
    state: &AppState,
    token: &Uuid,
) -> Result<Vec<ServerMapping>, GatewayError> {
    // 1. Fast path: cache hit.
    {
        let mut redis = state.redis.lock().await;
        if let Ok(Some(mappings)) = get_cached(&mut redis, token).await {
            return Ok(mappings);
        }
    }

    // 2. Backend call with retry.
    let backoff = [1u64, 2, 4]; // seconds
    let mut last_err = None;
    for (i, delay) in backoff.iter().enumerate() {
        match state.backend.authorize(*token).await {
            Ok(mappings) => {
                let mut redis = state.redis.lock().await;
                set_cached(&mut redis, token, &mappings).await;
                return Ok(mappings);
            }
            Err(e) => {
                last_err = Some(e);
                if i < backoff.len() - 1 {
                    tokio::time::sleep(std::time::Duration::from_secs(*delay)).await;
                }
            }
        }
    }

    // 3. Retries exhausted — attempt stale cache as final fallback.
    {
        let mut redis = state.redis.lock().await;
        if let Ok(Some(mappings)) = get_cached(&mut redis, token).await {
            tracing::warn!(
                "[AUTH] Backend unreachable, serving stale cache for token={}",
                token
            );
            return Ok(mappings);
        }
    }

    Err(last_err.unwrap_or(GatewayError::BackendUnreachable(
        "authorization retries exhausted".into(),
    )))
}
