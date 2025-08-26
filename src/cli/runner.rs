use std::path::PathBuf;
use tracing::{info, Level, warn};
use tracing_subscriber::FmtSubscriber;

use crate::codegraph::parser::CodeParser;
use crate::storage::PetGraphStorageManager;
use super::args::{Cli, Commands};

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

        match cli.command {
            Commands::Analyze { input, output, format } => {
                Self::run_analyze(input, output, format)?;
            }
            Commands::Repo { path, state_dir, incremental, search, stats } => {
                Self::run_repo_analysis(path, state_dir, incremental, search, stats)?;
            }
            Commands::Server { address: _ } => {
                // HTTP server is handled in main.rs
                return Err("HTTP server should be started from main.rs".into());
            }
        }

        Ok(())
    }

    fn run_analyze(
        input: Option<PathBuf>,
        output: Option<PathBuf>,
        format: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate that input is provided
        let input = input.ok_or("Input directory is required for analysis")?;

        info!("Starting CodeGraph analysis...");

        // Create parser and analyze code
        let mut parser = CodeParser::new();
        
        info!("Parsing directory: {:?}", input);
        let code_graph = parser.build_petgraph_code_graph(&input)?;
        
        // Get output path
        let output_path = output.unwrap_or_else(|| {
            let mut path = input.clone();
            path.push("codegraph.json");
            path
        });
        
        // Export based on format
        Self::export_code_graph(&code_graph, &format, &output_path)?;
        
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

    fn run_repo_analysis(
        path: PathBuf,
        state_dir: PathBuf,
        incremental: bool,
        search: Option<String>,
        stats: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::codegraph::repository::RepositoryManager;

        info!("Starting repository analysis for: {}", path.display());

        // 创建仓库管理器
        let mut repo_manager = RepositoryManager::new(path);

        // 尝试加载现有状态
        if state_dir.exists() {
            if let Err(e) = repo_manager.load_state(&state_dir) {
                warn!("Failed to load existing state: {}", e);
                info!("Starting fresh analysis...");
            } else {
                info!("Loaded existing state from: {}", state_dir.display());
            }
        }

        if incremental {
            // 增量更新模式
            info!("Running in incremental mode");
            // 这里可以实现文件监控和增量更新逻辑
        } else {
            // 全量分析模式
            info!("Running full repository analysis");
            repo_manager.initialize()?;
        }

        // 显示统计信息
        if stats {
            let stats = repo_manager.get_repository_stats();
            
        }

        // 执行搜索
        if let Some(query) = &search {
            info!("Searching for: {}", query);
            let results = repo_manager.search_entities(query);
            
            if results.is_empty() {
                // No results found
            } else {
                for result in results {
                    println!("  {} [{}] - {}:{}:{} ({})", 
                        result.name, 
                        result.entity_type, 
                        result.file_path.display(), 
                        result.line_start, 
                        result.line_end,
                        result.language
                    );
                }
            }
        }

        // 保存状态
        if let Err(e) = repo_manager.save_state(&state_dir) {
            warn!("Failed to save state: {}", e);
        } else {
            info!("Repository state saved to: {}", state_dir.display());
        }

        info!("Repository analysis completed successfully");
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
                PetGraphStorageManager::export_to_graphml(code_graph, output_path)?;
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