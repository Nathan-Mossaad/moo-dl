use tracing::{debug, trace};

use reqwest::Client;
use serde_json::Value;

use chromiumoxide::{
    browser::{self, Browser},
    BrowserConfig,
};
use tokio_stream::StreamExt;

use crate::Result;

/// Represents moodle credential
#[derive(Debug, Clone)]
pub struct Credential {
    /// base moodle instance url (e.g. https://moodle.example.com)
    instance_url: String,
    /// web service token (as used by the official moodle app)
    wstoken: String,
    /// cookie (as used on the moodle website)
    session_cookie: Option<String>,
}

impl Credential {
    /// Creates a new credential from a wstoken
    ///
    /// # Arguments
    ///
    /// * `instance_url` - base moodle instance url (e.g. https://moodle.example.com)
    /// * `wstoken` - web service token (as used by the official moodle app)
    pub fn from_wstoken(instance_url: String, wstoken: String) -> Self {
        debug!("Creating credential from (wstoken): {{ instance_url: \"{instance_url}\", wstoken: \"{wstoken}\" }}");
        Credential {
            instance_url,
            wstoken,
            session_cookie: None,
        }
    }

    /// Creates a new credential from (username/password)
    ///
    /// # Arguments
    ///
    /// * `instance_url` - base moodle instance url (e.g. https://moodle.example.com)
    /// * `username` - username
    /// * `password` - password
    /// * `wstoken` - web service token (as used by the official moodle app)
    /// * `client` - optional client to use for requests
    pub async fn from_username_password(
        instance_url: String,
        username: &str,
        password: &str,
        wstoken: Option<String>,
        client: Option<&Client>,
    ) -> Result<Self> {
        debug!("Creating credential from (username/password): {{ instance_url: \"{instance_url}\", username: \"{username}\", password: \"{password}\" }}");
        let client: &Client = match client {
            Some(client) => client,
            None => &Client::new(),
        };

        // Aquire wstoken
        let wstoken = match wstoken {
            Some(wstoken) => wstoken,
            None => {
                // Request wstoken
                let wstoken_req: Value = client
                    .post(format!("{}/login/token.php", instance_url))
                    .form(&vec![
                        ("username", username),
                        ("password", password),
                        ("service", "moodle_mobile_app"),
                    ])
                    .send()
                    .await?
                    .json()
                    .await?;
                debug!("Response token: {:?}", wstoken_req);
                match wstoken_req["token"].as_str() {
                    Some(wstoken) => wstoken.to_string(),
                    None => {
                        return Err(format!("Error on login: {:?} \n\n This probably means you have wrong login credentials! \n\n", wstoken_req).into());
                    }
                }
            }
        };

        // Attempt to get session cookie
        let session_cookie_req = client
            .post(format!("{}/login/index.php", instance_url))
            .form(&vec![("username", username), ("password", password)])
            .send()
            .await?;
        trace!(
            "Cookies: {:?}",
            session_cookie_req.cookies().collect::<Vec<_>>()
        );
        let session_cookie = session_cookie_req
            .cookies()
            .find(|cookie| cookie.name() == "MoodleSession")
            .map(|cookie| cookie.value().to_string());
        debug!("Response cookie: {:?}", session_cookie);

        Ok(Credential {
            instance_url,
            wstoken,
            session_cookie,
        })
    }

    /// Creates a new credential from a graphical login
    ///
    /// # Arguments
    /// * `instance_url` - base moodle instance url (e.g. https://moodle.example.com)
    /// * `wstoken` - web service token (as used by the official moodle app)
    /// * `browser` - optional browser to use for requests
    pub async fn from_graphical(
        instance_url: String,
        wstoken: Option<String>,
        browser: Option<&Browser>,
    ) -> Result<Self> {
        if !(instance_url.starts_with("http://") || instance_url.starts_with("https://")) {
            return Err(
                "Incorrect url, it has to start with either \"http://\" or \"https://\"".into(),
            );
        }

        debug!("Creating credential from (graphical): {{ instance_url: \"{instance_url}\" }}");
        let new_browser: Option<Browser>;
        let browser = match browser {
            Some(browser) => {
                new_browser = None;
                browser
            }
            None => {
                let (browser, mut handler) = Browser::launch(
                    BrowserConfig::builder()
                        .headless_mode(browser::HeadlessMode::False)
                        .build()
                        .unwrap(),
                )
                .await?;
                // Spawn a task to handle the browser events
                let _ = tokio::spawn(async move { while let Some(_) = handler.next().await {} });

                debug!("Login browser launched");

                new_browser = Some(browser);
                &new_browser.as_ref().unwrap()
            }
        };
        // Clear existing cookies as it may contain old session cookies
        // browser.clear_cookies().await?;

        // Get url and cookies
        let mut moo_dl_url = "".to_string();
        let login_page = browser.new_page(format!("{}/admin/tool/mobile/launch.php?service=moodle_mobile_app&passport=00000&urlscheme=moo-dl", instance_url)).await?;
        debug!("Login page loaded");
        println!("");
        let mut events = login_page.event_listener::<chromiumoxide::cdp::browser_protocol::network::EventRequestWillBeSent>().await?;
        while let Some(event) = events.next().await {
            trace!("LoginEvent: {:?}", event);
            if event.document_url.starts_with("moo-dl://") {
                debug!("Found \"moo-dl://\" event: {:?}", event);
                moo_dl_url = event.request.url.clone();
                break;
            }
        }
        let all_cookies = browser.get_cookies().await?;
        trace!("\nAll cookies: {:?}", all_cookies);
        // Check if browser is None
        if !new_browser.is_none() {
            let mut new_browser = new_browser.unwrap();
            new_browser.close().await?;
            new_browser.wait().await?;
        }

        // Get token
        let wstoken = match wstoken {
            Some(wstoken) => wstoken,
            None => wstoken_from_url(&moo_dl_url)?,
        };
        debug!("Found Token {:?}", wstoken);

        // Get cookie
        let session_cookie;
        let base_domain = instance_url.split("/").collect::<Vec<&str>>()[2];
        let cookie = all_cookies
            .iter()
            .find(|cookie| (cookie.domain == base_domain) && (cookie.name == "MoodleSession"));
        match cookie {
            Some(cookie_candidate) => {
                debug!("Found cookie: {:?}", cookie_candidate);
                session_cookie = cookie_candidate.value.clone();
            }
            None => {
                return Err("No login cookie found: {:?}".into());
            }
        }

        Ok(Credential {
            instance_url,
            wstoken,
            session_cookie: Some(session_cookie),
        })
    }
}

fn wstoken_from_url(moo_dl_url: &str) -> Result<String> {
    let token_base64 = match moo_dl_url.split("token=").last() {
        Some(token_base64) => token_base64,
        None => {
            return Err("Error on login: No token found in url".into());
        }
    };
    let token_decoded = String::from_utf8(base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        token_base64,
    )?)?;
    Ok(token_decoded.split(":::").collect::<Vec<&str>>()[1].to_string())
}
