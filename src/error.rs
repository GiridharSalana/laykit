//! Typed errors for LayKit I/O and parsing.

use std::fmt;
use std::io;

/// Primary error type for LayKit operations.
#[derive(Debug, Clone, PartialEq)]
pub enum LaykitError {
    /// Bytes are not a recognized GDSII or OASIS stream.
    UnknownFormat,
    /// Operating-system I/O failure.
    Io(String),
    /// Recognized format but parsing or serialization failed.
    Parse(String),
}

impl fmt::Display for LaykitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LaykitError::UnknownFormat => {
                write!(
                    f,
                    "unknown layout format: expected GDSII or OASIS stream data"
                )
            }
            LaykitError::Io(msg) => write!(f, "I/O error: {msg}"),
            LaykitError::Parse(msg) => write!(f, "parse error: {msg}"),
        }
    }
}

impl std::error::Error for LaykitError {}

impl From<io::Error> for LaykitError {
    fn from(err: io::Error) -> Self {
        LaykitError::Io(err.to_string())
    }
}

/// Alias used by the unified [`crate::layout`] loader.
pub type LoadError = LaykitError;
