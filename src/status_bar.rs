// TODO: remove
#![allow(dead_code)]

use std::path::Path;

use chrono::{Local, Utc};
use strip_ansi_escapes;
use tokio::sync::Mutex;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::{error, info};

use crate::Result;

#[derive(Debug, Default)]
pub struct StatusBar {
    unchanged: Mutex<usize>,
    skipped: Mutex<usize>,
    updated: Mutex<usize>,
    new: Mutex<usize>,
    err: Mutex<usize>,
    log: Mutex<Vec<String>>,
}

impl StatusBar {
    pub async fn get_overview(&self) -> String {
        let unchanged = self.unchanged.lock().await;
        let skipped = self.skipped.lock().await;
        let updated = self.updated.lock().await;
        let new = self.new.lock().await;
        let err = self.err.lock().await;

        format!(
            "Unchanged \x1b[90m{}\x1b[0m / Skipped \x1b[33m{}\x1b[0m / Updated \x1b[34m{}\x1b[0m / New \x1b[32m{}\x1b[0m / Err \x1b[31m{}\x1b[0m",
            unchanged, skipped, updated, new, err
        )
    }

    fn get_current_time() -> String {
        Utc::now().with_timezone(&Local).to_rfc3339()
    }

    async fn create_log_entry<'a>(&self, message: impl Into<&'a str>, log_type: &str) -> String {
        let mut log_entry_contents = log_type.to_string();
        log_entry_contents.push_str(message.into());

        let mut log = self.log.lock().await;
        let mut log_entry = StatusBar::get_current_time();
        log_entry.push_str(" ");
        log_entry.push_str(&log_entry_contents);
        log.push(log_entry.clone());

        log_entry_contents
    }

    pub async fn register_unchanged(&self) {
        let mut unchanged = self.unchanged.lock().await;
        *unchanged += 1;
    }

    pub async fn register_skipped(&self) {
        let mut skipped = self.skipped.lock().await;
        *skipped += 1;
    }

    pub async fn register_updated<'a>(&self, message: impl Into<&'a str>) {
        let mut updated = self.updated.lock().await;
        *updated += 1;
        let entry = self
            .create_log_entry(message, "\x1b[34mUpdated\x1b[0m: ")
            .await;
        info!("{}", entry);
    }

    pub async fn register_new<'a>(&self, message: impl Into<&'a str>) {
        let mut new = self.new.lock().await;
        *new += 1;
        let entry = self.create_log_entry(message, "\x1b[32mNew\x1b[0m: ").await;
        info!("{}", entry);
    }

    pub async fn register_err(&self, message: &str) {
        let mut err = self.err.lock().await;
        *err += 1;
        let entry = self.create_log_entry(message, "\x1b[31mErr\x1b[0m: ").await;
        error!("{}", entry);
    }

    // Appends contents of self.log to a log file
    pub async fn write_log_to_file(&self, file_path: &Path) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .await?;

        let mut buffer = Vec::new();

        {
            let self_log = self.log.lock().await;
            for log_entry in &*self_log {
                buffer.extend_from_slice(&strip_ansi_escapes::strip(&log_entry.as_bytes()));
                buffer.extend_from_slice(b"\n");
            }
        }

        buffer.extend_from_slice(b"Total: ");
        buffer.extend_from_slice(&strip_ansi_escapes::strip(
            self.get_overview().await.as_bytes(),
        ));
        buffer.extend_from_slice(b"     (Log generated at: ");
        buffer.extend_from_slice(StatusBar::get_current_time().as_bytes());
        buffer.extend_from_slice(b")\n\n");

        file.write_all(&buffer).await?;
        file.flush().await?;
        Ok(())
    }
}
