# MBBUT Project Goals

## Core Features
- Fast and reliable backup utility for migration scenarios
- Configurable blacklist for directories and file extensions
- Efficient file compression using zstd
- File hashing with blake3 for verification and resumability
- Parallel processing with rayon
- Command-line interface with clap

## User Interface
- Interactive CLI interface using cliclack
- Command-line arguments for headless operation:
  - `setup`: Configure a new backup job
  - `run`: Execute a backup with existing configuration
  - `resume`: Resume a previously interrupted backup

## Technical Requirements
- Efficient use of system resources
- Resume capability for interrupted transfers
- Robust error handling
- Full unit test coverage
- Detailed documentation