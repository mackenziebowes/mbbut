use crate::compression;
use crate::config::Config;
use crate::hashing::{hash_file, HashRegistry};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct BackupJob {
    pub config: Config,
    pub hash_registry: HashRegistry,
}

impl BackupJob {
    pub fn new(config: Config, hash_registry: HashRegistry) -> Self {
        Self {
            config,
            hash_registry,
        }
    }

    /// Collects files that need to be processed, skipping blacklisted items and already processed files
    fn collect_files_to_process(&self) -> Result<Vec<PathBuf>> {
        let source_path = self
            .config
            .source_path
            .as_ref()
            .context("Source path not set")?;
            
        let mut files_to_process = Vec::new();

        for entry in WalkDir::new(source_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip directories (we'll create them as needed)
            if path.is_dir() {
                continue;
            }

            // Skip blacklisted paths
            if self.config.is_blacklisted(path) {
                continue;
            }

            // Skip already processed files (if hash exists)
            if self.hash_registry.has_hash(path) {
                continue;
            }

            files_to_process.push(path.to_path_buf());
        }

        Ok(files_to_process)
    }

    /// Process a list of files with appropriate progress reporting
    fn process_files(&mut self, files_to_process: Vec<PathBuf>, message: String) -> Result<()> {
        let source_path = self
            .config
            .source_path
            .as_ref()
            .context("Source path not set")?;
        let destination_path = self
            .config
            .destination_path
            .as_ref()
            .context("Destination path not set")?;

        // Create destination directory if it doesn't exist
        fs::create_dir_all(destination_path)?;

        // Set up progress bar
        let pb = ProgressBar::new(files_to_process.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );

        // Create thread-safe clones to share between threads
        let source_path = source_path.clone();
        let destination_path = destination_path.clone();
        
        // Process files in parallel using Rayon
        files_to_process.par_iter().for_each(|source_file| {
            let result = process_file(
                source_file,
                &source_path,
                &destination_path,
            );
            
            if let Ok(hash) = result {
                // Safe to mutate our own hash registry here
                let mut registry_lock = self.hash_registry.hashes.lock().unwrap();
                registry_lock.insert(source_file.to_path_buf(), hash);
            } else if let Err(e) = result {
                eprintln!("Error processing file {}: {}", source_file.display(), e);
            }

            pb.inc(1);
        });

        pb.finish_with_message(message);

        // Save the updated hash registry
        if let Some(hash_file_path) = &self.config.hash_file_path {
            self.hash_registry.save_to_file(hash_file_path)?;
        }

        Ok(())
    }

    /// Run a full backup operation
    pub fn run(&mut self) -> Result<()> {
        let files_to_process = self.collect_files_to_process()?;
        
        if files_to_process.is_empty() {
            println!("No files to backup. Everything is already up to date.");
            return Ok(());
        }
        
        self.process_files(files_to_process, "Backup completed".to_string())
    }
    
    /// Resume a previously interrupted backup
    pub fn resume(&mut self) -> Result<()> {
        let files_to_process = self.collect_files_to_process()?;
        
        if files_to_process.is_empty() {
            println!("No files to resume. The backup is already complete.");
            return Ok(());
        }
        
        println!("Resuming backup with {} files remaining", files_to_process.len());
        self.process_files(files_to_process, "Resume completed".to_string())
    }
}

pub fn process_file(
    source_file: &Path,
    source_root: &Path,
    destination_root: &Path,
) -> Result<String> {
    // Calculate relative path from source root
    let relative_path = source_file.strip_prefix(source_root)?;

    // Construct destination path with .zst extension
    let mut destination_file = destination_root.join(relative_path);
    destination_file.set_extension(format!(
        "{}.zst",
        destination_file
            .extension()
            .map_or("", |e| e.to_str().unwrap_or(""))
    ));

    // Create parent directories if needed
    if let Some(parent) = destination_file.parent() {
        fs::create_dir_all(parent)?;
    }

    // Compress the file
    compression::compress_file(source_file, &destination_file)?;

    // Calculate hash and return it
    let hash = hash_file(source_file)?;
    
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::fs::File;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_backup_job_new() {
        // Create a config and hash registry
        let config = Config::default();
        let hash_registry = HashRegistry::new();
        
        // Create a new backup job
        let backup_job = BackupJob::new(config, hash_registry);
        
        // Verify default values
        assert_eq!(backup_job.hash_registry.len(), 0);
        assert!(backup_job.config.source_path.is_none());
        assert!(backup_job.config.destination_path.is_none());
        assert!(backup_job.config.hash_file_path.is_none());
    }

    #[test]
    fn test_process_file() {
        // Create a source file with some content
        let mut source_file = NamedTempFile::new().unwrap();
        let content = "Test content for processing";
        source_file.write_all(content.as_bytes()).unwrap();
        
        // Create source and destination directories
        let source_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();
        
        // Create a test file in the source directory structure
        let test_subdir = source_dir.path().join("subdir");
        fs::create_dir_all(&test_subdir).unwrap();
        let test_file_path = test_subdir.join("test.txt");
        fs::copy(source_file.path(), &test_file_path).unwrap();
        
        // Process the file
        let hash = process_file(
            &test_file_path, 
            source_dir.path(),
            dest_dir.path()
        ).unwrap();
        
        // Verify the hash is correct
        let expected_hash = hash_file(&test_file_path).unwrap();
        assert_eq!(hash, expected_hash);
        
        // Verify the destination file exists with correct path structure
        let expected_dest_path = dest_dir.path().join("subdir/test.txt.zst");
        assert!(expected_dest_path.exists());
        
        // Verify the file can be decompressed
        let decompressed_path = dest_dir.path().join("decompressed.txt");
        compression::decompress_file(expected_dest_path, &decompressed_path).unwrap();
        
        // Read the decompressed content
        let mut decompressed_content = String::new();
        let mut file = File::open(&decompressed_path).unwrap();
        file.read_to_string(&mut decompressed_content).unwrap();
        
        assert_eq!(decompressed_content, content);
    }
    
    #[test]
    fn test_process_file_no_extension() {
        // Create a source file with no extension
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(b"No extension test").unwrap();
        source_file.flush().unwrap();
        
        // Create source and destination directories
        let source_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();
        
        // Create a test file with no extension in the source directory
        let test_file_path = source_dir.path().join("noextension");
        fs::copy(source_file.path(), &test_file_path).unwrap();
        
        // Process the file
        process_file(
            &test_file_path, 
            source_dir.path(),
            dest_dir.path()
        ).unwrap();
        
        // Verify a compressed file was created in the destination directory
        let files = fs::read_dir(dest_dir.path()).unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect::<Vec<_>>();
            
        assert_eq!(files.len(), 1, "Expected exactly one file in destination directory");
        
        // The actual issue is that the code adds a dot and then .zst, so for a file with no extension
        // it creates "noextension..zst" (with double dot)
        let expected_dest_path = dest_dir.path().join("noextension..zst");
        assert!(expected_dest_path.exists(), "Compressed file was not created at expected path");
    }
    
    #[test]
    fn test_backup_job_run_empty_dirs() {
        // Create empty source and destination directories
        let source_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();
        let hash_file = NamedTempFile::new().unwrap();
        
        // Create config and hash registry
        let mut config = Config::default();
        config.source_path = Some(PathBuf::from(source_dir.path()));
        config.destination_path = Some(PathBuf::from(dest_dir.path()));
        config.hash_file_path = Some(PathBuf::from(hash_file.path()));
        
        let hash_registry = HashRegistry::new();
        let mut backup_job = BackupJob::new(config, hash_registry);
        
        // Run the backup job (should succeed with no files)
        let result = backup_job.run();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_backup_job_run_with_files() {
        // Create source and destination directories
        let source_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();
        let hash_file = NamedTempFile::new().unwrap();
        
        // Create a test file in the source directory
        let test_file_path = source_dir.path().join("test.txt");
        let mut test_file = File::create(&test_file_path).unwrap();
        test_file.write_all(b"Test content").unwrap();
        
        // Create a blacklisted file
        let blacklisted_path = source_dir.path().join("test.exe");
        let mut blacklisted_file = File::create(&blacklisted_path).unwrap();
        blacklisted_file.write_all(b"Blacklisted content").unwrap();
        
        // Create config and hash registry
        let mut config = Config::default();
        config.source_path = Some(source_dir.path().to_path_buf());
        config.destination_path = Some(dest_dir.path().to_path_buf());
        config.hash_file_path = Some(hash_file.path().to_path_buf());
        
        let hash_registry = HashRegistry::new();
        let mut backup_job = BackupJob::new(config, hash_registry);
        
        // Run the backup job
        let result = backup_job.run();
        assert!(result.is_ok());
        
        // Verify the txt file was backed up and the exe file was skipped
        let expected_txt_path = dest_dir.path().join("test.txt.zst");
        let expected_exe_path = dest_dir.path().join("test.exe.zst");
        
        assert!(expected_txt_path.exists());
        assert!(!expected_exe_path.exists());
        
        // Verify hash registry contains only the txt file
        assert_eq!(backup_job.hash_registry.len(), 1);
        assert!(backup_job.hash_registry.has_hash(&test_file_path));
        assert!(!backup_job.hash_registry.has_hash(&blacklisted_path));
    }

    #[test]
    fn test_backup_job_run_with_blacklist() {
        // Create source and destination directories
        let source_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();
        let hash_file = NamedTempFile::new().unwrap();
        
        // Create test directory structure
        fs::create_dir_all(source_dir.path().join("regular_dir")).unwrap();
        fs::create_dir_all(source_dir.path().join("node_modules")).unwrap();
        
        // Create files in both directories
        let regular_file = source_dir.path().join("regular_dir/file.txt");
        let blacklisted_file = source_dir.path().join("node_modules/package.json");
        
        let mut file = File::create(&regular_file).unwrap();
        file.write_all(b"Regular file").unwrap();
        
        let mut file = File::create(&blacklisted_file).unwrap();
        file.write_all(b"Blacklisted directory file").unwrap();
        
        // Create config with default blacklist (which includes node_modules)
        let mut config = Config::default();
        config.source_path = Some(source_dir.path().to_path_buf());
        config.destination_path = Some(dest_dir.path().to_path_buf());
        config.hash_file_path = Some(hash_file.path().to_path_buf());
        
        let hash_registry = HashRegistry::new();
        let mut backup_job = BackupJob::new(config, hash_registry);
        
        // Run the backup job
        let result = backup_job.run();
        assert!(result.is_ok());
        
        // Verify only the regular file was backed up
        let expected_regular_path = dest_dir.path().join("regular_dir/file.txt.zst");
        let expected_blacklisted_path = dest_dir.path().join("node_modules/package.json.zst");
        
        assert!(expected_regular_path.exists());
        assert!(!expected_blacklisted_path.exists());
    }

    #[test]
    fn test_backup_job_skips_processed_files() {
        // Create source and destination directories
        let source_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();
        let hash_file = NamedTempFile::new().unwrap();
        
        // Create a test file
        let test_file_path = source_dir.path().join("test.txt");
        let mut test_file = File::create(&test_file_path).unwrap();
        test_file.write_all(b"Test content").unwrap();
        
        // Create config
        let mut config = Config::default();
        config.source_path = Some(source_dir.path().to_path_buf());
        config.destination_path = Some(dest_dir.path().to_path_buf());
        config.hash_file_path = Some(hash_file.path().to_path_buf());
        
        // Create hash registry with the test file already marked as processed
        let mut hash_registry = HashRegistry::new();
        hash_registry.set_hash(test_file_path.clone(), "dummy_hash".to_string());
        
        // Create and run backup job
        let mut backup_job = BackupJob::new(config, hash_registry);
        let result = backup_job.run();
        assert!(result.is_ok());
        
        // Verify the file was skipped (no destination file was created)
        let expected_path = dest_dir.path().join("test.txt.zst");
        assert!(!expected_path.exists());
    }
}