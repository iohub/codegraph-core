use axum::{
    extract::{State, Query},
    response::{Json, Html},
    http::StatusCode,
};
use std::sync::Arc;
use crate::storage::StorageManager;
use crate::services::CodeAnalyzer;
use super::models::*;
use md5;
use uuid;
use serde_json::json;

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
    let _existing_graph: crate::codegraph::PetCodeGraph = if let Ok(Some(existing_graph)) = storage.get_persistence().load_graph(&project_id) {
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
    let max_depth = request.max_depth.unwrap_or(2); // Default max depth is 2
    
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
    
    for (related_func, relation) in relations {
        // Check if we already have this function in our list
        let existing_function = functions.iter_mut().find(|f| f.id == related_func.id.to_string());
        
        if let Some(existing_function) = existing_function {
            // Update existing function with new relations
            if is_caller {
                // Add caller relation
                let caller_relation = super::models::CallRelation {
                    function_name: related_func.name.clone(),
                    file_path: related_func.file_path.display().to_string(),
                    line_number: relation.line_number,
                };
                
                if !existing_function.callers.iter().any(|c| c.function_name == caller_relation.function_name) {
                    existing_function.callers.push(caller_relation);
                }
            } else {
                // Add callee relation
                let callee_relation = super::models::CallRelation {
                    function_name: related_func.name.clone(),
                    file_path: related_func.file_path.display().to_string(),
                    line_number: relation.line_number,
                };
                
                if !existing_function.callees.iter().any(|c| c.function_name == callee_relation.function_name) {
                    existing_function.callees.push(callee_relation);
                }
            }
        } else {
            // Create new function entry
            let mut new_function = super::models::FunctionInfo {
                id: related_func.id.to_string(),
                name: related_func.name.clone(),
                line_start: related_func.line_start,
                line_end: related_func.line_end,
                callers: Vec::new(),
                callees: Vec::new(),
            };
            
            if is_caller {
                // Add caller relation
                new_function.callers.push(super::models::CallRelation {
                    function_name: related_func.name.clone(),
                    file_path: related_func.file_path.display().to_string(),
                    line_number: relation.line_number,
                });
            } else {
                // Add callee relation
                new_function.callees.push(super::models::CallRelation {
                    function_name: related_func.name.clone(),
                    file_path: related_func.file_path.display().to_string(),
                    line_number: relation.line_number,
                });
            }
            
            functions.push(new_function);
        }
        
        // Recursively expand this function's relations
        expand_call_chain(graph, &related_func.id.to_string(), visited, functions, depth - 1, is_caller);
    }
}

/// New handler for hierarchical tree structure output
pub async fn query_hierarchical_graph(
    State(storage): State<Arc<StorageManager>>,
    Json(request): Json<super::models::QueryHierarchicalGraphRequest>,
) -> Result<Json<ApiResponse<super::models::QueryHierarchicalGraphResponse>>, StatusCode> {
    let max_depth = request.max_depth.unwrap_or(2); // Default max depth is 2
    let include_file_info = request.include_file_info.unwrap_or(true);
    
    // Try to find the project ID
    let project_id = if let Some(pid) = request.project_id {
        pid
    } else if let Ok(projects) = storage.get_persistence().list_projects() {
        // Use the first available project if none specified
        projects.first().cloned().ok_or(StatusCode::NOT_FOUND)?
    } else {
        return Err(StatusCode::NOT_FOUND);
    };
    
    // Load the code graph for the project
    let graph = match storage.get_persistence().load_graph(&project_id) {
        Ok(Some(graph)) => graph,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    let stats = graph.get_stats();
    let total_functions = stats.total_functions;
    let total_relations = stats.resolved_calls + stats.unresolved_calls;
    
    // Build hierarchical tree structure
    let tree_structure = if let Some(root_func_name) = &request.root_function {
        // Start from specific function
        build_hierarchical_tree_from_function(&graph, root_func_name, max_depth, include_file_info)
            .unwrap_or_else(|| create_default_tree_structure(&graph, include_file_info))
    } else {
        // Create default tree structure starting from main functions
        create_default_tree_structure(&graph, include_file_info)
    };
    
    let response = super::models::QueryHierarchicalGraphResponse {
        project_id,
        root_function: request.root_function.clone(),
        max_depth,
        tree_structure,
        total_functions,
        total_relations,
    };
    
    Ok(Json(ApiResponse {
        success: true,
        data: response,
    }))
}

/// Helper function to build hierarchical tree starting from a specific function
fn build_hierarchical_tree_from_function(
    graph: &crate::codegraph::types::PetCodeGraph,
    function_name: &str,
    max_depth: usize,
    include_file_info: bool,
) -> Option<super::models::HierarchicalNode> {
    // Find the function by name
    let functions = graph.find_functions_by_name(function_name);
    if functions.is_empty() {
        return None;
    }
    
    let root_function = &functions[0]; // Use the first match
    
    let mut visited = std::collections::HashSet::new();
    Some(build_hierarchical_node(
        graph,
        root_function,
        max_depth,
        0,
        &mut visited,
        include_file_info,
    ))
}

/// Helper function to create default tree structure
fn create_default_tree_structure(
    graph: &crate::codegraph::types::PetCodeGraph,
    _include_file_info: bool,
) -> super::models::HierarchicalNode {
    let _stats = graph.get_stats();
    
    // Create a root node that contains all functions
    let mut root_node = super::models::HierarchicalNode {
        name: "Project Functions".to_string(),
        function_id: None,
        file_path: None,
        line_start: None,
        line_end: None,
        children: Vec::new(),
        call_type: None,
    };
    
    // Group functions by file for better organization
    let mut file_groups: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
    
    for function in graph.get_all_functions() {
        let file_path = function.file_path.display().to_string();
        file_groups.entry(file_path).or_insert_with(Vec::new).push(function);
    }
    
    // Create file-level nodes
    for (file_path, functions) in file_groups {
        let mut file_node = super::models::HierarchicalNode {
            name: format!("📁 {}", std::path::Path::new(&file_path).file_name().unwrap_or_default().to_string_lossy()),
            function_id: None,
            file_path: Some(file_path.clone()),
            line_start: None,
            line_end: None,
            children: Vec::new(),
            call_type: None,
        };
        
        // Add functions to file node
        for function in functions {
            let function_node = super::models::HierarchicalNode {
                name: function.name.clone(),
                function_id: Some(function.id.to_string()),
                file_path: Some(function.file_path.display().to_string()),
                line_start: Some(function.line_start),
                line_end: Some(function.line_end),
                children: Vec::new(),
                call_type: Some("function".to_string()),
            };
            
            file_node.children.push(function_node);
        }
        
        root_node.children.push(file_node);
    }
    
    root_node
}

/// Recursive function to build hierarchical node structure
fn build_hierarchical_node(
    graph: &crate::codegraph::types::PetCodeGraph,
    function: &crate::codegraph::types::FunctionInfo,
    max_depth: usize,
    current_depth: usize,
    visited: &mut std::collections::HashSet<String>,
    include_file_info: bool,
) -> super::models::HierarchicalNode {
    if current_depth >= max_depth || visited.contains(&function.id.to_string()) {
        return super::models::HierarchicalNode {
            name: format!("{} (max depth reached)", function.name),
            function_id: Some(function.id.to_string()),
            file_path: if include_file_info { Some(function.file_path.display().to_string()) } else { None },
            line_start: if include_file_info { Some(function.line_start) } else { None },
            line_end: if include_file_info { Some(function.line_end) } else { None },
            children: Vec::new(),
            call_type: Some("max_depth".to_string()),
        };
    }
    
    visited.insert(function.id.to_string());
    
    // Get callees (functions called by this function)
    let callees = graph.get_callees(&function.id);
    
    let mut children = Vec::new();
    
    for (callee_func, _relation) in callees {
        let child_node = build_hierarchical_node(
            graph,
            callee_func,
            max_depth,
            current_depth + 1,
            visited,
            include_file_info,
        );
        children.push(child_node);
    }
    
    super::models::HierarchicalNode {
        name: function.name.clone(),
        function_id: Some(function.id.to_string()),
        file_path: if include_file_info { Some(function.file_path.display().to_string()) } else { None },
        line_start: if include_file_info { Some(function.line_start) } else { None },
        line_end: if include_file_info { Some(function.line_end) } else { None },
        children,
        call_type: Some("function".to_string()),
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
) -> Result<Json<ApiResponse<CodeSkeletonBatchResponse>>, StatusCode> {
    let mut skeletons = Vec::new();

    for filepath in &request.filepaths {
        // Read file contents
        let path = std::path::PathBuf::from(filepath);
        let code = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => {
                // Skip files that can't be read, but continue processing others
                tracing::warn!("Failed to read file: {}", filepath);
                continue;
            }
        };

        // Get parser and language
        let (mut parser, language_id) = match crate::codegraph::treesitter::parsers::get_ast_parser_by_filename(&path) {
            Ok(v) => v,
            Err(_) => {
                // Skip files that can't be parsed, but continue processing others
                tracing::warn!("Failed to get parser for file: {}", filepath);
                continue;
            }
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

        // Filter top-level struct/class and function symbols and build skeleton text
        use crate::codegraph::treesitter::structs::SymbolType;
        let class_symbols: Vec<_> = ast_markup
            .symbols_sorted_by_path_len
            .iter()
            .filter(|x| x.symbol_type == SymbolType::StructDeclaration || x.symbol_type == SymbolType::FunctionDeclaration)
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

        let skeleton_response = CodeSkeletonResponse {
            filepath: path.display().to_string(),
            language,
            skeleton_text,
        };

        skeletons.push(skeleton_response);
    }

    let response = CodeSkeletonBatchResponse {
        skeletons,
    };

    Ok(Json(ApiResponse {
        success: true,
        data: response,
    }))
} 

pub async fn draw_call_graph(
    State(storage): State<Arc<StorageManager>>,
    Query(query): Query<super::models::DrawCallGraphQuery>,
) -> Result<Html<String>, StatusCode> {
    // Check if we have the required parameters
    if query.filepath.is_empty() {
        return Ok(Html(generate_main_page_html()));
    }
    
    // First, get the call graph data using existing logic
    let call_graph_request = super::models::QueryCallGraphRequest {
        filepath: query.filepath.clone(),
        function_name: query.function_name.clone(),
        max_depth: query.max_depth,
    };
    
    match query_call_graph(State(storage.clone()), Json(call_graph_request)).await {
        Ok(resp) => {
            let call_graph_data = resp.0.data;
            let html_content = generate_echarts_call_graph_html(&call_graph_data);
            Ok(Html(html_content))
        }
        Err(status) => {
            let html = generate_error_page_html(
                &query.filepath,
                query.function_name.as_deref().unwrap_or(""),
                status,
            );
            Ok(Html(html))
        }
    }
}

fn generate_error_page_html(filepath: &str, function_name: &str, status: axum::http::StatusCode) -> String {
    let title = "Function Call Graph - Error";
    let status_text = format!("{} {}", status.as_u16(), status.canonical_reason().unwrap_or("Error"));
    let suggestion = if status == axum::http::StatusCode::NOT_FOUND {
        "Graph data not found. Make sure you have built the project graph first via POST /build_graph (with JSON {\"project_dir\": \"/path/to/project\"}). Also verify the filepath and function name exist."
            .to_string()
    } else {
        "An error occurred while generating the call graph. Please check server logs.".to_string()
    };

    let mut html = include_str!("templates/error_page.html").to_string();
    html = html.replace("__TITLE__", title);
    html = html.replace("__STATUS_TEXT__", &status_text);
    html = html.replace("__SUGGESTION__", &suggestion);
    html = html.replace("__FILEPATH__", filepath);
    html = html.replace("__FUNCTION_NAME__", function_name);
    html
}

// 新增：处理根路径的主页
pub async fn draw_call_graph_home() -> Html<String> {
    Html(generate_main_page_html())
}

fn generate_main_page_html() -> String {
    include_str!("templates/main_page.html").to_string()
}


fn generate_echarts_call_graph_html(call_graph_data: &super::models::QueryCallGraphResponse) -> String {
    // Prepare nodes with names and metadata (use function name for link resolution)
    let mut nodes: Vec<serde_json::Value> = Vec::new();
    let mut name_set: std::collections::HashSet<String> = std::collections::HashSet::new();

    for function in &call_graph_data.functions {
        name_set.insert(function.name.clone());
        nodes.push(json!({
            "id": function.name,
            "name": function.name,
            "file_path": call_graph_data.filepath,
            "line_start": function.line_start,
            "line_end": function.line_end
        }));
    }

    // Build links using function names (ECharts allows source/target by name)
    let mut links: Vec<serde_json::Value> = Vec::new();
    for function in &call_graph_data.functions {
        // callees: function -> callee
        for callee in &function.callees {
            if name_set.contains(&callee.function_name) {
                links.push(json!({
                    "source": function.name,
                    "target": callee.function_name,
                    "type": "calls"
                }));
            }
        }
        // callers: caller -> function
        for caller in &function.callers {
            if name_set.contains(&caller.function_name) {
                links.push(json!({
                    "source": caller.function_name,
                    "target": function.name,
                    "type": "called_by"
                }));
            }
        }
    }

    let graph_data = json!({
        "nodes": nodes,
        "links": links
    });

    // Load template and replace placeholders
    let mut html = include_str!("templates/echarts_call_graph.html").to_string();
    html = html.replace("__FILEPATH_INPUT__", &call_graph_data.filepath);
    let fn_input = call_graph_data
        .functions
        .first()
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "All functions".to_string());
    html = html.replace("__FUNCTION_NAME_INPUT__", &fn_input);
    html = html.replace("__GRAPH_JSON__", &serde_json::to_string(&graph_data).unwrap());

    html
} 