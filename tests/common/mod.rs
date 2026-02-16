//! Common test utilities

use mosaic_app::config::Config;

/// Create a test configuration
pub fn test_config() -> Config {
    Config {
        environment: "test".to_string(),
        debug: true,
        ..Default::default()
    }
}