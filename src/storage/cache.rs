use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    _created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
}

impl<T> CacheEntry<T> {
    fn new(data: T) -> Self {
        let now = Instant::now();
        Self {
            data,
            _created_at: now,
            last_accessed: now,
            access_count: 1,
        }
    }

    fn access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }
}

pub struct CacheManager {
    cache: Arc<RwLock<HashMap<String, CacheEntry<String>>>>,
    max_size: usize,
    ttl: Duration,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size: 1000,
            ttl: Duration::from_secs(3600), // 1 hour
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.write();
        if let Some(entry) = cache.get_mut(key) {
            if entry.last_accessed.elapsed() < self.ttl {
                entry.access();
                return Some(entry.data.clone());
            } else {
                cache.remove(key);
            }
        }
        None
    }

    pub fn set(&self, key: String, value: String) {
        let mut cache = self.cache.write();
        
        // Check if we need to evict some entries
        if cache.len() >= self.max_size {
            self.evict_lru(&mut cache);
        }
        
        cache.insert(key, CacheEntry::new(value));
    }

    pub fn remove(&self, key: &str) {
        let mut cache = self.cache.write();
        cache.remove(key);
    }

    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
    }

    pub fn size(&self) -> usize {
        self.cache.read().len()
    }

    fn evict_lru(&self, cache: &mut HashMap<String, CacheEntry<String>>) {
        // Simple approach: just remove a few random entries when cache is full
        let to_remove = (cache.len() / 10).max(1);
        let keys: Vec<String> = cache.keys().cloned().take(to_remove).collect();
        
        for key in keys {
            cache.remove(&key);
        }
    }
} 