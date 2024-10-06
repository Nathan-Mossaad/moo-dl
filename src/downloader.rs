use tracing::info;

use chrono::prelude::{DateTime, Local};
use std::{path::Path, time::UNIX_EPOCH};
use tokio::fs;

use crate::Result;

/// Options for the downloader
pub struct DownloadOptions {
    /// Strategy for handling file updates
    pub file_update_strategy: FileUpdateStrategy,
    /// Continaer format for storing websites
    pub site_store: SiteStore,
    /// Force update of files with unknown creation date
    pub force_update: bool,
}

/// Strategy for handling file updates
pub enum FileUpdateStrategy {
    /// Don't update files
    Ignore,
    /// Overwrite existing files
    Overwrite,
    /// Create a archived version of the file and redownload
    Archive,
}

/// Countainer formats for storing websites
pub enum SiteStore {
    /// Use singlefile see: https://github.com/gildas-lormeau/single-file-cli
    SingleFile,
    /// Create a PDF file consisting of a single page
    MonoPDF,
    /// Create a standard PDF file
    StandardPDF,
}

impl FileUpdateStrategy {
    /// Archives a given file if specified by the strategy
    /// If there is a need to download the file / it doesn't exist, it will return true
    /// If it is up to date, it will return false
    ///
    /// If the file system doesn't support file creation time, it will only check if the file exists
    pub async fn archive_file(&self, path: &Path, last_modified: Option<u64>) -> Result<bool> {
        // Get metadata and check if file exists
        let metadata = match fs::metadata(path).await {
            Ok(metadata) => metadata,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    return Ok(true);
                } else {
                    return Err(err.into());
                }
            }
        };

        // Check if there there is a last known modified time
        let last_modified = match last_modified {
            Some(last_modified) => last_modified,
            None => {
                // If there is no last known modified time, we don't have to archive it
                // We already know it exists
                return Ok(false);
            }
        };

        // Attempt to get the time since unix epoch
        let file_raw_time = match metadata.modified() {
            Ok(file_raw_time) => file_raw_time,
            Err(_) => {
                if let FileUpdateStrategy::Ignore = self {
                    info!("FS doesn't support modified time, disabling updating downloads");
                }
                return Ok(false);
            }
        };
        let file_time = file_raw_time.duration_since(UNIX_EPOCH)?.as_secs();

        // Check if file is up to date
        if file_time >= last_modified {
            // File is up to date
            return Ok(false);
        }

        // File is not up to date and has to be updated/archived
        match self {
            FileUpdateStrategy::Archive => {
                // Archive file
                version_file(path).await?;
                Ok(true)
            }
            FileUpdateStrategy::Overwrite => {
                // Remove file
                fs::remove_file(path).await?;
                Ok(true)
            }
            _ => {
                // Ignore file
                Ok(false)
            }
        }
    }
}

/// Versions a file by appending its modified date to the file name.
///
/// This function takes a file path as input and renames the file by appending the file's
/// modification date (in ISO 8601 format) to the end of the file name. This is useful for
/// creating versioned backups of files.
///
/// If the file does not exist or the modification date cannot be retrieved, the function will
/// return an error.
///
/// # Arguments
///
/// - `path`: The path to the file to be versioned.
///
/// # Returns
///
/// A `Result` that indicates whether the file was successfully versioned. If an error occurs
/// while retrieving the file metadata or renaming the file, the `Result` will contain the error.
///
/// # Example
///
/// ```rust
/// use std::path::Path;
///
/// #[tokio::main]
/// async fn main() -> Result<(), std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>> {
///     let file_path = Path::new("./example.txt");
///     version_file(file_path).await?;
///     println!("File versioned successfully!");
///     Ok(())
/// }
/// ```
async fn version_file(path: &Path) -> Result<()> {
    // Returns if file is not found, or modate is not valid
    let file_time = fs::metadata(&path)
        .await?
        .modified()?
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    let date = DateTime::from_timestamp(file_time as i64, 0)
        .unwrap()
        .with_timezone(&Local)
        .to_rfc3339();

    let mut new_path_string = path.to_string_lossy().to_string();
    new_path_string.push_str("_");
    new_path_string.push_str(&date);
    match path.extension() {
        Some(str) => {
            new_path_string.push_str(".");
            new_path_string.push_str(str.to_str().unwrap_or(""));
        }
        _ => {
            new_path_string.push_str("");
        }
    };

    fs::rename(path, new_path_string).await?;
    Ok(())
}
