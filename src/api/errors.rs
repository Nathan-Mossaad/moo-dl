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

#[derive(Debug, Clone)]
pub struct LoginFailedError {
    message: String,
}

impl LoginFailedError {
    pub fn new(message: impl Into<String>) -> Self {
        LoginFailedError {
            message: message.into(),
        }
    }
}

impl StdError for LoginFailedError {}

impl fmt::Display for LoginFailedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Login failed with reason: {}", self.message)
    }
}

#[derive(Debug, Clone)]
pub struct BrowserStartFailedError {
    message: String,
}

impl BrowserStartFailedError {
    pub fn new(message: impl Into<String>) -> Self {
        BrowserStartFailedError {
            message: message.into(),
        }
    }
}

impl StdError for BrowserStartFailedError {}

impl fmt::Display for BrowserStartFailedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Starting Browser failed with reason: {}", self.message)
    }
}
