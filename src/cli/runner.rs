use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::codegraph::parser::CodeParser;
use crate::codegraph::PetGraphStorageManager;
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

        info!("Starting CodeGraph analysis...");

        // Create parser and analyze code
        let mut parser = CodeParser::new();
        
        info!("Parsing directory: {:?}", cli.input);
        let code_graph = parser.build_petgraph_code_graph(&cli.input)?;
        
        // Get output path
        let output_path = cli.output.unwrap_or_else(|| {
            let mut path = cli.input.clone();
            path.push("codegraph.json");
            path
        });
        
        // Export based on format
        Self::export_code_graph(&code_graph, &cli.format, &output_path)?;
        
        // Print statistics
        let stats = code_graph.get_stats();
        info!("Analysis complete!");
        info!("  Total functions: {}", stats.total_functions);
        info!("  Total files: {}", stats.total_files);
        info!("  Total languages: {}", stats.total_languages);
        info!("  Resolved calls: {}", stats.resolved_calls);
        info!("  Unresolved calls: {}", stats.unresolved_calls);

        Ok(())
    }

    fn export_code_graph(
        code_graph: &crate::codegraph::PetCodeGraph,
        format: &str,
        output_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match format {
            "json" => {
                PetGraphStorageManager::save_to_file(code_graph, output_path)?;
                info!("Code graph saved to JSON file: {:?}", output_path);
            }
            "mermaid" => {
                let mermaid = code_graph.to_mermaid();
                std::fs::write(output_path, mermaid)?;
                info!("Code graph saved to Mermaid file: {:?}", output_path);
            }
            "dot" => {
                let dot = code_graph.to_dot();
                std::fs::write(output_path, dot)?;
                info!("Code graph saved to DOT file: {:?}", output_path);
            }
            "graphml" => {
                PetGraphStorageManager::export_to_graphml(code_graph, output_path)?;
                info!("Code graph saved to GraphML file: {:?}", output_path);
            }
            "gexf" => {
                PetGraphStorageManager::export_to_gexf(code_graph, output_path)?;
                info!("Code graph saved to GEXF file: {:?}", output_path);
            }
            _ => {
                eprintln!("Unsupported format: {}", format);
                std::process::exit(1);
            }
        }
        Ok(())
    }
} 