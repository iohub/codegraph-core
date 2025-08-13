use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct BuildGraphRequest {
    pub project_dir: String,
    pub force_rebuild: Option<bool>,
    pub exclude_patterns: Option<Vec<String>>,
    pub analysis_mode: Option<AnalysisMode>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum AnalysisMode {
    #[serde(rename = "standard")]
    Standard,
    #[serde(rename = "cha")]
    CHA,
    #[serde(rename = "simple_cha")]
    SimpleCHA,
}

impl Default for AnalysisMode {
    fn default() -> Self {
        AnalysisMode::Standard
    }
}

#[derive(Debug, Serialize)]
pub struct BuildGraphResponse {
    pub project_id: String,
    pub total_files: usize,
    pub total_functions: usize,
    pub build_time_ms: u64,
    pub cache_hit_rate: f64,
    pub analysis_mode: String,
} 