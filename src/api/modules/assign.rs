use std::str::FromStr;

use url::Url;

use super::*;

#[derive(Debug, Deserialize)]
pub struct Assign {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
}
impl Download for Assign {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        let path = path.join(&self.name);
        // Assignments don't provide most of their information via core_course_get_contents
        // Therefore we need to use mod_assign_get_submission_status instead
        // TODO: use: mod_assign_get_submission_status
        tracing::warn!("TODO: implement assign!");

        // We still want to save the overview (if available) (With full archiving support!)
        if let Some(hidden_html_contents) = &self.description {
            // Check that we have a full login (we only need to check, if we can generate a saved page)
            let _ = match config.get_cookie().await {
                Some(cookie) => cookie,
                None => {
                    config.status_bar.register_skipped().await;
                    return Ok(());
                }
            };

            let page_path = &path.join("description");
            let page_url = Url::from_str(&self.url)?;

            let hidden_html_path = path.join(".moo-dl.description.html");

            config
                .save_page_with_extra_file(
                    page_path,
                    &page_url,
                    &hidden_html_path,
                    hidden_html_contents,
                )
                .await?;
        }

        Ok(())
    }
}
