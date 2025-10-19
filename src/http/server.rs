use axum::{
    routing::{post, get},
    Router,
    response::Json,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use crate::storage::StorageManager;

use super::{
    handlers::{build_graph, query_call_graph, query_code_snippet, query_code_skeleton, query_hierarchical_graph, draw_call_graph, draw_call_graph_home, init, investigate_repo},
    models::ApiResponse,
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
        
        axum::serve(listener, app).await?;
        Ok(())
    }

    fn create_router(self) -> Router {
        // CORS configuration
        let cors = CorsLayer::permissive();

        Router::new()
            .route("/health", get(health_check))
            .route("/init", post(init))
            .route("/build_graph", post(build_graph))
            .route("/query_call_graph", post(query_call_graph))
            .route("/query_code_snippet", post(query_code_snippet))
            .route("/query_code_skeleton", post(query_code_skeleton))
            .route("/query_hierarchical_graph", post(query_hierarchical_graph))
            .route("/investigate_repo", post(investigate_repo))
            .route("/", get(draw_call_graph_home))
            .route("/draw_call_graph", get(draw_call_graph))
            .layer(cors)
            .with_state(self.storage)
    }
}

// Health check endpoint
async fn health_check() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse {
        success: true,
        data: "CodeGraph HTTP service is running",
    })
} 