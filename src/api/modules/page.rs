use std::str::FromStr;

use url::Url;

use super::{content_types::Content, *};

#[derive(Debug, Deserialize)]
pub struct Page {
    pub name: String,
    pub url: String,
    pub contents: Option<Vec<Content>>,
    pub contentsinfo: ContentsInfo,
}
#[derive(Debug, Deserialize)]
pub struct ContentsInfo {
    pub lastmodified: u64,
}

impl Download for Page {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        let path = path.join(&self.name);
        if let Some(contents) = &self.contents {
            // Create a task for each content
            let tasks = contents
                .into_iter()
                .map(|r| r.download(config.clone(), &path));
            // Return an error if one occured
            for res in join_all(tasks).await {
                res.context("Failed Resource")?;
            }
        }

        let url = Url::from_str(&self.url)?;
        config
            .save_page_with_timestamp(&path, &url, self.contentsinfo.lastmodified)
            .await
            .context("Failed Resource")?;

        Ok(())
    }
}
