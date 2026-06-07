use std::time::Instant;

use crate::backend::BackendClient;
use crate::config::Config;
use crate::ratelimit::RateLimiter;
use crate::registry::Registry;

pub struct AppState {
    pub config: Config,
    pub registry: Registry,
    pub backend: BackendClient,
    pub redis: redis::aio::ConnectionManager,
    pub rate_limiter: std::sync::Arc<RateLimiter>,
    pub start_time: Instant,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let redis_client = redis::Client::open(config.redis.url.as_str())?;
        let redis = redis::aio::ConnectionManager::new(redis_client).await?;
        let backend = BackendClient::new(
            config.backend.base_url.clone(),
            config.backend.hmac_secret_env.clone(),
            config.backend.request_timeout_secs,
        )?;
        let rate_limiter = std::sync::Arc::new(RateLimiter::new(config.ratelimit.requests_per_minute));
        Ok(Self {
            config,
            registry: Registry::new(),
            backend,
            redis,
            rate_limiter,
            start_time: Instant::now(),
        })
    }
}
