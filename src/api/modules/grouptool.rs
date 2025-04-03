use super::*;

#[derive(Debug, Deserialize)]
pub struct Grouptool {
    pub id: u64,
    pub name: String,
    pub url: String,
}