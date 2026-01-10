use clap::Parser;
use codegraph_cli::cli::{Cli, CodeGraphRunner};
use codegraph_cli::cli::args::Commands;
use codegraph_cli::http::CodeGraphServer;
use codegraph_cli::storage::StorageManager;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            if cli.verbose {
                tracing_subscriber::EnvFilter::new("debug")
            } else {
                tracing_subscriber::EnvFilter::new("info")
            }
        });

    tracing_subscriber::fmt()
        .with_env_filter(filter_layer)
        .init();

    match &cli.command {
        Commands::Server { address, storage_mode } => {
            let server_addr = address.as_deref().unwrap_or("127.0.0.1:8080");
            info!("Starting CodeGraph HTTP server on {}", server_addr);

            // Determine storage mode
            let storage_mode = storage_mode.as_ref().unwrap_or(&cli.storage_mode).clone();
            info!("Using storage mode: {:?}", storage_mode);

            let storage = Arc::new(StorageManager::with_storage_mode(storage_mode));
            let server = CodeGraphServer::new(storage);
            server.start(server_addr).await?;
        }
        Commands::Vectorize { .. } => {
            // 使用CodeGraphRunner处理vectorize命令
            CodeGraphRunner::run(cli).await?;
        }
    }

    Ok(())
}