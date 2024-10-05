pub mod errors;
pub mod login;
mod rest_api;

use reqwest::Client;

use login::ApiCredential;

use crate::Result;

#[derive(Debug, Clone)]
/// The main api struct
pub struct Api {
    /// The credential used for authentication
    pub api_credential: ApiCredential,
    /// The user id of the current user
    pub user_id: Option<u64>,
    /// The client used for requests
    pub client: Client,
}

impl Api {
    /// Get the user id of the current user
    pub async fn get_user_id(&mut self) -> Result<()> {
        let site_info = self.get_core_webservice_get_site_info().await?;
        self.user_id = Some(site_info.userid);
        Ok(())
    }
}
