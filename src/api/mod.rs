// TODO: remove
#![allow(dead_code)]

// Everthing from here on out should start with api_
pub mod helpers;
pub mod modules;
mod rest;

use std::path::{Path, PathBuf};
use std::fmt::Debug;
use std::sync::Arc;

use serde::Deserialize;
use tracing::debug;

use crate::config::sync_config::Config;

use crate::Result;

pub trait Download: Debug {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()>;
}

impl Config {
    /// Generic function to make a rest api request
    /// # Arguments
    /// * `query` - query parameters
    /// * `T` - type to deserialize to
    pub async fn api_request_json<T>(&self, query: &[(&str, &str)]) -> Result<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        debug!("Rest api request: {:?}", query);
        let response = self
            .client
            .get(format!(
                "{}/webservice/rest/server.php",
                self.get_moodle_url()
            ))
            .query(&[
                ("moodlewsrestformat", "json"),
                ("moodlewssettingraw", "false"),
                ("moodlewssettingfileurl", "true"),
                ("moodlewssettingfilter", "true"),
                ("wstoken", self.wstoken.as_str()),
            ])
            .query(query)
            .send()
            .await?;
        Ok(serde_json::from_str(&response.text().await?)?)
    }
}

/// Assemble a file path from the `api_filepath`, as provided by the api
pub fn assemble_path(path: &Path, api_filepath: &str, filename: &str) -> PathBuf {
    let custom_path = api_filepath.strip_prefix('/').unwrap_or(api_filepath);

    path.join(custom_path).join(filename)
}
