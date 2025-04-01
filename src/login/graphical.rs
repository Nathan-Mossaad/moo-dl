use std::path::Path;

use tracing::{debug, trace};

use chromiumoxide::{
    browser::{self, Browser},
    BrowserConfig,
};
use tokio_stream::StreamExt;

use super::*;

/// Creates a new session cookie from a graphical login
///
/// Warning: will open a new browser instance, make sure chrome dev tools api is open
pub async fn login_graphical(
    instance_url: &Url,
    executable_path: &Option<impl AsRef<Path>>,
    wstoken_request: bool,
) -> Result<LoginParams> {
    if !(instance_url.scheme() == "http" || instance_url.scheme() == "https") {
        return Err(anyhow!(
            "Incorrect url, it has to start with either \"http://\" or \"https://\""
        ));
    }

    // Setup browser config
    let mut browser_config = BrowserConfig::builder().headless_mode(browser::HeadlessMode::False);
    if let Some(path) = executable_path {
        browser_config = browser_config.chrome_executable(path);
    }
    let browser_config = browser_config.build().unwrap();

    // Launch browser
    let (mut browser, mut handler) = Browser::launch(browser_config).await?;
    // Spawn a task to handle the browser events
    let _ = tokio::spawn(async move { while let Some(_) = handler.next().await {} });
    debug!("Login browser launched");

    // Clear existing cookies as it may contain old session cookies
    browser.clear_cookies().await?;

    // Get url and cookies
    let mut moo_dl_url = "".to_string();
    let login_page = browser.new_page(format!("{}/admin/tool/mobile/launch.php?service=moodle_mobile_app&passport=00000&urlscheme=moo-dl", instance_url)).await?;
    debug!("Login page loaded");
    let mut events = login_page
        .event_listener::<chromiumoxide::cdp::browser_protocol::network::EventRequestWillBeSent>()
        .await?;
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

    // Browser is no longer needed, we can close it
    browser.close().await?;

    // Get token
    let wstoken = if wstoken_request {
        Some(wstoken_from_url(&moo_dl_url)?)
    } else {
        None
    };
    debug!("Found Token {:?}", wstoken);

    // Get cookie
    let session_cookie;
    let instance_host = instance_url
        .host_str()
        .ok_or(anyhow!("Invalid instance url"))?;
    let cookie = all_cookies
        .iter()
        .find(|cookie| (&cookie.domain == instance_host) && (cookie.name == "MoodleSession"));
    match cookie {
        Some(cookie_candidate) => {
            debug!("Found cookie: {:?}", cookie_candidate);
            session_cookie = cookie_candidate.value.clone();
        }
        None => {
            return Err(anyhow!("No login cookie found"));
        }
    }

    // We still need to wait, that the browser is fully closed
    browser.wait().await?;

    Ok(LoginParams {
        cookie: session_cookie,
        wstoken,
    })
}
