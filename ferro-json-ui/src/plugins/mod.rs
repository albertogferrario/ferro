//! Built-in plugin components for JSON-UI.
//!
//! Provides the Map plugin and a registration function that adds all
//! built-in plugins to the global `PluginRegistry`.

pub mod map;
pub mod rich_text_editor;

pub use map::MapPlugin;
pub use rich_text_editor::RichTextEditorPlugin;

/// Register all built-in plugins into the global registry.
///
/// Called during `PluginRegistry` global initialization so built-in
/// plugins are always available without explicit user registration.
pub fn register_built_in_plugins() {
    crate::plugin::register_plugin(MapPlugin);
    crate::plugin::register_plugin(RichTextEditorPlugin);
}
