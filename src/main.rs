mod backup;
mod compression;
mod config;
mod hashing;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cliclack::{confirm, intro, log, outro, select, input};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run backup with previously saved configuration
    Run {
        /// Path to the configuration file
        #[clap(short, long)]
        config: Option<PathBuf>,
    },
    /// Set up a new backup configuration
    Setup {
        /// Path to save the configuration file
        #[clap(short, long)]
        output: Option<PathBuf>,
    },
}

fn run_interactive_setup() -> Result<config::Config> {
    intro("MBBUT - Mackenzie Bowes' Back Up Tool")?;

    log::info("Setting up backup configuration...")?;

    let source_path: String = input("Source path")
        .placeholder("/path/to/source")
        .validate(|input: &String| {
            if input.is_empty() {
                Err("Path cannot be empty")
            } else {
                let path = PathBuf::from(input);
                if !path.exists() {
                    Err("Path does not exist")
                } else {
                    Ok(())
                }
            }
        })
        .interact()?;

    let destination_path: String = input("Destination path")
        .placeholder("/path/to/destination")
        .validate(|input: &String| {
            if input.is_empty() {
                Err("Path cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact()?;

    let hash_file_path: String = input("Path to store hash registry")
        .placeholder("/path/to/hashes.json")
        .validate(|input: &String| {
            if input.is_empty() {
                Err("Path cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact()?;

    // Create config with default blacklists
    let mut config = config::Config::default();
    config.source_path = Some(PathBuf::from(source_path));
    config.destination_path = Some(PathBuf::from(destination_path));
    config.hash_file_path = Some(PathBuf::from(hash_file_path));

    // Ask if user wants to customize blacklists
    let customize_blacklists =
        confirm("Do you want to customize blacklisted directories and extensions?").interact()?;

    if customize_blacklists {
        let blacklist_csv: String = input("Enter blacklist entities separated by spaces").interact()?;
        let blacklist_items: Vec<String> = blacklist_csv.split_whitespace().map(String::from).collect();

        for item in blacklist_items {
            if item.starts_with('.') {
                config.blacklist_extensions.insert(item.trim_start_matches('.').to_string());
            } else {
                config.blacklist_dirs.insert(item);
            }
        }
    }

    outro("Configuration complete!")?;

    Ok(config)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run { config }) => {
            // Load config
            let config_path = config.unwrap_or_else(|| PathBuf::from("mbbut_config.toml"));
            let config = config::Config::load_from_file(&config_path)
                .context("Failed to load configuration file")?;

            // Load hash registry
            let hash_file_path = config
                .hash_file_path
                .as_ref()
                .context("Hash file path not set in config")?;
            let hash_registry = hashing::HashRegistry::load_from_file(hash_file_path)
                .context("Failed to load hash registry")?;

            // Create and run backup job
            let mut backup_job = backup::BackupJob::new(config, hash_registry);
            backup_job.run()?;
        }
        Some(Commands::Setup { output }) => {
            // Interactive setup
            let config = run_interactive_setup()?;

            // Save configuration
            let output_path = output.unwrap_or_else(|| PathBuf::from("mbbut_config.toml"));
            config.save_to_file(output_path)?;
        }
        None => {
            // If no command is provided, run interactive mode
            let run_backup = select("What would you like to do?")
                .item("setup", "Set up a new backup configuration", "")
                .item("run", "Run backup with existing configuration", "")
                .interact()?;

            match run_backup {
                "setup" => {
                    let config = run_interactive_setup()?;
                    let output_path = PathBuf::from("mbbut_config.toml");
                    config.save_to_file(output_path)?;
                }
                "run" => {
                    let config_path = PathBuf::from("mbbut_config.toml");
                    if !config_path.exists() {
                        log::error("No configuration file found. Please run setup first.")?;
                        return Ok(());
                    }

                    let config = config::Config::load_from_file(&config_path)
                        .context("Failed to load configuration file")?;

                    let hash_file_path = config
                        .hash_file_path
                        .as_ref()
                        .context("Hash file path not set in config")?;
                    let hash_registry = hashing::HashRegistry::load_from_file(hash_file_path)
                        .context("Failed to load hash registry")?;

                    let mut backup_job = backup::BackupJob::new(config, hash_registry);
                    backup_job.run()?;
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}