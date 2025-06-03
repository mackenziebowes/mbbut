use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub blacklist_dirs: HashSet<String>,
    pub blacklist_extensions: HashSet<String>,
    pub source_path: Option<PathBuf>,
    pub destination_path: Option<PathBuf>,
    pub hash_file_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        let mut blacklist_dirs = HashSet::new();
        blacklist_dirs.insert("node_modules".to_string());
        blacklist_dirs.insert("target".to_string());
        blacklist_dirs.insert("dist".to_string());
        blacklist_dirs.insert(".git".to_string());

        let mut blacklist_extensions = HashSet::new();
        blacklist_extensions.insert("exe".to_string());
        blacklist_extensions.insert("dll".to_string());
        blacklist_extensions.insert("obj".to_string());

        Self {
            blacklist_dirs,
            blacklist_extensions,
            source_path: None,
            destination_path: None,
            hash_file_path: None,
        }
    }
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn is_blacklisted(&self, path: &Path) -> bool {
        // Check if any component of the path is in the blacklist
        if let Some(file_name) = path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                if self.blacklist_dirs.contains(file_name_str) {
                    return true;
                }
            }
        }

        // Check if the file extension is blacklisted
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                if self.blacklist_extensions.contains(ext_str) {
                    return true;
                }
            }
        }

        // Check for any parent directories in the blacklist
        for ancestor in path.ancestors().skip(1) {
            if let Some(dir_name) = ancestor.file_name() {
                if let Some(dir_name_str) = dir_name.to_str() {
                    if self.blacklist_dirs.contains(dir_name_str) {
                        return true;
                    }
                }
            }
        }

        false
    }
}
