pub mod cache;
pub mod persistence;
pub mod incremental;
pub mod petgraph_storage;

pub use cache::CacheManager;
pub use persistence::PersistenceManager;
pub use incremental::IncrementalManager;
pub use petgraph_storage::{PetGraphStorage, PetGraphStorageManager};

use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use crate::codegraph::types::PetCodeGraph;
use crate::cli::args::StorageMode;

pub struct StorageManager {
    cache: Arc<CacheManager>,
    persistence: Arc<PersistenceManager>,
    incremental: Arc<IncrementalManager>,
    graphs: Arc<RwLock<HashMap<String, PetCodeGraph>>>,
    storage_mode: StorageMode,
}

impl StorageManager {
    pub fn new() -> Self {
        Self::with_storage_mode(StorageMode::Json)
    }

    pub fn with_storage_mode(storage_mode: StorageMode) -> Self {
        Self {
            cache: Arc::new(CacheManager::new()),
            persistence: Arc::new(PersistenceManager::with_storage_mode(storage_mode.clone())),
            incremental: Arc::new(IncrementalManager::new()),
            graphs: Arc::new(RwLock::new(HashMap::new())),
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

    pub fn get_cache(&self) -> Arc<CacheManager> {
        self.cache.clone()
    }

    pub fn get_persistence(&self) -> Arc<PersistenceManager> {
        self.persistence.clone()
    }

    pub fn get_incremental(&self) -> Arc<IncrementalManager> {
        self.incremental.clone()
    }

    pub fn get_graphs(&self) -> Arc<RwLock<HashMap<String, PetCodeGraph>>> {
        self.graphs.clone()
    }
} 