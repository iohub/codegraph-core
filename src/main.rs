use clap::Parser;
use codegraph_cli::cli::{Cli, CodeGraphRunner};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    CodeGraphRunner::run(cli)
}