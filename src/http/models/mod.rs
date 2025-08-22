pub mod build;
pub mod query;
pub mod snippet;

pub use build::*;
pub use query::*;
pub use snippet::*;

use serde::{Deserialize, Serialize};

// Legacy response format for backward compatibility
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

// New standardized error format according to API documentation
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

// New standardized success response format
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSuccess<T> {
    pub data: T,
}

// Search code response models
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeSearchResponse {
    pub total_count: usize,
    pub items: Vec<CodeSnippet>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeSnippet {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub code_snippet: String,
    pub language: String,
    pub r#type: String,
    pub line_number: usize,
}

// Call graph response models
#[derive(Debug, Serialize, Deserialize)]
pub struct CallGraphResponse {
    pub function: String,
    pub direction: String,
    pub depth: usize,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub graph_definition: Option<GraphDefinition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphDefinition {
    pub mermaid: String,
}

// Symbol info response models
#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolInfoResponse {
    pub symbol: String,
    pub definition: SymbolDefinition,
    pub references: Vec<SymbolReference>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolDefinition {
    pub file_path: String,
    pub line_number: usize,
    pub code_snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolReference {
    pub file_path: String,
    pub line_number: usize,
    pub context: String,
}

// Dependencies response models
#[derive(Debug, Serialize, Deserialize)]
pub struct DependenciesResponse {
    pub r#type: String,
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyNode {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
}

// Query parameters for search
#[derive(Debug, Deserialize)]
pub struct CodeSearchQuery {
    pub q: String,
    pub r#type: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub fuzzy: Option<bool>,
}

// Query parameters for call graph
#[derive(Debug, Deserialize)]
pub struct CallGraphQuery {
    pub function: String,
    pub depth: Option<usize>,
    pub direction: Option<String>,
}

// Query parameters for dependencies
#[derive(Debug, Deserialize)]
pub struct DependenciesQuery {
    pub r#type: Option<String>,
} 