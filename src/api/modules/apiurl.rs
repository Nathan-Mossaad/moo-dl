use super::{content_types::Content, *};

#[derive(Debug, Deserialize)]
pub struct ApiUrl {
    pub id: u64,
    pub name: String,
    pub contents: Vec<Content>,
}