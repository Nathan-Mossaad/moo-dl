use std::path::Path;

use anyhow::Context;
use futures::future::join_all;

use indicatif::ProgressStyle;
use tracing::{instrument, Span};
use tracing_indicatif::span_ext::IndicatifSpanExt;

use crate::{api::Download, config::sync_config::Course};

use super::*;

impl Config {
    async fn download_course(config: Arc<Config>, path: &Path, course: &Course) -> Result<()> {
        let path = path.join(&course.name);

        let grade_config = config.clone();
        let grade_path = path.clone();
        let grade_course_id = course.id;
        let grade_handle = tokio::spawn(async move {
            grade_config
                .save_grades_table(&grade_path, grade_course_id)
                .await;
        });

        let context = format!("Failed getting course elements! Course: {}", &course.name);

        let course_elements = config
            .api_core_course_get_contents(course.id)
            .await
            .context(context)?;

        // Create a task for each content
        let tasks = course_elements
            .iter()
            .map(|r| r.download(config.clone(), &path));
        let res = join_all(tasks).await;

        // Wait for grades to be saved
        let _ = grade_handle.await;

        // Return an error if one occured
        for res in res {
            let context = format!("Failure in course: {}", course.id);
            res.context(context)?;
        }

        Ok(())
    }
    #[instrument(skip(config, path))]
    pub async fn download_courses(config: Arc<Config>, path: &Path) {
        let template = "{spinner:.green} [{elapsed_precise}] Checking for updates ... ".to_string();
        Span::current().pb_set_style(
            &ProgressStyle::default_spinner()
                .template(&template)
                .unwrap(),
        );

        // Create a task for each course
        let tasks = config
            .courses
            .iter()
            .map(|course| Config::download_course(config.clone(), path, course));
        // Print that an error occured in specific course
        for res in join_all(tasks).await {
            if let Err(e) = res {
                config.status_bar.register_err(&e.to_string()).await;
            }
        }
    }
}
