use clap::Parser;
use codegraph_cli::cli::{Cli, CodeGraphRunner};
use codegraph_cli::http::CodeGraphServer;
use codegraph_cli::storage::StorageManager;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Check if we should start HTTP server
    if let Some(server_addr) = cli.server {
        println!("Starting CodeGraph HTTP server on {}", server_addr);
        
        let storage = Arc::new(StorageManager::new());
        let server = CodeGraphServer::new(storage);
        server.start(&server_addr).await?;
    } else {
        // Run CLI mode
        CodeGraphRunner::run(cli)?;
    }
    
    Ok(())
}