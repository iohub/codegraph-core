use std::path::PathBuf;
use tracing::{info, Level, warn};
use tracing_subscriber::FmtSubscriber;

use crate::codegraph::parser::CodeParser;
use crate::storage::{PetGraphStorageManager, StorageManager};
use crate::cli::args::StorageMode;
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

        // Determine storage mode
        let storage_mode = match &cli.command {
            Commands::Analyze { storage_mode: Some(mode), .. } => mode.clone(),
            Commands::Repo { storage_mode: Some(mode), .. } => mode.clone(),
            Commands::Server { storage_mode: Some(mode), .. } => mode.clone(),
            _ => cli.storage_mode.clone(),
        };

        info!("Using storage mode: {:?}", storage_mode);

        match cli.command {
            Commands::Analyze { input, output, format, storage_mode: _ } => {
                Self::run_analyze(input, output, format, storage_mode)?;
            }
            Commands::Repo { path, state_dir, incremental, search, stats, storage_mode: _ } => {
                Self::run_repo_analysis(path, state_dir, incremental, search, stats, storage_mode)?;
            }
            Commands::Server { address: _, storage_mode: _ } => {
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
        storage_mode: StorageMode,
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
            match storage_mode {
                StorageMode::Json => path.push("codegraph.json"),
                StorageMode::Binary => path.push("codegraph.bin"),
                StorageMode::Both => path.push("codegraph"), // Will create both .json and .bin
            }
            path
        });
        
        // Export based on format and storage mode
        Self::export_code_graph(&code_graph, &format, &output_path, &storage_mode)?;
        
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
        storage_mode: StorageMode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::codegraph::repository::RepositoryManager;

        info!("Starting repository analysis for: {}", path.display());
        info!("Storage mode: {:?}", storage_mode);

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
            let _stats = repo_manager.get_repository_stats();
            
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

        // 保存状态（使用指定的存储模式）
        let storage_manager = StorageManager::with_storage_mode(storage_mode);
        let persistence = storage_manager.get_persistence();
        
        // 这里需要根据实际的代码图数据进行保存
        // 如果repo_manager有get_code_graph方法，可以这样调用：
        // if let Some(graph) = repo_manager.get_code_graph() {
        //     if let Err(e) = persistence.save_graph("main", &graph) {
        //         warn!("Failed to save graph: {}", e);
        //     }
        // }

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
        storage_mode: &StorageMode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match format {
            "json" => {
                match storage_mode {
                    StorageMode::Json => {
                        PetGraphStorageManager::save_to_file(code_graph, output_path)?;
                        info!("Code graph saved to JSON file: {:?}", output_path);
                    },
                    StorageMode::Binary => {
                        let mut binary_path = output_path.clone();
                        binary_path.set_extension("bin");
                        PetGraphStorageManager::save_to_binary(code_graph, &binary_path)?;
                        info!("Code graph saved to binary file: {:?}", binary_path);
                    },
                    StorageMode::Both => {
                        let mut json_path = output_path.clone();
                        json_path.set_extension("json");
                        let mut binary_path = output_path.clone();
                        binary_path.set_extension("bin");
                        
                        PetGraphStorageManager::save_to_file(code_graph, &json_path)?;
                        PetGraphStorageManager::save_to_binary(code_graph, &binary_path)?;
                        info!("Code graph saved to both JSON and binary files: {:?} and {:?}", 
                              json_path, binary_path);
                    },
                }
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