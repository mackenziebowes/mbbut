use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use zstd::stream::{copy_decode, copy_encode};

const COMPRESSION_LEVEL: i32 = 3; // Balanced between speed and size

pub fn compress_file<P: AsRef<Path>, Q: AsRef<Path>>(source: P, destination: Q) -> Result<()> {
    // Ensure the destination directory exists
    if let Some(parent) = destination.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    let source_file = File::open(source)?;
    let destination_file = File::create(destination)?;

    copy_encode(source_file, destination_file, COMPRESSION_LEVEL)?;

    Ok(())
}

pub fn decompress_file<P: AsRef<Path>, Q: AsRef<Path>>(source: P, destination: Q) -> Result<()> {
    // Ensure the destination directory exists
    if let Some(parent) = destination.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    let source_file = File::open(source)?;
    let destination_file = File::create(destination)?;

    copy_decode(source_file, destination_file)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::{NamedTempFile, tempdir};

    #[test]
    fn test_compress_empty_file() {
        // Create an empty source file
        let source_file = NamedTempFile::new().unwrap();
        
        // Create a destination path
        let temp_dir = tempdir().unwrap();
        let dest_path = temp_dir.path().join("empty.zst");
        
        // Compress the empty file
        compress_file(source_file.path(), &dest_path).unwrap();
        
        // Verify the compressed file exists and is not empty (zstd adds headers)
        assert!(dest_path.exists());
        let metadata = fs::metadata(&dest_path).unwrap();
        assert!(metadata.len() > 0); // Even empty files will have compression metadata
    }
    
    #[test]
    fn test_compress_text_file() {
        // Create a source file with text content
        let mut source_file = NamedTempFile::new().unwrap();
        let text_content = "This is a test file with text content that should compress well.".repeat(100);
        source_file.write_all(text_content.as_bytes()).unwrap();
        source_file.flush().unwrap();
        
        // Create a destination path
        let temp_dir = tempdir().unwrap();
        let dest_path = temp_dir.path().join("text.zst");
        
        // Compress the text file
        compress_file(source_file.path(), &dest_path).unwrap();
        
        // Verify the compressed file exists and is smaller than the original
        // (text should compress well)
        assert!(dest_path.exists());
        let original_size = fs::metadata(source_file.path()).unwrap().len();
        let compressed_size = fs::metadata(&dest_path).unwrap().len();
        assert!(compressed_size < original_size);
    }
    
    #[test]
    fn test_compress_binary_file() {
        // Create a source file with binary content (already compressed-like)
        let mut source_file = NamedTempFile::new().unwrap();
        let mut binary_content = Vec::with_capacity(10000);
        for i in 0..200u8 {
            // Create a pattern of bytes that repeats
            for _ in 0..50 {
                binary_content.push(i);
            }
        }
        source_file.write_all(&binary_content).unwrap();
        source_file.flush().unwrap();
        
        // Create a destination path
        let temp_dir = tempdir().unwrap();
        let dest_path = temp_dir.path().join("binary.zst");
        
        // Compress the binary file
        compress_file(source_file.path(), &dest_path).unwrap();
        
        // Verify the compressed file exists
        assert!(dest_path.exists());
    }
    
    #[test]
    fn test_decompress_file() {
        // Create a source file with content
        let mut source_file = NamedTempFile::new().unwrap();
        let original_content = "This is test content for compression and decompression.".repeat(50);
        source_file.write_all(original_content.as_bytes()).unwrap();
        source_file.flush().unwrap();
        
        // Compress the file
        let temp_dir = tempdir().unwrap();
        let compressed_path = temp_dir.path().join("compressed.zst");
        compress_file(source_file.path(), &compressed_path).unwrap();
        
        // Decompress the file
        let decompressed_path = temp_dir.path().join("decompressed.txt");
        decompress_file(&compressed_path, &decompressed_path).unwrap();
        
        // Read the decompressed content and verify it matches original
        let mut decompressed_content = String::new();
        let mut file = File::open(&decompressed_path).unwrap();
        file.read_to_string(&mut decompressed_content).unwrap();
        
        assert_eq!(decompressed_content, original_content);
    }
    
    #[test]
    fn test_decompress_nonexistent_file() {
        // Try to decompress a non-existent file
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("nonexistent.zst");
        let dest_path = temp_dir.path().join("output.txt");
        
        let result = decompress_file(source_path, dest_path);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_decompress_invalid_data() {
        // Create a file with invalid zstd data
        let mut invalid_file = NamedTempFile::new().unwrap();
        invalid_file.write_all(b"This is not valid zstd data").unwrap();
        invalid_file.flush().unwrap();
        
        // Try to decompress the invalid file
        let temp_dir = tempdir().unwrap();
        let dest_path = temp_dir.path().join("output.txt");
        
        let result = decompress_file(invalid_file.path(), dest_path);
        assert!(result.is_err()); // Should fail with a zstd error
    }
    
    #[test]
    fn test_compression_creates_parent_dirs() {
        // Create a source file
        let mut source_file = NamedTempFile::new().unwrap();
        source_file.write_all(b"Test content").unwrap();
        source_file.flush().unwrap();
        
        // Create a destination path with nested directories that don't exist yet
        let temp_dir = tempdir().unwrap();
        let nested_path = temp_dir.path().join("nested/dirs/that/dont/exist/yet.zst");
        
        // Compression should create all parent directories
        compress_file(source_file.path(), &nested_path).unwrap();
        
        // Verify the compressed file exists, meaning the directories were created
        assert!(nested_path.exists());
    }
}