use regex::Regex;
use tracing::warn;

use super::*;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "file")]
    File(ContentFile),
    #[serde(rename = "url")]
    Url(ContentUrl),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct ContentFile {
    filename: String,
    filepath: String,
    fileurl: String,
    timemodified: u64,
}

#[derive(Debug, Deserialize)]
pub struct ContentUrl {
    filename: String,
    fileurl: String,
    timemodified: u64,
}

impl Download for Content {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        match self {
            Content::File(content_file) => content_file.download(config, path).await,
            Content::Url(content_url) => content_url.download(config, path).await,
            Content::Unknown => {
                warn!("Not syncing unknown Content type, create an issue if you want this added!");
                Ok(())
            }
        }
    }
}

impl Download for ContentFile {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        // Filter regex
        if let Some(filters) = &config.file_filters {
            for filter in filters {
                let re = Regex::new(filter)
                    .map_err(|e| anyhow::anyhow!("Invalid regex {}: {}", filter, e))?;
                if re.is_match(&self.filename) {
                    // If the filename matches one of the filters, return early.
                    config.status_bar.register_skipped().await;
                    return Ok(());
                }
            }
        }

        let file_path = &self::assemble_path(path, &self.filepath, &self.filename);

        let request = config
            .client
            .get(&self.fileurl)
            .query(&[("token", &config.wstoken)]);

        let download_result = config
            .download_file_with_timestamp(file_path, request, self.timemodified)
            .await;

        if let Err(e) = download_result {
            config
                .status_bar
                .register_err(
                    &e.context(format!(
                        "Failed downloading file: {}",
                        &file_path.to_str().unwrap_or("Unavailable")
                    ))
                    .to_string(),
                )
                .await
        }

        Ok(())
    }
}

impl Download for ContentUrl {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        // let file_path = path.join(&self.filename);
        // TODO
        tracing::error!("Implement Download for ContentUrl");
        Ok(())
    }
}
