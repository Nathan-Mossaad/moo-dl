use tracing::debug;

use std::path::Path;

use futures::future::join_all;
use serde::Deserialize;

use crate::Result;

use super::errors::MissingUserIdError;
use super::Api;

pub mod course_modules;
use course_modules::{Download, Module};

// Descriptions taken from generated moodle docs these can be accessed on any moodle instance with administrator rights via: http://example.com/admin/webservice/documentation.php
#[derive(Debug, Deserialize)]
/// Return some site info / user info / list web service functions
pub struct CoreWebserviceGetSiteInfo {
    /// User id
    pub userid: u64,
}

// TODO remove dead_code warning
#[allow(dead_code)]
// Descriptions taken from generated moodle docs these can be accessed on any moodle instance with administrator rights via: http://example.com/admin/webservice/documentation.php
#[derive(Debug, Deserialize)]
/// Get the list of courses where a user is enrolled in
pub struct CoreEnrolGetUsersCourses {
    /// id of course
    pub id: u64,
    /// Short name of course
    pub shortname: String,
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
// This allows us to do multiple requests at once
pub struct ToolMobileCallExternalFunctions {
    pub responses: Vec<ToolMobileCallExternalFunctionsResponse>,
}
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ToolMobileCallExternalFunctionsResponse {
    pub data: String,
}
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct CoreCourseGetContents(Vec<CoreCourseGetContentsElement>);
impl Download for CoreCourseGetContents {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        let file_futures = self.0.iter().map(|element| element.download(api, &path));
        let downloads = join_all(file_futures).await;
        // Return error if any download fails
        for download in downloads {
            download?;
        }
        Ok(())
    }
}
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct CoreCourseGetContentsElement {
    pub id: u64,
    pub name: String,
    pub modules: Vec<Module>,
}
impl Download for CoreCourseGetContentsElement {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        let download_path = path.join(&self.name);
        let file_futures = self
            .modules
            .iter()
            .map(|module| module.download(api, &download_path));
        let downloads = join_all(file_futures).await;
        // Return error if any download fails
        for download in downloads {
            download?;
        }
        Ok(())
    }
}

impl Api {
    /// Generic function to make a rest api request
    /// # Arguments
    /// * `query` - query parameters
    /// * `T` - type to deserialize to
    pub async fn rest_api_request_json<T>(&self, query: &[(&str, &str)]) -> Result<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        debug!("Rest api request: {:?}", query);
        let response = self
            .client
            .get(format!(
                "{}/webservice/rest/server.php",
                self.api_credential.instance_url
            ))
            .query(&[
                ("moodlewsrestformat", "json"),
                ("moodlewssettingraw", "false"),
                ("moodlewssettingfileurl", "true"),
                ("moodlewssettingfilter", "true"),
                ("wstoken", self.api_credential.wstoken.as_str()),
            ])
            .query(query)
            .send()
            .await?;
        Ok(serde_json::from_str(&response.text().await?)?)
    }

    pub async fn get_core_webservice_get_site_info(&self) -> Result<CoreWebserviceGetSiteInfo> {
        Ok(self
            .rest_api_request_json::<CoreWebserviceGetSiteInfo>(&[(
                "wsfunction",
                "core_webservice_get_site_info",
            )])
            .await?)
    }

    pub async fn core_enrol_get_users_courses(&self) -> Result<Vec<CoreEnrolGetUsersCourses>> {
        Ok(self
            .rest_api_request_json::<Vec<CoreEnrolGetUsersCourses>>(&[
                ("wsfunction", "core_enrol_get_users_courses"),
                (
                    "userid",
                    &self.user_id.ok_or(MissingUserIdError)?.to_string(),
                ),
            ])
            .await?)
    }

    pub async fn core_course_get_contents(&self, course_id: u64) -> Result<CoreCourseGetContents> {
        Ok(self
            .rest_api_request_json::<CoreCourseGetContents>(&[
                ("wsfunction", "core_course_get_contents"),
                ("courseid", &course_id.to_string()),
            ])
            .await?)
    }

    /// Same as core_course_get_contents but with multiple courses
    pub async fn core_course_get_contents_mult(
        &self,
        courses: Vec<u64>,
    ) -> Result<Vec<CoreCourseGetContents>> {
        // Don't use grouped requests are, as parallel single requests are faster
        let courses_futures: Vec<_> = courses
            .iter()
            .map(|c| self.core_course_get_contents(*c))
            .collect();

        let responses = join_all(courses_futures).await;

        let mut result = vec![];
        for response in responses {
            match response {
                Ok(course_contents) => {
                    result.push(course_contents);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(result)
    }
}
