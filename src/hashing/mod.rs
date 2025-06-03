use anyhow::Result;
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
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