use std::process::Stdio;

use anyhow::{anyhow, Context};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use url::Url;

use indicatif::ProgressStyle;
use tracing::{instrument, trace, Span};
use tracing_indicatif::span_ext::IndicatifSpanExt;

use web2pdf_lib::{BrowserWeb2Pdf, PageWeb2Pdf};

use crate::config::sync_config::{ChromiumState, Config, PageConversion};

use super::super::*;

impl Config {
    /// Force create a new website save
    /// # Returns
    /// true, if page save has been sucessfull
    /// false, if saving pages is unavailable
    #[instrument(skip(self, file_path, url))]
    pub(in super) async fn force_save_page(&self, file_path: &Path, url: &Url) -> Result<bool> {
        let mut template = "{spinner:.green} [{elapsed_precise}] Creating page: ".to_string();
        template.push_str(file_path.to_str().unwrap_or("Unknown Filename"));
        Span::current().pb_set_style(&ProgressStyle::default_spinner().template(&template)?);

        // Make sure path exists
        ensure_path_exists(&file_path).await?;

        // New path for temporary file
        let tmp_path = file_path.with_extension("tmp_page_bZpbocXJQkxt_moo-dl");
        // Remove old file
        let _ = fs::remove_file(&tmp_path).await;

        let browser_guard = self.get_chromium().await;
        let browser = match &*browser_guard {
            ChromiumState::Unavailable => return Ok(false),
            ChromiumState::Browser(browser) => browser,
            ChromiumState::NotStarted => {
                return Err(anyhow!(
                    "Impossible, got invalid return from self.get_chromium"
                ));
            }
        };

        // Save page using single-file / standard pdf / mono pdf
        match &self.page_conversion {
            PageConversion::SingleFile(single_file_path) => {
                let mut cmd = Command::new(single_file_path);
                cmd.args(&[
                    // Set chrome dev url
                    "--browser-remote-debugging-URL",
                    browser.websocket_address(),
                    // Set url
                    url.as_str(),
                    // Set filename
                    tmp_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?,
                ]);
                // Run
                let mut child = cmd
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .context("Your single-file path is most likely wrong")?;

                // Print single-files output to trace
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();

                    while let Some(line) = lines.next_line().await? {
                        trace!("single-file output: {}", line);
                    }
                }

                let status = child.wait().await?;
                if !status.success() {
                    return Err(anyhow!("Single-file failed with exit code: {}", status));
                }
            }
            crate::config::sync_config::PageConversion::SinglePage => {
                let page = browser
                    .web2pdf_new_page(url.as_str())
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                page.web2pdf_save_pdf_mono_standard(&tmp_path).await?;
            }
            crate::config::sync_config::PageConversion::Standard => {
                let page = browser
                    .web2pdf_new_page(url.as_str())
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                page.web2pdf_save_pdf_standard(&tmp_path).await?;
            }
        }

        // Move file to destination
        fs::rename(tmp_path, file_path).await?;

        Ok(true)
    }
}
