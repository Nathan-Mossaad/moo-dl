use super::*;

#[derive(Debug, Deserialize)]
pub struct Assign {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
}
