use anyhow::Result;
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HashRegistry {
    #[serde(skip)]
    pub hashes: Mutex<HashMap<PathBuf, String>>,
    #[serde(rename = "hashes")]
    serialized_hashes: HashMap<PathBuf, String>,
}

impl HashRegistry {
    pub fn new() -> Self {
        Self {
            hashes: Mutex::new(HashMap::new()),
            serialized_hashes: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        match fs::read_to_string(path) {
            Ok(content) => {
                let registry: HashRegistry = serde_json::from_str(&content)?;
                let hashes_map = registry.serialized_hashes.clone();
                Ok(Self {
                    hashes: Mutex::new(hashes_map),
                    serialized_hashes: registry.serialized_hashes,
                })
            }
            Err(_) => {
                // Return empty registry if file doesn't exist
                Ok(Self::new())
            }
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        // Update serialized_hashes with current state
        let hashes_guard = self.hashes.lock().unwrap();
        let serialized = Self {
            hashes: Mutex::new(HashMap::new()),
            serialized_hashes: hashes_guard.clone(),
        };
        
        let content = serde_json::to_string(&serialized)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn has_hash(&self, path: &Path) -> bool {
        let hashes_guard = self.hashes.lock().unwrap();
        hashes_guard.contains_key(path)
    }

    pub fn get_hash(&self, path: &Path) -> Option<String> {
        let hashes_guard = self.hashes.lock().unwrap();
        hashes_guard.get(path).cloned()
    }

    pub fn set_hash(&mut self, path: PathBuf, hash: String) {
        let mut hashes_guard = self.hashes.lock().unwrap();
        hashes_guard.insert(path, hash);
    }

    pub fn len(&self) -> usize {
        let hashes_guard = self.hashes.lock().unwrap();
        hashes_guard.len()
    }
}

pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Hasher::new();

    let mut buffer = [0; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let hash = hasher.finalize();
    Ok(hash.to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, tempdir};

    #[test]
    fn test_hash_registry_new() {
        let registry = HashRegistry::new();
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_hash_registry_has_hash() {
        let mut registry = HashRegistry::new();
        let path = PathBuf::from("/test/file.txt");
        let hash = "test_hash".to_string();
        
        // Initially should return false
        assert!(!registry.has_hash(&path));
        
        // After setting, should return true
        registry.set_hash(path.clone(), hash);
        assert!(registry.has_hash(&path));
    }

    #[test]
    fn test_hash_registry_get_hash() {
        let mut registry = HashRegistry::new();
        let path = PathBuf::from("/test/file.txt");
        let hash = "test_hash".to_string();
        
        // Initially should return None
        assert_eq!(registry.get_hash(&path), None);
        
        // After setting, should return the hash
        registry.set_hash(path.clone(), hash.clone());
        assert_eq!(registry.get_hash(&path), Some(hash));
    }

    #[test]
    fn test_hash_registry_set_hash() {
        let mut registry = HashRegistry::new();
        let path = PathBuf::from("/test/file.txt");
        let hash = "test_hash".to_string();
        
        // Set hash and verify
        registry.set_hash(path.clone(), hash.clone());
        assert_eq!(registry.get_hash(&path), Some(hash));
        assert_eq!(registry.len(), 1);
        
        // Set different hash for same path and verify it's updated
        let new_hash = "new_hash".to_string();
        registry.set_hash(path.clone(), new_hash.clone());
        assert_eq!(registry.get_hash(&path), Some(new_hash));
        assert_eq!(registry.len(), 1); // Length should remain 1
    }

    #[test]
    fn test_hash_registry_len() {
        let mut registry = HashRegistry::new();
        assert_eq!(registry.len(), 0);
        
        registry.set_hash(PathBuf::from("/test/file1.txt"), "hash1".to_string());
        assert_eq!(registry.len(), 1);
        
        registry.set_hash(PathBuf::from("/test/file2.txt"), "hash2".to_string());
        assert_eq!(registry.len(), 2);
        
        registry.set_hash(PathBuf::from("/test/file1.txt"), "hash3".to_string()); // Update existing
        assert_eq!(registry.len(), 2); // Should still be 2
    }

    #[test]
    fn test_hash_registry_save_and_load() {
        // Create a registry and add some hashes
        let mut registry = HashRegistry::new();
        registry.set_hash(PathBuf::from("/test/file1.txt"), "hash1".to_string());
        registry.set_hash(PathBuf::from("/test/file2.txt"), "hash2".to_string());
        
        // Save to a temporary file
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("hashes.json");
        registry.save_to_file(&file_path).unwrap();
        
        // Load from the file into a new registry
        let loaded_registry = HashRegistry::load_from_file(&file_path).unwrap();
        
        // Verify the loaded registry has the same hashes
        assert_eq!(loaded_registry.len(), 2);
        assert_eq!(
            loaded_registry.get_hash(&PathBuf::from("/test/file1.txt")), 
            Some("hash1".to_string())
        );
        assert_eq!(
            loaded_registry.get_hash(&PathBuf::from("/test/file2.txt")), 
            Some("hash2".to_string())
        );
    }

    #[test]
    fn test_hash_registry_load_nonexistent_file() {
        // Load from a non-existent file
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("nonexistent.json");
        let registry = HashRegistry::load_from_file(file_path).unwrap();
        
        // Should return an empty registry
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_hash_file_with_content() {
        // Create a temporary file with known content
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "test content";
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        
        // Calculate hash of the file
        let hash = hash_file(temp_file.path()).unwrap();
        
        // The hash should be deterministic for the same content
        let expected_hash = {
            let mut hasher = Hasher::new();
            hasher.update(content.as_bytes());
            hasher.finalize().to_hex().to_string()
        };
        
        assert_eq!(hash, expected_hash);
    }

    #[test]
    fn test_hash_file_empty() {
        // Create an empty temporary file
        let temp_file = NamedTempFile::new().unwrap();
        
        // Calculate hash of empty file
        let hash = hash_file(temp_file.path()).unwrap();
        
        // The hash of an empty file should match this
        let expected_hash = {
            let hasher = Hasher::new();
            hasher.finalize().to_hex().to_string()
        };
        
        assert_eq!(hash, expected_hash);
    }

    #[test]
    fn test_hash_file_nonexistent() {
        // Try to hash a non-existent file
        let result = hash_file("/path/that/definitely/does/not/exist");
        assert!(result.is_err());
    }
}