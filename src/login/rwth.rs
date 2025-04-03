use reqwest::{cookie::Jar, Client};
use tokio::task;
use totp_rs::{Algorithm, Secret, TOTP};
use tracing::{debug, trace};

use super::*;

/// Creates a new session cookie from an rwth login
/// 
/// Based on <https://github.com/Romern/syncMyMoodle> (Thank you!)
pub async fn from_rwth(
    instance_url: &Url,
    username: &str,
    password: &str,
    totp: &str,
    totp_secret: &str,
    wstoken_request: bool,
) -> Result<LoginParams> {
    let cookie_jar = Arc::new(Jar::default());
    let totp_secret: String = totp_secret.into();
    let client = Client::builder()
        .cookie_provider(cookie_jar.clone())
        .build()?;

    debug!(
        "Logging in via RWTH sso using username: \"{}\", password: \"{}\", totp: \"{}\", totp_secret: \"{}\"",
        username, password, totp, totp_secret
    );

    // Intialize login process
    client.get(instance_url.as_ref()).send().await?;
    let response = client
        .get(instance_url.join("auth/shibboleth/index.php")?)
        .send()
        .await?;
    let resp_url = response.url().clone();
    debug!("Response URL: {:?}", resp_url);
    let csrf_token = get_token(response, "csrf_token").await?;
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
    let csrf_token = get_token(response, "csrf_token").await?;
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
    let csrf_token = get_token(response, "csrf_token").await?;
    debug!("CSRF token: {}", csrf_token);
    debug!("Completed login step 2");

    // Step 3 (Provide 2nd factor)
    // Generate token
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
    let html_clone = html.clone(); // Clone html so it can be moved into spawn_blocking
    let (relay_state, saml_response) = task::spawn_blocking(move || -> Result<(String, String)> {
        let document = Document::from(html_clone.as_str());
        let relay_state = document
            .find(Name("input"))
            .filter(|node| node.attr("name") == Some("RelayState"))
            .next()
            .and_then(|node| node.attr("value"))
            .ok_or_else(|| anyhow!("Could not extract relay state after entering totp code"))?;

        let saml_response = document
            .find(Name("input"))
            .filter(|node| node.attr("name") == Some("SAMLResponse"))
            .next()
            .and_then(|node| node.attr("value"))
            .ok_or_else(|| anyhow!("Could not extract saml response after entering totp code"))?;

        Ok((relay_state.to_string(), saml_response.to_string())) // Clone to String to make it Send
    })
    .await
    .map_err(|e| anyhow!("Error in spawn_blocking: {}", e))??; // Propagate errors

    trace!("RelayState: {:?}", relay_state);
    trace!("SAMLResponse: {:?}", saml_response);

    let response = client
        .post(instance_url.join("Shibboleth.sso/SAML2/POST")?)
        .form(&vec![
            ("RelayState", relay_state),
            ("SAMLResponse", saml_response),
        ])
        .send()
        .await?;
    let html = response.text().await?;
    trace!("Response HTML:\n {}", html);
    debug!("Completed final login step (4)");

    let session_cookie = extract_session_cookie(&instance_url, &cookie_jar)?;

    // Check if wstoken we need to get a new wstoken
    let wstoken = if wstoken_request {
        // Acuire wstoken
        let response_url = match client
            .get(instance_url.join("admin/tool/mobile/launch.php?service=moodle_mobile_app&passport=00000&urlscheme=moo-dl")?)
            .send()
            .await {
                Ok(_) => return Err(anyhow!("This should have resulted in an invalid url")),
                Err(e) => {
                    trace!("Attempting to parse theoretical Error: {:?}", e);
                    e.url().ok_or(anyhow!("No wstoken url"))?.to_string()
                }
            };
        trace!("Response URL: {}", response_url);
        let wstoken = wstoken_from_url(&response_url)?;
        debug!("Found Token {:?}", wstoken);
        Some(wstoken)
    } else {
        None
    };

    Ok(LoginParams {
        cookie: session_cookie,
        wstoken,
    })
}
