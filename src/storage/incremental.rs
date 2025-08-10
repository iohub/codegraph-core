use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

pub struct IncrementalManager {
    file_hashes: HashMap<String, String>,
}

impl IncrementalManager {
    pub fn new() -> Self {
        Self {
            file_hashes: HashMap::new(),
        }
    }

    pub fn calculate_file_hash(&self, file_path: &Path) -> Option<String> {
        if !file_path.exists() || !file_path.is_file() {
            return None;
        }

        fs::read_to_string(file_path)
            .ok()
            .map(|content| {
                let digest = md5::compute(content.as_bytes());
                format!("{:x}", digest)
            })
    }

    pub fn has_file_changed(&self, file_path: &str, current_hash: &str) -> bool {
        self.file_hashes
            .get(file_path)
            .map(|stored_hash| stored_hash != current_hash)
            .unwrap_or(true) // New file or no stored hash
    }

    pub fn update_file_hash(&mut self, file_path: String, hash: String) {
        self.file_hashes.insert(file_path, hash);
    }

    pub fn get_changed_files(
        &self,
        project_dir: &Path,
        exclude_patterns: &[String],
    ) -> Vec<PathBuf> {
        let mut changed_files = Vec::new();
        
        for entry in walkdir::WalkDir::new(project_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
                let file_path = entry.path();
                
                // Skip excluded patterns
                if self.should_exclude_file(file_path, exclude_patterns) {
                    continue;
                }
                
                // Check if file has changed
                if let Some(current_hash) = self.calculate_file_hash(file_path) {
                    let file_path_str = file_path.to_string_lossy();
                    if self.has_file_changed(&file_path_str, &current_hash) {
                        changed_files.push(file_path.to_path_buf());
                    }
                }
            }
        
        changed_files
    }

    fn should_exclude_file(&self, file_path: &Path, exclude_patterns: &[String]) -> bool {
        let file_path_str = file_path.to_string_lossy();
        
        for pattern in exclude_patterns {
            if file_path_str.contains(pattern) {
                return true;
            }
        }
        
        // Default exclusions
        let default_excludes = [
            "node_modules", ".venv", "__pycache__", "target", 
            ".git", ".svn", ".hg", ".DS_Store", "Thumbs.db"
        ];
        
        for exclude in &default_excludes {
            if file_path_str.contains(exclude) {
                return true;
            }
        }
        
        false
    }

    pub fn get_file_hashes(&self) -> &HashMap<String, String> {
        &self.file_hashes
    }

    pub fn set_file_hashes(&mut self, hashes: HashMap<String, String>) {
        self.file_hashes = hashes;
    }

    pub fn clear(&mut self) {
        self.file_hashes.clear();
    }
} 