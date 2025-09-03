use clap::Parser;
use codegraph_cli::cli::{Cli, CodeGraphRunner};
use codegraph_cli::cli::args::Commands;
use codegraph_cli::http::CodeGraphServer;
use codegraph_cli::storage::StorageManager;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Check if we should start HTTP server
    if let Commands::Server { address, storage_mode } = &cli.command {
        let server_addr = address.as_deref().unwrap_or("127.0.0.1:8080");
        println!("Starting CodeGraph HTTP server on {}", server_addr);
        
        // Determine storage mode
        let storage_mode = storage_mode.as_ref().unwrap_or(&cli.storage_mode).clone();
        println!("Using storage mode: {:?}", storage_mode);
        
        let storage = Arc::new(StorageManager::with_storage_mode(storage_mode));
        let server = CodeGraphServer::new(storage);
        server.start(server_addr).await?;
    } else {
        // Run CLI mode
        CodeGraphRunner::run(cli)?;
    }
    
    Ok(())
}