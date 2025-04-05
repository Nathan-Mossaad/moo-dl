use std::path::PathBuf;

use anyhow::{anyhow, Context};
use indicatif::ProgressBar;
use tokio::fs;
use url::Url;

use dialoguer::{Input, Select};

use crate::{
    config::sync_config::{rwth_url, Login},
    login::{graphical::login_graphical, rwth::from_rwth, user_pass::from_username_password},
};

use super::*;

pub async fn generate_config() -> Result<()> {
    // Get Login
    let (login, wstoken) = get_login().await?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Getting courses...");

    let mut config = Config::default();
    config.login = login;
    config.wstoken = wstoken;

    config.user_id = config
        .api_acquire_user_id()
        .await
        .context("Failed getting user_id")?;
    let courses = config
        .api_acquire_users_courses()
        .await
        .context("Failed getting courses")?;

    spinner.finish_with_message("Successfully loaded courses!");

    // Config head
    let mut conf = format!(
        r#"# Token for API
wstoken: {}
# Your moodle user id (needed for some operations)
user_id: {}

# Login parameters
login:
"#,
        config.wstoken, config.user_id
    );
    // Add Login parameters
    conf.push_str(&match &config.login {
        Login::ApiOnly { url } => {
            format!(
                r#"  type: ApiOnly
  url: {}

"#,
                url
            )
        }
        Login::Raw { url: _, cookie: _ } => panic!("Raw login config generation not supported!"),
        Login::Graphical { url } => {
            format!(
                r#"  type: Graphical
  url: {}

"#,
                url
            )
        }
        Login::UserPass {
            url,
            username,
            password,
        } => {
            format!(
                r#"  type: UserPass
  url: {}
  username: {}
  password: {}

"#,
                url, username, password
            )
        }
        Login::Rwth {
            url: _,
            username,
            password,
            totp,
            totp_secret,
        } => {
            format!(
                r#"  type: Rwth
  username: {}
  password: {}
  totp: {}
  totp_secret: {}

"#,
                username, password, totp, totp_secret
            )
        }
    });

    // Add Courses
    conf.push_str(
        r#"# Courses
courses:
"#,
    );
    for course in courses {
        conf.push_str(&format!("  - id: {}\n    name: {}\n", course.id, course.shortname));
    }

    conf.push_str(
        r#"

### Additional Options for fine tuning

# Modules to sync
modules:
  - Resource
  - Folder
  - Pdfannotator
  # Assignments:
  - Assign
  - Label
  - Url
  - Page
  - Quiz
  - Glossary
  - Vpl
  # Currently Lti is equivalent to opencast (which requires youtube to be enabled)
  - Lti
  - Grouptool

# Enables saving grades
grades: true

# One of keep "None / Update / Archive"
update_strategy: Archive

# Optionally set path of chrome executable (instead of autodetect
# (may be removed)
#chrome_executable: /usr/bin/chromium-browser

# Enables downloading youtube videos (may be removed)
youtube:
  path: yt-dlp
  params:
    - -N
    - 4
  parallel_downloads: 3

# How webpages should be saved (only one)
page_conversion:
  # # Use the sing-file tool to convert it to an html-document
  # type: SingleFile
  # path: /path/to/single-file
  # # Store entire file as pdf with a single page
  # type: SinglePage
  # Standard chrome pdf
  type: Standard

# Optional: Dir to sync to (may be removed)
# dir: ./cool/path

# Optional: Dir to sync to (may be removed)
log_file: moo-dl.log

# Optional: Regex to filter out files
# Warning: These only get applied for files directly served by the api,
#            to filter other files please remove the corresponding modules directly
file_filters:
  # - reg1
  # - reg2
"#,
    );

    let config_path = ".moo-dl-config.yml";
    fs::write(config_path, conf).await?;

    println!("Successfully written config to: {}", config_path);
    println!("You may now modify it to you liking!");

    Ok(())
}

// Return Login and wstoken
async fn get_login() -> Result<(Login, String)> {
    // Define login options.
    let items = vec![
        "API Only - Provide API capabilities only (needs a wstoken, if you do not know what this is use another option)",
        "Graphical - Use a graphical login interface (opens a webbrowser for login)",
        "User/Pass - Use username and password authentication",
        "RWTH - Use the RWTH SSO with TOTP",
    ];

    // Let the user select a login method.
    let selection = Select::new()
        .with_prompt("Please choose a login method")
        .items(&items)
        .interact()?;

    let (login, wstoken) = match selection {
        0 => {
            // API Only
            let url_str: String = Input::new()
                .with_prompt("Enter your moodle url (e.g. https://moodle.example.com)")
                .interact_text()?;

            let url = Url::parse(&url_str)
                .map_err(|e| anyhow!("Error parsing URL for API Only: {}", e))?;

            let wstoken: String = Input::new()
                .with_prompt("Enter your wstoken")
                .interact_text()?;

            (Login::ApiOnly { url }, wstoken)
        }
        1 => {
            // Graphical
            let url_str: String = Input::new()
                .with_prompt("Enter your moodle url (e.g. https://moodle.example.com)")
                .interact_text()?;

            let url = Url::parse(&url_str)
                .map_err(|e| anyhow!("Error parsing URL for Graphical: {}", e))?;

            let input: String = Input::new()
                .with_prompt("Enter a webbrowser path (leave empty for auto detection)")
                .allow_empty(true)
                .interact_text()
                .unwrap();

            let browser_path = if input.trim().is_empty() {
                None
            } else {
                Some(PathBuf::from(input))
            };

            let spinner = ProgressBar::new_spinner();
            spinner.set_message("Logging in...");

            let login_params = login_graphical(&url, &browser_path, true)
                .await
                .context("Graphical Login failed")?;
            let wstoken = login_params
                .wstoken
                .expect("Could not get wstoken from login");

            spinner.finish_with_message("Successfully logged in!");

            (Login::Graphical { url }, wstoken)
        }
        2 => {
            // User/Pass
            let url_str: String = Input::new()
                .with_prompt("Enter your moodle url (e.g. https://moodle.example.com)")
                .interact_text()?;

            let url = Url::parse(&url_str)
                .map_err(|e| anyhow!("Error parsing URL for User/Pass: {}", e))?;

            let username: String = Input::new()
                .with_prompt("Enter your username")
                .interact_text()?;
            let password: String = Input::new()
                .with_prompt("Enter your password")
                .interact_text()?;

            let spinner = ProgressBar::new_spinner();
            spinner.set_message("Logging in...");

            let login_params = from_username_password(&url, &username, &password, true)
                .await
                .context("Username/Password Login failed")?;
            let wstoken = login_params
                .wstoken
                .expect("Could not get wstoken from login");

            spinner.finish_with_message("Successfully logged in!");

            (
                Login::UserPass {
                    url,
                    username,
                    password,
                },
                wstoken,
            )
        }
        3 => {
            // RWTH
            let url = rwth_url();

            let username: String = Input::new()
                .with_prompt("Enter your RWTH username")
                .interact_text()?;
            let password: String = Input::new()
                .with_prompt("Enter your RWTH password")
                .interact_text()?;
            let totp: String = Input::new()
                .with_prompt("Enter your TOTP token (e.g. TOTP12345678, keep in mind only standard totp generators are supported)")
                .interact_text()?;
            let totp_secret: String = Input::new()
                .with_prompt(
                    "Enter your TOTP Authenticator key (e.g. ABCDEFGHIJKLMNOPQRSTUVWXYZABCDE)",
                )
                .interact_text()?;

            let spinner = ProgressBar::new_spinner();
            spinner.set_message("Logging in...");

            let login_params = from_rwth(&url, &username, &password, &totp, &totp_secret, true)
                .await
                .context("RWTH Login failed")?;
            let wstoken = login_params
                .wstoken
                .expect("Could not get wstoken from login");

            spinner.finish_with_message("Successfully logged in!");

            (
                Login::Rwth {
                    url,
                    username,
                    password,
                    totp,
                    totp_secret,
                },
                wstoken,
            )
        }
        _ => return Err(anyhow!("Invalid selection")),
    };

    Ok((login, wstoken))
}
