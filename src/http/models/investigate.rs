use serde::{Deserialize, Serialize};

use super::{CallRelation, CodeSkeletonResponse};

#[derive(Debug, Deserialize)]
pub struct InvestigateRepoRequest {
    pub project_dir: String,
}

#[derive(Debug, Serialize)]
pub struct InvestigateFunctionInfo {
    pub name: String,
    pub file_path: String,
    pub out_degree: usize,
    pub callers: Vec<CallRelation>,
    pub callees: Vec<CallRelation>,
}

#[derive(Debug, Serialize)]
pub struct DirectoryTreeNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<DirectoryTreeNode>>,
}

#[derive(Debug, Serialize)]
pub struct InvestigateRepoResponse {
    pub project_id: String,
    pub total_functions: usize,
    pub core_functions: Vec<InvestigateFunctionInfo>,
    pub file_skeletons: Vec<CodeSkeletonResponse>,
    pub directory_tree: Vec<DirectoryTreeNode>,
} 