// TODO: remove
#![allow(dead_code)]

use std::{
    fs::FileTimes,
    path::Path,
    time::{Duration, UNIX_EPOCH},
};

use chrono::prelude::{DateTime, Local};
use tokio::{
    fs::{self, File},
    io,
};

use crate::{config::sync_config::UpdateStrategy, Result};

impl UpdateStrategy {
    /// Check if a file exists
    /// # Returns
    /// true if file exists
    pub async fn check_exists(file_path: &Path) -> Result<bool> {
        match fs::metadata(file_path).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    /// Check if file is up date
    /// # Returns
    /// true if the file is up to date
    async fn check_file_date(file_path: &Path, timestamp: u64) -> Result<bool> {
        match fs::metadata(file_path).await {
            Ok(_) => Ok(get_file_creation(file_path).await? >= timestamp),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    /// Does the same as check_exists but additionally archives the file,
    /// if the passed timestamp is newer than the version on disk
    /// (Respects the setting in Update Strategy)
    ///
    /// May delete the file in case of archive or expects the user to overwrite the file in case of update
    pub async fn check_up_to_date(&self, file_path: &Path, timestamp: u64) -> Result<bool> {
        match self {
            UpdateStrategy::None => UpdateStrategy::check_exists(file_path).await,
            UpdateStrategy::Update => UpdateStrategy::check_file_date(file_path, timestamp).await,
            UpdateStrategy::Archive => {
                if UpdateStrategy::check_file_date(file_path, timestamp).await? {
                    Ok(true)
                } else {
                    // We only want to archive, if the file actually exists
                    if UpdateStrategy::check_exists(file_path).await? {
                        archive_file(file_path).await?;
                    }
                    Ok(false)
                }
            }
        }
    }
}

/// Returns the creation time of the file as a Unix timestamp
///
/// # Errors
///
/// If files creation time can not be read (e.g. filee does not exist)
async fn get_file_creation(file_path: &Path) -> Result<u64> {
    let metadata = fs::metadata(file_path).await?;

    // Attempt to get the time since unix epoch
    let file_raw_time = metadata.modified()?;

    Ok(file_raw_time.duration_since(UNIX_EPOCH)?.as_secs())
}

/// Writes the given Unix timestamp as creation date of the file
///
/// # Errors
///
/// If files creation time can not be set (e.g. file does not exist)
pub async fn set_file_creation(file_path: &Path, timestamp: u64) -> Result<()> {
    let new_age = UNIX_EPOCH + Duration::from_secs(timestamp);
    let times = FileTimes::new().set_accessed(new_age).set_modified(new_age);

    let dest = File::options()
        .write(true)
        .open(file_path)
        .await?
        .into_std()
        .await;

    tokio::task::spawn_blocking(move || {
        let _ = dest.set_times(times);
    })
    .await?;

    Ok(())
}

/// Archives a file by appending its modified date to the file name.
async fn archive_file(file_path: &Path) -> Result<()> {
    // Returns if file is not found, or modate is not valid
    let file_time = fs::metadata(&file_path)
        .await?
        .modified()?
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    let date = DateTime::from_timestamp(file_time as i64, 0)
        .unwrap()
        .with_timezone(&Local)
        .to_rfc3339();

    let new_path_string = if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
        format!("{}_{}.{}", file_path.to_string_lossy(), date, ext)
    } else {
        format!("{}_{}", file_path.to_string_lossy(), date)
    };

    fs::rename(file_path, new_path_string).await?;
    Ok(())
}
