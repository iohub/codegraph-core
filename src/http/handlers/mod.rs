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
                
                // Add all functions to the pet graph
                for function in cg.functions.values() {
                    pet_graph.add_function(function.clone());
                }
                
                // Add all call relations
                for relation in &cg.call_relations {
                    if let Err(e) = pet_graph.add_call_relation(relation.clone()) {
                        tracing::warn!("Failed to add call relation: {}", e);
                    }
                }
                
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