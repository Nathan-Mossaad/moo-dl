pub mod chromium;
pub mod raw_file;
pub mod request;
pub mod youtube;

use std::path::Path;

use tokio::fs;

use crate::{config::sync_config::Config, Result};

/// Ensures that the directory specified by the given `Path` exists.
async fn ensure_path_exists(path: &Path) -> Result<()> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir).await?;
    }
    Ok(())
}

impl Config {
    // Check if regex filter matches passed &str
    // # Returns
    // true, if some regex matched
    pub async fn check_filter(&self, str: &str) -> Result<bool> {
        if let Some(filters) = self.file_filters.as_ref() {
            for filter in filters {
                let re = regex::Regex::new(filter)
                    .map_err(|e| anyhow::anyhow!("Invalid regex {}: {}", filter, e))?;
                if re.is_match(&str) {
                    // If the filename matches one of the filters, return early.
                    self.status_bar.register_skipped().await;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
