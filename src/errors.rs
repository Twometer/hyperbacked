use std::{error, fmt};

#[derive(Debug)]
pub enum BackupError {
    SharksError(String),
}

impl fmt::Display for BackupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            BackupError::SharksError(message) => write!(f, "SharksError: {}", message),
        }
    }
}

impl error::Error for BackupError {}
