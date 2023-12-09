use std::{error::Error, fmt};

/// Error concerning decoding of base58 addresses.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DecodingError {
    /// Unexpected character (char).
    InvalidChar(char),
    /// Checksum failed (expected, actual).
    ChecksumFailed { expected: Vec<u8>, actual: Vec<u8> },
    /// Invalid length (length).
    InvalidLength(usize),
    /// Version byte was not recognized.
    InvalidVersion(u8),
}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodingError::InvalidChar(b) => write!(f, "invalid char ({})", b),
            DecodingError::ChecksumFailed { expected, actual } => write!(
                f,
                "invalid checksum (actual {:?} does not match expected {:?})",
                actual, expected
            ),
            DecodingError::InvalidLength(length) => write!(f, "invalid length ({})", length),
            DecodingError::InvalidVersion(v) => write!(f, "invalid version byte ({})", v),
        }
    }
}

impl Error for DecodingError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
    fn description(&self) -> &str {
        match *self {
            DecodingError::InvalidChar(_) => "invalid char",
            DecodingError::ChecksumFailed { .. } => "invalid checksum",
            DecodingError::InvalidLength(_) => "invalid length",
            DecodingError::InvalidVersion(_) => "invalid version",
        }
    }
}
