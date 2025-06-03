# Implementation Plan for MBBUT

1. ✅ Set up project structure
2. ✅ Implement interactive CLI interface with cliclack
3. ✅ Build TOML config parser for blacklist
4. ✅ Create file walking system with walkdir
5. ✅ Implement hashing with blake3
6. ✅ Add compression with zstd
7. ✅ Integrate parallel processing with rayon
8. ✅ Build resumability system
9. ✅ Add decompression CLI command

## Implementation Complete

All features have been implemented according to the project goals:

- Interactive CLI interface using cliclack
- TOML-based blacklist configuration for directories and file extensions
- File compression using zstd
- File decompression for compressed backup files
- File hashing with blake3 for resumability
- Parallel processing with rayon
- Command-line interface with clap

The program can be run with:
```
cargo run -- setup       # For first-time setup
cargo run -- run         # To run a backup
cargo run -- decompress  # To decompress a file
```

Or interactively:
```
cargo run
```

## Decompression Feature Plan

### Overview
Add a new CLI command to decompress `.zst` files using the existing decompression functionality in the compression module. This will enable users to decompress their backup files directly through the CLI.

### Implementation Steps

1. **Update CLI Structure**
   - Add a new `Decompress` command to the `Commands` enum in `src/main.rs`
   - The command will require source (compressed file) and destination (decompressed output) paths
   - Add appropriate documentation for the command

2. **CLI Command Handler**
   - Implement a handler for the `Decompress` command in the `main()` function
   - The handler will:
     - Validate input paths
     - Call the existing `decompress_file` function
     - Provide appropriate user feedback

3. **Interactive Mode Integration**
   - Add decompression as an option in the interactive mode selector
   - Implement an interactive flow to gather source and destination paths

4. **Testing**
   - Create unit tests for the decompression command
   - Test with various file sizes and types
   - Test error handling for invalid input paths

5. **Documentation**
   - Update CLI help text and documentation
   - Add examples of usage

### Code Changes

#### 1. Update Commands Enum in `src/main.rs`

```rust
#[derive(Subcommand)]
enum Commands {
    // Existing commands...
    
    /// Decompress a file
    Decompress {
        /// Path to the compressed file (.zst)
        #[clap(short, long)]
        source: PathBuf,
        
        /// Path where the decompressed file will be saved
        #[clap(short, long)]
        destination: PathBuf,
    },
}
```

#### 2. Add Command Handler in `main()`

```rust
match cli.command {
    // Existing commands...
    
    Some(Commands::Decompress { source, destination }) => {
        log::info("Decompressing file...")?;
        
        if !source.exists() {
            return Err(anyhow::anyhow!("Source file does not exist"));
        }
        
        compression::decompress_file(&source, &destination)
            .context("Failed to decompress file")?;
        
        log::success(&format!("File decompressed to {}", destination.display()))?;
    }
    
    // Other command handlers...
}
```

#### 3. Update Interactive Mode

```rust
let action = select("What would you like to do?")
    .item("setup", "Set up a new backup configuration", "")
    .item("run", "Run backup with existing configuration", "")
    .item("decompress", "Decompress a file", "")
    .interact()?;

match action {
    // Existing actions...
    
    "decompress" => {
        let source_path: String = input("Path to compressed file")
            .placeholder("/path/to/file.zst")
            .validate(|input: &String| {
                if input.is_empty() {
                    Err("Path cannot be empty")
                } else {
                    let path = PathBuf::from(input);
                    if !path.exists() {
                        Err("File does not exist")
                    } else {
                        Ok(())
                    }
                }
            })
            .interact()?;
            
        let destination_path: String = input("Destination path")
            .placeholder("/path/to/decompressed/file")
            .validate(|input: &String| {
                if input.is_empty() {
                    Err("Path cannot be empty")
                } else {
                    Ok(())
                }
            })
            .interact()?;
            
        compression::decompress_file(source_path, destination_path)
            .context("Failed to decompress file")?;
            
        log::success("File decompressed successfully!")?;
    }
    
    // Other action handlers...
}
```

### Testing Plan

1. **Unit Tests**
   - Test decompression of valid compressed files
   - Test handling of invalid source paths
   - Test handling of invalid destination paths

2. **Manual Testing**
   - Test CLI argument-based decompression
   - Test interactive mode decompression
   - Test with various file sizes and types

### Future Enhancements

1. **Batch Decompression**
   - Add ability to decompress multiple files at once
   - Support for directory-based decompression

2. **Progress Indicators**
   - Add progress bars for large file decompression

3. **Integrity Verification**
   - Verify decompressed file integrity after decompression