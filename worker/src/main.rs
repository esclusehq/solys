use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber;

mod config;
mod cron_eval;
mod prune;
mod queue;
mod agent;
mod webhook;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("RUST_LOG")
        .init();

    info!("Starting worker service...");

    let config = config::Config::new()?;

    // PostgreSQL connection for cron_tasks queries
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    let redis = redis::Client::open(config.redis_url.as_str())?
        .get_multiplexed_async_connection()
        .await?;

    // Start cron evaluation loop in background (D-02)
    let cron_pool = pool.clone();
    let cron_redis = redis.clone();
    tokio::spawn(async move {
        cron_eval::run_cron_evaluation_loop(cron_pool, cron_redis).await;
    });

    // Start prune task loop (D-15) — decoupled from backup eval, runs every 15 min
    let prune_pool = pool.clone();
    let prune_api_url = config.api_base_url.clone();
    tokio::spawn(async move {
        prune::run_prune_loop(prune_pool, prune_api_url).await;
    });

    // Start job processor with pool for DB access
    let mut processor = queue::JobProcessor::new(redis, pool);
    
    info!("Worker started, processing jobs...");
    
    processor.run().await;

    Ok(())
}
