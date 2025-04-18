use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use async_channel::{Receiver, Sender};
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::debug;
use url::Url;

use web2pdf_lib::Browser;

use crate::download::youtube::YoutubeVideo;
use crate::status_bar::StatusBar;
use crate::Result;

pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let contents = fs::read_to_string(path).with_context(|| "Failed to read config file")?;
    let config: Config = serde_yml::from_str(&contents)
        .with_context(|| "Could not parse config (There is most likely an error in the config)")?;
    debug!("Read config: {:?}", config);
    Ok(config)
}

/// Top-level configuration structure
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub wstoken: String,
    pub user_id: u64,
    pub login: Login,
    #[serde(skip)]
    pub cookie: Arc<RwLock<LoginState>>,
    pub courses: Vec<Course>,
    pub modules: HashSet<Module>,
    pub grades: bool,
    pub update_strategy: UpdateStrategy,
    pub chrome_executable: Option<PathBuf>,
    #[serde(skip, default)]
    pub chromium: RwLock<ChromiumState>,
    pub sciebo: bool,
    pub youtube: Option<Youtube>,
    #[serde(skip, default)]
    pub youtube_queue: YoutubeQueue,
    pub page_conversion: PageConversion,
    pub dir: Option<PathBuf>,
    pub log_file: Option<PathBuf>,
    pub file_filters: Option<Vec<String>>,
    #[serde(skip)]
    pub status_bar: Arc<StatusBar>,
    #[serde(skip, default = "create_standard_client")]
    pub client: Client,
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
    Graphical {
        url: Url,
    },
    UserPass {
        url: Url,
        username: String,
        password: String,
    },
    Rwth {
        #[serde(skip, default = "rwth_url")]
        url: Url,
        username: String,
        password: String,
        totp: String,
        totp_secret: String,
    },
}
impl Default for Login {
    /// Warning: this is a dummy
    fn default() -> Self {
        let url = Url::from_str("https://dummy.example.com/").unwrap();
        Login::ApiOnly { url }
    }
}
pub fn rwth_url() -> Url {
    Url::from_str("https://moodle.rwth-aachen.de/").unwrap()
}

#[derive(Debug, Default)]
pub enum LoginState {
    #[default]
    NotChecked,
    Unavailable,
    Cookie {
        cookie: Arc<String>,
    },
}

/// Course configuration (currently only room_id is provided).
#[derive(Debug, Deserialize)]
pub struct Course {
    pub id: u64,
    pub name: String,
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
#[derive(Debug, Deserialize, Hash, PartialEq, Eq, Default)]
pub enum UpdateStrategy {
    #[default]
    None,
    Update,
    Archive,
}

#[derive(Debug, Default)]
pub enum ChromiumState {
    #[default]
    NotStarted,
    Unavailable,
    Browser(Browser),
}

// Optional config to enable extraction and download of youtube videos
#[derive(Debug, Deserialize)]
pub struct Youtube {
    pub path: PathBuf,
    pub params: Vec<String>,
    pub parallel_downloads: u32,
}

#[derive(Debug)]
pub struct YoutubeQueue {
    pub sender: Sender<Arc<YoutubeVideo>>,
    pub receiver: Receiver<Arc<YoutubeVideo>>,
}
impl Default for YoutubeQueue {
    fn default() -> Self {
        let (s, r) = async_channel::unbounded::<Arc<YoutubeVideo>>();
        YoutubeQueue {
            sender: s,
            receiver: r,
        }
    }
}

/// Page conversion settings – only one of these should be set.
#[derive(Debug, Deserialize, Default)]
#[serde(tag = "type", content = "path")]
pub enum PageConversion {
    /// Use the single-file tool to convert it to an html-document
    SingleFile(PathBuf),
    /// Store entire file as pdf with a single page
    SinglePage,
    #[default]
    /// Standard chrome pdf conversion
    Standard,
}

fn create_standard_client() -> Client {
    // Don't enable cookie provider as it kills performance
    Client::builder()
        .gzip(true)
        .brotli(true)
        .zstd(true)
        .deflate(true)
        .build()
        .expect("Something went catastrophically wrong, could not create a reqwest client")
}
