use super::{assign_submission::ModAssignGetSubmissionStatus, modules::Module, *};

// Descriptions taken from generated moodle docs these can be accessed on any moodle instance with administrator rights via: http://example.com/admin/webservice/documentation.php
#[derive(Debug, Deserialize)]
/// Return some site info / user info / list web service functions
pub(super) struct CoreWebserviceGetSiteInfo {
    /// User id
    pub userid: u64,
}

// Descriptions taken from generated moodle docs these can be accessed on any moodle instance with administrator rights via: http://example.com/admin/webservice/documentation.php
#[derive(Debug, Deserialize)]
/// Get the list of courses where a user is enrolled in
pub struct CoreEnrolGetUsersCourses {
    /// id of course
    pub id: u64,
    /// Short name of course
    pub shortname: String,
}

#[derive(Debug, Deserialize)]
pub struct CoreCourseGetContentsElement {
    pub id: u64,
    pub name: String,
    pub modules: Vec<Module>,
}

impl Config {
    pub(super) async fn core_webservice_get_site_info(&self) -> Result<CoreWebserviceGetSiteInfo> {
        Ok(self
            .api_request_json::<CoreWebserviceGetSiteInfo>(&[(
                "wsfunction",
                "core_webservice_get_site_info",
            )])
            .await?)
    }

    pub(super) async fn core_enrol_get_users_courses(
        &self,
    ) -> Result<Vec<CoreEnrolGetUsersCourses>> {
        Ok(self
            .api_request_json::<Vec<CoreEnrolGetUsersCourses>>(&[
                ("wsfunction", "core_enrol_get_users_courses"),
                ("userid", &self.user_id.to_string()),
            ])
            .await?)
    }

    // TODO: change to pub(super) and implement download module in helpers
    pub async fn api_core_course_get_contents(
        &self,
        course_id: u64,
    ) -> Result<Vec<CoreCourseGetContentsElement>> {
        Ok(self
            .api_request_json::<Vec<CoreCourseGetContentsElement>>(&[
                ("wsfunction", "core_course_get_contents"),
                ("courseid", &course_id.to_string()),
            ])
            .await?)
    }

    pub(super) async fn mod_assign_get_submission_status(
        &self,
        assignid: u64,
    ) -> Result<ModAssignGetSubmissionStatus> {
        Ok(self
            .api_request_json::<ModAssignGetSubmissionStatus>(&[
                ("wsfunction", "mod_assign_get_submission_status"),
                ("assignid", &assignid.to_string()),
            ])
            .await?)
    }

    pub(super) async fn gradereport_user_get_grades_table(&self, course_id: u64) -> Result<String> {
        debug!(
            "Get gradereport_user_get_grades_table for id: {}",
            course_id
        );
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
                ("wsfunction", "gradereport_user_get_grades_table"),
                ("courseid", &course_id.to_string()),
                ("userid", &self.user_id.to_string()),
            ])
            .send()
            .await?;
        Ok(response.text().await?)
    }
}
