pub mod persistence;
pub mod incremental;
pub mod petgraph_storage;
pub mod traits;
pub mod prelude;

pub use persistence::PersistenceManager;
pub use incremental::IncrementalManager;
pub use petgraph_storage::{PetGraphStorage, PetGraphStorageManager};
pub use traits::{GraphPersistence, IncrementalUpdater, GraphSerializer};

use std::sync::Arc;
use parking_lot::RwLock;
use crate::codegraph::types::PetCodeGraph;
use crate::cli::args::StorageMode;

pub struct StorageManager {
    persistence: Arc<PersistenceManager>,
    incremental: Arc<IncrementalManager>,
    graph: Arc<RwLock<Option<PetCodeGraph>>>,
    storage_mode: StorageMode,
}

impl StorageManager {
    pub fn new() -> Self {
        Self::with_storage_mode(StorageMode::Json)
    }

    pub fn with_storage_mode(storage_mode: StorageMode) -> Self {
        Self {
            persistence: Arc::new(PersistenceManager::with_storage_mode(storage_mode.clone())),
            incremental: Arc::new(IncrementalManager::new()),
            graph: Arc::new(RwLock::new(None)),
            storage_mode,
        }
    }

    pub fn set_storage_mode(&mut self, storage_mode: StorageMode) {
        self.storage_mode = storage_mode.clone();
        // Update persistence manager's storage mode
        Arc::get_mut(&mut self.persistence)
            .unwrap()
            .set_storage_mode(storage_mode);
    }

    pub fn get_storage_mode(&self) -> &StorageMode {
        &self.storage_mode
    }

    pub fn get_persistence(&self) -> Arc<PersistenceManager> {
        self.persistence.clone()
    }

    pub fn get_incremental(&self) -> Arc<IncrementalManager> {
        self.incremental.clone()
    }

    pub fn get_graph(&self) -> Arc<RwLock<Option<PetCodeGraph>>> {
        self.graph.clone()
    }

    pub fn set_graph(&self, graph: PetCodeGraph) {
        *self.graph.write() = Some(graph);
    }

    pub fn get_graph_clone(&self) -> Option<PetCodeGraph> {
        self.graph.read().clone()
    }
} 