use std::process::Stdio;

use anyhow::{anyhow, Context};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use indicatif::ProgressStyle;
use tracing::{debug, instrument, trace, Span};
use tracing_indicatif::span_ext::IndicatifSpanExt;

use super::*;

use crate::config::sync_config::{UpdateStrategy, Youtube};
use crate::update::UpdateState;

impl Youtube {
    /// Downloads a video using yt-dlp and displays a progress bar.
    /// Provide a folder path instead of a filename, as yt-dlp chooses the file name
    ///
    /// Don't use directly, use the youtube download queue
    #[instrument(skip(self, url, output))]
    async fn force_download_youtube(&self, url: &str, output: &OutputType) -> Result<()> {
        // Make sure path exists
        ensure_path_exists(output.path()).await?;
        let ouput_path = match &output {
            OutputType::Folder(path_buf) => &path_buf.join("%(title)s [%(id)s].%(ext)s"),
            OutputType::File(path_buf) => path_buf,
        };

        let mut cmd = Command::new(&self.path);
        cmd.args(&[
            // Force new lines
            "--newline",
            // Get all available variables using: yt-dlp --progress-template '%(progress)#j'
            "--progress-template",
            "%(progress)#j",
            // Disable colors for easier parsing
            "--color",
            "no_color",
        ])
        .args(&self.params)
        // Set output
        .arg("-o")
        .arg(ouput_path.to_str().context("Invalid output path")?)
        .arg(url);

        debug!("yt-dlp params: {:?}", cmd);

        // Spawn the process
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start yt-dlp")?;

        // Prepare template
        let mut template = "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {percent:.1f}% ({eta})  Video: ".to_string();
        template.push_str(ouput_path.to_str().unwrap_or("Unknown file"));
        if let OutputType::Folder(_) = output {
            template.push_str(" Url: ");
            template.push_str(url);
        }

        Span::current().pb_set_style(
            &ProgressStyle::default_bar()
                .template(&template)?
                .progress_chars("#>-"),
        );
        Span::current().pb_set_length(1000);
        Span::current().pb_set_position(0);

        // Process yt-dlp's stdout line by line.
        let mut percent_extractor = PercentStrExtractor::default();
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Some(line) = lines.next_line().await? {
                percent_extractor.push_new_line(line);
                // Set to bar percent_extractor.percentage (is a f32)
                Span::current().pb_set_position((percent_extractor.percentage * 10.0) as u64);
            }
        }

        // Wait for yt-dlp to finish
        let status = child
            .wait()
            .await
            .context("yt-dlp process encountered an error")?;
        if !status.success() {
            return Err(anyhow!("yt-dlp exited with status: {}", status));
        }

        Ok(())
    }
}

impl Config {
    /// Same as `force_download_file` but only downloads if file does not exist
    /// Additionally writes the event to log
    ///
    /// Don't use directly, use the youtube download queue
    pub(super) async fn direct_download_youtube(
        &self,
        url: &Url,
        output: &OutputType,
    ) -> Result<()> {
        match UpdateStrategy::youtube_check_exists(url, output).await? {
            UpdateState::Missing => {
                match &self.youtube {
                    Some(yt) => {
                        yt.force_download_youtube(url.as_str(), output).await?;
                        let message_path = output
                            .path()
                            .to_str()
                            .ok_or_else(|| anyhow!("Invalid path"))?;
                        let message = format!("Video: {} with {} ", message_path, url.as_str());
                        self.status_bar.register_new(&message).await;
                    }
                    None => {
                        self.status_bar.register_skipped().await;
                    }
                }
                Ok(())
            }
            UpdateState::OutOfDate => Err(anyhow!("Impssossible OutOfDate")),
            UpdateState::UpToDate => {
                self.status_bar.register_unchanged().await;
                Ok(())
            }
        }
    }
}

/// Helper to extract the percentage from the yt-dlp output
#[derive(Debug, Default)]
struct PercentStrExtractor {
    pub percentage: f32,
    current_string: String,
}
impl PercentStrExtractor {
    fn push_new_line(&mut self, line: String) {
        if line == "{" {
            self.current_string = line;
            return;
        } else if line == "}" {
            self.current_string.push_str(&line);
            tracing::trace!("{}", self.current_string);

            let value: Value = match serde_json::from_str(&self.current_string) {
                Ok(val) => val,
                Err(_) => return,
            };
            let mut percent_string: String = match value.get("_percent_str") {
                Some(val) => val.to_string(),
                None => return,
            }
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '"')
            .collect();
            percent_string.pop();

            trace!("Extracted percentage: {}", percent_string);
            self.percentage = match percent_string.parse() {
                Ok(val) => {
                    if ((val - self.percentage) as f32).abs() < 50.0 {
                        self.percentage.max(val)
                    } else {
                        val
                    }
                }
                Err(_) => return,
            };
        } else {
            self.current_string.push_str(&line);
        }
    }
}
