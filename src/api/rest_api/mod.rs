use course_modules::Module;
use serde::Deserialize;
use tracing::debug;

use crate::Result;

use super::errors::MissingUserIdError;
use super::Api;

pub mod course_modules;

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
#[derive(Debug, Deserialize)]
pub struct CoreCourseGetContents(Vec<CoreCourseGetContentsElement>);
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CoreCourseGetContentsElement {
    pub id: u64,
    pub name: String,
    pub modules: Vec<Module>,
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
                self.credential.instance_url
            ))
            .query(&[
                ("moodlewsrestformat", "json"),
                ("moodlewssettingraw", "false"),
                ("moodlewssettingfileurl", "true"),
                ("moodlewssettingfilter", "true"),
                ("wstoken", self.credential.wstoken.as_str()),
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
        let mut query = vec![(
            "wsfunction".to_string(),
            "tool_mobile_call_external_functions".to_string(),
        )];
        for (pos, course) in courses.iter().enumerate() {
            query.extend([
                (format!("requests[{}][settingraw]", pos), "0".to_string()),
                (format!("requests[{}][settingfilter]", pos), "1".to_string()),
                (
                    format!("requests[{}][settingfileurl]", pos),
                    "1".to_string(),
                ),
                (
                    format!("requests[{}][function]", pos),
                    "core_course_get_contents".to_string(),
                ),
                (
                    format!("requests[{}][arguments]", pos),
                    format!("{{\"courseid\":\"{}\"}}", course),
                ),
            ]);
        }
        let query: Vec<(&str, &str)> = query
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let responses = self
            .rest_api_request_json::<ToolMobileCallExternalFunctions>(&query)
            .await?;
        let mut result = vec![];
        for response in responses.responses {
            let course_contents = serde_json::from_str::<CoreCourseGetContents>(&response.data)?;
            result.push(course_contents);
        }
        Ok(result)
    }
}
