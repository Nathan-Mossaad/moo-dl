use std::str::FromStr;

use anyhow::anyhow;
use regex::Regex;
use select::{document::Document, predicate::Name};
use tracing::trace;
use url::Url;

use super::*;

#[derive(Debug, Deserialize)]
pub struct Quiz {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub instance: u64,
}

impl Download for Quiz {
    // Warning this downloader requires scraping and therefore is increadibly slow!
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        let path = path.join(&self.name);

        let cookie = match config.get_cookie().await {
            Some(cookie) => cookie,
            None => {
                config.status_bar.register_skipped().await;
                return Ok(())
            },
        };
        trace!(
            "Attempting to get available quiz attempts for quiz id: {} name: {}",
            self.id,
            self.name
        );

        let response = config
            .client
            .get(&self.url)
            .header(
                "Cookie",
                "MoodleSession=".to_string() + &cookie,
            )
            .send()
            .await?;
        
        let html = response.text().await?;
        let document = Document::from(html.as_str());

        let url_start = config.get_moodle_url().to_string() + "mod/quiz/review.php";
        let url_contains = "cmid=".to_string() + self.id.to_string().as_str();

        let response_url = document
            .find(Name("a"))
            .map(|element| element.attr("href"))
            .flatten()
            .filter(|href| href.starts_with(&url_start) && href.contains(&url_contains));

        let tasks = response_url.map(|attempt_url| {
            // trace!("Quiz: Response token: {:?}", url);
            let config = config.clone();
            let path = path.clone();
            async move {
                let regex = Regex::new(r"attempt=(\d+)").unwrap();
                let attemptnr = regex
                    .captures(attempt_url)
                    .and_then(|captures| captures.get(1).map(|match_| match_.as_str()))
                    .ok_or(anyhow!("Could not extract attemptnr from url"))?;
                let attempt_path = path.join(attemptnr.to_string());
                config.save_page(
                    &attempt_path,
                    &Url::from_str(attempt_url)?,
                )
                .await
            }
        });

        // Return an error if one occured
        for res in join_all(tasks).await {
            res.context("Failed Resource")?;
        }
        
        Ok(())
    }
}
