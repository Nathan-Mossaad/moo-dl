use std::{
    fs::FileTimes,
    path::Path,
    time::{Duration, UNIX_EPOCH},
};

use tokio::fs::{self, File};

use crate::Result;

/// Returns the creation time of the file as a Unix timestamp
///
/// # Errors
///
/// If files creation time can not be read (e.g. filee does not exist)
pub async fn get_file_creation(file_path: &Path) -> Result<u64> {
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
