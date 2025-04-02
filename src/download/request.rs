use std::path::Path;

use anyhow::anyhow;
use indicatif::ProgressStyle;
use reqwest::RequestBuilder;
use tokio::{fs, fs::File, io::AsyncWriteExt};
use tokio_stream::StreamExt;
use tracing::{instrument, Span};
use tracing_indicatif::span_ext::IndicatifSpanExt;

use super::*;

use crate::{
    config::sync_config::{Config, UpdateStrategy},
    update::{timestamp::set_file_creation, UpdateState},
    Result,
};

/// Downloads a file from the specified URL asynchronously with a progress bar.
/// RequestBuilder should be created from a Client::get(url) call.
///
/// Uses a temporary file for downloads to prevent data loss in case of UpdateStrategy:Update
#[instrument(skip(file_path, request))]
async fn force_download_file(file_path: &Path, request: RequestBuilder) -> Result<()> {
    // Make sure path exists
    ensure_path_exists(file_path).await?;

    // New path for temporary file
    let tmp_path = file_path.with_extension("tmp_bZpbocXJQkxt_moo-dl");

    // Send request and get response
    let response = request.send().await?;
    let total_size = response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|ct_len| ct_len.to_str().ok())
        .and_then(|ct_len| ct_len.parse::<u64>().ok());

    // Animation
    if let Some(total_size) = total_size {
        let mut template = "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) File: ".to_string();
        let filename = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown Filename");
        template.push_str(filename);
        Span::current().pb_set_style(
            &ProgressStyle::default_bar()
                .template(&template)?
                .progress_chars("#>-"),
        );
        Span::current().pb_set_length(total_size);
    } else {
        let mut template = "{spinner:.green} [{elapsed_precise}] {bytes} File: ".to_string();
        let filename = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown Filename");
        template.push_str(filename);
        Span::current().pb_set_style(
            &ProgressStyle::default_spinner()
                .template(&template)?,
        );
    }
    Span::current().pb_set_position(0);
    let mut downloaded: u64 = 0;

    // Download file
    let mut file = File::create(&tmp_path).await?;
    let mut stream = response.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
        // Increment bar
        downloaded += chunk.len() as u64;
        Span::current().pb_set_position(downloaded);
    }
    file.flush().await?;

    // Move file to destination
    fs::rename(tmp_path, file_path).await?;

    Ok(())
}

impl Config {
    /// Same as `force_download_file` but only downloads if file does not exist
    /// Additionally writes the event to log
    pub async fn download_file(&self, file_path: &Path, request: RequestBuilder) -> Result<()> {
        match UpdateStrategy::check_exists(file_path).await? {
            UpdateState::Missing => {
                force_download_file(file_path, request).await?;
                let message = file_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;
                self.status_bar.register_new(message).await;
                Ok(())
            }
            UpdateState::OutOfDate => Err(anyhow!("Impssossible OutOfDate")),
            UpdateState::UpToDate => {
                self.status_bar.register_unchanged().await;
                Ok(())
            }
        }
    }

    /// Same as `download_file` utilises the given time for updating / archiving (if requested)
    /// Additionally writes the event to log
    pub async fn download_file_with_timestamp(
        &self,
        file_path: &Path,
        request: RequestBuilder,
        timestamp: u64,
    ) -> Result<()> {
        match self
            .update_strategy
            .timestamp_check_up_to_date(file_path, timestamp)
            .await?
        {
            UpdateState::Missing => {
                force_download_file(file_path, request).await?;
                set_file_creation(file_path, timestamp).await?;

                let message = file_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;
                self.status_bar.register_new(message).await;
                Ok(())
            }
            UpdateState::OutOfDate => {
                force_download_file(file_path, request).await?;
                set_file_creation(file_path, timestamp).await?;

                let message = file_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;
                self.status_bar.register_updated(message).await;
                Ok(())
            }
            UpdateState::UpToDate => {
                self.status_bar.register_unchanged().await;
                Ok(())
            }
        }
    }
}
