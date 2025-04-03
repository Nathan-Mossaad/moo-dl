use super::*;

#[derive(Debug, Deserialize)]
pub struct Glossary {
    pub id: u64,
    pub name: String,
}