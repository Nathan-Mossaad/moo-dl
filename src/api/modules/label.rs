use super::*;

#[derive(Debug, Deserialize)]
pub struct Label {
    pub name: String,
    pub description: String,
}

impl Download for Label {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        // Check for youtube vidoes
        config
            .queue_youtube_vidoes_extract(&self.description, path.to_owned())
            .await?;
        // Check for sciebo links
        Config::extract_sciebo_download(config.clone(), &self.description, path.to_owned()).await?;

        let file_name = format!("{}.html", self.name);
        let path = path.join(file_name);

        config.write_file_contents(&path, &self.description).await?;

        Ok(())
    }
}
