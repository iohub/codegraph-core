pub mod graph;
pub mod parser;
pub mod types;
pub mod petgraph_storage;
pub mod treesitter;
pub mod analyzer;
pub mod snippet_service;
pub mod repository_manager;
pub mod cha;
pub mod cha_simple;

pub use graph::CodeGraph;
pub use types::{
    CallRelation, FunctionInfo, GraphNode, GraphRelation, PetCodeGraph,
    ClassInfo, ClassType, EntityNode, EntityEdge, EntityEdgeType, EntityGraph,
    FileMetadata, FileIndex, SnippetIndex, SnippetInfo
};
pub use petgraph_storage::{PetGraphStorage, PetGraphStorageManager};
pub use treesitter::TreeSitterParser;
pub use analyzer::CodeGraphAnalyzer;
pub use snippet_service::SnippetService;
pub use repository_manager::{RepositoryManager, RepositoryStats, SearchResult};
pub use cha::{
    ClassHierarchyBuilder, ClassHierarchyAnalysis, CallSiteExtractor,
    EnhancedClassInfo, CallSite, CallType
};
pub use cha_simple::{SimpleCHA, SimpleClassInfo, SimpleCallSite};