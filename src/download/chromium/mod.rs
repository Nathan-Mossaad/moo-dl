use anyhow::anyhow;
use tokio::sync::RwLockReadGuard;

use web2pdf_lib::{Browser, BrowserWeb2Pdf};

use crate::config::sync_config::{ChromiumState, Config};

impl Config {
    /// Get's the currently loaded browser from the config, will attempt to start a browser if there has been no previous attempt
    ///
    /// Should only return ChromiumState::Unavailable or ChromiumState::Browser
    pub async fn get_chromium(&self) -> RwLockReadGuard<'_, ChromiumState> {
        {
            let browser_guard = self.chromium.read().await;
            match &*browser_guard {
                ChromiumState::NotStarted => {}
                _ => return browser_guard,
            }
        }

        // Browser is not running Attempt to create a new browser
        let mut browser_guard = self.chromium.write().await;
        // Another thread may have started the browser by now
        match &*browser_guard {
            ChromiumState::NotStarted => {}
            _ => return browser_guard.downgrade(),
        }

        // We need to wait for Login first to prevent issues with starting two browsers
        let cookie = match self.get_cookie().await {
            Some(cookie) => cookie,
            None => {
                *browser_guard = ChromiumState::Unavailable;
                return browser_guard.downgrade();
            }
        };

        // Actually attempt to start (now that we have exclusive write access, and no other thread is attempting a start)
        let browser_result = match &self.chrome_executable {
            None => Browser::web2pdf_launch().await,
            Some(path) => Browser::web2pdf_launch_from_executable_path(path).await,
        };
        match browser_result {
            Ok(browser) => *browser_guard = ChromiumState::Browser(browser),
            Err(e) => {
                let err = anyhow!(e.to_string());
                self.status_bar
                    .register_err(&err.context("Could not launch browser").to_string())
                    .await;
                *browser_guard = ChromiumState::Unavailable;
            }
        }

        // Load cookie for moodle
        if let ChromiumState::Browser(browser) = &*browser_guard {
            let browser_cookies = vec![
                chromiumoxide::cdp::browser_protocol::network::CookieParam::builder()
                    .domain(self.get_moodle_url().host_str().expect("This should not be possible, report this: Invalid moodle Url"))
                    .name("MoodleSession")
                    .value(cookie.to_string())
                    .source_port(-1)
                    .build()
                    .expect("This should not be possible, report this: cookieparam error")
            ];
            match browser.set_cookies(browser_cookies).await {
                Ok(_) => {},
                Err(e) => {
                    let err = anyhow!(e.to_string());
                    self.status_bar
                        .register_err(&err.context("Could not set browser cookie (Webbrowser unavailable)").to_string())
                        .await;
                    *browser_guard = ChromiumState::Unavailable;
                },
            }
        } else {
            panic!("This should not be possible, report this: browser startup error")
        }

        browser_guard.downgrade()
    }
}
