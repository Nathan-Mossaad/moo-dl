use anyhow::ensure;
use reqwest::header::COOKIE;

use super::*;
use crate::{config::sync_config::UpdateStrategy, update::UpdateState};

#[derive(Debug, Deserialize)]
pub struct Scorm {
    name: String,
    #[serde(rename = "contextid")]
    context_id: u64,
}

impl Download for Scorm {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        let file_path = path.join(&self.name).join("index.html");
        if let Err(error) = self.download_index(&config, &file_path).await {
            config
                .status_bar
                .register_err(&format!(
                    "Failed downloading SCORM activity {}: {error:#}",
                    self.name
                ))
                .await;
        }
        Ok(())
    }
}

impl Scorm {
    async fn download_index(&self, config: &Config, file_path: &Path) -> Result<()> {
        if UpdateStrategy::check_exists(file_path).await? == UpdateState::UpToDate {
            config.status_bar.register_unchanged().await;
            return Ok(());
        }

        let Some(cookie) = config.get_cookie().await else {
            config.status_bar.register_skipped().await;
            return Ok(());
        };

        let url = config.get_moodle_url().join(&format!(
            "pluginfile.php/{}/mod_scorm/content/100000/index.html",
            self.context_id
        ))?;
        let response = config
            .client
            .get(url.clone())
            .header(COOKIE, format!("MoodleSession={cookie}"))
            .send()
            .await?
            .error_for_status()?;
        ensure!(
            response.status().is_success()
                && response.url().origin() == url.origin()
                && response.url().path() == url.path(),
            "Unexpected response while downloading index.html (HTTP {})",
            response.status()
        );

        let html = response.text().await?;
        config.write_file_contents(file_path, &html).await
    }
}
