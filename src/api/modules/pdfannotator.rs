use super::{content_types::Content, *};

#[derive(Debug, Deserialize)]
pub struct Pdfannotator {
    pub id: u64,
    pub name: String,
    pub contents: Option<Vec<Content>>,
}