use std::str::FromStr as _;

use anyhow::anyhow;
use once_cell::sync::Lazy;
use regex::Regex;
use select::{document::Document, predicate::Attr};
use tracing::trace;
use url::Url;

use crate::download::youtube::OutputType;

use super::*;

static RE_VIDEO_IDENTIFIER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"video_identifier=([a-f0-9\-]+)&").unwrap());

// This currently is basically RWTH opencast
#[derive(Debug, Deserialize)]
pub struct Lti {
    pub id: u64,
    pub name: String,
    pub modicon: String,
    pub description: Option<String>,
}

impl Download for Lti {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        // Check if we have an rwth-opencast video (only rwth opencast has an opencast icon)
        if self.modicon == "https://moodle.rwth-aachen.de/theme/image.php/boost_union_rwth/theme_boost_union_rwth/-1/opencast_episode?filtericon=1" {
            // We can use the publically available m3u8
            // https://streaming.rwth-aachen.de/rwth_production/_definst_/smil:prod/smil:engage-player_{video_identifier}_presentation.smil/playlist.m3u8
            let re = &RE_VIDEO_IDENTIFIER;

            // Try to get vid_id via description
            let vid_id = match &self.description {
                Some(description) => {
                    // We can get the id using the description
                    if let Some(captures) = re.captures(description) {
                        Some(captures[1].to_owned())
                    } else {
                        None
                    }
                },
                None => None,
            };
            let vid_id = match vid_id {
                Some(vid_id) => vid_id,
                None => {
                    // We couldn't get the vid_id via the description, fallback to extracting it using a SessionCookie
                    // Inspired by <https://github.com/Romern/syncMyMoodle> (Thank you!)
                    let cookie = match config.get_cookie().await {
                        Some(cookie) => cookie,
                        None => {
                            config.status_bar.register_skipped().await;
                            return  Ok(());
                        },
                    };
                    let url = format!("https://moodle.rwth-aachen.de/mod/lti/launch.php?id={}&triggerview=0", &self.id);
                    let response = config
                        .client
                        .get(url)
                        .header(
                            "Cookie",
                            "MoodleSession=".to_string() + &cookie,
                        )
                        .send()
                        .await?;

                    let html = response.text().await?;
                    let document = Document::from(html.as_str());

                    document
                        .find(Attr("name", "custom_id"))
                        .filter_map(|node| node.attr("value"))
                        .next()
                        .ok_or_else(|| anyhow!("custom_id not found in the HTML"))?.to_string()
                },
            };

            trace!("Opencast Video id: {:?}", vid_id);

            let url = Url::from_str(&format!("https://streaming.rwth-aachen.de/rwth_production/_definst_/smil:prod/smil:engage-player_{}_presentation.smil/playlist.m3u8", vid_id))?;
            let file_path = path.join(&self.name).with_extension("mp4");

            config.queue_youtube_video(url, OutputType::File(file_path)).await.context("Failed Opencast")?;
        }

        Ok(())
    }
}
