use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use std::sync::Arc;
use crate::storage::StorageManager;
use crate::codegraph::CodeGraphAnalyzer;
use super::models::*;

pub async fn build_graph(
    State(storage): State<Arc<StorageManager>>,
    Json(request): Json<BuildGraphRequest>,
) -> Result<Json<ApiResponse<BuildGraphResponse>>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    // Generate project ID
    let project_id = uuid::Uuid::new_v4().to_string();
    
    // Get project directory path
    let project_dir = std::path::Path::new(&request.project_dir);
    if !project_dir.exists() || !project_dir.is_dir() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Check for existing graph
    let mut cache_hit_rate = 0.0;
    let _existing_graph: crate::codegraph::PetCodeGraph = if let Ok(Some(existing_graph)) = storage.get_persistence().load_graph(&project_id) {
        cache_hit_rate = 0.8; // Assume 80% cache hit for existing graph
        existing_graph
    } else {
        // Create new graph using CodeGraphAnalyzer
        let mut analyzer = CodeGraphAnalyzer::new();
        
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
    
    // Get exclude patterns
    let exclude_patterns = request.exclude_patterns.unwrap_or_else(|| {
        vec![
            "node_modules".to_string(),
            ".venv".to_string(),
            "__pycache__".to_string(),
            "target".to_string(),
        ]
    });
    
    // Get changed files for incremental update (currently not used but kept for future incremental processing)
    let _changed_files = storage.get_incremental().get_changed_files(
        project_dir,
        &exclude_patterns,
    );
    
    // Use CodeGraphAnalyzer to build the actual graph
    let mut analyzer = CodeGraphAnalyzer::new();
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
    State(_storage): State<Arc<StorageManager>>,
    Json(request): Json<QueryCodeSnippetRequest>,
) -> Result<Json<ApiResponse<CodeSnippetResponse>>, StatusCode> {
    // TODO: Implement query code snippet logic
    let response = CodeSnippetResponse {
        filepath: request.filepath,
        function_name: request.function_name,
        code_snippet: "// TODO: Implement".to_string(),
        line_start: 0,
        line_end: 0,
        language: "unknown".to_string(),
    };
    
    Ok(Json(ApiResponse {
        success: true,
        data: response,
    }))
} 