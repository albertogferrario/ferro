use ferro::{Config, DatabaseConfig as FerroDatabaseConfig};

/// Register all application configs.
pub fn register_all() {
    Config::register(FerroDatabaseConfig::from_env());
}
