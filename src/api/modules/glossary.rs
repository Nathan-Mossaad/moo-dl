use super::*;

#[derive(Debug, Deserialize)]
pub struct Glossary {
    pub id: u64,
    pub name: String,
}
impl Download for Glossary {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        let file_path = path.join(self.name.to_string());
        let mut glossary_url = config.get_moodle_url().join("mod/glossary/print.php")?;
        glossary_url
            .query_pairs_mut()
            .append_pair("id", &self.id.to_string())
            .append_pair("mode", "")
            .append_pair("hook", "ALL")
            .append_pair("sortkey", "")
            .append_pair("sortorder", "")
            .append_pair("offset", "0")
            .append_pair("pagelimit", "0");

        config.save_page(&file_path, &glossary_url).await?;
        Ok(())
    }
}
