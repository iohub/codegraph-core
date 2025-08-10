use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use std::sync::Arc;
use crate::storage::StorageManager;
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
    let graph = if let Ok(Some(existing_graph)) = storage.get_persistence().load_graph(&project_id) {
        cache_hit_rate = 0.8; // Assume 80% cache hit for existing graph
        existing_graph
    } else {
        // Create new graph
        crate::codegraph::types::PetCodeGraph::new()
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
    
    // Get changed files for incremental update
    let changed_files = storage.get_incremental().get_changed_files(
        project_dir,
        &exclude_patterns,
    );
    
    // TODO: Implement actual code parsing and graph building
    // For now, return mock data
    let total_files = changed_files.len();
    let total_functions = total_files * 2; // Mock: assume 2 functions per file
    
    let build_time_ms = start_time.elapsed().as_millis() as u64;
    
    // Save graph and file hashes
    if let Err(_) = storage.get_persistence().save_graph(&project_id, &graph) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
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
    State(_storage): State<Arc<StorageManager>>,
    Json(request): Json<QueryCallGraphRequest>,
) -> Result<Json<ApiResponse<QueryCallGraphResponse>>, StatusCode> {
    // TODO: Implement query call graph logic
    let response = QueryCallGraphResponse {
        filepath: request.filepath,
        functions: vec![],
    };
    
    Ok(Json(ApiResponse {
        success: true,
        data: response,
    }))
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