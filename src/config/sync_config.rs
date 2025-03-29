// TODO: remove
#![allow(dead_code)]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Deserialize;
use tracing::debug;
use url::Url;

use crate::Result;

pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let contents = fs::read_to_string(path).with_context(|| "Failed to read config file")?;
    let config: Config = serde_yml::from_str(&contents)
        .with_context(|| "Could not parse config (There is most likely an error in the config)")?;
    debug!("Read config: {:?}", config);
    Ok(config)
}

/// Top-level configuration structure
#[derive(Debug, Deserialize)]
pub struct Config {
    pub wstoken: String,
    pub login: Login,
    pub courses: Vec<Course>,
    pub modules: HashSet<Module>,
    pub points: bool,
    pub update_strategy: UpdateStrategy,
    pub chrome_executable: Option<PathBuf>,
    pub youtube: Option<Youtube>,
    pub page_conversion: PageConversion,
    pub dir: Option<String>,
    pub file_filters: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Login {
    // Provide api capabilities only
    ApiOnly {
        url: Url,
    },
    Raw {
        url: Url,
        cookie: String,
    },
    UserPass {
        url: Url,
        username: String,
        password: String,
    },
    Graphical {
        url: Url,
    },
    Rwth {
        username: String,
        password: String,
        totp: String,
        totp_secret: String,
    },
}

/// Course configuration (currently only room_id is provided).
#[derive(Debug, Deserialize)]
pub struct Course {
    id: u64,
    name: String,
}

// List of supported modules
#[derive(Debug, Deserialize, Hash, PartialEq, Eq)]
pub enum Module {
    Resource,
    Folder,
    Pdfannotator,
    Assign,
    Label,
    Url,
    Page,
    Quiz,
    Feedback,
    Glossary,
    Vpl,
    Lti,
    Forum,
    Hsuforum,
    Grouptool,
}

// Update strategy
#[derive(Debug, Deserialize, Hash, PartialEq, Eq)]
pub enum UpdateStrategy {
    None,
    Update,
    Archive,
}

// Optional config to enable extraction and download of youtube videos
#[derive(Debug, Deserialize)]
pub struct Youtube {
    path: PathBuf,
    params: String,
    parallel_downloads: u32,
}

/// Page conversion settings – only one of these should be set.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "path")]
pub enum PageConversion {
    /// Use the single-file tool to convert it to an html-document
    SingleFile(PathBuf),
    /// Store entire file as pdf with a single page
    SinglePage,
    /// Standard chrome pdf conversion
    Standard,
}
