use std::{error::Error, fmt};

/// Message has unexpected schema or other parsing error.
#[derive(Debug)]
pub struct KeybaseMessageParseError;

impl Error for KeybaseMessageParseError {}

impl fmt::Display for KeybaseMessageParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error parsing Keybase message.")
    }
}
