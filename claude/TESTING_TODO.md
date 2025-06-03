# MBBUT Testing ToDo

## Overview
This document outlines the testing plan for MBBUT (Mackenzie Bowes' Back Up Tool). As per Charles' mandate, we have ensured full unit test coverage with detailed comments and proper type declarations.

## Testing Strategy
1. ✅ Create unit tests for each module
2. ✅ Test edge cases and error conditions
3. ✅ Create mock objects where needed
4. ✅ Document all tests with clear examples

## Test Coverage Completed

### Config Module
- [x] Test `Config::default()` - Verify default values
- [x] Test `Config::load_from_file()` - Test with valid and invalid TOML
- [x] Test `Config::save_to_file()` - Verify file is saved correctly
- [x] Test `Config::is_blacklisted()` - Test with various path types
  - [x] Blacklisted directories at different levels
  - [x] Blacklisted file extensions
  - [x] Non-blacklisted items

### Hashing Module
- [x] Test `hash_file()` - Verify hash generation
  - [x] Test with empty files
  - [x] Test with known content files
- [x] Test `HashRegistry::new()` - Verify initialization
- [x] Test `HashRegistry::load_from_file()` - Test with valid and missing files
- [x] Test `HashRegistry::save_to_file()` - Verify serialization
- [x] Test `HashRegistry::has_hash()` - Verify presence checking
- [x] Test `HashRegistry::get_hash()` - Verify retrieval
- [x] Test `HashRegistry::set_hash()` - Verify setting works
- [x] Test `HashRegistry::len()` - Verify count accuracy

### Compression Module
- [x] Test `compress_file()` - Verify compression
  - [x] Test with empty files
  - [x] Test with binary data
  - [x] Test with text data
- [x] Test `decompress_file()` - Verify decompression
  - [x] Test round-trip compression/decompression
  - [x] Test with invalid compressed data

### Backup Module
- [x] Test `BackupJob::new()` - Verify initialization
- [x] Test `BackupJob::run()` - End-to-end test
- [x] Test `process_file()` - Verify individual file processing
  - [x] Test relative path calculation
  - [x] Test destination file creation
  - [x] Test with various file types

## Implementation Completed ✅
1. ✅ Set up test directory structure
2. ✅ Implement config tests
3. ✅ Implement hashing tests 
4. ✅ Implement compression tests
5. ✅ Implement backup tests
6. ✅ All tests passing with `cargo test`

## Test Results
```
running 31 tests
test backup::tests::test_backup_job_new ... ok
test backup::tests::test_backup_job_run_empty_dirs ... ok
test backup::tests::test_backup_job_skips_processed_files ... ok
test compression::tests::test_compress_empty_file ... ok
test compression::tests::test_compress_binary_file ... ok
test backup::tests::test_process_file_no_extension ... ok
test compression::tests::test_decompress_nonexistent_file ... ok
test config::tests::test_config_default ... ok
test backup::tests::test_backup_job_run_with_files ... ok
test compression::tests::test_compress_text_file ... ok
test compression::tests::test_decompress_invalid_data ... ok
test config::tests::test_is_blacklisted_directory ... ok
test backup::tests::test_process_file ... ok
test compression::tests::test_decompress_file ... ok
test config::tests::test_is_blacklisted_extension ... ok
test backup::tests::test_backup_job_run_with_blacklist ... ok
test config::tests::test_is_blacklisted_mixed ... ok
test config::tests::test_config_load_from_file_invalid ... ok
test hashing::tests::test_hash_file_nonexistent ... ok
test config::tests::test_config_load_from_file ... ok
test config::tests::test_config_save_to_file ... ok
test hashing::tests::test_hash_registry_len ... ok
test hashing::tests::test_hash_registry_get_hash ... ok
test hashing::tests::test_hash_registry_has_hash ... ok
test hashing::tests::test_hash_registry_new ... ok
test hashing::tests::test_hash_registry_set_hash ... ok
test hashing::tests::test_hash_file_empty ... ok
test hashing::tests::test_hash_registry_load_nonexistent_file ... ok
test hashing::tests::test_hash_file_with_content ... ok
test compression::tests::test_compression_creates_parent_dirs ... ok
test hashing::tests::test_hash_registry_save_and_load ... ok

test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```