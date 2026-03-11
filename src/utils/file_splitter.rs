//! File splitter utility for splitting large files into manageable chunks
//! 
//! This module provides functionality to split large files (like Twitter archives)
//! into smaller chunks for easier processing and distribution.

use anyhow::{Context, Result, bail};
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::fmt;

/// Configuration for file splitting operations
#[derive(Debug, Clone)]
pub struct SplitConfig {
    /// Path to the input file to split
    pub input_path: PathBuf,
    /// Directory where chunks will be written (defaults to input file's directory)
    pub output_dir: Option<PathBuf>,
    /// Size of each chunk in bytes
    pub chunk_size: u64,
    /// Prefix for chunk filenames (defaults to input filename)
    pub prefix: Option<String>,
    /// Number of digits for chunk numbering (default: 3)
    pub digits: u8,
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            input_path: PathBuf::new(),
            output_dir: None,
            chunk_size: 1024 * 1024, // 1MB default
            prefix: None,
            digits: 3,
        }
    }
}

/// Information about a created file chunk
#[derive(Debug, Clone)]
pub struct ChunkInfo {
    /// Path to the chunk file
    pub path: PathBuf,
    /// Size of the chunk in bytes
    pub size: u64,
    /// Chunk number (1-based)
    pub number: usize,
}

impl fmt::Display for ChunkInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chunk {}: {} ({} bytes)", 
               self.number, 
               self.path.display(), 
               self.size)
    }
}

/// Result of a file splitting operation
#[derive(Debug)]
pub struct SplitResult {
    /// Original input file path
    pub input_path: PathBuf,
    /// Output directory used
    pub output_dir: PathBuf,
    /// Chunk size used
    pub chunk_size: u64,
    /// Information about created chunks
    pub chunks: Vec<ChunkInfo>,
    /// Total size of original file
    pub total_size: u64,
}

impl fmt::Display for SplitResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "📄 Split '{}' into {} chunks", 
                 self.input_path.display(), 
                 self.chunks.len())?;
        writeln!(f, "📁 Output directory: {}", self.output_dir.display())?;
        writeln!(f, "📊 Total size: {} bytes", self.total_size)?;
        writeln!(f, "🔢 Chunk size: {} bytes", self.chunk_size)?;
        writeln!(f, "\n📋 Created chunks:")?;
        
        for chunk in &self.chunks {
            writeln!(f, "  {}", chunk)?;
        }
        
        Ok(())
    }
}

/// Split a file into chunks according to the provided configuration
pub fn split_file(config: &SplitConfig) -> Result<SplitResult> {
    validate_config(config)?;
    
    let input_path = config.input_path.canonicalize()
        .context("Failed to resolve input file path")?;
    
    let output_dir = determine_output_dir(config, &input_path)?;
    let (base_name, extension) = determine_filename_parts(config, &input_path);
    
    let file_size = input_path.metadata()
        .context("Failed to read input file metadata")?
        .len();
    
    if file_size == 0 {
        bail!("Input file is empty");
    }
    
    let chunks = create_chunks(&input_path, &output_dir, &base_name, &extension, config)?;
    
    Ok(SplitResult {
        input_path,
        output_dir,
        chunk_size: config.chunk_size,
        chunks,
        total_size: file_size,
    })
}

/// Validate the split configuration
fn validate_config(config: &SplitConfig) -> Result<()> {
    if config.chunk_size == 0 {
        bail!("Chunk size must be greater than 0");
    }
    
    if config.digits == 0 || config.digits > 10 {
        bail!("Digits must be between 1 and 10");
    }
    
    if !config.input_path.exists() {
        bail!("Input file does not exist: {}", config.input_path.display());
    }
    
    if !config.input_path.is_file() {
        bail!("Input path is not a file: {}", config.input_path.display());
    }
    
    Ok(())
}

/// Determine the output directory for chunks
fn determine_output_dir(config: &SplitConfig, input_path: &Path) -> Result<PathBuf> {
    let output_dir = match &config.output_dir {
        Some(dir) => {
            if !dir.exists() {
                std::fs::create_dir_all(dir)
                    .with_context(|| format!("Failed to create output directory: {}", dir.display()))?;
            }
            dir.clone()
        }
        None => {
            input_path.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        }
    };
    
    output_dir.canonicalize()
        .context("Failed to resolve output directory path")
}

/// Determine the base name and extension for chunk files
fn determine_filename_parts(config: &SplitConfig, input_path: &Path) -> (String, String) {
    match &config.prefix {
        Some(prefix) => (prefix.clone(), String::new()),
        None => {
            let file_name = input_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("chunk");
            
            // Split on first dot to preserve complex extensions like .tar.gz
            if let Some(dot_pos) = file_name.find('.') {
                let base = file_name[..dot_pos].to_string();
                let ext = file_name[dot_pos..].to_string();
                (base, ext)
            } else {
                (file_name.to_string(), String::new())
            }
        }
    }
}

/// Create the actual chunk files
fn create_chunks(
    input_path: &Path,
    output_dir: &Path,
    base_name: &str,
    extension: &str,
    config: &SplitConfig,
) -> Result<Vec<ChunkInfo>> {
    let mut input_file = BufReader::new(
        File::open(input_path)
            .with_context(|| format!("Failed to open input file: {}", input_path.display()))?
    );
    
    let mut chunks = Vec::new();
    let mut buffer = vec![0u8; config.chunk_size as usize];
    let mut chunk_number = 1;
    
    loop {
        let bytes_read = input_file.read(&mut buffer)
            .context("Failed to read from input file")?;
        
        if bytes_read == 0 {
            break; // End of file
        }
        
        let chunk_path = output_dir.join(format!(
            "{}-{:0width$}{}",
            base_name,
            chunk_number,
            extension,
            width = config.digits as usize
        ));
        
        let mut output_file = BufWriter::new(
            File::create(&chunk_path)
                .with_context(|| format!("Failed to create chunk file: {}", chunk_path.display()))?
        );
        
        output_file.write_all(&buffer[..bytes_read])
            .context("Failed to write chunk data")?;
        
        output_file.flush()
            .context("Failed to flush chunk file")?;
        
        chunks.push(ChunkInfo {
            path: chunk_path,
            size: bytes_read as u64,
            number: chunk_number,
        });
        
        chunk_number += 1;
    }
    
    Ok(chunks)
}

/// Parse a size string like "1M", "500K", "2G" into bytes
pub fn parse_size_string(size_str: &str) -> Result<u64> {
    let size_str = size_str.trim().to_uppercase();
    
    if size_str.is_empty() {
        bail!("Empty size string");
    }
    
    // Find where the number ends and unit begins
    let split_pos = size_str.chars()
        .position(|c| !c.is_ascii_digit())
        .unwrap_or(size_str.len());
    
    let (num_str, unit) = size_str.split_at(split_pos);
    
    let num: u64 = num_str.parse()
        .with_context(|| format!("Invalid number in size string: {}", num_str))?;
    
    let multiplier = match unit {
        "" | "B" => 1,
        "K" | "KB" => 1024,
        "M" | "MB" => 1024 * 1024,
        "G" | "GB" => 1024 * 1024 * 1024,
        "T" | "TB" => 1024_u64.pow(4),
        _ => bail!("Invalid size unit: {}. Use B, K, M, G, or T", unit),
    };
    
    Ok(num * multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    
    fn create_test_file(dir: &Path, name: &str, content: &[u8]) -> Result<PathBuf> {
        let path = dir.join(name);
        fs::write(&path, content)?;
        Ok(path)
    }
    
    #[test]
    fn test_split_config_default() {
        let config = SplitConfig::default();
        assert_eq!(config.chunk_size, 1024 * 1024);
        assert_eq!(config.digits, 3);
        assert!(config.output_dir.is_none());
        assert!(config.prefix.is_none());
    }
    
    #[test]
    fn test_parse_size_string() -> Result<()> {
        assert_eq!(parse_size_string("1024")?, 1024);
        assert_eq!(parse_size_string("1K")?, 1024);
        assert_eq!(parse_size_string("1M")?, 1024 * 1024);
        assert_eq!(parse_size_string("1G")?, 1024 * 1024 * 1024);
        assert_eq!(parse_size_string("2M")?, 2 * 1024 * 1024);
        
        // Test with lowercase
        assert_eq!(parse_size_string("1k")?, 1024);
        assert_eq!(parse_size_string("1m")?, 1024 * 1024);
        
        // Test with whitespace
        assert_eq!(parse_size_string(" 1M ")?, 1024 * 1024);
        
        Ok(())
    }
    
    #[test]
    fn test_parse_size_string_invalid() {
        assert!(parse_size_string("").is_err());
        assert!(parse_size_string("abc").is_err());
        assert!(parse_size_string("1X").is_err());
    }
    
    #[test]
    fn test_split_small_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let input_path = create_test_file(temp_dir.path(), "test.txt", b"Hello, World!")?;
        
        let config = SplitConfig {
            input_path,
            chunk_size: 5,
            ..Default::default()
        };
        
        let result = split_file(&config)?;
        
        assert_eq!(result.chunks.len(), 3); // "Hello", ", Wor", "ld!"
        assert_eq!(result.chunks[0].size, 5);
        assert_eq!(result.chunks[1].size, 5);
        assert_eq!(result.chunks[2].size, 3);
        assert_eq!(result.total_size, 13);
        
        Ok(())
    }
    
    #[test]
    fn test_split_with_extension_preservation() -> Result<()> {
        let temp_dir = tempdir()?;
        let input_path = create_test_file(temp_dir.path(), "document.txt", b"test content")?;
        
        let config = SplitConfig {
            input_path,
            chunk_size: 4,
            ..Default::default()
        };
        
        let result = split_file(&config)?;
        
        // Should create document-001.txt, document-002.txt, document-003.txt
        assert_eq!(result.chunks.len(), 3);
        assert!(result.chunks[0].path.file_name().unwrap().to_str().unwrap().ends_with(".txt"));
        assert!(result.chunks[0].path.file_name().unwrap().to_str().unwrap().starts_with("document-001"));
        
        Ok(())
    }
    
    #[test]
    fn test_split_with_custom_prefix() -> Result<()> {
        let temp_dir = tempdir()?;
        let input_path = create_test_file(temp_dir.path(), "test.txt", b"test content")?;
        
        let config = SplitConfig {
            input_path,
            chunk_size: 4,
            prefix: Some("custom".to_string()),
            ..Default::default()
        };
        
        let result = split_file(&config)?;
        
        assert!(result.chunks[0].path.file_name().unwrap().to_str().unwrap().starts_with("custom-001"));
        
        Ok(())
    }
    
    #[test]
    fn test_split_with_custom_output_dir() -> Result<()> {
        let temp_dir = tempdir()?;
        let input_path = create_test_file(temp_dir.path(), "test.txt", b"test content")?;
        let output_dir = temp_dir.path().join("chunks");
        
        let config = SplitConfig {
            input_path,
            chunk_size: 4,
            output_dir: Some(output_dir.clone()),
            ..Default::default()
        };
        
        let result = split_file(&config)?;
        
        let canonical_output_dir = output_dir.canonicalize()?;
        assert_eq!(result.output_dir, canonical_output_dir);
        assert!(result.chunks[0].path.starts_with(&canonical_output_dir));
        
        Ok(())
    }
    
    #[test]
    fn test_validate_config_invalid_chunk_size() {
        let config = SplitConfig {
            chunk_size: 0,
            ..Default::default()
        };
        
        assert!(validate_config(&config).is_err());
    }
    
    #[test]
    fn test_validate_config_nonexistent_file() {
        let config = SplitConfig {
            input_path: PathBuf::from("nonexistent.txt"),
            ..Default::default()
        };
        
        assert!(validate_config(&config).is_err());
    }
    
    #[test]
    fn test_empty_file_handling() -> Result<()> {
        let temp_dir = tempdir()?;
        let input_path = create_test_file(temp_dir.path(), "empty.txt", b"")?;
        
        let config = SplitConfig {
            input_path,
            chunk_size: 1024,
            ..Default::default()
        };
        
        let result = split_file(&config);
        assert!(result.is_err());
        
        Ok(())
    }
    
    #[test]
    fn test_complex_extension_preservation() -> Result<()> {
        let temp_dir = tempdir()?;
        let input_path = create_test_file(temp_dir.path(), "archive.tar.gz", b"compressed data")?;
        
        let config = SplitConfig {
            input_path,
            chunk_size: 5,
            ..Default::default()
        };
        
        let result = split_file(&config)?;
        
        // Should preserve the full .tar.gz extension
        assert!(result.chunks[0].path.file_name().unwrap().to_str().unwrap().ends_with(".tar.gz"));
        assert!(result.chunks[0].path.file_name().unwrap().to_str().unwrap().starts_with("archive-001"));
        
        Ok(())
    }
}
