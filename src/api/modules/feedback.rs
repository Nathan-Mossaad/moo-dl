use super::*;

#[derive(Debug, Deserialize)]
pub struct Feedback {
    pub id: u64,
    pub name: String,
}