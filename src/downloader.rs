use tracing::info;

use chrono::prelude::{DateTime, Local};
use std::{
    fs::FileTimes,
    path::Path,
    time::{Duration, UNIX_EPOCH},
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tokio_stream::StreamExt;

use chromiumoxide::cdp::browser_protocol::target::CreateTargetParams;
use reqwest::RequestBuilder;
use web2pdf_lib::{Browser, BrowserWeb2Pdf, PageWeb2Pdf};

use crate::Result;

/// Options for the downloader
#[derive(Debug, Clone)]
pub struct DownloadOptions {
    /// Strategy for handling file updates
    pub file_update_strategy: FileUpdateStrategy,
    /// Continaer format for storing websites
    pub site_store: SiteStore,
    /// Force update of files with unknown creation date
    pub force_update: bool,
}

/// Strategy for handling file updates
#[derive(Debug, Clone)]
pub enum FileUpdateStrategy {
    /// Create a archived version of the file and redownload
    Archive,
    /// Overwrite existing files
    Overwrite,
    /// Don't update files
    Ignore,
}

/// Countainer formats for storing websites
#[derive(Debug, Clone)]
pub enum SiteStore {
    /// Create a PDF file consisting of a single page
    MonoPDF,
    /// Create a standard PDF file
    StandardPDF,
    /// Disable downloading websites
    None,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            file_update_strategy: FileUpdateStrategy::Archive,
            site_store: SiteStore::MonoPDF,
            force_update: false,
        }
    }
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
                // Will be overwritten by download
                Ok(true)
            }
            _ => {
                // Ignore file
                Ok(false)
            }
        }
    }

    /// Downloads a file from the specified URL asynchronously.
    ///
    /// RequestBuilder should be created from a Client::get(url) call.
    pub async fn download_from_requestbuilder(
        &self,
        request: RequestBuilder,
        path: &Path,
        last_modified: Option<u64>,
    ) -> Result<()> {
        if self.archive_file(path, last_modified).await? {
            info!("Downloading/Updating file {}", path.display());
            download_file_using_tmp(request, path, last_modified).await?;
        }
        Ok(())
    }
}

impl SiteStore {
    pub async fn save_page(
        &self,
        browser: &Browser,
        url: impl Into<String>,
        file_path: &Path,
    ) -> Result<()> {
        // Check if PDF creation is wanted
        if let SiteStore::None = self {
            return Ok(());
        }

        let new_page_params = CreateTargetParams::builder().url(url).build()?;
        let page = browser.web2pdf_new_page(new_page_params).await?;

        // Make sure file doesn't exist
        match fs::remove_file(file_path).await {
            Ok(_) => {}
            // File could not be removed, path might also not exists:
            Err(_) => {
                ensure_path_exists(file_path).await?;
            }
        }

        // Save PDF
        page.web2pdf_save_pdf_mono_standard(file_path).await?;
        Ok(())
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
        _ => {}
    };

    fs::rename(path, new_path_string).await?;
    Ok(())
}

/// Sets the modified and accessed times of a file to a specified unix timestamp.
pub async fn write_modified_time(path: &Path, last_modified: u64) -> Result<()> {
    let dest = File::options()
        .write(true)
        .open(path)
        .await?
        .into_std()
        .await;
    write_modified_time_from_file(dest, last_modified).await?;
    Ok(())
}
pub async fn write_modified_time_from_file(dest: std::fs::File, last_modified: u64) -> Result<()> {
    let new_age = UNIX_EPOCH + Duration::from_secs(last_modified);
    let times = FileTimes::new().set_accessed(new_age).set_modified(new_age);
    tokio::task::spawn_blocking(move || {
        let _ = dest.set_times(times);
    })
    .await?;
    Ok(())
}

/// Ensures that the directory specified by the given `Path` exists.
async fn ensure_path_exists(path: &Path) -> Result<()> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir).await?;
    }
    Ok(())
}

/// Downloads a file from the specified URL asynchronously.
/// RequestBuilder should be created from a Client::get(url) call.
async fn chunked_download(
    request: RequestBuilder,
    path: &Path,
    last_modified: Option<u64>,
) -> Result<()> {
    // Make sure path exists
    ensure_path_exists(path).await?;

    let mut file = File::create(path).await?;

    let mut stream = request.send().await?.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
    }

    file.flush().await?;

    // Write modified time
    if let Some(last_modified) = last_modified {
        write_modified_time_from_file(file.into_std().await, last_modified).await?;
    }

    Ok(())
}

/// Same as chunked_download but writes to a temporary file first, to avoid partially downloaded files
/// Also results in the old file existing until the new one is fully downloaded
///
/// RequestBuilder should be created from a Client::get(url) call.
pub async fn download_file_using_tmp(
    request: RequestBuilder,
    path: &Path,
    last_modified: Option<u64>,
) -> Result<()> {
    // New path for temporary file
    let tmp_path = path.with_extension("tmp_bZpbocXJQkxt_moo-dl");

    // Make sure file doesn't exist
    let _ = fs::remove_file(&tmp_path).await;
    // Download file
    chunked_download(request, &tmp_path, last_modified).await?;

    // Make sure file doesn't exist
    let _ = fs::remove_file(&path).await;
    // Move file to destination
    fs::rename(tmp_path, path).await?;

    Ok(())
}
