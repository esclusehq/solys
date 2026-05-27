use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber;

mod config;
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

    let redis = redis::Client::open(config.redis_url.as_str())?
        .get_multiplexed_async_connection()
        .await?;

    let mut processor = queue::JobProcessor::new(redis);
    
    info!("Worker started, processing jobs...");
    
    processor.run().await;

    Ok(())
}
