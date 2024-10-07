use tracing::{debug, info, trace};

use std::{ops::Deref, path::Path, sync::Arc};
use tokio::sync::{Mutex, RwLock};

use reqwest::{cookie::Jar, Client, IntoUrl};

pub mod errors;
pub mod login;
pub mod rest_api;

use crate::{downloader::DownloadOptions, Result};
use errors::LoginFailedError;

use login::{
    from_params::{LoginFromParams, LoginParams},
    ApiCredential, Credential,
};
#[derive(Debug, Clone)]
/// The main api struct
pub struct Api {
    /// The credential used for api authentication
    pub api_credential: ApiCredential,
    /// The credential used for authentication
    credential: Arc<RwLock<Option<Credential>>>,
    /// The login params used to create a credential if needed
    pub login_params: Arc<Mutex<LoginParams>>,
    /// The cookie jar that may contain the session cookie
    pub cookie_jar: Arc<Jar>,
    /// The client used for requests
    pub client: Client,
    /// Download options
    pub download_options: DownloadOptions,
    /// The user id of the current user
    pub user_id: Option<u64>,
}

impl Api {
    /// Create a new builder for the api
    pub fn builder() -> ApiBuilder {
        ApiBuilder::new()
    }

    async fn get_credential(&self) -> Option<tokio::sync::RwLockReadGuard<'_, Option<Credential>>> {
        let credential_guard = self.credential.read().await;
        if credential_guard.as_ref().is_some() {
            Some(credential_guard)
        } else {
            None
        }
    }

    /// Acuire a credential
    /// If a credential already exists, it will be returned
    /// If no credential exists, it will be created
    ///
    /// If the login fails, it will return an error for all calls
    pub async fn acuire_credential(
        &self,
    ) -> Result<tokio::sync::RwLockReadGuard<'_, Option<Credential>>> {
        {
            // Check if credential exists
            let credential_guard = self.get_credential().await;
            if credential_guard.is_some() {
                return Ok(credential_guard.unwrap());
            }
            // credential_guard goes out of scope
        }

        trace!("No existing credential found, trying to acuire one");
        // Acquire lock on login params
        let mut login_params_guard = self.login_params.lock().await;
        match login_params_guard.deref() {
            LoginParams::LoginFailed => {
                return Err(Box::new(LoginFailedError::new(
                    "Login already failed in other thread",
                )));
            }
            LoginParams::None => {
                return Err(Box::new(LoginFailedError::new("No login params set")));
            }
            LoginParams::LoginComplete => {
                let credential_guard = self.get_credential().await;
                match credential_guard {
                    Some(credential) => return Ok(credential),
                    None => return Err(Box::new(LoginFailedError::new(
                        "Catastrophic error, could not get existing credential: Pls report this",
                    ))),
                }
            }
            _ => {}
        }
        info!("Logging in");
        debug!("Login params: {:?}", login_params_guard);
        let credential = login_params_guard
            .deref()
            .clone()
            .login(&self.api_credential, self.cookie_jar.clone())
            .await;
        info!("Logged in!");
        match credential {
            Ok(credential) => {
                self.credential.write().await.replace(credential);
                *login_params_guard = LoginParams::LoginComplete;
                Ok(self.credential.read().await)
            }
            Err(e) => {
                *self.login_params.lock().await = LoginParams::LoginFailed;
                Err(e)
            }
        }
    }

    /// Get the user id of the current user
    pub async fn acuire_user_id(&mut self) -> Result<()> {
        let site_info = self.get_core_webservice_get_site_info().await?;
        self.user_id = Some(site_info.userid);
        Ok(())
    }

    /// Download a file that is directly accessible via the moodle api
    pub async fn download_file<U: IntoUrl>(
        &self,
        url: U,
        download_path: &Path,
        last_modified: Option<u64>,
    ) -> Result<()> {
        let request = self
            .client
            .get(url)
            .query(&[("token", self.api_credential.wstoken.as_str())]);

        self.download_options
            .file_update_strategy
            .download_from_requestbuilder(request, download_path, last_modified)
            .await?;

        Ok(())
    }

    /// Download a file from api data
    ///
    /// # Arguments
    /// * `path` - The folder where the file should be downloaded to
    /// * `filename` - The name of the file according to the moodle api
    /// * `filepath` - The path to the file accroding to the moodle api
    /// * `fileurl` - The url to the file according to the moodle api
    /// * `timemodified` - The time the file was last modified according to the moodle api
    pub async fn download_file_from_api_params(
        &self,
        path: &Path,
        filename: &str,
        filepath: &str,
        fileurl: &str,
        timemodified: u64,
    ) -> Result<()> {
        let mut fixed_filepath;
        let custom_path = if filepath.starts_with("/") {
            fixed_filepath = ".".to_string();
            fixed_filepath.push_str(filepath);
            &fixed_filepath
        } else {
            filepath
        };

        let download_path = &path.join(custom_path).join(filename);

        self.download_file(fileurl, download_path, Some(timemodified))
            .await?;

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
    login_params: Option<LoginParams>,
    cookie_jar: Option<Arc<Jar>>,
    download_options: Option<DownloadOptions>,
    user_id: Option<u64>,
}

impl ApiBuilder {
    pub fn new() -> Self {
        ApiBuilder {
            api_credential: None,
            credential: None,
            login_params: None,
            cookie_jar: None,
            download_options: None,
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

    pub fn login_params(mut self, login_params: LoginParams) -> Self {
        self.login_params = Some(login_params);
        self
    }

    pub fn cookie_jar(mut self, cookie_jar: Arc<Jar>) -> Self {
        self.cookie_jar = Some(cookie_jar);
        self
    }

    pub fn download_options(mut self, download_options: DownloadOptions) -> Self {
        self.download_options = Some(download_options);
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
        let login_params = match self.login_params {
            Some(login_params) => Arc::new(Mutex::new(login_params)),
            None => Arc::new(Mutex::new(LoginParams::None)),
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
        let download_options = match self.download_options {
            Some(download_options) => download_options,
            None => DownloadOptions::default(),
        };
        Ok(Api {
            api_credential,
            credential: Arc::new(RwLock::new(credential)),
            login_params: login_params.clone(),
            cookie_jar,
            client,
            download_options,
            user_id: self.user_id,
        })
    }
}
