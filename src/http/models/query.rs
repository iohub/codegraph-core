use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct QueryCallGraphRequest {
    pub filepath: String,
    pub function_name: Option<String>,
    pub max_depth: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct FunctionInfo {
    pub id: String,
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub callers: Vec<CallRelation>,
    pub callees: Vec<CallRelation>,
}

#[derive(Debug, Serialize)]
pub struct CallRelation {
    pub function_name: String,
    pub file_path: String,
    pub line_number: usize,
}

#[derive(Debug, Serialize)]
pub struct QueryCallGraphResponse {
    pub filepath: String,
    pub functions: Vec<FunctionInfo>,
} 