use super::*;

#[derive(Debug, Deserialize)]
pub struct Label {
    pub id: u64,
    pub name: String,
    pub description: String,
}
