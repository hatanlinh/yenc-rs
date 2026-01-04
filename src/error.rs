//! Error types for yEnc operations

use std::fmt;
use std::io;

/// Main error type for yEnc operations
#[derive(Debug)]
pub enum YencError {
    /// I/O error occurred
    Io(io::Error),
    /// Invalid yEnc header
    InvalidHeader(String),
    /// Invalid yEnc data
    InvalidData(String),
    /// Missing required header field
    MissingField(String),
    /// CRC mismatch
    CrcMismatch { expected: u32, actual: u32 },
}

impl fmt::Display for YencError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YencError::Io(err) => write!(f, "I/O error: {}", err),
            YencError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
            YencError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            YencError::MissingField(field) => write!(f, "Missing required field: {}", field),
            YencError::CrcMismatch { expected, actual } => {
                write!(
                    f,
                    "CRC mismatch: expected {:#x}, got {:#x}",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for YencError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            YencError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for YencError {
    fn from(err: io::Error) -> Self {
        YencError::Io(err)
    }
}

/// A specialized `Result` type for yEnc operations
pub type Result<T> = std::result::Result<T, YencError>;
