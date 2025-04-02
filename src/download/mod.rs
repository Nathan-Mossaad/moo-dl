// TODO: remove
#![allow(dead_code)]

pub mod raw_file;
pub mod request;
pub mod youtube;
pub mod chromium;

use std::path::Path;

use tokio::fs;

use crate::Result;

/// Ensures that the directory specified by the given `Path` exists.
async fn ensure_path_exists(path: &Path) -> Result<()> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir).await?;
    }
    Ok(())
}
