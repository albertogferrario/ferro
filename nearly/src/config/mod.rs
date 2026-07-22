//! Application configuration registration.

use ferro::{Config, DatabaseConfig};

/// Register all application configs with the framework.
pub fn register_all() {
    Config::register(DatabaseConfig::from_env());
}
