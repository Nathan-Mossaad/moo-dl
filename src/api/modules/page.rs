use super::{content_types::Content, *};

#[derive(Debug, Deserialize)]
pub struct Page {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub contents: Option<Vec<Content>>,
}