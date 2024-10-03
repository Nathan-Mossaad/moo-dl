pub mod login;
mod rest_api;

use reqwest::Client;

use login::Credential;

use crate::Result;

#[derive(Debug, Clone)]
pub struct Api {
    pub credential: Credential,
    pub user_id: Option<i32>,
    pub client: Client,
}

impl Api {
    pub async fn get_user_id(&mut self) -> Result<()> {
        let site_info = self.get_core_webservice_get_site_info().await?;
        self.user_id = Some(site_info.userid);
        Ok(())
    }
}
