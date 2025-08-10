use std::path::PathBuf;
use std::fs;
use std::io;
use crate::codegraph::types::PetCodeGraph;

pub struct PersistenceManager {
    base_dir: PathBuf,
}

impl PersistenceManager {
    pub fn new() -> Self {
        let base_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".codegraph_cache");
        
        // Create base directory if it doesn't exist
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).ok();
        }
        
        Self { base_dir }
    }

    pub fn save_graph(&self, project_id: &str, graph: &PetCodeGraph) -> io::Result<()> {
        let project_dir = self.base_dir.join(project_id);
        fs::create_dir_all(&project_dir)?;
        
        let graph_file = project_dir.join("graph.json");
        let json = serde_json::to_string_pretty(graph)?;
        fs::write(graph_file, json)?;
        
        Ok(())
    }

    pub fn load_graph(&self, project_id: &str) -> io::Result<Option<PetCodeGraph>> {
        let graph_file = self.base_dir.join(project_id).join("graph.json");
        
        if !graph_file.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(graph_file)?;
        let graph: PetCodeGraph = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        Ok(Some(graph))
    }

    pub fn save_file_hash(&self, project_id: &str, file_path: &str, hash: &str) -> io::Result<()> {
        let project_dir = self.base_dir.join(project_id);
        fs::create_dir_all(&project_dir)?;
        
        let hash_file = project_dir.join("file_hashes.json");
        let mut hashes: HashMap<String, String> = if hash_file.exists() {
            let content = fs::read_to_string(&hash_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };
        
        hashes.insert(file_path.to_string(), hash.to_string());
        let json = serde_json::to_string_pretty(&hashes)?;
        fs::write(hash_file, json)?;
        
        Ok(())
    }

    pub fn load_file_hashes(&self, project_id: &str) -> io::Result<HashMap<String, String>> {
        let hash_file = self.base_dir.join(project_id).join("file_hashes.json");
        
        if !hash_file.exists() {
            return Ok(HashMap::new());
        }
        
        let content = fs::read_to_string(hash_file)?;
        let hashes: HashMap<String, String> = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        Ok(hashes)
    }

    pub fn delete_project(&self, project_id: &str) -> io::Result<()> {
        let project_dir = self.base_dir.join(project_id);
        if project_dir.exists() {
            fs::remove_dir_all(project_dir)?;
        }
        Ok(())
    }

    pub fn list_projects(&self) -> io::Result<Vec<String>> {
        let mut projects = Vec::new();
        
        if self.base_dir.exists() {
            for entry in fs::read_dir(&self.base_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        projects.push(name.to_string());
                    }
                }
            }
        }
        
        Ok(projects)
    }
}

use std::collections::HashMap; 