pub mod graphical;

use std::result::Result::Ok;
use std::sync::Arc;

use anyhow::anyhow;
use tracing::{info, warn};
use url::Url;

use crate::config::sync_config::{Config, Login, LoginState};
use crate::Result;

pub struct LoginParams {
    cookie: String,
    wstoken: Option<String>,
}

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
        let mut cookie_guard = self.cookie.write().await;

        match &self.login {
            Login::ApiOnly { .. } => {
                *cookie_guard = LoginState::Unavailable;
                warn!("No full login method provided: Running with limited functionality");
                Ok(())
            }
            Login::Raw { url: _, cookie } => {
                *cookie_guard = LoginState::Cookie {
                    cookie: cookie.to_string(),
                };
                info!("Logged in using, raw params!");
                Ok(())
            }
            Login::Graphical { url } => {
                let login_result =
                    graphical::login_graphical(url, &self.chrome_executable, false).await?;
                *cookie_guard = LoginState::Cookie {
                    cookie: login_result.cookie,
                };
                info!("Logged in using, raw params!");
                Ok(())
            }
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

fn wstoken_from_url(moo_dl_url: &str) -> Result<String> {
    let token_base64 = match moo_dl_url.split("token=").last() {
        Some(token_base64) => token_base64,
        None => {
            return Err(anyhow!("Error on login: No token found in url"));
        }
    };
    let token_decoded = String::from_utf8(base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        token_base64,
    )?)?;
    Ok(token_decoded.split(":::").collect::<Vec<&str>>()[1].to_string())
}
