use std::path::PathBuf;
use clap::{Parser, Subcommand};

/// CodeGraph CLI - Analyze code dependencies and generate code graphs
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Verbose mode
    #[clap(short, long, action)]
    pub verbose: bool,

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

        /// Output format (json, mermaid, dot, graphml, gexf)
        #[clap(short, long, value_parser, default_value = "json")]
        format: String,
    },

    /// Analyze codebase using the new orchestrator
    AnalyzeV2 {
        /// Root directory to analyze
        #[clap(short, long, default_value = ".")]
        root: PathBuf,

        /// Languages to analyze (comma-separated: rust,java,python,cpp,typescript,javascript)
        #[clap(long)]
        languages: Option<String>,

        /// Output directory
        #[clap(long, default_value = "target/codegraph")]
        output_dir: PathBuf,

        /// Output formats (comma-separated: json,mermaid,dot)
        #[clap(long, default_value = "json,mermaid,dot")]
        formats: String,

        /// Maximum number of worker threads
        #[clap(long)]
        workers: Option<usize>,

        /// Include test files
        #[clap(long)]
        include_tests: bool,

        /// Follow symbolic links
        #[clap(long)]
        follow_symlinks: bool,
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
    },

    /// Start HTTP server on specified address (e.g., 127.0.0.1:8080)
    Server {
        #[clap(long, value_parser)]
        address: Option<String>,
    },
} 