use std::path::Path;

use tokio::{fs::File, io::AsyncReadExt};

use super::*;

impl UpdateStrategy {
    /// Check if file is up date
    async fn file_check_file_contents(file_path: &Path, new_content: &str) -> Result<UpdateState> {
        match get_file_contents(file_path).await {
            Ok(file_content) => Ok(if file_content == new_content {
                UpdateState::UpToDate
            } else {
                UpdateState::OutOfDate
            }),
            Err(e) => {
                if let Some(io_err) = e.downcast_ref::<io::Error>() {
                    if io_err.kind() == io::ErrorKind::NotFound {
                        return Ok(UpdateState::Missing);
                    }
                }
                Err(e)
            }
        }
    }

    /// Check if file is up date
    /// Behaves like `timestamp_check_up_to_date` but uses the file content instead of the creation date
    pub async fn file_check_up_to_date(
        &self,
        file_path: &Path,
        new_content: &str,
    ) -> Result<UpdateState> {
        match self {
            UpdateStrategy::None => UpdateStrategy::check_exists(file_path).await,
            UpdateStrategy::Update => {
                UpdateStrategy::file_check_file_contents(file_path, new_content).await
            }
            UpdateStrategy::Archive => {
                let state =
                    UpdateStrategy::file_check_file_contents(file_path, new_content).await?;
                if state == UpdateState::OutOfDate {
                    archive_file(file_path).await?;
                }
                Ok(state)
            }
        }
    }
}

async fn get_file_contents(file_path: &Path) -> Result<String> {
    let mut file = File::open(file_path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}
