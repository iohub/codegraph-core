use axum::{
    routing::{get, post},
    Router,
    response::Json,
    http::StatusCode,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use crate::storage::StorageManager;

use super::{
    handlers::{
        search_code,
        get_call_graph,
        get_symbol_info,
        get_dependencies,
        health_check,
    },
    models::{ApiResponse, ApiError},
};

pub struct CodeGraphServer {
    storage: Arc<StorageManager>,
}

impl CodeGraphServer {
    pub fn new(storage: Arc<StorageManager>) -> Self {
        Self { storage }
    }

    pub async fn start(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.create_router();
        
        let listener = TcpListener::bind(addr).await?;
        println!("🚀 CodeGraph HTTP server starting on {}", addr);
        println!("📚 API Documentation: http://{}/v1/docs", addr);
        
        axum::serve(listener, app).await?;
        Ok(())
    }

    fn create_router(self) -> Router {
        // CORS configuration
        let cors = CorsLayer::permissive();

        Router::new()
            // Health check
            .route("/health", get(health_check))
            
            // API v1 routes
            .route("/v1/search/code", get(search_code))
            .route("/v1/analysis/callgraph", get(get_call_graph))
            .route("/v1/symbol/:symbol_name", get(get_symbol_info))
            .route("/v1/analysis/dependencies", get(get_dependencies))
            
            // Legacy routes for backward compatibility
            .route("/build_graph", post(super::handlers::build_graph))
            .route("/query_call_graph", post(super::handlers::query_call_graph))
            .route("/query_code_snippet", post(super::handlers::query_code_snippet))
            
            .layer(cors)
            .with_state(self.storage)
            .fallback(not_found_handler)
    }
}

// 404 handler for unmatched routes
async fn not_found_handler() -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiError {
            code: "ROUTE_NOT_FOUND".to_string(),
            message: "The requested endpoint does not exist".to_string(),
        })
    )
} 