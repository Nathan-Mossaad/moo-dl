pub mod save_page;

use anyhow::anyhow;
use tokio::sync::RwLockReadGuard;
use tracing::{debug, warn};
use url::Url;

use web2pdf_lib::{Browser, BrowserWeb2Pdf};

use crate::{
    config::sync_config::{ChromiumState, Config, PageConversion, UpdateStrategy},
    update::{timestamp::set_file_creation, UpdateState},
};

use super::*;

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
            Some(path) => {
                debug!("Launching non default chrome executable: {}", path.to_str().unwrap_or("Path unavailable"));
                Browser::web2pdf_launch_from_executable_path(path).await
            },
        };
        match browser_result {
            Ok(browser) => {
                debug!("Remote debugging URL: {}", &browser.websocket_address());
                *browser_guard = ChromiumState::Browser(browser)
            }
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
                    .domain(
                        self.get_moodle_url()
                            .host_str()
                            .expect("This should not be possible, report this: Invalid moodle Url"),
                    )
                    .name("MoodleSession")
                    .value(cookie.to_string())
                    .source_port(-1)
                    .build()
                    .expect("This should not be possible, report this: cookieparam error"),
            ];
            match browser.set_cookies(browser_cookies).await {
                Ok(_) => {}
                Err(e) => {
                    let err = anyhow!(e.to_string());
                    self.status_bar
                        .register_err(
                            &err.context("Could not set browser cookie (Webbrowser unavailable)")
                                .to_string(),
                        )
                        .await;
                    *browser_guard = ChromiumState::Unavailable;
                }
            }
        } else {
            panic!("This should not be possible, report this: browser startup error")
        }

        browser_guard.downgrade()
    }

    /// A wrapper around `chromiumoxide::browser::close()`, that only gets only executed, if chromium is actually loaded
    /// Only run this, if you are sure that nothing is accessing the browser
    pub async fn chromium_close(&self) {
        let mut browser_guard = self.chromium.write().await;

        if let ChromiumState::Browser(browser) = &mut *browser_guard {
            if let Err(e) = browser.close().await {
                let err = anyhow!(e.to_string());
                warn!("{}", err.context("Could not stop Chromium"));
            }
        }
    }

    // A wrapper around `chromiumoxide::browser::wait()`, that only gets only executed, if chromium is actually loaded
    pub async fn chromium_wait(&self) {
        let mut browser_guard = self.chromium.write().await;

        if let ChromiumState::Browser(browser) = &mut *browser_guard {
            if let Err(e) = browser.wait().await {
                let err = anyhow!(e.to_string());
                warn!("{}", err.context("Could not wait for Chromium stop"));
            }
        }
    }
}

// Implementations for direct consumption in Api
impl Config {
    /// Same as `force_save_page` but only downloads if file does not exist
    /// Additionally writes the event to log
    /// 
    /// Don't set file extension, it will be generated automatically
    pub async fn save_page(&self, file_path: &Path, url: &Url) -> Result<()> {
        // We need to set the correct file extension
        let file_path = if let PageConversion::SingleFile(_) = &self.page_conversion {
            file_path.with_extension("html")
        } else {
            file_path.with_extension("pdf")
        };
        
        match UpdateStrategy::check_exists(&file_path).await? {
            UpdateState::Missing => {
                self.force_save_page(&file_path, url).await?;
                let message = file_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;
                self.status_bar.register_new(message).await;
                Ok(())
            }
            UpdateState::OutOfDate => Err(anyhow!("Impssossible OutOfDate")),
            UpdateState::UpToDate => {
                self.status_bar.register_unchanged().await;
                Ok(())
            }
        }
    }

    /// Same as `download_file` utilises the given time for updating / archiving (if requested)
    /// Additionally writes the event to log
    /// 
    /// Don't set file extension, it will be generated automatically
   pub async fn save_page_with_timestamp(
        &self,
        file_path: &Path,
        url: &Url,
        timestamp: u64,
    ) -> Result<()> {
        // We need to set the correct file extension
        let file_path = if let PageConversion::SingleFile(_) = &self.page_conversion {
            file_path.with_extension("html")
        } else {
            file_path.with_extension("pdf")
        };        
        
        match self
            .update_strategy
            .timestamp_check_up_to_date(&file_path, timestamp)
            .await?
        {
            UpdateState::Missing => {
                self.force_save_page(&file_path, url).await?;
                set_file_creation(&file_path, timestamp).await?;

                let message = file_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;
                self.status_bar.register_new(message).await;
                Ok(())
            }
            UpdateState::OutOfDate => {
                self.force_save_page(&file_path, url).await?;
                set_file_creation(&file_path, timestamp).await?;

                let message = file_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;
                self.status_bar.register_updated(message).await;
                Ok(())
            }
            UpdateState::UpToDate => {
                self.status_bar.register_unchanged().await;
                Ok(())
            }
        }
    }
}
