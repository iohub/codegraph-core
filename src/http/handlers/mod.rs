use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use std::sync::Arc;
use crate::storage::StorageManager;
use crate::services::CodeAnalyzer;
use super::models::*;
use md5;
use uuid;

pub async fn build_graph(
    State(storage): State<Arc<StorageManager>>,
    Json(request): Json<BuildGraphRequest>,
) -> Result<Json<ApiResponse<BuildGraphResponse>>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    // Get project directory path
    let project_dir = std::path::Path::new(&request.project_dir);
    
    // Generate project ID using MD5 hash of project directory
    let project_id = format!("{:x}", md5::compute(request.project_dir.as_bytes()));
    if !project_dir.exists() || !project_dir.is_dir() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Check for existing graph
    let mut cache_hit_rate = 0.0;
    let _existing_graph: crate::codegraph::PetCodeGraph = if let Ok(Some(existing_graph)) = storage.get_persistence().load_graph(&project_id) {
        cache_hit_rate = 0.8; // Assume 80% cache hit for existing graph
        existing_graph
    } else {
        // Create new graph using CodeAnalyzer
        let mut analyzer = CodeAnalyzer::new();
        
        // Analyze directory and build code graph
        match analyzer.analyze_directory(project_dir) {
            Ok(_code_graph) => {
                // Convert CodeGraph to PetCodeGraph
                if let Some(_cg) = analyzer.get_code_graph() {
                    // For now, create a new PetCodeGraph since we need to convert from CodeGraph
                    // In the future, we could modify the analyzer to return PetCodeGraph directly
                    crate::codegraph::types::PetCodeGraph::new()
                } else {
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            },
            Err(e) => {
                tracing::error!("Failed to analyze directory: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };
    
    
    // Use CodeAnalyzer to build the actual graph
    let mut analyzer = CodeAnalyzer::new();
    let mut total_files = 0;
    let mut total_functions = 0;
    
    match analyzer.analyze_directory(project_dir) {
        Ok(_code_graph) => {
            if let Some(stats) = analyzer.get_stats() {
                total_files = stats.total_files;
                total_functions = stats.total_functions;
            }
            
            // Get the actual code graph for saving
            if let Some(cg) = analyzer.get_code_graph() {
                // Convert to PetCodeGraph for storage
                // This is a simplified conversion - in practice you might want to implement
                // a proper conversion method from CodeGraph to PetCodeGraph
                let mut pet_graph = crate::codegraph::types::PetCodeGraph::new();
                
                // First, add all functions to the pet graph
                for function in cg.functions.values() {
                    pet_graph.add_function(function.clone());
                }
                
                tracing::info!("Added {} functions to PetCodeGraph", cg.functions.len());
                
                // Then, add all call relations
                let mut successful_relations = 0;
                for relation in &cg.call_relations {
                    if let Err(e) = pet_graph.add_call_relation(relation.clone()) {
                        tracing::warn!("Failed to add call relation: {}", e);
                    } else {
                        successful_relations += 1;
                    }
                }
                
                tracing::info!("Successfully added {}/{} call relations to PetCodeGraph", 
                              successful_relations, cg.call_relations.len());
                
                // Update stats
                pet_graph.update_stats();
                
                // Save the converted graph
                if let Err(_) = storage.get_persistence().save_graph(&project_id, &pet_graph) {
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        },
        Err(e) => {
            tracing::error!("Failed to analyze directory: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    
    let build_time_ms = start_time.elapsed().as_millis() as u64;
    
    let response = BuildGraphResponse {
        project_id,
        total_files,
        total_functions,
        build_time_ms,
        cache_hit_rate,
    };
    
    Ok(Json(ApiResponse {
        success: true,
        data: response,
    }))
}

pub async fn query_call_graph(
    State(storage): State<Arc<StorageManager>>,
    Json(request): Json<QueryCallGraphRequest>,
) -> Result<Json<ApiResponse<QueryCallGraphResponse>>, StatusCode> {
    // Extract request parameters
    let filepath = request.filepath;
    let function_name = request.function_name;
    let max_depth = request.max_depth.unwrap_or(3); // Default max depth is 3
    
    // Try to find the project ID by searching through stored graphs
    // In a real implementation, you might want to store project_id -> project_dir mapping
    let project_id = if let Ok(projects) = storage.get_persistence().list_projects() {
        // For now, use the first available project
        // In practice, you'd want to implement a proper project lookup mechanism
        projects.first().cloned()
    } else {
        return Err(StatusCode::NOT_FOUND);
    };
    
    let project_id = project_id.ok_or(StatusCode::NOT_FOUND)?;
    
    // Load the code graph for the project
    let graph = match storage.get_persistence().load_graph(&project_id) {
        Ok(Some(graph)) => graph,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // Debug: Log graph information
    tracing::info!("Loaded graph with {} functions", graph.get_stats().total_functions);
    
    let mut functions = Vec::new();
    
    if let Some(func_name) = function_name {
        // Query specific function by name
        let matching_functions = graph.find_functions_by_name(&func_name);
        
        tracing::info!("Found {} functions matching name '{}'", matching_functions.len(), func_name);
        
        for function in matching_functions {
            tracing::info!("Processing function: {} (ID: {})", function.name, function.id);
            
            // Debug: Log function-specific debug info
            if let Some(func) = graph.get_function_by_id(&function.id) {
                tracing::debug!("Function debug info: {} at {}:{}", func.name, func.file_path.display(), func.line_start);
            }
            
            let callers = graph.get_callers(&function.id);
            let callees = graph.get_callees(&function.id);
            
            tracing::info!("Function {} has {} callers and {} callees", function.name, callers.len(), callees.len());
            
            // Convert to API response format
            let api_function = super::models::FunctionInfo {
                id: function.id.to_string(),
                name: function.name.clone(),
                line_start: function.line_start,
                line_end: function.line_end,
                callers: callers.iter().map(|(caller_func, relation)| {
                    super::models::CallRelation {
                        function_name: caller_func.name.clone(),
                        file_path: caller_func.file_path.display().to_string(),
                        line_number: relation.line_number,
                    }
                }).collect(),
                callees: callees.iter().map(|(callee_func, relation)| {
                    super::models::CallRelation {
                        function_name: callee_func.name.clone(),
                        file_path: callee_func.file_path.display().to_string(),
                        line_number: relation.line_number,
                    }
                }).collect(),
            };
            
            functions.push(api_function);
        }
    } else {
        // Query all functions in the specified file
        let file_path = std::path::PathBuf::from(&filepath);
        let file_functions = graph.find_functions_by_file(&file_path);
        
        tracing::info!("Found {} functions in file '{}'", file_functions.len(), filepath);
        
        for function in file_functions {
            tracing::info!("Processing function: {} (ID: {})", function.name, function.id);
            
            // Debug: Log function-specific debug info
            if let Some(func) = graph.get_function_by_id(&function.id) {
                tracing::debug!("Function debug info: {} at {}:{}", func.name, func.file_path.display(), func.line_start);
            }
            
            let callers = graph.get_callers(&function.id);
            let callees = graph.get_callees(&function.id);
            
            tracing::info!("Function {} has {} callers and {} callees", function.name, callers.len(), callees.len());
            
            // Convert to API response format
            let api_function = super::models::FunctionInfo {
                id: function.id.to_string(),
                name: function.name.clone(),
                line_start: function.line_start,
                line_end: function.line_end,
                callers: callers.iter().map(|(caller_func, relation)| {
                    super::models::CallRelation {
                        function_name: caller_func.name.clone(),
                        file_path: caller_func.file_path.display().to_string(),
                        line_number: relation.line_number,
                    }
                }).collect(),
                callees: callees.iter().map(|(callee_func, relation)| {
                    super::models::CallRelation {
                        function_name: callee_func.name.clone(),
                        file_path: callee_func.file_path.display().to_string(),
                        line_number: relation.line_number,
                    }
                }).collect(),
            };
            
            functions.push(api_function);
        }
    }
    
    // If max_depth > 1, expand the call chains
    if max_depth > 1 {
        let mut expanded_functions = functions.clone();
        
        for function in &functions {
            // Expand callers chain
            let mut visited = std::collections::HashSet::new();
            expand_call_chain(&graph, &function.id, &mut visited, &mut expanded_functions, max_depth - 1, true);
            
            // Expand callees chain
            let mut visited = std::collections::HashSet::new();
            expand_call_chain(&graph, &function.id, &mut visited, &mut expanded_functions, max_depth - 1, false);
        }
        
        functions = expanded_functions;
    }
    
    let response = QueryCallGraphResponse {
        filepath,
        functions,
    };
    
    Ok(Json(ApiResponse {
        success: true,
        data: response,
    }))
}

/// Helper function to expand call chains recursively
fn expand_call_chain(
    graph: &crate::codegraph::types::PetCodeGraph,
    function_id: &str,
    visited: &mut std::collections::HashSet<String>,
    functions: &mut Vec<super::models::FunctionInfo>,
    depth: usize,
    is_caller: bool,
) {
    if depth == 0 || visited.contains(function_id) {
        return;
    }
    
    visited.insert(function_id.to_string());
    
    // Parse UUID from string
    let uuid = match uuid::Uuid::parse_str(function_id) {
        Ok(uuid) => uuid,
        Err(_) => return,
    };
    
    let relations = if is_caller {
        graph.get_callers(&uuid)
    } else {
        graph.get_callees(&uuid)
    };
    
    for (related_func, _relation) in relations {
        let related_id = related_func.id.to_string();
        
        // Check if this function is already in our list
        if !functions.iter().any(|f| f.id == related_id) {
            let callers = graph.get_callers(&related_func.id);
            let callees = graph.get_callees(&related_func.id);
            
            let api_function = super::models::FunctionInfo {
                id: related_id.clone(),
                name: related_func.name.clone(),
                line_start: related_func.line_start,
                line_end: related_func.line_end,
                callers: callers.iter().map(|(caller_func, relation)| {
                    super::models::CallRelation {
                        function_name: caller_func.name.clone(),
                        file_path: caller_func.file_path.display().to_string(),
                        line_number: relation.line_number,
                    }
                }).collect(),
                callees: callees.iter().map(|(callee_func, relation)| {
                    super::models::CallRelation {
                        function_name: callee_func.name.clone(),
                        file_path: callee_func.file_path.display().to_string(),
                        line_number: relation.line_number,
                    }
                }).collect(),
            };
            
            functions.push(api_function);
        }
        
        // Recursively expand
        expand_call_chain(graph, &related_id, visited, functions, depth - 1, is_caller);
    }
}

pub async fn query_code_snippet(
    State(storage): State<Arc<StorageManager>>,
    Json(request): Json<QueryCodeSnippetRequest>,
) -> Result<Json<ApiResponse<CodeSnippetResponse>>, StatusCode> {
    // Try to find the project ID by searching through stored graphs
    let project_id = if let Ok(projects) = storage.get_persistence().list_projects() {
        projects.first().cloned()
    } else {
        return Err(StatusCode::NOT_FOUND);
    };
    
    let project_id = project_id.ok_or(StatusCode::NOT_FOUND)?;
    
    // Load the code graph for the project
    let graph = match storage.get_persistence().load_graph(&project_id) {
        Ok(Some(graph)) => graph,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // Find the target function
    let target_function = if let Some(func_name) = &request.function_name {
        // Query specific function by name
        let matching_functions = graph.find_functions_by_name(func_name);
        if matching_functions.is_empty() {
            return Err(StatusCode::NOT_FOUND);
        }
        // For now, take the first matching function
        // In a real implementation, you might want to handle multiple matches
        matching_functions[0]
    } else {
        // Query all functions in the specified file and take the first one
        let file_path = std::path::PathBuf::from(&request.filepath);
        let file_functions = graph.find_functions_by_file(&file_path);
        if file_functions.is_empty() {
            return Err(StatusCode::NOT_FOUND);
        }
        file_functions[0]
    };
    
    // Read the file contents
    let file_contents = match std::fs::read_to_string(&target_function.file_path) {
        Ok(contents) => contents,
        Err(e) => {
            tracing::error!("Failed to read file {}: {}", target_function.file_path.display(), e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Split file into lines
    let lines: Vec<&str> = file_contents.lines().collect();
    
    // Calculate line range for the snippet
    let context_lines = request.context_lines.unwrap_or(3);
    let include_context = request.include_context.unwrap_or(true);
    
    let (line_start, line_end) = if include_context {
        let start = target_function.line_start.saturating_sub(context_lines);
        let end = (target_function.line_end + context_lines).min(lines.len());
        (start, end)
    } else {
        (target_function.line_start, target_function.line_end)
    };
    
    // Extract the code snippet
    let code_snippet = if line_start < lines.len() && line_end <= lines.len() && line_start < line_end {
        lines[line_start..line_end].join("\n")
    } else {
        // Fallback: return the entire function range
        if target_function.line_start < lines.len() && target_function.line_end <= lines.len() {
            lines[target_function.line_start..target_function.line_end].join("\n")
        } else {
            "// Function not found in file".to_string()
        }
    };
    
    // Determine language from file extension
    let language: String = target_function.file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "rs" => "rust",
            "py" => "python",
            "js" => "javascript",
            "ts" => "typescript",
            "java" => "java",
            "cpp" | "cc" | "cxx" => "cpp",
            "c" => "c",
            "go" => "go",
            "php" => "php",
            "rb" => "ruby",
            "swift" => "swift",
            "kt" => "kotlin",
            "scala" => "scala",
            "cs" => "csharp",
            _ => "unknown"
        })
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    let response = CodeSnippetResponse {
        filepath: target_function.file_path.display().to_string(),
        function_name: Some(target_function.name.clone()),
        code_snippet,
        line_start: target_function.line_start,
        line_end: target_function.line_end,
        language,
    };
    
    Ok(Json(ApiResponse {
        success: true,
        data: response,
    }))
} 

pub async fn query_code_skeleton(
    State(_storage): State<Arc<StorageManager>>,
    Json(request): Json<QueryCodeSkeletonRequest>,
) -> Result<Json<ApiResponse<CodeSkeletonResponse>>, StatusCode> {
    // Read file contents
    let path = std::path::PathBuf::from(&request.filepath);
    let code = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    // Get parser and language
    let (mut parser, language_id) = match crate::codegraph::treesitter::parsers::get_ast_parser_by_filename(&path) {
        Ok(v) => v,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    // Parse and build symbol maps
    let symbols = parser.parse(&code, &path);
    let symbols_struct: Vec<crate::codegraph::treesitter::ast_instance_structs::SymbolInformation> =
        symbols.iter().map(|s| s.read().symbol_info_struct()).collect();

    // Build guid maps similar to tests
    use uuid::Uuid;
    use std::collections::HashMap;
    let guid_to_children: HashMap<Uuid, Vec<Uuid>> = symbols
        .iter()
        .map(|s| (s.read().guid().clone(), s.read().childs_guid().clone()))
        .collect();

    // Build a minimal FileASTMarkup-compatible list
    let ast_markup = crate::codegraph::treesitter::file_ast_markup::FileASTMarkup {
        symbols_sorted_by_path_len: symbols_struct.clone(),
    };
    let guid_to_info: HashMap<Uuid, &crate::codegraph::treesitter::ast_instance_structs::SymbolInformation> =
        ast_markup
            .symbols_sorted_by_path_len
            .iter()
            .map(|s| (s.guid.clone(), s))
            .collect();

    // Make formatter
    let formatter = crate::codegraph::treesitter::skeletonizer::make_formatter(&language_id);

    // Filter top-level struct/class symbols and build skeleton text
    use crate::codegraph::treesitter::structs::SymbolType;
    let class_symbols: Vec<_> = ast_markup
        .symbols_sorted_by_path_len
        .iter()
        .filter(|x| x.symbol_type == SymbolType::StructDeclaration)
        .collect();

    let mut lines: Vec<String> = Vec::new();
    for symbol in class_symbols {
        let skeleton_line = formatter.make_skeleton(&symbol, &code.to_string(), &guid_to_children, &guid_to_info);
        lines.push(skeleton_line);
    }

    let skeleton_text = if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n\n")
    };

    let language = language_id.to_string();

    let response = CodeSkeletonResponse {
        filepath: path.display().to_string(),
        language,
        skeleton_text,
    };

    Ok(Json(ApiResponse {
        success: true,
        data: response,
    }))
} 