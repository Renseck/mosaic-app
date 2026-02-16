pub mod cli;
pub mod config;
pub mod core;
pub mod error;

pub use error::{Error, Result};

/// Project version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");