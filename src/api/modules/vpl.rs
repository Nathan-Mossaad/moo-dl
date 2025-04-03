use super::*;

#[derive(Debug, Deserialize)]
pub struct Vpl {
    pub id: u64,
    pub name: String,
    pub url: String,
}
