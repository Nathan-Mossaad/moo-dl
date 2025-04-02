use anyhow::anyhow;
use tokio_stream::{wrappers::ReadDirStream, StreamExt};
use url::Url;

use super::*;

impl UpdateStrategy {
    /// Checks if the provided youtube video has already been downloaded in the folder
    /// Provide a folder path instead of a filename, as yt-dlp chooses the file name
    pub async fn youtube_check_exists(url: &Url, dir: &Path) -> Result<UpdateState> {
        let video_id = format!(
            "[{}]",
            get_vid_id(url).ok_or(anyhow!("Could not extract youtube video id"))?
        );

        let read_dir = match fs::read_dir(dir).await {
            Ok(read_dir) => read_dir,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(UpdateState::Missing),
            Err(e) => return Err(e.into()),
        };
        // Wrap the read_dir stream using tokio_stream.
        let mut entries = ReadDirStream::new(read_dir);

        let mut found_valid_file = false;

        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Only consider files that contain the video_id
            if file_name.contains(&video_id) {
                if file_name.ends_with(".ytdl") {
                    // A file with the .ytdl extension was found, so return Missing.
                    return Ok(UpdateState::Missing);
                } else {
                    found_valid_file = true;
                }
            }
        }

        if found_valid_file {
            Ok(UpdateState::UpToDate)
        } else {
            Ok(UpdateState::Missing)
        }
    }
}

/// Extract the YouTube video ID
fn get_vid_id(url: &Url) -> Option<String> {
    match url.host_str()? {
        "www.youtube.com" | "youtube.com" | "www.youtube-nocookie.com" | "youtube-nocookie.com" => {
            // For a standard YouTube URL, try to get the video ID from the query parameter "v".
            if let Some(video_id) = url
                .query_pairs()
                .find(|(key, _)| key == "v")
                .map(|(_, value)| value.into_owned())
            {
                return Some(video_id);
            }
            // If not found, see if the URL is an embed URL.
            let mut segments = url.path_segments()?;
            if let Some(first_segment) = segments.next() {
                if first_segment == "embed" {
                    return segments.next().map(|s| s.to_string());
                }
            }
            None
        }
        "youtu.be" => {
            // For a shortened URL, the video ID is the first segment of the path.
            url.path_segments()
                .and_then(|segments| segments.into_iter().next())
                .map(|s| s.to_string())
        }
        _ => None,
    }
}
