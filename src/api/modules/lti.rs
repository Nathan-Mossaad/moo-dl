use std::str::FromStr as _;

use regex::Regex;
use tracing::trace;
use url::Url;

use crate::download::youtube::OutputType;

use super::*;

// This currently is basically RWTH opencast
#[derive(Debug, Deserialize)]
pub struct Lti {
    pub id: u64,
    pub name: String,
    pub modicon: String,
    pub description: String,
}

impl Download for Lti {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        // Check if we have an rwth-opencast video (only rwth opencast has an opencast icon)
        if self.modicon == "https://moodle.rwth-aachen.de/theme/image.php/boost_union_rwth/theme_boost_union_rwth/-1/opencast_episode?filtericon=1" {
            // We can use the publically available m3u8
            // https://streaming.rwth-aachen.de/rwth_production/_definst_/smil:prod/smil:engage-player_{video_identifier}_presentation.smil/playlist.m3u8
            let re = Regex::new(r"video_identifier=([a-f0-9\-]+)&").unwrap();

            if let Some(captures) = re.captures(&self.description) {
                // Capture group 1 holds the value of the identifier
                let vid_id = &captures[1];
                trace!("Opencast Video id: {:?}", vid_id);

                let url = Url::from_str(&format!("https://streaming.rwth-aachen.de/rwth_production/_definst_/smil:prod/smil:engage-player_{}_presentation.smil/playlist.m3u8", vid_id))?;
                let file_path = path.join(&self.name).with_extension("mp4");

                config.queue_youtube_video(url, OutputType::File(file_path)).await.context("Failed Opencast")?;
            } else {
                config.status_bar.register_err(&format!("Failed getting Opencast: {}", &self.id)).await;
            }
        }

        Ok(())
    }
}
