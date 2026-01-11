use axum::{
    extract::State,
    Json,
    http::StatusCode as AxumStatusCode,
};
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use crate::storage::StorageManager;
use crate::services::vectorize_service::VectorizeService;
use crate::http::models::{ApiResponse, BuildEmbeddingRequest, BuildEmbeddingResponse, SemanticSearchRequest, SemanticSearchResponse};

struct TaskGuard {
    tasks: Arc<Mutex<HashSet<String>>>,
    repo_path: String,
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.remove(&self.repo_path);
    }
}

pub async fn build_embedding_index(
    State(storage): State<Arc<StorageManager>>,
    Json(request): Json<BuildEmbeddingRequest>,
) -> Result<Json<ApiResponse<BuildEmbeddingResponse>>, AxumStatusCode> {
    let repo_path = request.repo_path.clone();
    
    // Check and set lock
    {
        let mut tasks = storage.vector_tasks.lock().unwrap();
        if tasks.contains(&repo_path) {
            return Ok(Json(ApiResponse {
                success: false,
                data: BuildEmbeddingResponse { message: "Task already running for this repo".to_string() }
            }));
        }
        tasks.insert(repo_path.clone());
    }
    
    // RAII guard to remove from set on return/panic
    let _guard = TaskGuard {
        tasks: storage.vector_tasks.clone(),
        repo_path: repo_path.clone(),
    };
    
    // Get config
    let config = storage.get_config().ok_or(AxumStatusCode::INTERNAL_SERVER_ERROR)?;
    let db_path = config.codegraph.db_uri.clone();
    let collection = config.codegraph.collection.clone();
    
    // Create service and run vectorization
    let service = VectorizeService::new(&db_path, collection, Some(&config)).await
        .map_err(|e| {
            tracing::error!("Failed to create vectorize service: {}", e);
            AxumStatusCode::INTERNAL_SERVER_ERROR
        })?;
        
    // Ensure collection exists
    service.ensure_collection().await
        .map_err(|e| {
            tracing::error!("Failed to ensure collection: {}", e);
            AxumStatusCode::INTERNAL_SERVER_ERROR
        })?;
        
    // Vectorize directory
    service.vectorize_directory(&repo_path).await
        .map_err(|e| {
            tracing::error!("Vectorization failed: {}", e);
            AxumStatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(ApiResponse {
        success: true,
        data: BuildEmbeddingResponse { message: "Index built successfully".to_string() }
    }))
}

pub async fn semantic_search(
    State(storage): State<Arc<StorageManager>>,
    Json(request): Json<SemanticSearchRequest>,
) -> Result<Json<ApiResponse<SemanticSearchResponse>>, AxumStatusCode> {
    // Get config
    let config = storage.get_config().ok_or(AxumStatusCode::INTERNAL_SERVER_ERROR)?;
    let db_path = config.codegraph.db_uri.clone();
    let collection = config.codegraph.collection.clone();
    
    // Create service
    let service = VectorizeService::new(&db_path, collection, Some(&config)).await
        .map_err(|e| {
            tracing::error!("Failed to create vectorize service: {}", e);
            AxumStatusCode::INTERNAL_SERVER_ERROR
        })?;
        
    let limit = request.limit.unwrap_or(10);
    
    // Search
    let results = service.search(&request.text, limit).await
        .map_err(|e| {
            tracing::error!("Search failed: {}", e);
            AxumStatusCode::INTERNAL_SERVER_ERROR
        })?;
        
    Ok(Json(ApiResponse {
        success: true,
        data: SemanticSearchResponse { results }
    }))
}
