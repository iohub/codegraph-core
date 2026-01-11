use serde::{Deserialize, Serialize};
use crate::services::vectorize_service::SearchResult;

#[derive(Deserialize)]
pub struct BuildEmbeddingRequest {
    pub repo_path: String,
}

#[derive(Serialize)]
pub struct BuildEmbeddingResponse {
    pub message: String,
}

#[derive(Deserialize)]
pub struct SemanticSearchRequest {
    pub text: String,
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct SemanticSearchResponse {
    pub results: Vec<SearchResult>,
}
