use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use super::args::Cli;

pub struct CodeGraphRunner;

impl CodeGraphRunner {
    pub fn new() -> Self {
        Self
    }

    pub fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize logging
        let subscriber = FmtSubscriber::builder()
            .with_max_level(if cli.verbose { Level::DEBUG } else { Level::INFO })
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;

        info!("Server mode only; no CLI commands to execute");

        Ok(())
    }
} 