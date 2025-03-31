use reqwest::{cookie::Jar, Client};
use serde_json::Value;
use tracing::{debug, trace};

use super::*;

/// Creates a new session cookie from a graphical login
pub async fn from_username_password(
    instance_url: &Url,
    username: &str,
    password: &str,
    wstoken_request: bool,
) -> Result<LoginParams> {
    let cookie_jar = Arc::new(Jar::default());

    let client = Client::builder()
        .cookie_provider(cookie_jar.clone())
        .build()?;

    // Aquire wstoken
    let wstoken = if wstoken_request {
        // Request wstoken
        let wstoken_req: Value = client
            .post(instance_url.join("login/token.php")?)
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
        Some(wstoken_req["token"]
            .as_str()
            .ok_or(anyhow!("Error on login: {:?} \n\n This probably means you have wrong login credentials! \n\n", wstoken_req))?
            .to_string())
    } else {
        None
    };

    // Get session cookie
    // Get login token
    let get_login_token_req = client
        .post(instance_url.join("login/index.php")?)
        .send()
        .await?;
    let login_url = get_login_token_req.url().clone();
    let login_token = get_token(get_login_token_req, "logintoken").await?;
    let session_cookie_req = client
        .post(login_url)
        .form(&vec![
            ("anchor", ""),
            ("logintoken", &login_token),
            ("username", username),
            ("password", password),
        ])
        .send()
        .await?;
    trace!(
        "Cookies: {:?}",
        session_cookie_req.cookies().collect::<Vec<_>>()
    );
    let session_cookie = extract_session_cookie(&instance_url, &cookie_jar)?;

    Ok(LoginParams {
        cookie: session_cookie,
        wstoken,
    })
}
