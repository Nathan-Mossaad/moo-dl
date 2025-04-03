use super::{content_types::Content, *};

#[derive(Debug, Deserialize)]
pub struct Resource {
    pub id: u64,
    pub name: String,
    pub contents: Option<Vec<Content>>,
}

impl Download for Resource {
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
        Ok(())
    }
}
