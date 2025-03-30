use tokio::{fs::File, io::AsyncWriteExt};

use super::*;

// Write content to file (may overwrite)
pub async fn write_file_contents(file_path: &Path, new_content: &str) -> Result<()> {
    // Make sure path exists
    ensure_path_exists(file_path).await?;

    let mut file = File::create(file_path).await?;
    file.write_all(new_content.as_bytes()).await?;
    file.flush().await?;
    Ok(())
}
