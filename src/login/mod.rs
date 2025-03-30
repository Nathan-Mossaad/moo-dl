use std::sync::Arc;

use anyhow::Ok;
use tracing::{info, warn};
use url::Url;

use crate::config::sync_config::{Config, Login, LoginState};
use crate::Result;

impl Login {
    pub fn get_url(&self) -> &Url {
        match self {
            Login::ApiOnly { url } => url,
            Login::Raw { url, .. } => url,
            Login::UserPass { url, .. } => url,
            Login::Graphical { url } => url,
            Login::Rwth { url, .. } => url,
        }
    }
}

impl Config {
    /// Returns a thread that will run in the background to attempt a login
    /// Will log to the status_bar, if an error occurs
    pub async fn login_thread(config: Arc<Config>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let login_state = Config::login(&config).await;
            if let Err(error) = login_state {
                let error_string = error.context("Login failed").to_string();
                config.status_bar.register_err(&error_string).await;
            }
        })
    }
    async fn login(&self) -> Result<()> {
        let mut cookie = self.cookie.write().await;

        match &self.login {
            Login::ApiOnly { .. } => {
                *cookie = LoginState::Failed;
                tracing::warn!("No full login method provided: Running with limited functionality");
                Ok(())
            }
            Login::Raw { url, cookie } => todo!(),
            Login::Graphical { url } => todo!(),
            Login::UserPass {
                url,
                username,
                password,
            } => todo!(),
            Login::Rwth {
                url,
                username,
                password,
                totp,
                totp_secret,
            } => todo!(),
        }
    }
}
