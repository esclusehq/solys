use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub backend: BackendConfig,
    pub redis: RedisConfig,
    pub tunnel: TunnelConfig,
    pub ratelimit: RateLimitConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub tunnel_bind: String,
    pub player_bind: String,
    pub metrics_bind: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackendConfig {
    pub base_url: String,
    pub hmac_secret_env: String,
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TunnelConfig {
    pub heartbeat_interval_secs: u64,
    pub heartbeat_missed_threshold: u64,
    pub max_tunnels_per_server: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        // Try to load from `relay-gateway.toml` in CWD; allow override via RELAY_CONFIG env.
        let config_path = std::env::var("RELAY_CONFIG")
            .unwrap_or_else(|_| "relay-gateway.toml".to_string());
        let settings = config::Config::builder()
            .add_source(config::File::with_name(&config_path).required(false))
            .add_source(
                config::Environment::with_prefix("RELAY")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;
        let cfg: Config = settings.try_deserialize()?;
        Ok(cfg)
    }
}
