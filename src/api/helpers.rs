use super::*;

impl Config {
    /// Get the user id of the current user
    pub async fn api_acquire_user_id(&self) -> Result<u64> {
        let site_info = self.api_core_webservice_get_site_info().await?;
        Ok(site_info.userid)
    }
}



