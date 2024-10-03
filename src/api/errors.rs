use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, Clone)]
pub struct MissingUserIdError;

impl StdError for MissingUserIdError {}

impl fmt::Display for MissingUserIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing user ID")
    }
}
