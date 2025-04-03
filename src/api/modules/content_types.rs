use super::*;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "file")]
    File(File),
    #[serde(rename = "url")]
    Url(Url),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct File {
    filename: String,
    filepath: String,
    fileurl: String,
    timemodified: u64,
}

#[derive(Debug, Deserialize)]
pub struct Url {
    filename: String,
    fileurl: String,
    timemodified: u64,
}
