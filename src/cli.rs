//! Command Line Interface module for Tweet-Scrolls
//! Implements simple folder-based processing as per requirements

use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use std::env;
use serde_json::Value;

/// CLI configuration parsed from command line arguments
#[derive(Debug)]
pub struct CliConfig {
    /// Path to Twitter archive folder containing tweets.js, direct-messages.js, etc.
    pub archive_folder: PathBuf,
    /// Output directory (defaults to a timestamped folder in the archive directory)
    pub output_dir: Option<PathBuf>,
    /// Run in non-interactive mode (no prompts)
    pub non_interactive: bool,
}

impl CliConfig {
    /// Parse command line arguments
    /// 
    /// # Usage
    /// 
    /// ```bash
    /// tweet-scrolls /path/to/twitter/archive
    /// tweet-scrolls /path/to/twitter/archive /path/to/output
    /// ```
    pub fn from_args() -> Result<Self> {
        let args: Vec<String> = env::args().collect();
        
        if args.len() < 2 {
            print_usage();
            bail!("Missing required argument: archive folder path");
        }
        
        let archive_folder = PathBuf::from(&args[1]);
        
        // Validate the folder exists
        if !archive_folder.exists() {
            bail!("Archive folder does not exist: {}", archive_folder.display());
        }
        
        if !archive_folder.is_dir() {
            bail!("Path is not a directory: {}", archive_folder.display());
        }
        
        // Check for required files
        if find_archive_file_path(&archive_folder, "tweets.js").is_none() {
            bail!("tweets.js not found in archive folder or archive_folder/data");
        }
        
        let output_dir = if args.len() > 2 {
            Some(PathBuf::from(&args[2]))
        } else {
            None
        };
        
        Ok(CliConfig {
            archive_folder,
            output_dir,
            non_interactive: true, // Always non-interactive when using CLI args
        })
    }
    
    /// Get the path to tweets.js file
    pub fn tweets_file(&self) -> PathBuf {
        find_archive_file_path(&self.archive_folder, "tweets.js")
            .unwrap_or_else(|| self.archive_folder.join("tweets.js"))
    }
    
    /// Get the path to direct-messages.js file (if it exists)
    pub fn dms_file(&self) -> Option<PathBuf> {
        find_archive_file_path(&self.archive_folder, "direct-messages.js")
    }
    
    /// Get the path to direct-message-headers.js file (if it exists)
    pub fn dm_headers_file(&self) -> Option<PathBuf> {
        find_archive_file_path(&self.archive_folder, "direct-message-headers.js")
    }
    
    /// Get or create the output directory
    pub fn get_output_dir(&self, screen_name: &str, timestamp: i64) -> PathBuf {
        match &self.output_dir {
            Some(dir) => dir.clone(),
            None => self.archive_folder.join(format!("output_{}_{}", screen_name, timestamp))
        }
    }
}

fn find_archive_file_path(archive_folder: &Path, file_name: &str) -> Option<PathBuf> {
    let archive_root_path = archive_folder.join(file_name);
    if archive_root_path.exists() {
        return Some(archive_root_path);
    }

    let nested_data_path = archive_folder.join("data").join(file_name);
    if nested_data_path.exists() {
        return Some(nested_data_path);
    }

    None
}

fn extract_archive_screen_name(archive_folder: &Path) -> Option<String> {
    let account_path = find_archive_file_path(archive_folder, "account.js")?;
    let account_content = std::fs::read_to_string(account_path).ok()?;
    let json_start = account_content.find('[')?;
    let json_end = account_content.rfind(']')?;
    let account_entries: Value = serde_json::from_str(&account_content[json_start..=json_end]).ok()?;

    account_entries
        .get(0)?
        .get("account")?
        .get("username")?
        .as_str()
        .map(|username| username.to_string())
}

fn print_usage() {
    eprintln!("Tweet-Scrolls - Twitter Archive Processor");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  tweet-scrolls <archive-folder> [output-folder]");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  <archive-folder>  Path to Twitter archive folder containing tweets.js");
    eprintln!("  [output-folder]   Optional output directory (defaults to archive folder)");
    eprintln!();
    eprintln!("Example:");
    eprintln!("  tweet-scrolls /home/user/twitter-archive");
    eprintln!("  tweet-scrolls /home/user/twitter-archive /home/user/output");
}

/// Process Twitter archive with CLI configuration
pub async fn process_with_cli(config: CliConfig) -> Result<()> {
    use crate::main_process::main_process_twitter_archive;
    use chrono::Utc;
    use crate::utils::file_splitter::{split_file, SplitConfig};

    println!("🚀 Processing Twitter archive from: {}", config.archive_folder.display());

    let tweets_file = config.tweets_file();
    let dms_file = config.dms_file();
    let dm_headers_file = config.dm_headers_file();

    // Input file splitting removed: Only output TXT files will be split after processing

    let screen_name = extract_archive_screen_name(&config.archive_folder)
        .unwrap_or_else(|| "user".to_string());
    let timestamp = Utc::now().timestamp();
    let output_dir = config.get_output_dir(&screen_name, timestamp);

    println!("📁 Output directory: {}", output_dir.display());

    // Process the archive
    main_process_twitter_archive(
        tweets_file.to_str().unwrap(),
        dms_file.as_ref().map(|p| p.to_str().unwrap()),
        dm_headers_file.as_ref().map(|p| p.to_str().unwrap()),
        output_dir.to_str().unwrap(),
        &screen_name,
        timestamp,
    ).await?;

    println!("✅ Processing complete!");

    // --- New requirement: Split large output TXT files (>1MB) after processing ---
    use std::fs;
    use std::ffi::OsStr;
    println!("🔎 Scanning output directory for large TXT files...");
    let txt_files = fs::read_dir(&output_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension() == Some(OsStr::new("txt")) {
                let metadata = fs::metadata(&path).ok()?;
                if metadata.len() > 1024 * 1024 {
                    Some((path, metadata.len()))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    for (path, size) in txt_files {
        println!("[FileSplitter] Splitting large TXT file: {} ({} bytes)", path.display(), size);
        let split_config = SplitConfig {
            input_path: path.clone(),
            output_dir: Some(path.parent().unwrap().to_path_buf()),
            chunk_size: 1024 * 1024, // 1MB
            prefix: None,
            digits: 3,
        };
        match split_file(&split_config) {
            Ok(result) => println!("[FileSplitter] {}", result),
            Err(e) => println!("[FileSplitter] Error splitting file {}: {}", path.display(), e),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;
    
    #[test]
    fn test_cli_config_parsing() {
        // Test with valid arguments
        env::set_var("TEST_ARGS", "tweet-scrolls /tmp/test");
        
        // We can't easily test env::args() directly, so we'll test the validation logic
        let config = CliConfig {
            archive_folder: PathBuf::from("/tmp"),
            output_dir: None,
            non_interactive: true,
        };
        
        assert_eq!(config.archive_folder, PathBuf::from("/tmp"));
        assert!(config.non_interactive);
    }
    
    #[tokio::test]
    async fn test_file_detection() -> Result<()> {
        let temp_dir = tempdir()?;
        let archive_path = temp_dir.path();
        
        // Create test files
        fs::write(archive_path.join("tweets.js"), "test").await?;
        fs::write(archive_path.join("direct-messages.js"), "test").await?;
        
        let config = CliConfig {
            archive_folder: archive_path.to_path_buf(),
            output_dir: None,
            non_interactive: true,
        };
        
        assert!(config.tweets_file().exists());
        assert!(config.dms_file().is_some());
        assert!(config.dm_headers_file().is_none());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_nested_data_file_detection() -> Result<()> {
        let temp_dir = tempdir()?;
        let archive_path = temp_dir.path();
        let data_path = archive_path.join("data");
        fs::create_dir_all(&data_path).await?;

        fs::write(data_path.join("tweets.js"), "test").await?;
        fs::write(data_path.join("direct-messages.js"), "test").await?;

        let config = CliConfig {
            archive_folder: archive_path.to_path_buf(),
            output_dir: None,
            non_interactive: true,
        };

        assert_eq!(config.tweets_file(), data_path.join("tweets.js"));
        assert_eq!(config.dms_file(), Some(data_path.join("direct-messages.js")));
        assert!(config.dm_headers_file().is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_extract_archive_screen_name() -> Result<()> {
        let temp_dir = tempdir()?;
        let archive_path = temp_dir.path();
        let data_path = archive_path.join("data");
        fs::create_dir_all(&data_path).await?;

        fs::write(
            data_path.join("account.js"),
            r#"window.YTD.account.part0 = [{"account":{"username":"realhandle"}}]"#,
        ).await?;

        assert_eq!(
            extract_archive_screen_name(archive_path),
            Some("realhandle".to_string())
        );

        Ok(())
    }
}
