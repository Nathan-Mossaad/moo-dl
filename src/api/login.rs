use std::sync::Arc;

use tracing::{debug, trace};

use regex::Regex;
use reqwest::{cookie::CookieStore, Client, Response, Url};
use serde_json::Value;

use select::document::Document;
use select::predicate::Name;
use totp_rs::{Algorithm, Secret, TOTP};

use chromiumoxide::{
    browser::{self, Browser},
    BrowserConfig,
};
use tokio_stream::StreamExt;

use crate::Result;

// TODO remove dead_code warning
#[allow(dead_code)]

/// Represents moodle credential
#[derive(Debug, Clone)]
pub struct Credential {
    /// base moodle instance url (e.g. https://moodle.example.com)
    pub instance_url: String,
    /// web service token (as used by the official moodle app)
    pub wstoken: String,
    /// cookie (as used on the moodle website)
    pub session_cookie: Option<String>,
}

// TODO remove dead_code warning
#[allow(dead_code)]

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

    /// Creates a new credential from RWTH sso
    ///
    /// # Arguments
    ///
    /// * `username` - username
    /// * `password` - password
    /// * `totp` - totp
    /// * `totp_secret` - totp secret
    /// * `wstoken` - web service token (this will skip requesting a new one)
    /// * `jar` - cookie jar (This will contain all created cookies)
    ///
    /// Based on https://github.com/Romern/syncMyMoodle
    ///
    /// Example:
    /// ```rust
    /// use moo_dl::api::login::Credential;
    /// use std::sync::Arc;
    /// use reqwest::cookie::Jar;
    ///
    /// let credential = Credential::from_rwth(
    ///     "ab123456",
    ///     "password",
    ///     "TOTPAIEOFJE",
    ///     "AGJJHGGJ3HIJI45920N3I4J3",
    ///     None,
    ///     Arc::new(Jar::default()),
    /// )
    /// .await?;
    /// println!("{:?}", credential);
    /// ```
    pub async fn from_rwth<C: CookieStore + 'static>(
        username: &str,
        password: &str,
        totp: &str,
        totp_secret: impl Into<String>,
        wstoken: Option<String>,
        cookie_jar: Arc<C>,
    ) -> Result<Self> {
        let totp_secret: String = totp_secret.into();
        debug!(
            "Logging in via RWTH sso using username: \"{}\", password: \"{}\", totp: \"{}\", totp_secret: \"{}\"",
            username, password, totp, totp_secret
        );
        let client = Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()?;

        // Intialize login process
        client.get("https://moodle.rwth-aachen.de/").send().await?;
        let response = client
            .get("https://moodle.rwth-aachen.de/auth/shibboleth/index.php")
            .send()
            .await?;
        let resp_url = response.url().clone();
        debug!("Response URL: {:?}", resp_url);
        let csrf_token = get_csrf_token(response).await?;
        debug!("CSRF token: {}", csrf_token);

        // Login via SSO
        // Step 1 (username and password)
        let response = client
            .post(resp_url)
            .form(&vec![
                ("csrf_token", csrf_token.as_str()),
                ("j_username", username),
                ("j_password", password),
                ("_eventId_proceed", ""),
            ])
            .send()
            .await?;
        let resp_url = response.url().clone();
        debug!("Response URL: {:?}", resp_url);
        let csrf_token = get_csrf_token(response).await?;
        debug!("CSRF token: {}", csrf_token);
        debug!("Completed login step 1");

        // Step 2 (Select TOTP provider)
        let response = client
            .post(resp_url)
            .form(&vec![
                ("csrf_token", csrf_token.as_str()),
                ("fudis_selected_token_ids_input", totp),
                ("_eventId_proceed", ""),
            ])
            .send()
            .await?;
        let resp_url = response.url().clone();
        debug!("Response URL: {:?}", resp_url);
        let csrf_token = get_csrf_token(response).await?;
        debug!("CSRF token: {}", csrf_token);
        debug!("Completed login step 2");

        // Step 3 (Provide 2nd factor)
        // Generate token
        // let secret = Secret::Encoded("OBWGC2LOFVZXI4TJNZTS243FMNZGK5BNGEZDG".to_string());
        // let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret.to_bytes().unwrap()).unwrap();
        let totp_generator = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(totp_secret).to_bytes()?,
        )?;
        let totp_token = totp_generator.generate_current().unwrap();
        debug!("Generated token: {}", totp_token);
        let response = client
            .post(resp_url)
            .form(&vec![
                ("csrf_token", csrf_token.as_str()),
                ("fudis_otp_input", &totp_token),
                ("_eventId_proceed", ""),
            ])
            .send()
            .await?;
        let html = response.text().await?;
        trace!("Response HTML:\n {}", html);
        debug!("Completed login step 3");

        // Step 4 (Pass tokens to moodle)
        // A short pause of a sec might be necessary according to github.com/Romern/syncMyMoodle
        let document = Document::from(html.as_str());
        // trace!("Document: {:?}", document);
        let relay_state = document
            .find(Name("input"))
            .filter(|node| node.attr("name") == Some("RelayState"))
            .next()
            .and_then(|node| node.attr("value"));
        let relay_state: &str = match relay_state {
            Some(relay_state) => relay_state,
            None => return Err("Could not extract relay state after entering totp code".into()),
        };
        let saml_response = document
            .find(Name("input"))
            .filter(|node| node.attr("name") == Some("SAMLResponse"))
            .next()
            .and_then(|node| node.attr("value"));
        let saml_response = match saml_response {
            Some(saml_response) => saml_response,
            None => return Err("Could not extract saml response after entering totp code".into()),
        };
        trace!("RelayState: {:?}", relay_state);
        trace!("SAMLResponse: {:?}", saml_response);
        let response = client
            .post("https://moodle.rwth-aachen.de/Shibboleth.sso/SAML2/POST")
            .form(&vec![
                ("RelayState", relay_state),
                ("SAMLResponse", saml_response),
            ])
            .send()
            .await?;
        let html = response.text().await?;
        trace!("Response HTML:\n {}", html);
        debug!("Completed final login step (4)");

        // Print all cookies
        let url = Url::parse("https://moodle.rwth-aachen.de/").unwrap();
        let cookies = cookie_jar.cookies(&url);
        trace!("Cookies: {:?}", cookies);

        let session_cookie;
        if let Some(header_value) = cookie_jar.cookies(&url) {
            let header_value = header_value.to_str()?;
            let regex = Regex::new(r"MoodleSession=([^ ;]+)")?;
            let regex_capture = match regex.captures(header_value) {
                Some(captures) => captures[0].to_string(),
                None => return Err("RWTH Login: No Session Cookie found".into()),
            };
            let mut parts = regex_capture.split('=');
            let _ = parts.next();
            session_cookie = match parts.next() {
                Some(cookie) => cookie.to_string(),
                None => return Err("RWTH Login: No Session Cookie found".into()),
            };
        } else {
            return Err("RWTH Login: No Session Cookie found".into());
        }
        debug!("Found Session Cookie {:?}", session_cookie);

        // Check if wstoken we need to get a new wstoken
        let wstoken = match wstoken {
            Some(wstoken) => wstoken,
            None => {
                // Acuire wstoken
                let response_url = match client
                    .get("https://moodle.rwth-aachen.de/admin/tool/mobile/launch.php?service=moodle_mobile_app&passport=00000&urlscheme=moo-dl")
                    .send()
                    .await {
                        Ok(_) => return Err("This should have resulted in an invalid url".into()),
                        Err(e) => {
                            trace!("Attempting to parse theoretical Error: {:?}", e);
                            match e.url() {
                                Some(url) => url.to_string(),
                                None => return Err("No wstoken url".into()),
                            }
                        }
                    };
                trace!("Response URL: {}", response_url);
                let wstoken = wstoken_from_url(&response_url)?;
                debug!("Found Token {:?}", wstoken);
                wstoken
            }
        };

        Ok(Credential {
            instance_url: "https://moodle.rwth-aachen.de".to_string(),
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

/// RWTH specific function to get the csrf token
async fn get_csrf_token(response: Response) -> Result<String> {
    let html = response.text().await?;
    trace!("Parsing HTML:\n {}", html);

    let document = Document::from(html.as_str());
    let csrf_token = document
        .find(Name("input"))
        .filter(|node| node.attr("name") == Some("csrf_token"))
        .next()
        .and_then(|node| node.attr("value"));
    let csrf_token = match csrf_token {
        Some(csrf_token) => csrf_token,
        None => {
            return Err("Error on login: Couldn't extract csrf token".into());
        }
    };
    Ok(csrf_token.to_string())
}
