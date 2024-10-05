use std::fmt::Debug;
use std::sync::Arc;

use reqwest::cookie::CookieStore;

use crate::{api::errors::LoginFailedError, Result};

use super::{ApiCredential, Credential};

/// Represents parameters for Credential::from_raw, that can be used to login
pub trait LoginFromParams: Debug + Clone {
    async fn login<C: CookieStore + 'static>(
        self,
        api_credential: &ApiCredential,
        cookie_jar: Arc<C>,
    ) -> Result<Credential>;
}

#[derive(Debug, Clone)]
pub enum LoginParams {
    Raw(CredentialFromRawParams),
    UsernamePassword(CredentialFromUsernamePasswordParams),
    Graphical(CredentialFromGraphicalParams),
    Rwth(CredentialFromRwthParams),
    LoginFailed,
    LoginComplete,
    None,
}

impl LoginFromParams for LoginParams {
    async fn login<C: CookieStore + 'static>(
        self,
        api_credential: &ApiCredential,
        cookie_jar: Arc<C>,
    ) -> Result<Credential> {
        match self {
            LoginParams::Raw(params) => params.login(api_credential, cookie_jar).await,
            LoginParams::UsernamePassword(params) => params.login(api_credential, cookie_jar).await,
            LoginParams::Graphical(params) => params.login(api_credential, cookie_jar).await,
            LoginParams::Rwth(params) => params.login(api_credential, cookie_jar).await,
            LoginParams::LoginFailed => Err(Box::new(LoginFailedError::new(
                "Login failed in other thread",
            ))),
            LoginParams::LoginComplete => Err(Box::new(LoginFailedError::new(
                "Login already complete in other thread",
            ))),
            LoginParams::None => Err(Box::new(LoginFailedError::new("No login params set"))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CredentialFromRawParams {
    pub session_cookie: String,
}

impl LoginFromParams for CredentialFromRawParams {
    async fn login<C: CookieStore + 'static>(
        self,
        api_credential: &ApiCredential,
        cookie_jar: Arc<C>,
    ) -> Result<Credential> {
        Credential::from_raw(
            api_credential.instance_url.clone(),
            api_credential.wstoken.clone(),
            self.session_cookie,
            cookie_jar,
        )
    }
}

#[derive(Debug, Clone)]
pub struct CredentialFromUsernamePasswordParams {
    pub username: String,
    pub password: String,
}

impl LoginFromParams for CredentialFromUsernamePasswordParams {
    async fn login<C: CookieStore + 'static>(
        self,
        api_credential: &ApiCredential,
        cookie_jar: Arc<C>,
    ) -> Result<Credential> {
        Ok(Credential::from_username_password(
            api_credential.instance_url.clone(),
            &self.username,
            &self.password,
            Some(api_credential.wstoken.clone()),
            cookie_jar,
        )
        .await?)
    }
}

#[derive(Debug, Clone)]
pub struct CredentialFromGraphicalParams {}

impl LoginFromParams for CredentialFromGraphicalParams {
    async fn login<C: CookieStore + 'static>(
        self,
        api_credential: &ApiCredential,
        cookie_jar: Arc<C>,
    ) -> Result<Credential> {
        Ok(Credential::from_graphical(
            api_credential.instance_url.clone(),
            Some(api_credential.wstoken.clone()),
            None,
            cookie_jar,
        )
        .await?)
    }
}

#[derive(Debug, Clone)]
pub struct CredentialFromRwthParams {
    pub username: String,
    pub password: String,
    pub totp: String,
    pub totp_secret: String,
}

impl LoginFromParams for CredentialFromRwthParams {
    async fn login<C: CookieStore + 'static>(
        self,
        api_credential: &ApiCredential,
        cookie_jar: Arc<C>,
    ) -> Result<Credential> {
        Ok(Credential::from_rwth(
            &self.username,
            &self.password,
            &self.totp,
            &self.totp_secret,
            Some(api_credential.wstoken.clone()),
            cookie_jar,
        )
        .await?)
    }
}
