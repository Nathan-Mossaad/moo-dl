// TODO: remove
#![allow(dead_code)]

// Everthing from here on out should start with api_
pub mod helpers;
pub mod modules;
mod rest;

use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use futures::future::join_all;
use rest::CoreCourseGetContentsElement;
use serde::Deserialize;
use tracing::debug;

use crate::config::sync_config::Config;
use crate::config::sync_config::Module as ConfigModule;
use modules::Module;

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

impl Download for CoreCourseGetContentsElement {
    // Downloads only the elements, that are requested in regards to the config
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        // Respect course element names
        let path = path.join(&self.name);

        // Create a task for each element
        let (skip_count, tasks): (u32, Vec<_>) =
            self.modules
                .iter()
                .fold((0, Vec::new()), |(mut skip_count, mut tasks), module| {
                    if let Some(cfg_module) = config_module_of(module) {
                        if config.modules.contains(&cfg_module) {
                            tasks.push(module.download(config.clone(), &path));
                        } else {
                            skip_count += 1;
                        }
                    };

                    (skip_count, tasks)
                });

        // Register the correct amount of skipps
        for _ in 0..skip_count {
            config.status_bar.register_skipped().await;
        }

        // Return an error if one occured
        for res in join_all(tasks).await {
            res.context("Failed Module")?;
        }
        Ok(())
    }
}

// Helper to convert Modules to ConfigModules
fn config_module_of(m: &Module) -> Option<ConfigModule> {
    // We return None if the module is Unknown or unsupported.
    match m {
        Module::Resource(_) => Some(ConfigModule::Resource),
        Module::Folder(_) => Some(ConfigModule::Folder),
        Module::Pdfannotator(_) => Some(ConfigModule::Pdfannotator),
        Module::Assign(_) => Some(ConfigModule::Assign),
        Module::Label(_) => Some(ConfigModule::Label),
        Module::ApiUrl(_) => Some(ConfigModule::Url),
        Module::Page(_) => Some(ConfigModule::Page),
        Module::Quiz(_) => Some(ConfigModule::Quiz),
        Module::Feedback(_) => Some(ConfigModule::Feedback),
        Module::Glossary(_) => Some(ConfigModule::Glossary),
        Module::Vpl(_) => Some(ConfigModule::Vpl),
        Module::Lti(_) => Some(ConfigModule::Lti),
        Module::Forum(_) => Some(ConfigModule::Forum),
        Module::HsuForum(_) => Some(ConfigModule::Hsuforum),
        Module::Grouptool(_) => Some(ConfigModule::Grouptool),
        Module::Unknown => None,
    }
}
