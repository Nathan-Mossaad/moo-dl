use super::{rest::CoreEnrolGetUsersCourses, *};

impl Config {
    /// Get the user id of the current user
    pub async fn api_acquire_user_id(&self) -> Result<u64> {
        let site_info = self.core_webservice_get_site_info().await?;
        Ok(site_info.userid)
    }
    
    /// Get the users' currently enrolled courses
    pub async fn api_acquire_users_courses(&self, user_id: u64) -> Result<Vec<CoreEnrolGetUsersCourses>> {
        let courses = self.core_enrol_get_users_courses(user_id).await?;
        Ok(courses)
    }
}



