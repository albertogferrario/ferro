//! Plugin system for JSON-UI custom interactive components.
//!
//! Provides a trait-based extension point where custom components (Map, Chart,
//! Editor, etc.) register themselves with the framework. Each plugin declares
//! its component type name, props schema, render function, and required
//! JS/CSS assets.
//!
//! A global `PluginRegistry` (mirroring the `LayoutRegistry` pattern) maps
//! component type names to plugin implementations. The renderer checks
//! built-in components first, then falls back to the plugin registry for
//! unknown types.

use std::collections::{HashMap, HashSet};
use std::sync::{OnceLock, RwLock};

// ── Asset type ─────────────────────────────────────────────────────────

/// A JS or CSS asset required by a plugin.
///
/// Rendered as a `<script>` or `<link>` tag in the HTML output.
/// Optional `integrity` and `crossorigin` attributes enable
/// Subresource Integrity (SRI) for CDN-loaded assets.
pub struct Asset {
    /// URL of the asset (JS or CSS file).
    pub url: String,
    /// SRI hash for integrity verification (e.g., "sha256-...").
    pub integrity: Option<String>,
    /// Crossorigin attribute value (e.g., "" for anonymous).
    pub crossorigin: Option<String>,
}

impl Asset {
    /// Create a new asset with just a URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            integrity: None,
            crossorigin: None,
        }
    }

    /// Set the integrity hash (builder pattern).
    pub fn integrity(mut self, hash: impl Into<String>) -> Self {
        self.integrity = Some(hash.into());
        self
    }

    /// Set the crossorigin attribute (builder pattern).
    pub fn crossorigin(mut self, value: impl Into<String>) -> Self {
        self.crossorigin = Some(value.into());
        self
    }
}

// ── Plugin trait ───────────────────────────────────────────────────────

/// Trait for JSON-UI plugin components.
///
/// Plugins provide custom interactive components that require client-side
/// JS/CSS. Each plugin declares a unique component type name, a JSON
/// Schema for its props (enabling MCP/agent discovery), a render function
/// producing HTML, and asset declarations for the page.
///
/// Implementations must be `Send + Sync` for use in the global registry
/// across threads.
pub trait JsonUiPlugin: Send + Sync {
    /// Unique component type name (e.g., "Map").
    ///
    /// Used in JSON: `{"type": "Map", ...}`. Must not collide with
    /// built-in component type names.
    fn component_type(&self) -> &str;

    /// JSON Schema describing accepted props.
    ///
    /// Used by MCP/agents for discovery and validation. Should return
    /// a valid JSON Schema object.
    fn props_schema(&self) -> serde_json::Value;

    /// Render the component to an HTML string.
    ///
    /// Receives the raw props and the view data for data_path resolution.
    fn render(&self, props: &serde_json::Value, data: &serde_json::Value) -> String;

    /// CSS assets to load in `<head>`.
    ///
    /// Called once per page; results are deduplicated by URL across all
    /// plugin instances on the page.
    fn css_assets(&self) -> Vec<Asset>;

    /// JS assets to load before `</body>`.
    ///
    /// Called once per page; results are deduplicated by URL across all
    /// plugin instances on the page.
    fn js_assets(&self) -> Vec<Asset>;

    /// Inline initialization JS emitted once per page after assets load.
    ///
    /// Returns `None` if no initialization is needed.
    fn init_script(&self) -> Option<String>;
}

// ── Plugin registry ────────────────────────────────────────────────────

/// Registry mapping component type names to plugin implementations.
///
/// Created empty by default. Plugins are registered at application startup.
/// Follows the same `HashMap<String, Box<dyn T>>` pattern as `LayoutRegistry`.
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn JsonUiPlugin>>,
}

impl PluginRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin. Replaces any existing plugin with the same component type.
    pub fn register(&mut self, plugin: impl JsonUiPlugin + 'static) {
        let name = plugin.component_type().to_string();
        self.plugins.insert(name, Box::new(plugin));
    }

    /// Look up a plugin by component type name.
    pub fn get(&self, component_type: &str) -> Option<&dyn JsonUiPlugin> {
        self.plugins.get(component_type).map(|p| p.as_ref())
    }

    /// Return a sorted list of all registered plugin type names.
    pub fn registered_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self.plugins.keys().cloned().collect();
        types.sort();
        types
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Global registry ────────────────────────────────────────────────────

static GLOBAL_PLUGIN_REGISTRY: OnceLock<RwLock<PluginRegistry>> = OnceLock::new();

/// Access the global plugin registry.
///
/// Lazily initialized on first call with built-in plugins registered.
pub fn global_plugin_registry() -> &'static RwLock<PluginRegistry> {
    GLOBAL_PLUGIN_REGISTRY.get_or_init(|| {
        let mut registry = PluginRegistry::new();
        registry.register(crate::plugins::MapPlugin);
        RwLock::new(registry)
    })
}

/// Register a plugin in the global registry.
///
/// Convenience wrapper around `global_plugin_registry().write()`.
pub fn register_plugin(plugin: impl JsonUiPlugin + 'static) {
    global_plugin_registry()
        .write()
        .expect("plugin registry poisoned")
        .register(plugin);
}

/// Look up a plugin by component type name in the global registry.
///
/// Acquires a read lock on the global registry, checks if the plugin
/// exists, and calls the provided closure with a reference to it.
/// Returns `None` if no plugin is registered for the given type.
///
/// The closure pattern avoids lifetime issues with returning references
/// through the RwLock guard.
pub fn with_plugin<R>(component_type: &str, f: impl FnOnce(&dyn JsonUiPlugin) -> R) -> Option<R> {
    let guard = global_plugin_registry()
        .read()
        .expect("plugin registry poisoned");
    guard.get(component_type).map(f)
}

/// Return a sorted list of all registered plugin type names.
///
/// Useful for MCP/agent discovery of available plugin components.
pub fn registered_plugin_types() -> Vec<String> {
    global_plugin_registry()
        .read()
        .expect("plugin registry poisoned")
        .registered_types()
}

// ── Asset collection ───────────────────────────────────────────────────

/// Collected and deduplicated assets from all plugins used on a page.
pub struct CollectedAssets {
    /// CSS `<link>` tags for `<head>`.
    pub css: Vec<Asset>,
    /// JS `<script>` tags for before `</body>`.
    pub js: Vec<Asset>,
    /// Inline init scripts to emit after JS assets.
    pub init_scripts: Vec<String>,
}

/// Collect and deduplicate assets from a list of plugin type names.
///
/// Given the set of plugin types used on a page, looks up each plugin
/// in the global registry and aggregates their CSS assets, JS assets,
/// and init scripts. Assets are deduplicated by URL.
pub fn collect_plugin_assets(plugin_types: &[String]) -> CollectedAssets {
    let registry = global_plugin_registry()
        .read()
        .expect("plugin registry poisoned");

    let mut css_seen = HashSet::new();
    let mut js_seen = HashSet::new();
    let mut css = Vec::new();
    let mut js = Vec::new();
    let mut init_scripts = Vec::new();

    for type_name in plugin_types {
        if let Some(plugin) = registry.get(type_name) {
            for asset in plugin.css_assets() {
                if css_seen.insert(asset.url.clone()) {
                    css.push(asset);
                }
            }
            for asset in plugin.js_assets() {
                if js_seen.insert(asset.url.clone()) {
                    js.push(asset);
                }
            }
            if let Some(script) = plugin.init_script() {
                init_scripts.push(script);
            }
        }
    }

    CollectedAssets {
        css,
        js,
        init_scripts,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// A test plugin for verification.
    struct TestPlugin;

    impl JsonUiPlugin for TestPlugin {
        fn component_type(&self) -> &str {
            "TestWidget"
        }

        fn props_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "label": { "type": "string" }
                }
            })
        }

        fn render(&self, props: &serde_json::Value, _data: &serde_json::Value) -> String {
            let label = props
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            format!("<div class=\"test-widget\">{label}</div>")
        }

        fn css_assets(&self) -> Vec<Asset> {
            vec![Asset::new("https://cdn.example.com/widget.css")
                .integrity("sha256-abc123")
                .crossorigin("")]
        }

        fn js_assets(&self) -> Vec<Asset> {
            vec![Asset::new("https://cdn.example.com/widget.js")]
        }

        fn init_script(&self) -> Option<String> {
            Some("initWidgets();".to_string())
        }
    }

    struct NoAssetPlugin;

    impl JsonUiPlugin for NoAssetPlugin {
        fn component_type(&self) -> &str {
            "NoAsset"
        }

        fn props_schema(&self) -> serde_json::Value {
            serde_json::json!({})
        }

        fn render(&self, _props: &serde_json::Value, _data: &serde_json::Value) -> String {
            "<span>no-asset</span>".to_string()
        }

        fn css_assets(&self) -> Vec<Asset> {
            vec![]
        }

        fn js_assets(&self) -> Vec<Asset> {
            vec![]
        }

        fn init_script(&self) -> Option<String> {
            None
        }
    }

    // ── Asset tests ────────────────────────────────────────────────

    #[test]
    fn asset_builder_sets_all_fields() {
        let asset = Asset::new("https://example.com/lib.js")
            .integrity("sha256-xyz")
            .crossorigin("anonymous");

        assert_eq!(asset.url, "https://example.com/lib.js");
        assert_eq!(asset.integrity.as_deref(), Some("sha256-xyz"));
        assert_eq!(asset.crossorigin.as_deref(), Some("anonymous"));
    }

    #[test]
    fn asset_new_has_no_integrity_or_crossorigin() {
        let asset = Asset::new("https://example.com/lib.js");
        assert!(asset.integrity.is_none());
        assert!(asset.crossorigin.is_none());
    }

    // ── PluginRegistry tests ───────────────────────────────────────

    #[test]
    fn registry_starts_empty() {
        let registry = PluginRegistry::new();
        assert!(registry.registered_types().is_empty());
    }

    #[test]
    fn registry_register_and_get() {
        let mut registry = PluginRegistry::new();
        registry.register(TestPlugin);

        let plugin = registry.get("TestWidget");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().component_type(), "TestWidget");
    }

    #[test]
    fn registry_get_returns_none_for_unknown() {
        let registry = PluginRegistry::new();
        assert!(registry.get("NonExistent").is_none());
    }

    #[test]
    fn registry_registered_types_sorted() {
        let mut registry = PluginRegistry::new();
        registry.register(TestPlugin);
        registry.register(NoAssetPlugin);

        let types = registry.registered_types();
        assert_eq!(types, vec!["NoAsset", "TestWidget"]);
    }

    #[test]
    fn registry_register_replaces_existing() {
        let mut registry = PluginRegistry::new();

        struct PluginV1;
        impl JsonUiPlugin for PluginV1 {
            fn component_type(&self) -> &str {
                "Same"
            }
            fn props_schema(&self) -> serde_json::Value {
                serde_json::json!({"v": 1})
            }
            fn render(&self, _: &serde_json::Value, _: &serde_json::Value) -> String {
                "v1".to_string()
            }
            fn css_assets(&self) -> Vec<Asset> {
                vec![]
            }
            fn js_assets(&self) -> Vec<Asset> {
                vec![]
            }
            fn init_script(&self) -> Option<String> {
                None
            }
        }

        struct PluginV2;
        impl JsonUiPlugin for PluginV2 {
            fn component_type(&self) -> &str {
                "Same"
            }
            fn props_schema(&self) -> serde_json::Value {
                serde_json::json!({"v": 2})
            }
            fn render(&self, _: &serde_json::Value, _: &serde_json::Value) -> String {
                "v2".to_string()
            }
            fn css_assets(&self) -> Vec<Asset> {
                vec![]
            }
            fn js_assets(&self) -> Vec<Asset> {
                vec![]
            }
            fn init_script(&self) -> Option<String> {
                None
            }
        }

        registry.register(PluginV1);
        registry.register(PluginV2);

        let plugin = registry.get("Same").unwrap();
        let html = plugin.render(&serde_json::json!({}), &serde_json::json!({}));
        assert_eq!(html, "v2");
    }

    // ── Plugin rendering tests ─────────────────────────────────────

    #[test]
    fn plugin_renders_html() {
        let plugin = TestPlugin;
        let html = plugin.render(
            &serde_json::json!({"label": "Hello"}),
            &serde_json::json!({}),
        );
        assert_eq!(html, "<div class=\"test-widget\">Hello</div>");
    }

    #[test]
    fn plugin_returns_schema() {
        let plugin = TestPlugin;
        let schema = plugin.props_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["label"].is_object());
    }

    // ── collect_plugin_assets tests ────────────────────────────────

    #[test]
    fn collect_assets_from_registry() {
        // Register plugins globally for this test
        register_plugin(TestPlugin);
        register_plugin(NoAssetPlugin);

        let assets = collect_plugin_assets(&["TestWidget".to_string()]);
        assert_eq!(assets.css.len(), 1);
        assert_eq!(assets.css[0].url, "https://cdn.example.com/widget.css");
        assert_eq!(assets.js.len(), 1);
        assert_eq!(assets.js[0].url, "https://cdn.example.com/widget.js");
        assert_eq!(assets.init_scripts.len(), 1);
        assert_eq!(assets.init_scripts[0], "initWidgets();");
    }

    #[test]
    fn collect_assets_deduplicates_by_url() {
        // Ensure plugin is registered (idempotent due to global registry)
        register_plugin(TestPlugin);

        // Requesting same plugin type twice should not duplicate assets.
        let assets = collect_plugin_assets(&["TestWidget".to_string(), "TestWidget".to_string()]);
        assert_eq!(assets.css.len(), 1);
        assert_eq!(assets.js.len(), 1);
    }

    #[test]
    fn collect_assets_empty_for_unknown_types() {
        let assets = collect_plugin_assets(&["NonExistentPlugin".to_string()]);
        assert!(assets.css.is_empty());
        assert!(assets.js.is_empty());
        assert!(assets.init_scripts.is_empty());
    }

    #[test]
    fn collect_assets_handles_no_asset_plugin() {
        register_plugin(NoAssetPlugin);
        let assets = collect_plugin_assets(&["NoAsset".to_string()]);
        assert!(assets.css.is_empty());
        assert!(assets.js.is_empty());
        assert!(assets.init_scripts.is_empty());
    }

    // ── Global registry tests ──────────────────────────────────────

    #[test]
    fn global_registry_returns_valid_registry() {
        let reg = global_plugin_registry();
        let guard = reg.read().unwrap();
        // The key test is that it doesn't panic when accessing the global registry.
        let _ = guard.registered_types();
    }

    #[test]
    fn registered_plugin_types_returns_sorted_list() {
        // Already registered TestWidget and NoAsset above
        let types = registered_plugin_types();
        // The global registry persists across tests, so we just check it doesn't panic
        // and returns a sorted list
        let mut sorted = types.clone();
        sorted.sort();
        assert_eq!(types, sorted);
    }
}
