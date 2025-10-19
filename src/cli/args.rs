use clap::{Parser, Subcommand, ValueEnum};

/// 存储方式配置
#[derive(Debug, Clone, ValueEnum)]
pub enum StorageMode {
    /// 仅JSON格式存储
    Json,
    /// 仅二进制格式存储
    Binary,
    /// 同时保存JSON和二进制格式
    Both,
}

impl Default for StorageMode {
    fn default() -> Self {
        StorageMode::Json
    }
}

/// CodeGraph CLI - Analyze code dependencies and generate code graphs
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Verbose mode
    #[clap(short, long, action)]
    pub verbose: bool,

    /// Storage mode for code graph persistence
    #[clap(long, value_enum, default_value = "json")]
    pub storage_mode: StorageMode,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start HTTP server on specified address (e.g., 127.0.0.1:8080)
    Server {
        #[clap(long, value_parser)]
        address: Option<String>,

        /// Storage mode override for this command
        #[clap(long, value_enum)]
        storage_mode: Option<StorageMode>,
    },
    /// Vectorize code blocks and save to Qdrant
    Vectorize {
        /// Path to the directory to vectorize
        #[clap(long, value_parser)]
        path: String,
        
        /// Qdrant collection name
        #[clap(long, value_parser)]
        collection: String,
        
        /// Qdrant server URL
        #[clap(long, value_parser, default_value = "http://localhost:6334")]
        qdrant_url: String,
    },
}