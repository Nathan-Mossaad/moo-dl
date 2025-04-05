pub mod downloader;
pub mod queue;

use std::path::PathBuf;

use url::Url;

use crate::config::sync_config::Config;

use super::*;

pub struct YoutubeVideo {
    url: Url,
    output: OutputType,
}

pub enum OutputType {
    Folder(PathBuf),
    File(PathBuf),
}
impl OutputType {
    pub fn path(&self) -> &PathBuf {
        match &self {
            OutputType::Folder(path_buf) => path_buf,
            OutputType::File(path_buf) => path_buf,
        }
    }
}
