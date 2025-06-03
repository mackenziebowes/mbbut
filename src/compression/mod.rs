use anyhow::Result;
use std::fs::{self, File};
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