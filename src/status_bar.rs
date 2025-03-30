use std::{fmt, path::Path};

use chrono::{Local, Utc};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::{info, warn};

extern crate strip_ansi_escapes;

use crate::Result;

#[derive(Debug, Default)]
pub struct StatusBar {
    unchanged: usize,
    skipped: usize,
    updated: usize,
    new: usize,
    err: usize,
    log: Vec<String>,
}

impl fmt::Display for StatusBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unchanged \x1b[90m{}\x1b[0m / Skipped \x1b[33m{}\x1b[0m / Updated \x1b[34m{}\x1b[0m / New \x1b[32m{}\x1b[0m / Err \x1b[31m{}\x1b[0m",
            self.unchanged, self.skipped, self.updated, self.new, self.err
        )
    }
}

impl StatusBar {
    fn get_current_time() -> String {
        Utc::now().with_timezone(&Local).to_rfc3339()
    }

    fn create_log_entry<'a>(&mut self, message: impl Into<&'a str>, log_type: &str) -> String {
        // Create log entry contents for both tracing and log file
        let mut log_entry_contents = log_type.to_string();
        log_entry_contents.push_str(message.into());

        // Create entry for log file
        let mut log_entry = StatusBar::get_current_time();
        log_entry.push_str(" ");
        log_entry.push_str(&log_entry_contents);
        self.log.push(log_entry.clone());

        log_entry_contents
    }

    pub fn register_unchanged(&mut self) {
        self.unchanged += 1;
    }
    pub fn register_skipped(&mut self) {
        self.skipped += 1;
    }
    pub fn register_updated<'a>(&mut self, message: impl Into<&'a str>) {
        self.updated += 1;
        let entry = self.create_log_entry(message, "\x1b[34mUpdated\x1b[0m: ");
        info!("{}", entry);
    }
    pub fn register_new<'a>(&mut self, message: impl Into<&'a str>) {
        self.new += 1;
        let entry = self.create_log_entry(message, "\x1b[32mNew\x1b[0m: ");
        info!("{}", entry);
    }
    pub fn register_err<'a>(&mut self, message: impl Into<&'a str>) {
        self.err += 1;
        let entry = self.create_log_entry(message, "\x1b[31mErr\x1b[0m: ");
        warn!("{}", entry);
    }

    // Appends contents of self.log to a log file
    pub async fn write_log_to_file(&self, file_path: &Path) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .await?;

        let mut buffer = Vec::new();

        for log_entry in &self.log {
            buffer.extend_from_slice(&strip_ansi_escapes::strip(&log_entry.as_bytes()));
            buffer.extend_from_slice(b"\n");
        }

        buffer.extend_from_slice(b"Total: ");
        buffer.extend_from_slice(&strip_ansi_escapes::strip(self.to_string().as_bytes()));
        buffer.extend_from_slice(b"     (Log generated at: ");
        buffer.extend_from_slice(StatusBar::get_current_time().as_bytes());
        buffer.extend_from_slice(b")\n\n");

        file.write_all(&buffer).await?;
        Ok(())
    }
}
