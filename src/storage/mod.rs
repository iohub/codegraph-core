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
use std::sync::Mutex;
use std::collections::{HashMap, HashSet};
use notify::RecommendedWatcher;
use parking_lot::RwLock;
use crate::codegraph::types::PetCodeGraph;
use crate::cli::args::StorageMode;
use crate::config::Config;

pub struct StorageManager {
    persistence: Arc<PersistenceManager>,
    incremental: Arc<IncrementalManager>,
    graph: Arc<RwLock<Option<PetCodeGraph>>>,
    storage_mode: StorageMode,
    watchers: Arc<Mutex<HashMap<String, RecommendedWatcher>>>,
    pub vector_tasks: Arc<Mutex<HashSet<String>>>,
    pub config: Arc<RwLock<Option<Config>>>,
}

impl StorageManager {
    pub fn new() -> Self {
        Self::with_storage_mode(StorageMode::Json)
    }

    pub fn with_storage_mode(storage_mode: StorageMode) -> Self {
        let base_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(".codegraph_db");

        Self {
            persistence: Arc::new(PersistenceManager::with_storage_mode(storage_mode.clone(), base_dir)),
            incremental: Arc::new(IncrementalManager::new()),
            graph: Arc::new(RwLock::new(None)),
            storage_mode,
            watchers: Arc::new(Mutex::new(HashMap::new())),
            vector_tasks: Arc::new(Mutex::new(HashSet::new())),
            config: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_config(storage_mode: StorageMode, config: Config) -> Self {
        let base_dir = std::path::PathBuf::from(&config.codegraph.graph_db_uri);

        Self {
            persistence: Arc::new(PersistenceManager::with_storage_mode(storage_mode.clone(), base_dir)),
            incremental: Arc::new(IncrementalManager::new()),
            graph: Arc::new(RwLock::new(None)),
            storage_mode,
            watchers: Arc::new(Mutex::new(HashMap::new())),
            vector_tasks: Arc::new(Mutex::new(HashSet::new())),
            config: Arc::new(RwLock::new(Some(config))),
        }
    }

    pub fn set_config(&self, config: Config) {
        *self.config.write() = Some(config);
    }

    pub fn get_config(&self) -> Option<Config> {
        self.config.read().clone()
    }

    pub fn add_watcher(&self, project_id: String, watcher: RecommendedWatcher) {
        self.watchers.lock().unwrap().insert(project_id, watcher);
    }

    pub fn has_watcher(&self, project_id: &str) -> bool {
        self.watchers.lock().unwrap().contains_key(project_id)
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