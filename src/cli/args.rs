use std::path::PathBuf;
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
    /// Analyze code dependencies and generate code graphs
    Analyze {
        /// Input directory to analyze
        #[clap(short, long, value_parser)]
        input: Option<PathBuf>,

        /// Output file path
        #[clap(short, long, value_parser)]
        output: Option<PathBuf>,

        /// Output format (json, dot, graphml, gexf)
        #[clap(short, long, value_parser, default_value = "json")]
        format: String,

        /// Storage mode override for this command
        #[clap(long, value_enum)]
        storage_mode: Option<StorageMode>,
    },

    /// Repository analysis with incremental updates
    Repo {
        /// 要分析的仓库路径
        #[clap(short, long, default_value = ".")]
        path: PathBuf,

        /// 输出状态目录
        #[clap(long, default_value = "./.codegraph")]
        state_dir: PathBuf,

        /// 是否增量更新
        #[clap(short, long)]
        incremental: bool,

        /// 搜索查询
        #[clap(long)]
        search: Option<String>,

        /// 显示统计信息
        #[clap(short, long)]
        stats: bool,

        /// Storage mode override for this command
        #[clap(long, value_enum)]
        storage_mode: Option<StorageMode>,
    },

    /// Start HTTP server on specified address (e.g., 127.0.0.1:8080)
    Server {
        #[clap(long, value_parser)]
        address: Option<String>,

        /// Storage mode override for this command
        #[clap(long, value_enum)]
        storage_mode: Option<StorageMode>,
    },
} 