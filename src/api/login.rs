use reqwest::Client;
use serde_json::Value;
use tracing::{debug, trace};

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
    pub fn from_wstoken(instance_url: String, wstoken: String) -> Self {
        debug!("Creating credential from (wstoken): {{ instance_url: \"{instance_url}\", wstoken: \"{wstoken}\" }}");
        Credential {
            instance_url,
            wstoken,
            session_cookie: None,
        }
    }
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
}
