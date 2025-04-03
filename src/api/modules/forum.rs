use super::*;

#[derive(Debug, Deserialize)]
pub struct Forum {
    pub id: u64,
    pub name: String,
}