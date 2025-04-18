pub mod downloader;

use std::{path::PathBuf, str::FromStr, sync::Arc};

use anyhow::Context;
use futures::future::join_all;
use once_cell::sync::Lazy;
use percent_encoding::percent_decode;
use regex::{Captures, Regex};
use url::Url;

use super::*;

static RE_SCIEBO: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"https:\/\/[a-zA-Z0-9-]+\.sciebo\.de\/s\/[a-zA-Z0-9-]+"#).unwrap());

impl Config {
    /// Extracts Sciebo URLs from the given `search_space` and downloads them.
    pub async fn extract_sciebo_download(
        config: Arc<Config>,
        search_space: &str,
        output_folder: PathBuf,
    ) -> Result<()> {
        if !config.sciebo {
            config.status_bar.register_skipped().await;
            return Ok(());
        }
        tracing::trace!("Looking for sciebo links in: {}", search_space);

        let re = &RE_SCIEBO;
        // Iterate over every match in the search_space.
        let tasks = re
            .captures_iter(search_space)
            .map(|cap| process_capture(config.clone(), &output_folder, cap));

        // Return an error if one occured
        for res in join_all(tasks).await {
            res.context("Failed sciebo download")?;
        }
        Ok(())
    }
}
async fn process_capture(
    config: Arc<Config>,
    output_folder: &Path,
    cap: Captures<'_>,
) -> Result<()> {
    if let Some(url_str) = cap.get(0) {
        let url_value = url_str.as_str();

        // Validate and parse the url into the Url struct.
        if let Ok(parsed_url) = Url::parse(url_value) {
            tracing::trace!("Sciebo URL: {:?}", &parsed_url);
            // Start download
            // We can use .unwrap(), as the regex insures, that the string has both a host and an id
            let url = Url::from_str(&format!(
                "https://{}/public.php/webdav/",
                parsed_url.host().unwrap()
            ))?;
            let username = final_url_segment(parsed_url.as_str()).unwrap();
            let pass = None;
            Config::download_webdav(
                config,
                output_folder,
                &url,
                Some(&parsed_url),
                &username,
                pass,
            )
            .await?;
        }
    }
    Ok(())
}

fn final_url_segment(href: &str) -> Option<String> {
    // Parse the URL, trying with the original href first, and then with a dummy scheme/domain if needed
    let url = Url::parse(href)
        .or_else(|_| Url::parse(&format!("http://dummy{}", href)))
        .ok()?;

    url.path_segments()?
        // Filter out empty ones
        .filter(|segment| !segment.is_empty())
        .last()
        // Decode percent encoding
        .map(|s| {
            percent_decode(s.as_bytes())
                .decode_utf8_lossy()
                .into_owned()
        })
}
