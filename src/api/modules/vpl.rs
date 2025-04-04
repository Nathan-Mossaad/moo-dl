use anyhow::anyhow;
use regex::Regex;
use select::{document::Document, predicate::Name};
use url::Url;

use super::*;

#[derive(Debug, Deserialize)]
pub struct Vpl {
    pub id: u64,
    pub name: String,
    pub url: String,
}
impl Download for Vpl {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        let path = path.join(self.name.to_string() + ".vpl");

        let cookie = match config.get_cookie().await {
            Some(cookie) => cookie,
            None => {
                config.status_bar.register_skipped().await;
                return Ok(());
            }
        };

        // Part 1. Get "static" files

        // Get description files
        let description_file_path = path.join("description.zip");
        let description_files_url =
            self.get_url(&config, "mod/vpl/views/downloadrequiredfiles.php")?;
        let description_request = config
            .client
            .get(description_files_url)
            .header("Cookie", "MoodleSession=".to_string() + &cookie);
        let download_result = config
            .download_file(&description_file_path, description_request)
            .await;
        if let Err(e) = download_result {
            config
                .status_bar
                .register_err(
                    &e.context(format!(
                        "Failed downloading vpl description files: {}",
                        &description_file_path.to_str().unwrap_or("Unavailable")
                    ))
                    .to_string(),
                )
                .await
        }

        // Save pages, as they usually include usefull information (and sometimes feedback, even if it doesn't get synced)
        let description_page_path = path.join("description");
        let submission_page_path = path.join("submission");
        let description_page_url = self.get_url(&config, "mod/vpl/view.php")?;
        let submission_page_url = self.get_url(&config, "mod/vpl/forms/submissionview.php")?;

        config
            .save_page(&description_page_path, &description_page_url)
            .await?;
        config
            .save_page(&submission_page_path, &submission_page_url)
            .await?;

        // Part 2. Get own submission files
        let response = config
            .client
            .get(submission_page_url)
            .header("Cookie", "MoodleSession=".to_string() + &cookie)
            .send()
            .await?
            .text()
            .await?;
        let document = Document::from(response.as_str());

        let url_start = config
            .get_moodle_url()
            .join("mod/vpl/views/downloadsubmission.php")?
            .to_string();
        let url_contains = "id=".to_string() + self.id.to_string().as_str();

        let response_urls = document
            .find(Name("a"))
            .map(|element| element.attr("href"))
            .flatten()
            .filter(|href| href.starts_with(&url_start) && href.contains(&url_contains));

        let submission_futures = response_urls.into_iter().map(|url| {
            let download_path = path.clone();
            let config = config.clone();
            let cookie = cookie.clone();
            async move {
                let regex = Regex::new(r"submissionid=(\d+)").unwrap();
                let submissionid = regex
                    .captures(url)
                    .and_then(|captures| captures.get(1).map(|match_| match_.as_str()))
                    .ok_or(anyhow!("Could not extract submissionid from url"))?;
                let request = config
                    .client
                    .get(url)
                    .header("Cookie", "MoodleSession=".to_string() + &cookie);
                config
                    .download_file(
                        &download_path.join(
                            "submission-files_submissionid-".to_string() + submissionid + ".zip",
                        ),
                        request,
                    )
                    .await
            }
        });

        // Return an error if one occured
        for res in join_all(submission_futures).await {
            res.context("Failed vpl")?;
        }

        Ok(())
    }
}

impl Vpl {
    fn get_url(&self, config: &Arc<Config>, page_path: &str) -> Result<Url> {
        let mut url = config.get_moodle_url().join(page_path)?;
        url.query_pairs_mut()
            .append_pair("id", &self.id.to_string());
        Ok(url)
    }
}
