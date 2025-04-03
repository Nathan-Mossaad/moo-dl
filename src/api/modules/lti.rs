use super::*;

// This currently is basically RWTH opencast
#[derive(Debug, Deserialize)]
pub struct Lti {
    pub id: u64,
    pub name: String,
}
