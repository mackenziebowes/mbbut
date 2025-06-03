# mbbut (mackenzie bowes' back up tool)

![Vibe Coded](https://img.shields.io/badge/vibe-coded-622118)

A command line interface for backing it up.

## Features

- Fast file-by-file compression using ZStandard
- Incremental backups with BLAKE3 hash tracking
- Parallel processing for improved performance
- Configurable file and directory exclusions
- Resume interrupted backups
- Decompress backed-up files when needed

## Installation

### From source

```bash
# Clone the repository
git clone https://github.com/mackenziebowes/mbbut.git
cd mbbut

# Build and install
cargo build --release
cargo install --path .
```

### From crates.io (coming soon, maybe)

```bash
cargo install mbbut
```

## Usage

```bash
# Run a backup using a saved configuration
mbbut run --config mbbut_config.toml

# Set up a new backup configuration
mbbut setup --output mbbut_config.toml

# Resume a previously interrupted backup
mbbut resume --config mbbut_config.toml

# Decompress a file
mbbut decompress --source backup.txt.zst --destination original.txt
```

## Configuration

Configuration is stored in TOML format with the following options:

```toml
source_path = "/path/to/backup"
destination_path = "/path/to/store/backup"
hash_file_path = "/path/to/hash/registry"
blacklist_dirs = ["node_modules", "target", "dist", ".git"]
blacklist_extensions = ["exe", "dll", "obj"]
```

## Why?

Friendship ended with Windows, Linux is my new best friend.

- Local LLMs are finally good
- Windows is a joke for system management
- Therefore, I have to convert my Windows devices to Linux
- Something something NTFS
- I am storing a decade of memories to an SSD so I can flash my tower drives
- Maybe you want to do the same with the various laptops hanging around your place

Or, yknow, you just want to back stuff up.

## Support

lol. lmao.

## License

MIT
