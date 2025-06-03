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

# Project Goals

## Core Features

1. Easy backup configuration through an interactive CLI
2. Resumable backups with hash-based change detection
3. File compression for efficient storage
4. File decompression for easy restoration
5. Blacklisting for directories and file extensions
6. Multi-threaded operation for performance

## CLI Features

- Interactive mode for first-time users
- Command-line arguments for automation
- Run command to execute backups
- Setup command for configuration
- Decompress command to restore compressed files

## Future Enhancements

- Batch decompression
- Progress indicators for large files
- Integrity verification
