//! Error types and handling

use thiserror::Error;

/// Result type alias using our Error type
pub type Result = std::result::Result;

/// Application error types
#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}