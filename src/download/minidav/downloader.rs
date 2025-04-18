use anyhow::{anyhow, Context};
use chrono::{DateTime, FixedOffset};
use futures::future::join_all;
use reqwest::header::CONTENT_DISPOSITION;
use serde::{de, Deserialize, Deserializer};
use tracing::{debug, trace};
use url::Url;

use super::*;

#[derive(Debug, Deserialize)]
#[serde(rename = "multistatus")]
struct MultiStatus {
    #[serde(default)]
    response: Vec<Response>,
}

#[derive(Debug, Deserialize)]
struct Response {
    href: String,
    propstat: PropStat,
}

#[derive(Debug, Deserialize)]
struct PropStat {
    prop: Prop,
    status: String,
}

#[derive(Debug, Deserialize)]
struct Prop {
    #[serde(rename = "quota-used-bytes")]
    quota_used_bytes: Option<u64>,
    #[serde(deserialize_with = "deserialize_http_timestamp")]
    getlastmodified: Option<u64>,
}

/// Deserialize getlastmodified string to a Unix timestamp
fn deserialize_http_timestamp<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(date_str) = s {
        let dt: DateTime<FixedOffset> =
            DateTime::parse_from_rfc2822(&date_str).map_err(de::Error::custom)?;
        Ok(Some(dt.timestamp() as u64))
    } else {
        Ok(None)
    }
}

impl Config {
    /// Download all files from webdav respecting updates/archives
    /// Recurses and checks every file manually, as the modification date of folders are unreliable
    pub(super) async fn download_webdav(
        config: Arc<Config>,
        output_folder: &Path,
        url: &Url,
        share_url: Option<&Url>,
        username: &str,
        pass: Option<&str>,
    ) -> Result<()> {
        trace!("Syncing webdav url: {:?}", url);
        // Check against regex filters
        if config
            .check_filter(
                &percent_decode(url.as_str().as_bytes())
                    .decode_utf8_lossy()
                    .into_owned(),
            )
            .await?
        {
            config.status_bar.register_skipped().await;
            return Ok(());
        }

        let deserialized = get_deserialized(&config, url, username, pass).await?;

        // Get respones, ignoring the current folder
        let responses = match deserialized.response.split_first() {
            None => return Ok(()),
            Some((first, rest))
                if first.href.trim_end_matches('/') == url.path().trim_end_matches('/') =>
            {
                // Fallback to a single file sciebo download
                if rest.len() == 0 {
                    if let Some(share_url) = share_url {
                        return download_sciebo_single(
                            config,
                            output_folder,
                            share_url,
                            first.propstat.prop.getlastmodified,
                        )
                        .await;
                    }
                }
                rest
            }
            Some(_) => &deserialized.response[..],
        };

        // Download files or recurse for folders
        let mut folder_tasks: Vec<_> = Vec::new();
        let mut file_tasks: Vec<_> = Vec::new();

        for response in responses.iter() {
            // Check that the resource is available
            if !response.propstat.status.contains("200 OK") {
                continue;
            }

            if response.href.ends_with('/') {
                // Recurse for folders
                folder_tasks.push(async {
                    let url_segment = final_url_segment(&response.href).unwrap_or(".".to_string());
                    let path = output_folder.join(&url_segment);
                    let mut url = url.clone();
                    url.set_path(&response.href);
                    Config::download_webdav(config.clone(), &path, &url, None, username, pass).await
                });
            } else {
                // Create file requests
                file_tasks.push(async {
                    let file = final_url_segment(&response.href).unwrap_or(".".to_string());
                    // Check against regex filters
                    if config.check_filter(&file).await? {
                        config.status_bar.register_skipped().await;
                        return Ok(());
                    }

                    let path = output_folder.join(&file);

                    let mut url = url.clone();
                    url.set_path(&response.href);
                    let request = config.client.get(url).basic_auth(username, pass);
                    config
                        .download_file_option_timestamp(
                            &path,
                            request,
                            response.propstat.prop.getlastmodified,
                            response.propstat.prop.quota_used_bytes,
                        )
                        .await
                });
            }
        }

        // Return an error if one occured
        let (folder_res, file_res) = tokio::join!(join_all(folder_tasks), join_all(file_tasks));
        for res in folder_res {
            res.context("Failed Downloading file in sciebo")?;
        }
        for res in file_res {
            res.context("Failed Downloading file in sciebo")?;
        }
        Ok(())
    }
}

/// Download a shared sciebo link as a single file (Needed for single files, as they don't get exposed via webdav)
async fn download_sciebo_single(
    config: Arc<Config>,
    output_folder: &Path,
    sciebo_share_url: &Url,
    timestamp: Option<u64>,
) -> Result<()> {
    // Get download url
    let mut sciebo_share_url = sciebo_share_url.to_owned();
    sciebo_share_url
        .path_segments_mut()
        .map_err(|_| anyhow!("Could net get mutable path segments"))?
        .push("download");
    trace!("Syncing sciebo file: {:?}", sciebo_share_url);

    // Get filename
    let filename = get_filename_from_url_simple(&config, &sciebo_share_url).await?;
    // Check against regex filters
    if config.check_filter(&filename).await? {
        config.status_bar.register_skipped().await;
        return Ok(());
    }

    // Download file
    let path = output_folder.join(&filename);

    let request = config.client.get(sciebo_share_url);
    config
        .download_file_option_timestamp(&path, request, timestamp, None)
        .await?;

    Ok(())
}

async fn get_deserialized(
    config: &Config,
    url: &Url,
    username: &str,
    pass: Option<&str>,
) -> Result<MultiStatus> {
    let response = config
        .client
        .request(reqwest::Method::from_bytes(b"PROPFIND")?, url.to_owned())
        .basic_auth(username, pass)
        .send()
        .await?;

    let xml = response.text().await?;

    Ok(quick_xml::de::from_str(&xml)?)
}

pub async fn get_filename_from_url_simple(config: &Config, url: &Url) -> Result<String> {
    let response = config.client.head(url.as_str()).send().await?;

    let headers = response.headers();

    let header_value = headers
        .get(CONTENT_DISPOSITION)
        .ok_or(anyhow!("Content-Disposition header not found"))?
        .to_str()?;
    trace!("Content-Disposition header: {:?}", header_value);

    let re = Regex::new(r#"filename\*=UTF-8''.*;"#)?;
    if let Some(capture) = re.captures(header_value) {
        if let Some(name) = capture.get(0) {
            let name = &name.as_str()[17..name.len() - 1];
            debug!("Extracted sciebo name: {:?}", name);
            return Ok(name.to_string());
        }
    }
    Err(anyhow!("unknown_filename"))
}
