use anyhow::{Context, Result};
use dotenv::dotenv;
use std::env;

pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub worker_id: String,
    pub worker_poll_interval_ms: u64,
    pub worker_concurrency: u32,
    pub jwt_secret: String,
    pub app_url: String,
    pub api_base_url: String,
}

impl Config {
    pub fn new() -> Result<Self> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL must be set")?;

        let redis_url = env::var("REDIS_URL")
            .context("REDIS_URL must be set")?;

        let worker_id = env::var("WORKER_ID")
            .unwrap_or_else(|_| "worker-01".to_string());

        let worker_poll_interval_ms = env::var("WORKER_POLL_INTERVAL_MS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<u64>()
            .unwrap_or(1000);

        let worker_concurrency = env::var("WORKER_CONCURRENCY")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .unwrap_or(5);

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "dev-secret-key-min-32-chars-long".to_string());

        let app_url = env::var("APP_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let api_base_url = env::var("API_BASE_URL")
            .unwrap_or_else(|_| "http://api:3000".to_string());

        Ok(Self {
            database_url,
            redis_url,
            worker_id,
            worker_poll_interval_ms,
            worker_concurrency,
            jwt_secret,
            app_url,
            api_base_url,
        })
    }
}
