use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use super::args::{Cli, Commands};
use super::vectorize::run_vectorize;

pub struct CodeGraphRunner;

impl CodeGraphRunner {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize logging
        let subscriber = FmtSubscriber::builder()
            .with_max_level(if cli.verbose { Level::DEBUG } else { Level::INFO })
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;

        match cli.command {
            Commands::Server { address: _, storage_mode: _ } => {
                info!("Starting server mode");
                // TODO: 启动HTTP服务器
                info!("Server mode not fully implemented yet");
            }
            Commands::Vectorize { path, collection, qdrant_url } => {
                info!("Starting vectorize mode");
                run_vectorize(path, collection, qdrant_url).await?;
            }
        }

        Ok(())
    }
}