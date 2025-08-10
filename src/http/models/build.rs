use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct BuildGraphRequest {
    pub project_dir: String,
    pub force_rebuild: Option<bool>,
    pub exclude_patterns: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct BuildGraphResponse {
    pub project_id: String,
    pub total_files: usize,
    pub total_functions: usize,
    pub build_time_ms: u64,
    pub cache_hit_rate: f64,
} 