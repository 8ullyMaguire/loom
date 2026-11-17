//! Bundle invariants I1-I3 from the specification.
use base64::Engine;
use rand::RngCore;
use thiserror::Error;

pub const MAX_FEEDS: usize = 100;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BundleError {
    #[error("a bundle needs between 1 and 100 feeds, got {0}")]
    BadFeedCount(usize),
    #[error("a bundle name must be 1 to 80 characters")]
    BadName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bundle {
    pub name: String,
    pub uris: Vec<String>,
    pub edit_token: String,
}

impl Bundle {
    pub fn new(name: &str, uris: Vec<String>) -> Result<Self, BundleError> {
        if name.is_empty() || name.chars().count() > 80 {
            return Err(BundleError::BadName);
        }
        if uris.is_empty() || uris.len() > MAX_FEEDS {
            return Err(BundleError::BadFeedCount(uris.len()));
        }
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        let edit_token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
        Ok(Self {
            name: name.to_string(),
            uris,
            edit_token,
        })
    }
}
