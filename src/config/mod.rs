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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        
        // Verify default blacklisted directories
        assert!(config.blacklist_dirs.contains("node_modules"));
        assert!(config.blacklist_dirs.contains("target"));
        assert!(config.blacklist_dirs.contains("dist"));
        assert!(config.blacklist_dirs.contains(".git"));
        assert_eq!(config.blacklist_dirs.len(), 4);
        
        // Verify default blacklisted extensions
        assert!(config.blacklist_extensions.contains("exe"));
        assert!(config.blacklist_extensions.contains("dll"));
        assert!(config.blacklist_extensions.contains("obj"));
        assert_eq!(config.blacklist_extensions.len(), 3);
        
        // Verify paths are None by default
        assert!(config.source_path.is_none());
        assert!(config.destination_path.is_none());
        assert!(config.hash_file_path.is_none());
    }

    #[test]
    fn test_config_load_from_file() {
        // Create a temporary file with valid TOML content
        let mut temp_file = NamedTempFile::new().unwrap();
        let toml_content = r#"
            blacklist_dirs = ["test_dir", "another_dir"]
            blacklist_extensions = ["log", "tmp"]
            source_path = "/tmp/source"
            destination_path = "/tmp/destination"
            hash_file_path = "/tmp/hashes.json"
        "#;
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        
        // Load config from the temporary file
        let config = Config::load_from_file(temp_file.path()).unwrap();
        
        // Verify loaded values
        assert_eq!(config.blacklist_dirs.len(), 2);
        assert!(config.blacklist_dirs.contains("test_dir"));
        assert!(config.blacklist_dirs.contains("another_dir"));
        
        assert_eq!(config.blacklist_extensions.len(), 2);
        assert!(config.blacklist_extensions.contains("log"));
        assert!(config.blacklist_extensions.contains("tmp"));
        
        assert_eq!(config.source_path, Some(PathBuf::from("/tmp/source")));
        assert_eq!(config.destination_path, Some(PathBuf::from("/tmp/destination")));
        assert_eq!(config.hash_file_path, Some(PathBuf::from("/tmp/hashes.json")));
    }

    #[test]
    fn test_config_load_from_file_invalid() {
        // Create a temporary file with invalid TOML content
        let mut temp_file = NamedTempFile::new().unwrap();
        let invalid_toml = r#"
            blacklist_dirs = "this should be an array"
            source_path = 123  # This should be a string
        "#;
        temp_file.write_all(invalid_toml.as_bytes()).unwrap();
        
        // Attempt to load config and verify it fails
        let result = Config::load_from_file(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_config_save_to_file() {
        // Create a config to save
        let mut config = Config::default();
        config.source_path = Some(PathBuf::from("/test/source"));
        config.destination_path = Some(PathBuf::from("/test/dest"));
        
        // Save to a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        config.save_to_file(temp_file.path()).unwrap();
        
        // Read back the file and parse it to verify
        let content = fs::read_to_string(temp_file.path()).unwrap();
        let loaded_config: Config = toml::from_str(&content).unwrap();
        
        // Verify saved values match original config
        assert_eq!(loaded_config.source_path, config.source_path);
        assert_eq!(loaded_config.destination_path, config.destination_path);
        assert_eq!(loaded_config.blacklist_dirs, config.blacklist_dirs);
        assert_eq!(loaded_config.blacklist_extensions, config.blacklist_extensions);
    }

    #[test]
    fn test_is_blacklisted_directory() {
        let config = Config::default();
        
        // Test with blacklisted directory
        let path = PathBuf::from("/some/path/node_modules/file.js");
        assert!(config.is_blacklisted(&path));
        
        // Test with non-blacklisted directory
        let path = PathBuf::from("/some/path/src/file.js");
        assert!(!config.is_blacklisted(&path));
        
        // Test with blacklisted directory as part of the path
        let path = PathBuf::from("/some/node_modules/path/file.js");
        assert!(config.is_blacklisted(&path));
    }

    #[test]
    fn test_is_blacklisted_extension() {
        let config = Config::default();
        
        // Test with blacklisted extension
        let path = PathBuf::from("/some/path/program.exe");
        assert!(config.is_blacklisted(&path));
        
        // Test with non-blacklisted extension
        let path = PathBuf::from("/some/path/program.rs");
        assert!(!config.is_blacklisted(&path));
    }

    #[test]
    fn test_is_blacklisted_mixed() {
        let mut config = Config::default();
        config.blacklist_dirs.insert("custom_dir".to_string());
        config.blacklist_extensions.insert("log".to_string());
        
        // Test with both blacklisted directory and extension
        let path = PathBuf::from("/some/path/custom_dir/file.log");
        assert!(config.is_blacklisted(&path));
        
        // Test with non-blacklisted path
        let path = PathBuf::from("/some/path/allowed_dir/file.txt");
        assert!(!config.is_blacklisted(&path));
    }
}
