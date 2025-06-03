use crate::compression;
use crate::config::Config;
use crate::hashing::{hash_file, HashRegistry};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::Path;
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
    fn collect_files_to_process(&self) -> Result<Vec<std::path::PathBuf>> {
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
    fn process_files(&mut self, files_to_process: Vec<std::path::PathBuf>, message: String) -> Result<()> {
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

fn process_file(
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