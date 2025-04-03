use tokio::{fs::File, io::AsyncWriteExt};
use anyhow::anyhow;

use crate::{config::sync_config::Config, update::UpdateState};

use super::*;

/// Write content to file (may overwrite)
async fn force_write_file_contents(file_path: &Path, new_content: &str) -> Result<()> {
    // Make sure path exists
    ensure_path_exists(file_path).await?;

    let mut file = File::create(file_path).await?;
    file.write_all(new_content.as_bytes()).await?;
    file.flush().await?;
    Ok(())
}

impl Config {
    /// Same as `write_file_contents` but respects update preferences
    /// Additionally writes the event to log
    pub async fn write_file_contents(&self, file_path: &Path, new_content: &str) -> Result<()> {
        match self
            .update_strategy
            .file_check_up_to_date(file_path, new_content)
            .await?
        {
            UpdateState::Missing => {
                force_write_file_contents(file_path, new_content).await?;

                let message = file_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;
                self.status_bar.register_new(message).await;
                Ok(())
            }
            UpdateState::OutOfDate => {
                force_write_file_contents(file_path, new_content).await?;

                let message = file_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;
                self.status_bar.register_updated(message).await;
                Ok(())
            }
            UpdateState::UpToDate => {
                self.status_bar.register_unchanged().await;
                Ok(())
            }
        }

    }
}