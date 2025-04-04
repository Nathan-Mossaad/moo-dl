use super::{rest::CoreEnrolGetUsersCourses, *};

impl Config {
    /// Get the user id of the current user
    pub async fn api_acquire_user_id(&self) -> Result<u64> {
        let site_info = self.core_webservice_get_site_info().await?;
        Ok(site_info.userid)
    }

    /// Get the users' currently enrolled courses
    pub async fn api_acquire_users_courses(
        &self,
        user_id: u64,
    ) -> Result<Vec<CoreEnrolGetUsersCourses>> {
        let courses = self.core_enrol_get_users_courses(user_id).await?;
        Ok(courses)
    }

    async fn save_grades_table_inner(&self, path: &Path, course_id: u64) -> Result<()> {
        let page_path = &path.join("grades");
        let mut page_url = self.get_moodle_url().join("grade/report/user/index.php")?;
        page_url
            .query_pairs_mut()
            .append_pair("id", &course_id.to_string());

        let hidden_json_path = &path.join(".moo-dl.grades.json");
        let hidden_json = self.gradereport_user_get_grades_table(course_id).await?;

        self.save_page_with_extra_file(page_path, &page_url, &hidden_json_path, &hidden_json)
            .await?;

        Ok(())
    }

    /// Save a grade table (if requested by config)
    pub async fn save_grades_table(&self, path: &Path, course_id: u64) {
        if self.grades {
            if let Err(e) = self.save_grades_table_inner(path, course_id).await {
                let message = e
                    .context(format!("Failed saving grades {}", course_id))
                    .to_string();
                self.status_bar.register_err(&message).await;
            }
        }
    }
}
