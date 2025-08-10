pub mod cache;
pub mod persistence;
pub mod incremental;

pub use cache::CacheManager;
pub use persistence::PersistenceManager;
pub use incremental::IncrementalManager;

use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use crate::codegraph::types::PetCodeGraph;

pub struct StorageManager {
    cache: Arc<CacheManager>,
    persistence: Arc<PersistenceManager>,
    incremental: Arc<IncrementalManager>,
    graphs: Arc<RwLock<HashMap<String, PetCodeGraph>>>,
}

impl StorageManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(CacheManager::new()),
            persistence: Arc::new(PersistenceManager::new()),
            incremental: Arc::new(IncrementalManager::new()),
            graphs: Arc::new(RwLock::new(HashMap::new())),
        }
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