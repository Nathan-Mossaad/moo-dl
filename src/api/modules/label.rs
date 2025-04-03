use super::*;

#[derive(Debug, Deserialize)]
pub struct Label {
    pub id: u64,
    pub name: String,
    pub description: String,
}

impl Download for Label {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        let file_name = format!("{}.html", self.name);
        let path = path.join(file_name);

        config.write_file_contents(&path, &self.description).await?;
        
        Ok(())
    }
}
