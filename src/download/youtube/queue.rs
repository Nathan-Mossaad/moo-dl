use std::sync::Arc;

use futures::future::join_all;
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
        let youtube_vid = Arc::new(YoutubeVideo {url, output_folder});
        self.youtube_queue.sender.send(youtube_vid).await?;
        Ok(())
    }
}

impl YoutubeDownloadQueue {
    pub async fn wait_for_completeion(self) {
        join_all(self.threads).await;
    }
}
