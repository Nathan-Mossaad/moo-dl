use super::*;

#[derive(Debug, Deserialize)]
pub struct Quiz {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub instance: u64,
}