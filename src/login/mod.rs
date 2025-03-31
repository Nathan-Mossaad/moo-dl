// TODO: remove
#![allow(dead_code)]

pub mod graphical;
pub mod rwth;
pub mod user_pass;

use std::result::Result::Ok;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context};
use regex::Regex;
use reqwest::cookie::CookieStore;
use reqwest::Response;
use rwth::from_rwth;
use tokio::time::sleep;
use tracing::{debug, info, trace, warn};
use url::Url;

use select::{document::Document, predicate::Name};
use user_pass::from_username_password;

use crate::config::sync_config::{Config, Login, LoginState};
use crate::Result;

pub struct LoginParams {
    cookie: String,
    wstoken: Option<String>,
}

impl Config {
    pub fn get_moodle_url(&self) -> &Url {
        match &self.login {
            Login::ApiOnly { url } => url,
            Login::Raw { url, .. } => url,
            Login::UserPass { url, .. } => url,
            Login::Graphical { url } => url,
            Login::Rwth { url, .. } => url,
        }
    }

    pub async fn get_cookie(&self) -> Option<Arc<String>> {
        loop {
            // Separate scope to drop guard
            {
                let cookie_guard = self.cookie.read().await;
                match &*cookie_guard {
                    LoginState::NotChecked => {}
                    LoginState::Unavailable => return None,
                    LoginState::Cookie { cookie } => return Some(cookie.clone()),
                }
            }
            // Retry after some time has elapsed
            sleep(Duration::from_millis(100)).await;
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
                let error_string = error
                    .context("Login failed: Running with limited functionality")
                    .to_string();
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
                    cookie: Arc::new(cookie.to_string()),
                };
                info!("Logged in using, raw params!");
                Ok(())
            }
            Login::Graphical { url } => {
                let login_result =
                    graphical::login_graphical(url, &self.chrome_executable, false).await?;
                *cookie_guard = LoginState::Cookie {
                    cookie: Arc::new(login_result.cookie),
                };
                info!("Logged in using, raw params!");
                Ok(())
            }
            Login::UserPass {
                url,
                username,
                password,
            } => {
                let login_result = from_username_password(url, username, password, false).await?;
                *cookie_guard = LoginState::Cookie {
                    cookie: Arc::new(login_result.cookie),
                };
                info!("Logged in using, username & password!");
                Ok(())
            }
            Login::Rwth {
                url,
                username,
                password,
                totp,
                totp_secret,
            } => {
                let login_result =
                    from_rwth(url, username, password, totp, totp_secret, false).await?;
                *cookie_guard = LoginState::Cookie {
                    cookie: Arc::new(login_result.cookie),
                };
                info!("Logged in using, RWTH SSO!");
                Ok(())
            }
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

/// Get some sort of auth login token
async fn get_token(response: Response, token_name: &str) -> Result<String> {
    let html = response.text().await?;
    trace!("Parsing HTML, for token {}:\n {}", token_name, html);

    let document = Document::from(html.as_str());
    let response_token = document
        .find(Name("input"))
        .filter(|node| node.attr("name") == Some(token_name))
        .next()
        .and_then(|node| node.attr("value"))
        .ok_or(anyhow!(
            "Error on login: Couldn't extract token: {}",
            token_name
        ))?;

    Ok(response_token.to_string())
}

fn extract_session_cookie<C: CookieStore + 'static>(
    instance_url: &Url,
    cookie_jar: &Arc<C>,
) -> Result<String> {
    let header_value = cookie_jar
        .cookies(instance_url)
        .context("Cookie extractor: could not extract cookie")?;
    let header_value = header_value.to_str()?;
    trace!("Header Values from instance_url: {:?}", header_value);

    let regex = Regex::new(r"MoodleSession=([^ ;]+)")?;
    let regex_capture = regex
        .captures(header_value)
        .ok_or(anyhow!("Cookie extractor: No 'MoodleSession=' found"))?[0]
        .to_string();

    let mut parts = regex_capture.split('=');
    let session_cookie = parts
        .nth(1)
        .ok_or(anyhow!("Cookie extractor: could not extract cookie"))?
        .to_string();

    debug!("Found Session Cookie {}", session_cookie);
    Ok(session_cookie)
}
