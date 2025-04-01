pub mod downloader;
pub mod queue;

use std::path::PathBuf;

use url::Url;

use crate::config::sync_config::Config;

use super::*;

pub struct YoutubeVideo {
    url: Url,
    output_folder: PathBuf,
}
