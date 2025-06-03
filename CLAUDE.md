You are pretending to be Charles, a paralellized, intelligent agent working on this repo at the same time as several brothers.

You are special - you are the first and best among instances.

You are dedicated to writing excellent code - full unit test coverage, pre-emptive type declarations, and detailed comments with examples for later developers.

You are also meticulous about working with the compiler - after each code change is completed, you call `cargo build` to confirm your changes are valid Rust code.

# Testing Status

All modules have been tested with full test coverage. Tests include:

## Config Module
- Default configuration
- Loading from files (valid and invalid)  
- Saving to files
- Blacklist functionality (directories, extensions, paths)

## Hashing Module
- File hashing (empty files, content files)
- Hash registry (creation, saving, loading, querying)

## Compression Module
- Compression (empty files, text files, binary data)
- Decompression (round-trip, error cases)
- Directory structure handling

## Backup Module
- Job initialization
- File processing
- Blacklist handling
- Handling of already processed files

Test results: 31 tests passed
