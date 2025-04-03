use std::path::Path;

use anyhow::Context;
use futures::future::join_all;

use crate::{api::Download, config::sync_config::Course};

use super::*;

impl Config {
    async fn download_course(config: Arc<Config>, path: &Path, course: &Course) -> Result<()> {
        let path = path.join(&course.name);
        // We create a new Arc for each Course to reduce the number of threads accessing the same Arc

        let course_elements = config
            .api_core_course_get_contents(course.id)
            .await
            .context("Failed getting course elements!")?;

        // Create a task for each content
        let tasks = course_elements.iter().map(|r| r.download(config.clone(), &path));
        // Return an error if one occured
        for res in join_all(tasks).await {
            let context = format!("Failure in course: {}", course.id);
            res.context(context)?;
        }

        Ok(())
    }
    pub async fn download_courses(config: Arc<Config>, path: &Path) {
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
