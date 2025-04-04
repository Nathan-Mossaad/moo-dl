pub mod raw_file;
pub mod timestamp;
pub mod youtube;

use std::{path::Path, time::UNIX_EPOCH};

use chrono::prelude::{DateTime, Local};
use tokio::{fs, io};

use crate::{config::sync_config::UpdateStrategy, Result};

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateState {
    Missing,
    OutOfDate,
    UpToDate,
}

impl UpdateStrategy {
    /// Check if a file exists
    /// # Returns
    /// UpToDate if file exists
    /// Missing if not
    pub async fn check_exists(file_path: &Path) -> Result<UpdateState> {
        match fs::metadata(file_path).await {
            Ok(_) => Ok(UpdateState::UpToDate),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(UpdateState::Missing),
            Err(e) => Err(e.into()),
        }
    }
}

/// Archives a file by appending its modified date to the file name.
pub async fn archive_file(file_path: &Path) -> Result<()> {
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
