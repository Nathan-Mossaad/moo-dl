pub mod errors;
pub mod login;
mod rest_api;

use std::sync::{Arc, RwLock};

use reqwest::{cookie::Jar, Client};

use login::{ApiCredential, Credential};

use crate::Result;

#[derive(Debug, Clone)]
/// The main api struct
pub struct Api {
    /// The credential used for api authentication
    pub api_credential: ApiCredential,
    /// The credential used for authentication
    pub credential: Arc<RwLock<Option<Credential>>>,
    /// The cookie jar that may contain the session cookie
    pub cookie_jar: Arc<Jar>,
    /// The client used for requests
    pub client: Client,
    /// The user id of the current user
    pub user_id: Option<u64>,
}

impl Api {
    /// Create a new builder for the api
    pub fn builder() -> ApiBuilder {
        ApiBuilder::new()
    }

    /// Get the user id of the current user
    pub async fn get_user_id(&mut self) -> Result<()> {
        let site_info = self.get_core_webservice_get_site_info().await?;
        self.user_id = Some(site_info.userid);
        Ok(())
    }
}

/// Builder for the api
///
/// Eiter api_credential or credential must be set
/// If both are set, credential will be used
pub struct ApiBuilder {
    api_credential: Option<ApiCredential>,
    credential: Option<Credential>,
    cookie_jar: Option<Arc<Jar>>,
    user_id: Option<u64>,
}

impl ApiBuilder {
    pub fn new() -> Self {
        ApiBuilder {
            api_credential: None,
            credential: None,
            cookie_jar: None,
            user_id: None,
        }
    }

    pub fn api_credential(mut self, api_credential: ApiCredential) -> Self {
        self.api_credential = Some(api_credential);
        self
    }

    pub fn credential(mut self, credential: Credential) -> Self {
        self.credential = Some(credential);
        self
    }

    pub fn cookie_jar(mut self, cookie_jar: Arc<Jar>) -> Self {
        self.cookie_jar = Some(cookie_jar);
        self
    }

    pub fn user_id(mut self, user_id: u64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn build(self) -> Result<Api> {
        let credential = self.credential;
        let api_credential = match &credential {
            Some(credential) => credential.clone().into(),
            None => self
                .api_credential
                .ok_or("No api credential or credential provided")?,
        };
        let cookie_jar = match self.cookie_jar {
            Some(cookie_jar) => cookie_jar,
            None => {
                let cookie_jar = Arc::new(reqwest::cookie::Jar::default());
                if let Some(credential) = &credential {
                    login::add_moodle_session_cookie(
                        &cookie_jar,
                        &credential.session_cookie,
                        &credential.instance_url,
                    )?;
                }
                cookie_jar
            }
        };
        let client = Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()?;
        Ok(Api {
            api_credential,
            credential: Arc::new(RwLock::new(credential)),
            cookie_jar,
            client,
            user_id: self.user_id,
        })
    }
}
