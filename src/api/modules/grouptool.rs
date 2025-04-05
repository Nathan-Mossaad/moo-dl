use std::str::FromStr;

use url::Url;

use super::*;

#[derive(Debug, Deserialize)]
pub struct Grouptool {
    pub name: String,
    pub url: String,
}
impl Download for Grouptool {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        let file_path = path.join(self.name.to_string());

        let url = Url::from_str(&self.url)?;

        config.save_page(&file_path, &url).await?;
        Ok(())
    }
}
