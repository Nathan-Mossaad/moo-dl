use serde::Deserialize;
use tracing::debug;

use crate::Result;

use super::errors::MissingUserIdError;
use super::Api;

// Descriptions taken from generated moodle docs these can be accessed on any moodle instance with administrator rights via: http://example.com/admin/webservice/documentation.php

#[derive(Debug, Deserialize)]
/// Return some site info / user info / list web service functions
pub struct CoreWebserviceGetSiteInfo {
    /// User id
    pub userid: i32,
}

#[derive(Debug, Deserialize)]
// Get the list of courses where a user is enrolled in
pub struct CoreEnrolGetUsersCourses {
    /// id of course
    pub id: i32,
    /// Short name of course
    pub shortname: String,
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
        Ok(response.json::<T>().await?)
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
}
