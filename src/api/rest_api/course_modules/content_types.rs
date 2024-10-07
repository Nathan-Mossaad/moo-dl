use std::path::Path;

use serde::Deserialize;

use crate::api::Api;
use crate::Result;

use super::Download;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "file")]
    File(File),
    #[serde(rename = "url")]
    Url(Url),
    #[serde(other)]
    Unknown,
}
impl Download for Content {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        match self {
            Content::File(file) => file.download(api, path).await,
            _ => {
                // TODO add missing module downloaders
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct File {
    pub filename: String,
    pub filepath: String,
    pub fileurl: String,
    pub timemodified: u64,
}
impl Download for File {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        api.download_file_from_api_params(
            path,
            &self.filename,
            &self.filepath,
            &self.fileurl,
            self.timemodified,
        )
        .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Url {
    pub filename: String,
    pub fileurl: String,
    pub timemodified: u64,
}
