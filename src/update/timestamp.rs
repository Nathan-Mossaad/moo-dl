use std::{
    fs::FileTimes,
    path::Path,
    time::{Duration, UNIX_EPOCH},
};

use tokio::{
    fs::{self, File},
    io,
};

use super::*;

impl UpdateStrategy {
    /// Check if file is up date
    async fn timestamp_check_file_date(file_path: &Path, timestamp: u64) -> Result<UpdateState> {
        match fs::metadata(file_path).await {
            Ok(_) => {
                if timestamp > get_file_creation(file_path).await? {
                    Ok(UpdateState::OutOfDate)
                } else {
                    Ok(UpdateState::UpToDate)
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(UpdateState::Missing),
            Err(e) => Err(e.into()),
        }
    }

    /// Does the same as check_exists but additionally archives the file,
    /// if the passed timestamp is newer than the version on disk
    /// (Respects the setting in Update Strategy)
    ///
    /// May move the file in case of archive or expects the user to overwrite the file in case of update
    pub async fn timestamp_check_up_to_date(&self, file_path: &Path, timestamp: u64) -> Result<UpdateState> {
        match self {
            UpdateStrategy::None => UpdateStrategy::check_exists(file_path).await,
            UpdateStrategy::Update => UpdateStrategy::timestamp_check_file_date(file_path, timestamp).await,
            UpdateStrategy::Archive => {
                let state = UpdateStrategy::timestamp_check_file_date(file_path, timestamp).await?;
                if state == UpdateState::OutOfDate {
                    archive_file(file_path).await?;
                }
                Ok(state)
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
