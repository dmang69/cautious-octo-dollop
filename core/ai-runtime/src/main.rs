mod context_manager;
mod inference;
mod telemetry;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("AI Runtime daemon starting");

    let manager = context_manager::ContextManager::new().await?;
    manager.run().await?;

    info!("AI Runtime daemon stopped");
    Ok(())
}
