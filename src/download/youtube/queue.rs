use std::sync::Arc;

use futures::future::join_all;
use regex::Regex;
use tokio::task::JoinHandle;

use super::*;

pub struct YoutubeDownloadQueue {
    threads: Vec<JoinHandle<()>>,
}

impl Config {
    async fn download_thread(&self) {
        while let Ok(vid) = self.youtube_queue.receiver.recv().await {
            if let Err(e) = self
                .direct_download_youtube(&vid.url, &vid.output_folder)
                .await
            {
                let context = format!("Failed to download video {}", vid.url.as_str());
                self.status_bar
                    .register_err(&e.context(context).to_string())
                    .await;
            }
        }
    }

    pub async fn create_youtube_download_threads(config: Arc<Config>) -> YoutubeDownloadQueue {
        let mut youtube_download_queue = YoutubeDownloadQueue { threads: vec![] };

        if let Some(yt_conf) = &config.youtube {
            for _ in 0..yt_conf.parallel_downloads {
                // Create new thread
                let thread_self = config.clone();
                let thread_handle = tokio::spawn(async move {
                    thread_self.download_thread().await;
                });
                youtube_download_queue.threads.push(thread_handle);
            }
        }

        youtube_download_queue
    }

    pub async fn queue_youtube_video(&self, url: Url, output_folder: PathBuf) -> Result<()> {
        let youtube_vid = Arc::new(YoutubeVideo { url, output_folder });
        self.youtube_queue.sender.send(youtube_vid).await?;
        Ok(())
    }

    /// Extracts YouTube URLs from the given `search_space` and queues them for download.
    pub async fn queue_youtube_vidoes_extract(
        &self,
        search_space: &str,
        output_folder: PathBuf,
    ) -> Result<()> {
        // Regex adapted from: https://stackoverflow.com/questions/19377262/regex-for-youtube-url
        let re = Regex::new(r#"(?:https?:?\/\/)?((?:www|m)\.)?((?:youtube(?:-nocookie)?\.com|youtu.be))(\/(?:[\w\-]+\?v=|embed\/|live\/|v\/)?)([\w\-]+)(\S+)?"#)?;
        // Iterate over every match in the search_space.
        for cap in re.captures_iter(search_space) {
            if let Some(url_str) = cap.get(0) {
                let url_value = url_str.as_str();

                // Validate and parse the url into the Url struct.
                if let Ok(parsed_url) = Url::parse(url_value) {
                    tracing::trace!("Invalid URL: {:?}", &parsed_url);
                    // Clone output_folder for each async call.
                    self.queue_youtube_video(parsed_url, output_folder.clone()).await?;
                } else {
                    
                }
            }
        }
        Ok(())
    }
}

impl YoutubeDownloadQueue {
    pub async fn wait_for_completion(self) {
        join_all(self.threads).await;
    }
}
