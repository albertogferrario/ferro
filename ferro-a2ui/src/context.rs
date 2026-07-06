//! Rendering context for the A2UI renderer.

use crate::catalog::CatalogTier;
use ferro_projections::render::BaseContext;
use ferro_theme::ThemeTemplates;
use serde_json::Value;

/// How collection archetypes emit records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmissionMode {
    /// One shared row template via template binding; guard-invariant actions only.
    Template,
    /// Per-record components built from `A2uiContext::records`; guard-accurate actions.
    Materialized,
}

/// App identity for `surfaceProperties`, read from framework conventions.
#[derive(Debug, Clone, Default)]
pub struct A2uiConfig {
    /// `APP_NAME` — emitted as `agentDisplayName`.
    pub app_name: Option<String>,
    /// `APP_URL` — reserved for future attribution use.
    pub app_url: Option<String>,
}

impl A2uiConfig {
    /// Reads `APP_NAME` / `APP_URL` (the same env vars `AppConfig` consumes).
    pub fn from_env() -> Self {
        Self {
            app_name: std::env::var("APP_NAME").ok(),
            app_url: std::env::var("APP_URL").ok(),
        }
    }
}

/// Context consumed by [`crate::A2uiRenderer`].
#[derive(Debug, Clone, Default)]
pub struct A2uiContext {
    /// Modality-agnostic fields (intent index, evaluated guards, …).
    pub base: BaseContext,
    /// Catalog tier to emit against.
    pub tier: CatalogTier,
    /// Emission-mode override; `None` uses the per-archetype default
    /// (Process → Materialized, everything else → Template).
    pub emission_mode: Option<EmissionMode>,
    /// Surface ID override; `None` derives `ferro-<service>-<intent>`.
    pub surface_id: Option<String>,
    /// Theme slot-template overrides.
    pub templates: Option<ThemeTemplates>,
    /// Live records for materialized emission. Each record may carry the
    /// reserved key `_allowed_actions: [string]` (host-evaluated guards).
    pub records: Option<Vec<Value>>,
    /// Sets `sendDataModel` on the surface (input surfaces set this too).
    pub send_data_model: bool,
    /// App identity for `surfaceProperties`.
    pub config: A2uiConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_default_is_basic_tier_no_overrides() {
        let ctx = A2uiContext::default();
        assert_eq!(ctx.tier, crate::catalog::CatalogTier::Basic);
        assert!(ctx.emission_mode.is_none());
        assert!(ctx.surface_id.is_none());
        assert!(ctx.templates.is_none());
        assert!(ctx.records.is_none());
        assert!(!ctx.send_data_model);
    }

    #[test]
    fn config_from_env_reads_app_identity() {
        // Serialized test: env vars are process-global; this is the only test touching them.
        std::env::set_var("APP_NAME", "Test App");
        std::env::set_var("APP_URL", "https://test.example");
        let cfg = A2uiConfig::from_env();
        assert_eq!(cfg.app_name.as_deref(), Some("Test App"));
        assert_eq!(cfg.app_url.as_deref(), Some("https://test.example"));
        std::env::remove_var("APP_NAME");
        std::env::remove_var("APP_URL");
    }
}
