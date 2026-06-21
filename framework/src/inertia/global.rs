//! Process-global Inertia configuration.
//!
//! Set once at bootstrap via [`set_inertia_config`] (or `App::set_inertia_config`),
//! read by the common render path. Falls back to `InertiaConfig::from_env()`/default()
//! when unset, so apps that never set it keep working unchanged.

use ferro_inertia::InertiaConfig;
use std::sync::OnceLock;

static INERTIA_CONFIG: OnceLock<InertiaConfig> = OnceLock::new();

/// Set the process-global `InertiaConfig`. Call once from bootstrap before the
/// server starts accepting requests. Subsequent calls are ignored (`OnceLock`
/// semantics); a warning is emitted on the second call to surface the mistake.
pub fn set_inertia_config(config: InertiaConfig) {
    if INERTIA_CONFIG.set(config).is_err() {
        eprintln!("Warning: InertiaConfig already set; second call ignored");
    }
}

/// Get the active `InertiaConfig`, falling back to `from_env()`/`default()` when unset.
pub fn get_inertia_config() -> InertiaConfig {
    INERTIA_CONFIG
        .get()
        .cloned()
        .unwrap_or_else(InertiaConfig::default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_inertia_config_falls_back_to_default_when_unset() {
        // In a fresh test process with nothing set, get() returns the env/default config.
        // Assert structural defaults that hold regardless of ambient env.
        let c = get_inertia_config();
        assert_eq!(c.mount_id, "app");
    }
}
