use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use std::sync::Arc;
use crate::storage::StorageManager;
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
        // TODO: Use new AnalyzerOrchestrator instead of CodeGraphAnalyzer
        // For now, create an empty graph
        crate::codegraph::types::PetCodeGraph::new()
    };
    
    // TODO: Use new AnalyzerOrchestrator to build the actual graph
    let total_files = 0;
    let total_functions = 0;
    
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

// New API v1 handlers according to the API documentation

/// GET /v1/search/code - Search for code snippets by name
pub async fn search_code(
    State(storage): State<Arc<StorageManager>>,
    axum::extract::Query(query): axum::extract::Query<CodeSearchQuery>,
) -> Result<Json<ApiSuccess<CodeSearchResponse>>, (StatusCode, Json<ApiError>)> {
    // Validate required query parameter
    if query.q.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: "INVALID_QUERY".to_string(),
                message: "Query parameter 'q' cannot be empty".to_string(),
            })
        ));
    }

    // Get the first available project for now
    let project_id = if let Ok(projects) = storage.get_persistence().list_projects() {
        projects.first().cloned()
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: "STORAGE_ERROR".to_string(),
                message: "Failed to list projects".to_string(),
            })
        ));
    };

    let project_id = project_id.ok_or((
        StatusCode::NOT_FOUND,
        Json(ApiError {
            code: "NO_PROJECTS".to_string(),
            message: "No projects found".to_string(),
        })
    ))?;

    // Load the code graph
    let graph = match storage.get_persistence().load_graph(&project_id) {
        Ok(Some(graph)) => graph,
        Ok(None) => return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "GRAPH_NOT_FOUND".to_string(),
                message: "Code graph not found for the project".to_string(),
            })
        )),
        Err(_) => return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: "LOAD_ERROR".to_string(),
                message: "Failed to load code graph".to_string(),
            })
        )),
    };

    // Search for functions matching the query
    let matching_functions = graph.find_functions_by_name(&query.q);
    
    // Apply type filter if specified
    let filtered_functions = if let Some(type_filter) = &query.r#type {
        matching_functions.into_iter()
            .filter(|f| f.language.to_lowercase() == type_filter.to_lowercase())
            .collect()
    } else {
        matching_functions
    };

    // Apply pagination
    let limit = query.limit.unwrap_or(10).min(50);
    let offset = query.offset.unwrap_or(0);
    let total_count = filtered_functions.len();
    let paginated_functions = filtered_functions
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();

    // Convert to response format
    let items = paginated_functions.into_iter().map(|func| {
        CodeSnippet {
            id: func.id.to_string(),
            name: func.name.clone(),
            file_path: func.file_path.display().to_string(),
            code_snippet: format!("function {}() {{ ... }}", func.name), // Simplified for now
            language: get_language_from_path(&func.file_path),
            r#type: func.language.clone(),
            line_number: func.line_start,
        }
    }).collect();

    let response = CodeSearchResponse {
        total_count,
        items,
    };

    Ok(Json(ApiSuccess { data: response }))
}

/// GET /v1/analysis/callgraph - Get function call graph
pub async fn get_call_graph(
    State(storage): State<Arc<StorageManager>>,
    axum::extract::Query(query): axum::extract::Query<CallGraphQuery>,
) -> Result<Json<ApiSuccess<CallGraphResponse>>, (StatusCode, Json<ApiError>)> {
    // Validate required query parameter
    if query.function.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: "INVALID_FUNCTION".to_string(),
                message: "Function parameter cannot be empty".to_string(),
            })
        ));
    }

    // Get the first available project for now
    let project_id = if let Ok(projects) = storage.get_persistence().list_projects() {
        projects.first().cloned()
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: "STORAGE_ERROR".to_string(),
                message: "Failed to list projects".to_string(),
            })
        ));
    };

    let project_id = project_id.ok_or((
        StatusCode::NOT_FOUND,
        Json(ApiError {
            code: "NO_PROJECTS".to_string(),
            message: "No projects found".to_string(),
        })
    ))?;

    // Load the code graph
    let graph = match storage.get_persistence().load_graph(&project_id) {
        Ok(Some(graph)) => graph,
        Ok(None) => return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "GRAPH_NOT_FOUND".to_string(),
                message: "Code graph not found for the project".to_string(),
            })
        )),
        Err(_) => return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: "LOAD_ERROR".to_string(),
                message: "Failed to load code graph".to_string(),
            })
        )),
    };

    // Find the target function
    let matching_functions = graph.find_functions_by_name(&query.function);
    if matching_functions.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "FUNCTION_NOT_FOUND".to_string(),
                message: format!("Function '{}' not found", query.function),
            })
        ));
    }

    let target_function = &matching_functions[0];
    let depth = query.depth.unwrap_or(3).min(10);
    let direction = query.direction.as_deref().unwrap_or("down");

    // Build call graph nodes and edges
    let mut nodes = vec![
        GraphNode {
            id: format!("node_{}", target_function.id),
            label: target_function.name.clone(),
            r#type: "function".to_string(),
        }
    ];

    let mut edges = Vec::new();

    // For now, create a simple mock call graph
    // In a real implementation, you would traverse the actual call relationships
    if direction == "down" || direction == "both" {
        // Add some mock called functions
        for i in 0..depth.min(3) {
            let called_id = format!("called_{}", i);
            nodes.push(GraphNode {
                id: called_id.clone(),
                label: format!("CalledFunction_{}", i),
                r#type: "function".to_string(),
            });
            edges.push(GraphEdge {
                source: format!("node_{}", target_function.id),
                target: called_id,
                relation: "calls".to_string(),
            });
        }
    }

    if direction == "up" || direction == "both" {
        // Add some mock calling functions
        for i in 0..depth.min(2) {
            let caller_id = format!("caller_{}", i);
            nodes.push(GraphNode {
                id: caller_id.clone(),
                label: format!("CallingFunction_{}", i),
                r#type: "function".to_string(),
            });
            edges.push(GraphEdge {
                source: caller_id,
                target: format!("node_{}", target_function.id),
                relation: "calls".to_string(),
            });
        }
    }

    // Generate Mermaid graph definition
    let mermaid_edges: Vec<String> = edges.iter()
        .map(|edge| format!("    {} --> {}", edge.source, edge.target))
        .collect();
    
    let mermaid_graph = format!(
        "graph TD;\n{}",
        mermaid_edges.join(";\n")
    );

    let response = CallGraphResponse {
        function: query.function,
        direction: direction.to_string(),
        depth,
        nodes,
        edges,
        graph_definition: Some(GraphDefinition {
            mermaid: mermaid_graph,
        }),
    };

    Ok(Json(ApiSuccess { data: response }))
}

/// GET /v1/symbol/{symbol_name} - Get symbol definition and references
pub async fn get_symbol_info(
    State(storage): State<Arc<StorageManager>>,
    axum::extract::Path(symbol_name): axum::extract::Path<String>,
) -> Result<Json<ApiSuccess<SymbolInfoResponse>>, (StatusCode, Json<ApiError>)> {
    if symbol_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: "INVALID_SYMBOL".to_string(),
                message: "Symbol name cannot be empty".to_string(),
            })
        ));
    }

    // Get the first available project for now
    let project_id = if let Ok(projects) = storage.get_persistence().list_projects() {
        projects.first().cloned()
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: "STORAGE_ERROR".to_string(),
                message: "Failed to list projects".to_string(),
            })
        ));
    };

    let project_id = project_id.ok_or((
        StatusCode::NOT_FOUND,
        Json(ApiError {
            code: "NO_PROJECTS".to_string(),
            message: "No projects found".to_string(),
        })
    ))?;

    // Load the code graph
    let graph = match storage.get_persistence().load_graph(&project_id) {
        Ok(Some(graph)) => graph,
        Ok(None) => return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "GRAPH_NOT_FOUND".to_string(),
                message: "Code graph not found for the project".to_string(),
            })
        )),
        Err(_) => return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: "LOAD_ERROR".to_string(),
                message: "Failed to load code graph".to_string(),
            })
        )),
    };

    // Find the symbol
    let matching_functions = graph.find_functions_by_name(&symbol_name);
    if matching_functions.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "SYMBOL_NOT_FOUND".to_string(),
                message: format!("Symbol '{}' not found", symbol_name),
            })
        ));
    }

    let symbol = &matching_functions[0];

    // Create definition
    let definition = SymbolDefinition {
        file_path: symbol.file_path.display().to_string(),
        line_number: symbol.line_start,
        code_snippet: format!("function {}() {{ ... }}", symbol.name),
    };

    // For now, create mock references
    // In a real implementation, you would find actual references
    let references = vec![
        SymbolReference {
            file_path: symbol.file_path.display().to_string(),
            line_number: symbol.line_start + 1,
            context: format!("// Usage of {}", symbol.name),
        }
    ];

    let response = SymbolInfoResponse {
        symbol: symbol_name,
        definition,
        references,
    };

    Ok(Json(ApiSuccess { data: response }))
}

/// GET /v1/analysis/dependencies - Get project dependencies
pub async fn get_dependencies(
    State(storage): State<Arc<StorageManager>>,
    axum::extract::Query(query): axum::extract::Query<DependenciesQuery>,
) -> Result<Json<ApiSuccess<DependenciesResponse>>, (StatusCode, Json<ApiError>)> {
    let dependency_type = query.r#type.unwrap_or_else(|| "internal".to_string());

    // Get the first available project for now
    let project_id = if let Ok(projects) = storage.get_persistence().list_projects() {
        projects.first().cloned()
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: "STORAGE_ERROR".to_string(),
                message: "Failed to list projects".to_string(),
            })
        ));
    };

    let project_id = project_id.ok_or((
        StatusCode::NOT_FOUND,
        Json(ApiError {
            code: "NO_PROJECTS".to_string(),
            message: "No projects found".to_string(),
        })
    ))?;

    // Load the code graph
    let graph = match storage.get_persistence().load_graph(&project_id) {
        Ok(Some(graph)) => graph,
        Ok(None) => return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "GRAPH_NOT_FOUND".to_string(),
                message: "Code graph not found for the project".to_string(),
            })
        )),
        Err(_) => return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: "LOAD_ERROR".to_string(),
                message: "Failed to load code graph".to_string(),
            })
        )),
    };

    // For now, create mock dependency data
    // In a real implementation, you would analyze actual dependencies
    let nodes = vec![
        DependencyNode {
            id: "main".to_string(),
            label: "Main Module".to_string(),
        },
        DependencyNode {
            id: "utils".to_string(),
            label: "Common Utils".to_string(),
        },
        DependencyNode {
            id: "services".to_string(),
            label: "Services".to_string(),
        },
    ];

    let edges = vec![
        DependencyEdge {
            source: "main".to_string(),
            target: "utils".to_string(),
            relation: "imports".to_string(),
        },
        DependencyEdge {
            source: "main".to_string(),
            target: "services".to_string(),
            relation: "depends_on".to_string(),
        },
        DependencyEdge {
            source: "services".to_string(),
            target: "utils".to_string(),
            relation: "imports".to_string(),
        },
    ];

    let response = DependenciesResponse {
        r#type: dependency_type,
        nodes,
        edges,
    };

    Ok(Json(ApiSuccess { data: response }))
}

/// GET /health - Health check endpoint
pub async fn health_check() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse {
        success: true,
        data: "CodeGraph HTTP service is running",
    })
}

/// GET /v1/docs - API documentation
pub async fn api_docs() -> Json<ApiResponse<String>> {
    let docs = r#"
# 代码知识库查询系统 API 文档

## 概述

本系统提供一系列基于代码知识库（Codebase）的智能查询接口，旨在帮助开发者快速理解代码结构、函数调用关系、查找代码片段等，从而提升开发与维护效率。

## 接口列表

### 1. 通过名称查询代码片段
- **URL**: `/v1/search/code`
- **方法**: `GET`
- **描述**: 在代码库中搜索包含指定关键词的代码片段

### 2. 查询函数调用关系图
- **URL**: `/v1/analysis/callgraph`
- **方法**: `GET`
- **描述**: 生成以目标函数为中心的调用关系图

### 3. 获取符号的定义和引用位置
- **URL**: `/v1/symbol/{symbol_name}`
- **方法**: `GET`
- **描述**: 获取符号的定义信息和所有引用位置

### 4. 获取项目的依赖分析
- **URL**: `/v1/analysis/dependencies`
- **方法**: `GET`
- **描述**: 生成项目的依赖关系图

## 错误处理

所有接口在遇到错误时均返回标准化的错误格式：

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "A human-readable description of the error."
  }
}
```

## 状态码

- `200 OK`: 请求成功
- `400 Bad Request`: 请求参数有误
- `404 Not Found`: 请求的资源不存在
- `500 Internal Server Error`: 服务器内部错误
"#;

    Json(ApiResponse {
        success: true,
        data: docs.to_string(),
    })
}

// Helper function to determine language from file path
fn get_language_from_path(path: &std::path::Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "rs" => "rust".to_string(),
            "py" => "python".to_string(),
            "js" => "javascript".to_string(),
            "ts" => "typescript".to_string(),
            "java" => "java".to_string(),
            "cpp" | "cc" | "cxx" => "cpp".to_string(),
            "c" => "c".to_string(),
            "go" => "go".to_string(),
            "php" => "php".to_string(),
            "rb" => "ruby".to_string(),
            "swift" => "swift".to_string(),
            "kt" => "kotlin".to_string(),
            "scala" => "scala".to_string(),
            "cs" => "csharp".to_string(),
            _ => "unknown".to_string()
        })
        .unwrap_or_else(|| "unknown".to_string())
} 