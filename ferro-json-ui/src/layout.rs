//! Layout system for JSON-UI page rendering.
//!
//! Provides a trait-based layout system where named layouts wrap rendered
//! component HTML in full page shells. Three built-in layouts are provided:
//! `DefaultLayout` (minimal), `AppLayout` (dashboard with nav + sidebar),
//! and `AuthLayout` (centered card).
//!
//! A global `LayoutRegistry` maps layout names to implementations. Views
//! specify a layout via `JsonUiView.layout`, and the render pipeline looks
//! it up in the registry.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use crate::render::html_escape;

// ── Layout context ──────────────────────────────────────────────────────

/// Context passed to layout render functions.
///
/// Contains all data a layout needs to produce a complete HTML page:
/// the rendered component HTML, page metadata, and serialized view/data
/// for potential frontend hydration.
pub struct LayoutContext<'a> {
    /// Page title for the `<title>` element.
    pub title: &'a str,
    /// Rendered component HTML fragment (output of `render_to_html`).
    pub content: &'a str,
    /// Additional `<head>` content (Tailwind CDN link, custom styles).
    pub head: &'a str,
    /// CSS classes for the `<body>` element.
    pub body_class: &'a str,
    /// Serialized view JSON for the `data-view` attribute.
    pub view_json: &'a str,
    /// Serialized data JSON for the `data-props` attribute.
    pub data_json: &'a str,
    /// JS assets and init scripts for plugins, injected before closing body tag.
    pub scripts: &'a str,
}

// ── Layout trait ────────────────────────────────────────────────────────

/// Trait for layout implementations.
///
/// Layouts produce a complete HTML page string wrapping the rendered
/// component content. They must be `Send + Sync` for use in the global
/// registry across threads.
pub trait Layout: Send + Sync {
    /// Render a complete HTML page using the provided context.
    fn render(&self, ctx: &LayoutContext) -> String;
}

// ── Base document helper ────────────────────────────────────────────────

/// Produce the common `<!DOCTYPE html>` shell shared by all built-in layouts.
///
/// All three built-in layouts delegate to this function to avoid duplicating
/// the HTML/head/body boilerplate. The `body_content` parameter receives the
/// inner body HTML which varies per layout.
fn base_document(
    title: &str,
    head: &str,
    body_class: &str,
    body_content: &str,
    scripts: &str,
) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    {head}
</head>
<body class="{body_class}">
    {body_content}
    {scripts}
</body>
</html>"#,
        title = html_escape(title),
        head = head,
        body_class = html_escape(body_class),
        body_content = body_content,
        scripts = scripts,
    )
}

/// Produce the ferro-json-ui wrapper div with data attributes.
fn ferro_wrapper(ctx: &LayoutContext) -> String {
    format!(
        r#"<div id="ferro-json-ui" data-view="{view}" data-props="{props}">{content}</div>"#,
        view = html_escape(ctx.view_json),
        props = html_escape(ctx.data_json),
        content = ctx.content,
    )
}

// ── DefaultLayout ───────────────────────────────────────────────────────

/// Minimal layout wrapping content in a valid HTML page.
///
/// Produces the same structure as the existing framework HTML shell:
/// doctype, meta tags, title, head content, body with the ferro-json-ui
/// wrapper div containing the rendered components.
pub struct DefaultLayout;

impl Layout for DefaultLayout {
    fn render(&self, ctx: &LayoutContext) -> String {
        let wrapper = ferro_wrapper(ctx);
        base_document(ctx.title, ctx.head, ctx.body_class, &wrapper, ctx.scripts)
    }
}

// ── AppLayout ───────────────────────────────────────────────────────────

/// Dashboard-style layout with navigation bar, sidebar, and main content area.
///
/// Uses a flex layout with the sidebar on the left and main content on the
/// right. The ferro-json-ui wrapper div is placed inside the `<main>` element.
///
/// By default, renders empty navigation and sidebar placeholders. Users create
/// custom Layout implementations that call the partial functions with real data.
pub struct AppLayout;

impl Layout for AppLayout {
    fn render(&self, ctx: &LayoutContext) -> String {
        let nav = navigation(&[]);
        let side = sidebar(&[]);
        let wrapper = ferro_wrapper(ctx);

        let body = format!(
            r#"{nav}
    <div class="flex">
        {side}
        <main class="flex-1 p-6">
            {wrapper}
        </main>
    </div>"#,
            nav = nav,
            side = side,
            wrapper = wrapper,
        );

        base_document(ctx.title, ctx.head, ctx.body_class, &body, ctx.scripts)
    }
}

// ── AuthLayout ──────────────────────────────────────────────────────────

/// Centered card layout for authentication pages (login, register).
///
/// Centers the content vertically and horizontally within a max-width
/// container. No navigation or sidebar.
pub struct AuthLayout;

impl Layout for AuthLayout {
    fn render(&self, ctx: &LayoutContext) -> String {
        let wrapper = ferro_wrapper(ctx);

        let body = format!(
            r#"<div class="min-h-screen flex items-center justify-center">
        <div class="w-full max-w-md">
            <div class="bg-white rounded-lg shadow-md p-8">
                {wrapper}
            </div>
        </div>
    </div>"#,
            wrapper = wrapper,
        );

        base_document(ctx.title, ctx.head, ctx.body_class, &body, ctx.scripts)
    }
}

// ── Partial types and functions ─────────────────────────────────────────

/// A navigation link item.
pub struct NavItem {
    /// Display label for the link.
    pub label: String,
    /// URL the link points to.
    pub url: String,
    /// Whether this item represents the current page.
    pub active: bool,
}

impl NavItem {
    /// Create a new navigation item (inactive by default).
    pub fn new(label: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            url: url.into(),
            active: false,
        }
    }

    /// Mark this navigation item as active (builder pattern).
    pub fn active(mut self) -> Self {
        self.active = true;
        self
    }
}

/// A sidebar section containing a title and a list of navigation items.
pub struct SidebarSection {
    /// Section heading.
    pub title: String,
    /// Navigation items in this section.
    pub items: Vec<NavItem>,
}

impl SidebarSection {
    /// Create a new sidebar section.
    pub fn new(title: impl Into<String>, items: Vec<NavItem>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }
}

/// Render a horizontal navigation bar.
///
/// Produces a `<nav>` element with Tailwind CSS classes. Active items
/// are highlighted with blue text and medium font weight.
pub fn navigation(items: &[NavItem]) -> String {
    let mut html =
        String::from("<nav class=\"bg-white border-b border-gray-200 px-4 py-3\"><div class=\"flex items-center space-x-6\">");

    for item in items {
        let class = if item.active {
            "text-blue-600 font-medium"
        } else {
            "text-gray-600 hover:text-gray-900"
        };
        html.push_str(&format!(
            "<a href=\"{}\" class=\"{}\">{}</a>",
            html_escape(&item.url),
            class,
            html_escape(&item.label),
        ));
    }

    html.push_str("</div></nav>");
    html
}

/// Render a vertical sidebar with sections.
///
/// Produces an `<aside>` element with sections, each containing a heading
/// and a list of navigation links.
pub fn sidebar(sections: &[SidebarSection]) -> String {
    let mut html =
        String::from("<aside class=\"w-64 bg-gray-50 border-r border-gray-200 p-4 min-h-screen\">");

    for section in sections {
        html.push_str("<div class=\"mb-6\">");
        html.push_str(&format!(
            "<h3 class=\"text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2\">{}</h3>",
            html_escape(&section.title),
        ));
        html.push_str("<ul class=\"space-y-1\">");
        for item in &section.items {
            let class = if item.active {
                "text-blue-600 font-medium"
            } else {
                "text-gray-600 hover:text-gray-900"
            };
            html.push_str(&format!(
                "<li><a href=\"{}\" class=\"block px-2 py-1 text-sm {}\">{}</a></li>",
                html_escape(&item.url),
                class,
                html_escape(&item.label),
            ));
        }
        html.push_str("</ul></div>");
    }

    html.push_str("</aside>");
    html
}

/// Render a simple footer.
///
/// Produces a `<footer>` element with centered text.
pub fn footer(text: &str) -> String {
    format!(
        "<footer class=\"border-t border-gray-200 px-4 py-3 text-center text-sm text-gray-500\">{}</footer>",
        html_escape(text),
    )
}

// ── Layout registry ─────────────────────────────────────────────────────

/// Registry mapping layout names to implementations.
///
/// Created with three built-in layouts: "default" (`DefaultLayout`),
/// "app" (`AppLayout`), and "auth" (`AuthLayout`). Additional layouts
/// can be registered at application startup.
pub struct LayoutRegistry {
    layouts: HashMap<String, Box<dyn Layout>>,
    default: String,
}

impl LayoutRegistry {
    /// Create a new registry with the three built-in layouts.
    pub fn new() -> Self {
        let mut layouts: HashMap<String, Box<dyn Layout>> = HashMap::new();
        layouts.insert("default".to_string(), Box::new(DefaultLayout));
        layouts.insert("app".to_string(), Box::new(AppLayout));
        layouts.insert("auth".to_string(), Box::new(AuthLayout));

        Self {
            layouts,
            default: "default".to_string(),
        }
    }

    /// Register a layout by name. Replaces any existing layout with the same name.
    pub fn register(&mut self, name: impl Into<String>, layout: impl Layout + 'static) {
        self.layouts.insert(name.into(), Box::new(layout));
    }

    /// Render using the named layout. Falls back to default if name is None
    /// or the name is not found in the registry.
    pub fn render(&self, name: Option<&str>, ctx: &LayoutContext) -> String {
        let layout_name = name.unwrap_or(&self.default);
        let layout = self
            .layouts
            .get(layout_name)
            .or_else(|| self.layouts.get(&self.default))
            .expect("default layout must exist in registry");
        layout.render(ctx)
    }

    /// Check whether a layout with the given name is registered.
    pub fn has(&self, name: &str) -> bool {
        self.layouts.contains_key(name)
    }
}

impl Default for LayoutRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Global registry ─────────────────────────────────────────────────────

static GLOBAL_REGISTRY: OnceLock<RwLock<LayoutRegistry>> = OnceLock::new();

/// Access the global layout registry.
///
/// Lazily initialized on first call with the three built-in layouts.
pub fn global_registry() -> &'static RwLock<LayoutRegistry> {
    GLOBAL_REGISTRY.get_or_init(|| RwLock::new(LayoutRegistry::new()))
}

/// Register a layout in the global registry.
///
/// Convenience wrapper around `global_registry().write()`.
pub fn register_layout(name: impl Into<String>, layout: impl Layout + 'static) {
    global_registry()
        .write()
        .expect("layout registry poisoned")
        .register(name, layout);
}

/// Render using the global registry.
///
/// Convenience wrapper around `global_registry().read()`.
pub fn render_layout(name: Option<&str>, ctx: &LayoutContext) -> String {
    global_registry()
        .read()
        .expect("layout registry poisoned")
        .render(name, ctx)
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ctx() -> LayoutContext<'static> {
        LayoutContext {
            title: "Test Page",
            content: "<p>Hello</p>",
            head: "<link rel=\"stylesheet\" href=\"/style.css\">",
            body_class: "bg-white",
            view_json: "{\"schema\":\"v1\"}",
            data_json: "{\"key\":\"value\"}",
            scripts: "",
        }
    }

    // ── base_document tests ─────────────────────────────────────────

    #[test]
    fn base_document_produces_valid_html_structure() {
        let html = base_document("Title", "<style></style>", "my-class", "<p>body</p>", "");
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("<meta charset=\"UTF-8\">"));
        assert!(html.contains("<meta name=\"viewport\""));
        assert!(html.contains("<title>Title</title>"));
        assert!(html.contains("<style></style>"));
        assert!(html.contains("<body class=\"my-class\">"));
        assert!(html.contains("<p>body</p>"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn base_document_escapes_title() {
        let html = base_document("Tom & Jerry <script>", "", "", "", "");
        assert!(html.contains("<title>Tom &amp; Jerry &lt;script&gt;</title>"));
    }

    #[test]
    fn base_document_escapes_body_class() {
        let html = base_document("T", "", "a\"b", "", "");
        assert!(html.contains("class=\"a&quot;b\""));
    }

    // ── DefaultLayout tests ─────────────────────────────────────────

    #[test]
    fn default_layout_renders_all_context_fields() {
        let ctx = test_ctx();
        let html = DefaultLayout.render(&ctx);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test Page</title>"));
        assert!(html.contains("href=\"/style.css\""));
        assert!(html.contains("class=\"bg-white\""));
        assert!(html.contains("id=\"ferro-json-ui\""));
        assert!(html.contains("data-view=\""));
        assert!(html.contains("data-props=\""));
        assert!(html.contains("<p>Hello</p>"));
    }

    #[test]
    fn default_layout_contains_ferro_wrapper() {
        let ctx = test_ctx();
        let html = DefaultLayout.render(&ctx);
        assert!(html.contains("<div id=\"ferro-json-ui\""));
    }

    // ── AppLayout tests ─────────────────────────────────────────────

    #[test]
    fn app_layout_includes_nav_and_sidebar() {
        let ctx = test_ctx();
        let html = AppLayout.render(&ctx);

        assert!(html.contains("<nav"));
        assert!(html.contains("<aside"));
        assert!(html.contains("<main class=\"flex-1 p-6\">"));
        assert!(html.contains("<div id=\"ferro-json-ui\""));
        assert!(html.contains("<p>Hello</p>"));
    }

    #[test]
    fn app_layout_has_flex_structure() {
        let ctx = test_ctx();
        let html = AppLayout.render(&ctx);
        assert!(html.contains("class=\"flex\""));
    }

    // ── AuthLayout tests ────────────────────────────────────────────

    #[test]
    fn auth_layout_centers_content() {
        let ctx = test_ctx();
        let html = AuthLayout.render(&ctx);

        assert!(html.contains("flex items-center justify-center"));
        assert!(html.contains("max-w-md"));
        assert!(html.contains("rounded-lg shadow-md"));
        assert!(html.contains("<div id=\"ferro-json-ui\""));
    }

    #[test]
    fn auth_layout_has_no_nav_or_sidebar() {
        let ctx = test_ctx();
        let html = AuthLayout.render(&ctx);
        assert!(!html.contains("<nav"));
        assert!(!html.contains("<aside"));
    }

    // ── LayoutRegistry tests ────────────────────────────────────────

    #[test]
    fn registry_returns_default_for_none_name() {
        let registry = LayoutRegistry::new();
        let ctx = test_ctx();
        let html = registry.render(None, &ctx);
        // DefaultLayout produces the simple wrapper (no nav/sidebar)
        assert!(html.contains("<div id=\"ferro-json-ui\""));
        assert!(!html.contains("<nav"));
    }

    #[test]
    fn registry_returns_default_for_unknown_name() {
        let registry = LayoutRegistry::new();
        let ctx = test_ctx();
        let html = registry.render(Some("nonexistent"), &ctx);
        // Falls back to default
        assert!(html.contains("<div id=\"ferro-json-ui\""));
        assert!(!html.contains("<nav"));
    }

    #[test]
    fn registry_renders_named_layout() {
        let registry = LayoutRegistry::new();
        let ctx = test_ctx();
        let html = registry.render(Some("app"), &ctx);
        assert!(html.contains("<nav"));
        assert!(html.contains("<aside"));
    }

    #[test]
    fn registry_renders_auth_layout() {
        let registry = LayoutRegistry::new();
        let ctx = test_ctx();
        let html = registry.render(Some("auth"), &ctx);
        assert!(html.contains("flex items-center justify-center"));
    }

    #[test]
    fn registry_has_returns_true_for_registered() {
        let registry = LayoutRegistry::new();
        assert!(registry.has("default"));
        assert!(registry.has("app"));
        assert!(registry.has("auth"));
    }

    #[test]
    fn registry_has_returns_false_for_unknown() {
        let registry = LayoutRegistry::new();
        assert!(!registry.has("nonexistent"));
    }

    #[test]
    fn registry_register_adds_custom_layout() {
        let mut registry = LayoutRegistry::new();
        struct Custom;
        impl Layout for Custom {
            fn render(&self, _ctx: &LayoutContext) -> String {
                "CUSTOM".to_string()
            }
        }
        registry.register("custom", Custom);
        assert!(registry.has("custom"));

        let ctx = test_ctx();
        let html = registry.render(Some("custom"), &ctx);
        assert_eq!(html, "CUSTOM");
    }

    #[test]
    fn registry_register_replaces_existing() {
        let mut registry = LayoutRegistry::new();
        struct Replacement;
        impl Layout for Replacement {
            fn render(&self, _ctx: &LayoutContext) -> String {
                "REPLACED".to_string()
            }
        }
        registry.register("default", Replacement);
        let ctx = test_ctx();
        let html = registry.render(None, &ctx);
        assert_eq!(html, "REPLACED");
    }

    // ── Global registry tests ───────────────────────────────────────

    #[test]
    fn global_registry_returns_valid_registry() {
        let reg = global_registry();
        let guard = reg.read().unwrap();
        assert!(guard.has("default"));
        assert!(guard.has("app"));
        assert!(guard.has("auth"));
    }

    #[test]
    fn render_layout_global_function_works() {
        let ctx = test_ctx();
        let html = render_layout(None, &ctx);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<div id=\"ferro-json-ui\""));
    }

    // ── Partial tests ───────────────────────────────────────────────

    #[test]
    fn navigation_renders_empty_gracefully() {
        let html = navigation(&[]);
        assert!(html.contains("<nav"));
        assert!(html.contains("</nav>"));
    }

    #[test]
    fn navigation_renders_items_with_correct_classes() {
        let items = vec![NavItem::new("Home", "/"), NavItem::new("Users", "/users")];
        let html = navigation(&items);
        assert!(html.contains("href=\"/\""));
        assert!(html.contains(">Home</a>"));
        assert!(html.contains("href=\"/users\""));
        assert!(html.contains(">Users</a>"));
        // Both should be inactive
        assert!(html.contains("text-gray-600 hover:text-gray-900"));
    }

    #[test]
    fn navigation_marks_active_item() {
        let items = vec![
            NavItem::new("Home", "/").active(),
            NavItem::new("Users", "/users"),
        ];
        let html = navigation(&items);
        assert!(html.contains("text-blue-600 font-medium"));
    }

    #[test]
    fn sidebar_renders_sections_with_headers() {
        let sections = vec![SidebarSection::new(
            "Main Menu",
            vec![
                NavItem::new("Dashboard", "/"),
                NavItem::new("Settings", "/settings"),
            ],
        )];
        let html = sidebar(&sections);
        assert!(html.contains("<aside"));
        assert!(html.contains("Main Menu"));
        assert!(html.contains("Dashboard"));
        assert!(html.contains("Settings"));
        assert!(html.contains("</aside>"));
    }

    #[test]
    fn sidebar_renders_empty_gracefully() {
        let html = sidebar(&[]);
        assert!(html.contains("<aside"));
        assert!(html.contains("</aside>"));
    }

    #[test]
    fn footer_renders_text() {
        let html = footer("Copyright 2026");
        assert!(html.contains("<footer"));
        assert!(html.contains("Copyright 2026"));
        assert!(html.contains("</footer>"));
    }

    #[test]
    fn partials_escape_user_strings() {
        let items = vec![NavItem::new("Tom & Jerry", "/a&b")];
        let html = navigation(&items);
        assert!(html.contains("Tom &amp; Jerry"));
        assert!(html.contains("href=\"/a&amp;b\""));

        let sections = vec![SidebarSection::new(
            "A<B",
            vec![NavItem::new("<script>", "/x\"y")],
        )];
        let html = sidebar(&sections);
        assert!(html.contains("A&lt;B"));
        assert!(html.contains("&lt;script&gt;"));

        let html = footer("<script>alert('xss')</script>");
        assert!(html.contains("&lt;script&gt;"));
    }

    // ── ferro_wrapper tests ─────────────────────────────────────────

    #[test]
    fn ferro_wrapper_includes_data_attributes() {
        let ctx = test_ctx();
        let html = ferro_wrapper(&ctx);
        assert!(html.contains("id=\"ferro-json-ui\""));
        assert!(html.contains("data-view=\""));
        assert!(html.contains("data-props=\""));
        assert!(html.contains("<p>Hello</p>"));
    }
}
