use std::path::PathBuf;
use clap::Parser;

/// CodeGraph CLI - Analyze code dependencies and generate code graphs
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Input directory to analyze
    #[clap(short, long, value_parser)]
    pub input: PathBuf,

    /// Output file path
    #[clap(short, long, value_parser)]
    pub output: Option<PathBuf>,

    /// Output format (json, mermaid, dot, graphml, gexf)
    #[clap(short, long, value_parser, default_value = "json")]
    pub format: String,

    /// Verbose mode
    #[clap(short, long, action)]
    pub verbose: bool,
} 