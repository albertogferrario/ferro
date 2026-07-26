//! Layout system for JSON-UI page rendering.
//!
//! Provides a trait-based layout system where named layouts wrap rendered
//! component HTML in full page shells. Three built-in layouts are provided:
//! `DefaultLayout` (minimal), `AppLayout` (dashboard with nav + sidebar),
//! and `AuthLayout` (centered, no card chrome). `DashboardLayout` is an optional
//! layout that users register themselves with per-app config.
//!
//! A global `LayoutRegistry` maps layout names to implementations. Specs
//! specify a layout via `Spec.layout`, and the render pipeline looks it up
//! in the registry.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use crate::component::{HeaderProps, SidebarGroup, SidebarNavItem, SidebarProps};
use crate::render::classes::INTERACTIVE_BASE;
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
    /// Rendered component HTML fragment (output of `render_spec_to_html`).
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

/// Produce the common `<!DOCTYPE html>` shell with optional extra body attributes.
///
/// Extends `base_document` with a `body_data` parameter for additional
/// `data-*` attributes on the `<body>` element (e.g., `data-sse-url`).
fn base_document_ext(
    title: &str,
    head: &str,
    body_class: &str,
    body_data: &str,
    body_content: &str,
    scripts: &str,
) -> String {
    let body_data_attr = if body_data.is_empty() {
        String::new()
    } else {
        format!(" {body_data}")
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    {head}
</head>
<body class="{body_class}"{body_data_attr}>
    {body_content}
    {scripts}
</body>
</html>"#,
        title = html_escape(title),
        head = head,
        body_class = html_escape(body_class),
        body_data_attr = body_data_attr,
        body_content = body_content,
        scripts = scripts,
    )
}

// ── DashboardLayout helpers ─────────────────────────────────────────────

/// Render a sidebar nav item for the layout shell.
fn layout_sidebar_nav_item(item: &SidebarNavItem) -> String {
    let disabled = item.disabled.unwrap_or(false);
    // Appearance is handled by the fjui-sidebar__nav-item skin rule (SKIN-01).
    // Layout utilities (flex items-center gap-*) stay inline (D-02).
    let (tag, classes) = if disabled {
        (
            "span",
            "fjui-sidebar__nav-item flex items-center gap-2 opacity-50 pointer-events-none select-none".to_string(),
        )
    } else if item.active {
        (
            "a",
            "fjui-sidebar__nav-item fjui-sidebar__nav-item--active flex items-center gap-2".to_string(),
        )
    } else {
        (
            "a",
            "fjui-sidebar__nav-item flex items-center gap-2".to_string(),
        )
    };
    let mut html = if disabled {
        format!("<{tag} aria-disabled=\"true\" class=\"{classes}\">")
    } else {
        format!(
            "<{tag} href=\"{}\" class=\"{classes}\">",
            html_escape(&item.href),
        )
    };
    if let Some(ref icon) = item.icon {
        html.push_str(&format!(
            "<span class=\"inline-flex items-center justify-center w-5 h-5 shrink-0\">{icon}</span>" // raw SVG
        ));
    }
    html.push_str(&format!("{}</{tag}>", html_escape(&item.label)));
    html
}

/// Render a sidebar group for the layout shell.
fn layout_sidebar_group(group: &SidebarGroup) -> String {
    let mut html = String::from("<div data-sidebar-group");
    if group.collapsed {
        html.push_str(" data-collapsed");
    }
    html.push('>');
    // Appearance (font-size, color, text-transform) handled by fjui-sidebar__group-label skin rule.
    html.push_str(&format!(
        "<p class=\"fjui-sidebar__group-label\">{}</p>",
        html_escape(&group.label)
    ));
    html.push_str("<nav class=\"space-y-1\">");
    for item in &group.items {
        html.push_str(&layout_sidebar_nav_item(item));
    }
    html.push_str("</nav></div>");
    html
}

/// Render the sidebar shell from SidebarProps for DashboardLayout.
fn layout_sidebar_html(props: &SidebarProps) -> String {
    // fjui-sidebar: appearance (bg, border-right, width, position:fixed) handled by skin rule (SKIN-01).
    // Layout utilities (inset-y-0 left-0 z-40 flex flex-col hidden md:flex) stay inline (D-02).
    let mut html = String::from(
        "<aside data-sidebar class=\"fjui-sidebar inset-y-0 left-0 z-40 flex flex-col hidden md:flex\">",
    );
    if !props.fixed_top.is_empty() {
        html.push_str("<nav class=\"px-4 pt-4 pb-1 space-y-1\">");
        for item in &props.fixed_top {
            html.push_str(&layout_sidebar_nav_item(item));
        }
        html.push_str("</nav>");
    }
    if !props.groups.is_empty() {
        html.push_str("<div class=\"flex-1 overflow-y-auto px-4 pb-4 pt-0 space-y-2\">");
        for group in &props.groups {
            html.push_str(&layout_sidebar_group(group));
        }
        html.push_str("</div>");
    }
    if !props.fixed_bottom.is_empty() {
        // pb-safe: env(safe-area-inset-bottom) prevents bottom items from being clipped
        // by Chrome mobile's dynamic URL bar when the sidebar is open full-height (dvh fix).
        html.push_str("<nav class=\"fjui-sidebar__bottom p-4 space-y-1 border-t border-border\">");
        for item in &props.fixed_bottom {
            html.push_str(&layout_sidebar_nav_item(item));
        }
        html.push_str("</nav>");
    }
    html.push_str("</aside>");
    // Backdrop for mobile sidebar overlay — sibling of aside so it covers the viewport behind it.
    html.push_str(
        "<div data-sidebar-backdrop class=\"fixed inset-0 z-30 bg-black/50 hidden md:hidden\"></div>",
    );
    html
}

/// Render the header shell from HeaderProps for DashboardLayout.
fn layout_header_html(props: &HeaderProps) -> String {
    // fjui-header: appearance (bg, border-bottom, height, sticky positioning, padding) handled by skin rule (SKIN-01).
    // Layout utilities (z-30 flex items-center) stay inline (D-02). `relative` is intentionally
    // absent: the skin sets `position: sticky` on .fjui-header, and `relative` (a Tailwind
    // utility outside @layer components) would override it, causing the header to scroll away
    // (Finding A — sticky header fix). Sticky positioning creates its own stacking context,
    // so z-30 remains effective without an explicit `relative`. The header sits inside the
    // sidebar-offset content column, so it must NOT re-pad for the sidebar itself
    // (a legacy md:pl-72 here doubled the offset and pushed the workspace label off-left).
    // md:pl-64 offsets the header *content* to clear the sidebar, while the
    // border-bottom on the fjui-header element spans the full viewport width
    // (Finding 3 — full-width separator). The header element itself is
    // full-width; only its inner padding mirrors the content column offset.
    let mut html = String::from("<header class=\"fjui-header z-30 flex items-center md:pl-64\">");
    // Mobile hamburger button — visible only on small screens. -ml-2 cancels the
    // button's own padding so the icon glyph (not the touch target) aligns with
    // the 12px content gutter below.
    html.push_str(&format!(
        "<button data-sidebar-toggle class=\"md:hidden p-2 -ml-2 rounded-md text-text-muted \
         hover:text-text hover:bg-surface {INTERACTIVE_BASE}\" aria-label=\"Toggle sidebar\">\
         <svg class=\"h-6 w-6\" fill=\"none\" stroke=\"currentColor\" viewBox=\"0 0 24 24\">\
         <path stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" \
         d=\"M4 6h16M4 12h16M4 18h16\"/></svg></button>"
    ));
    // Business name — left-aligned workspace label (CHR-01).
    html.push_str(&format!(
        "<span class=\"fjui-header__workspace\">{}</span>",
        html_escape(&props.business_name)
    ));
    html.push_str("<div class=\"ml-auto flex items-center gap-4\">");
    // Notification bell with dropdown toggle.
    html.push_str("<div class=\"relative\">");
    if let Some(count) = props.notification_count {
        if count > 0 {
            html.push_str(&format!(
                "<button data-notification-toggle class=\"relative p-2 rounded-md text-text-muted hover:text-text {INTERACTIVE_BASE}\">\
                 <svg class=\"h-5 w-5\" fill=\"none\" stroke=\"currentColor\" viewBox=\"0 0 24 24\">\
                 <path stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" \
                 d=\"M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9\"/></svg>\
                 <span class=\"absolute top-1 right-1 inline-flex items-center justify-center h-4 w-4 \
                 text-xs font-bold text-primary-foreground bg-destructive rounded-full\">{count}</span></button>",
            ));
        } else {
            html.push_str(&format!(
                "<button data-notification-toggle class=\"p-2 rounded-md text-text-muted hover:text-text {INTERACTIVE_BASE}\">\
                 <svg class=\"h-5 w-5\" fill=\"none\" stroke=\"currentColor\" viewBox=\"0 0 24 24\">\
                 <path stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" \
                 d=\"M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9\"/></svg></button>"
            ));
        }
    }
    html.push_str(
        "<div data-notification-dropdown class=\"hidden absolute right-0 top-full mt-1 w-80 \
         bg-card rounded-lg shadow-lg border border-border z-50\"></div></div>",
    );
    // Search affordance button (CHR-01 / UX-02): magnifier + ⌘K kbd chip + tooltip.
    // Dispatches fjui:open-command-palette — handler wired in Phase 249 (D-06).
    html.push_str(&format!(
        "<button type=\"button\" class=\"fjui-header__search-btn inline-flex items-center gap-2 {INTERACTIVE_BASE}\" \
         data-tooltip=\"Cerca\" aria-label=\"Cerca (⌘K)\" \
         onclick=\"document.dispatchEvent(new CustomEvent('fjui:open-command-palette'))\">\
         <svg class=\"h-4 w-4\" fill=\"none\" stroke=\"currentColor\" viewBox=\"0 0 24 24\" stroke-width=\"2\" \
           stroke-linecap=\"round\" stroke-linejoin=\"round\">\
           <circle cx=\"11\" cy=\"11\" r=\"8\"/><path d=\"m21 21-4.35-4.35\"/>\
         </svg>\
         <kbd class=\"fjui-kbd hidden md:inline\">\u{2318}K</kbd></button>"
    ));
    // Avatar initials button opens the fjui-avatar-menu popover (CHR-01).
    // Falls back to business_name initials when user_name is absent.
    let name_source = props
        .user_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(&props.business_name);
    let initials: String = name_source
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    html.push_str(&format!(
        "<button class=\"fjui-avatar fjui-avatar--md inline-flex items-center justify-center \
         cursor-pointer {INTERACTIVE_BASE}\" \
         popovertarget=\"fjui-avatar-menu\" aria-label=\"Menu utente\" aria-haspopup=\"true\">{}</button>",
        html_escape(&initials)
    ));
    // Avatar menu popover panel: Profilo / Tema toggle (only when `theme_url`
    // is configured) / separator / Esci POST form.
    // Esci is a plain <form method="post"> with no CSRF token — matching how
    // consumer apps handle their other POST forms. The operative cross-site
    // mitigation is the app's SameSite=Lax session cookie (cross-site POSTs
    // arrive unauthenticated). Note ferro's CsrfMiddleware validates only the
    // X-CSRF-TOKEN/X-XSRF-TOKEN headers, so it would reject token-less form
    // posts; apps enabling it must exempt or tokenize this form themselves.
    // Tema onclick POSTs to `theme_url`; the dark class toggles only on a 2xx
    // response so visual state stays in sync with the persisted preference.
    let logout_action = props.logout_url.as_deref().unwrap_or("/logout");
    let theme_item = match props.theme_url.as_deref() {
        Some(url) => {
            // JS-string-escape then HTML-escape: the URL sits inside a
            // single-quoted JS literal within a double-quoted onclick attribute
            // (entities decode before JS parses, so `'` alone would break out).
            let js_url = url.replace('\\', "\\\\").replace('\'', "\\'");
            format!(
                "<button type=\"button\" class=\"fjui-avatar-menu__item\" \
                 onclick=\"fetch('{}',{{method:'POST',body:'theme='+(document.documentElement.classList.contains('dark')?'light':'dark'),headers:{{'Content-Type':'application/x-www-form-urlencoded'}}}}).then(function(r){{if(r.ok)document.documentElement.classList.toggle('dark')}})\">Tema</button>",
                html_escape(&js_url)
            )
        }
        None => String::new(),
    };
    let profile_item = match props.profile_url.as_deref() {
        Some(url) => format!(
            "<a href=\"{}\" class=\"fjui-avatar-menu__item\">Profilo</a>",
            html_escape(url)
        ),
        None => String::new(),
    };
    html.push_str(&format!(
        "<div popover id=\"fjui-avatar-menu\" data-popover-menu class=\"fjui-avatar-menu\">\
           {profile_item}\
           {theme_item}\
           <div class=\"fjui-avatar-menu__separator\"></div>\
           <form method=\"post\" action=\"{}\">\
             <button type=\"submit\" class=\"fjui-avatar-menu__item fjui-avatar-menu__item--destructive\">Esci</button>\
           </form>\
         </div>",
        html_escape(logout_action)
    ));
    html.push_str("</div></header>");
    html
}

/// Combine plugin scripts with the built-in JS runtime.
fn with_runtime(ctx_scripts: &str) -> String {
    let runtime = format!(
        "<script>\n{}\n</script>",
        crate::runtime::FERRO_RUNTIME_JS.as_str()
    );
    if ctx_scripts.is_empty() {
        runtime
    } else {
        format!("{ctx_scripts}\n{runtime}")
    }
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
        let scripts = with_runtime(ctx.scripts);
        base_document(ctx.title, ctx.head, ctx.body_class, &wrapper, &scripts)
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
        <main class="flex-1 px-3 py-4 md:p-6">
            <div class="mx-auto w-full max-w-7xl">
                {wrapper}
            </div>
        </main>
    </div>"#,
        );

        let scripts = with_runtime(ctx.scripts);
        base_document(ctx.title, ctx.head, ctx.body_class, &body, &scripts)
    }
}

// ── AuthLayout ──────────────────────────────────────────────────────────

/// Centered layout for authentication pages (login, register).
///
/// Centers the content vertically and horizontally within a max-width
/// container. No navigation or sidebar. No card chrome — the spec's
/// root component is responsible for its own card styling (D-05).
pub struct AuthLayout;

impl Layout for AuthLayout {
    fn render(&self, ctx: &LayoutContext) -> String {
        let wrapper = ferro_wrapper(ctx);

        let body = format!(
            r#"<div class="min-h-screen flex items-center justify-center">
        <div class="w-full max-w-md">
            {wrapper}
        </div>
    </div>"#,
        );

        let scripts = with_runtime(ctx.scripts);
        base_document(ctx.title, ctx.head, ctx.body_class, &body, &scripts)
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
        String::from("<nav class=\"bg-background border-b border-border px-4 py-3\"><div class=\"flex items-center space-x-6\">");

    for item in items {
        let class = if item.active {
            "text-primary font-medium"
        } else {
            "text-text-muted hover:text-text"
        };
        html.push_str(&format!(
            "<a href=\"{}\" class=\"{} {INTERACTIVE_BASE}\">{}</a>",
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
        String::from("<aside class=\"w-64 bg-surface border-r border-border p-4 min-h-screen\">");

    for section in sections {
        html.push_str("<div class=\"mb-6\">");
        html.push_str(&format!(
            "<h3 class=\"text-xs font-semibold text-text-muted uppercase tracking-wider mb-2\">{}</h3>",
            html_escape(&section.title),
        ));
        html.push_str("<ul class=\"space-y-1\">");
        for item in &section.items {
            let class = if item.active {
                "text-primary font-medium"
            } else {
                "text-text-muted hover:text-text"
            };
            html.push_str(&format!(
                "<li><a href=\"{}\" class=\"block px-2 py-1 text-sm rounded-md {} {INTERACTIVE_BASE}\">{}</a></li>",
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
        "<footer class=\"border-t border-border px-4 py-3 text-center text-sm text-text-muted\">{}</footer>",
        html_escape(text),
    )
}

// ── DashboardLayout ─────────────────────────────────────────────────────

/// Configuration for `DashboardLayout`.
///
/// Provides the per-application sidebar navigation and header data needed
/// to render the persistent dashboard shell. Users construct this at app
/// startup and register it with the layout registry.
///
/// # Example
///
/// ```rust
/// use ferro_json_ui::{DashboardLayout, DashboardLayoutConfig, HeaderProps, SidebarProps, register_layout};
///
/// register_layout("dashboard", DashboardLayout::new(DashboardLayoutConfig {
///     sidebar: SidebarProps { fixed_top: vec![], groups: vec![], fixed_bottom: vec![] },
///     header: HeaderProps {
///         business_name: "My App".to_string(),
///         notification_count: None,
///         user_name: Some("Alice".to_string()),
///         user_avatar: None,
///         logout_url: Some("/logout".to_string()),
///         theme_url: Some("/theme".to_string()),
///         profile_url: Some("/settings".to_string()),
///     },
///     sse_url: None,
/// }));
/// ```
pub struct DashboardLayoutConfig {
    /// Sidebar navigation data for the persistent sidebar shell.
    pub sidebar: SidebarProps,
    /// Header data for the persistent header shell.
    pub header: HeaderProps,
    /// Optional SSE endpoint URL. When set, the JS runtime opens an
    /// `EventSource` connection to this URL and dispatches live-value
    /// and toast updates from incoming messages.
    pub sse_url: Option<String>,
}

/// Dashboard layout with persistent sidebar, header, and main content area.
///
/// Renders a full-page shell with a fixed sidebar on the left (desktop)
/// and a sticky header at the top. The rendered view content appears in
/// the `<main>` area. The built-in JS runtime (`FERRO_RUNTIME_JS`) is
/// injected once as a `<script>` tag, enabling SSE, live-value updates,
/// and toast notifications.
///
/// Mobile: sidebar is hidden by default and toggled via the hamburger button
/// in the header (using responsive Tailwind classes).
///
/// This layout is NOT auto-registered. Users must register it at startup:
///
/// ```rust
/// use ferro_json_ui::{DashboardLayout, DashboardLayoutConfig, HeaderProps, SidebarProps, register_layout};
///
/// register_layout("dashboard", DashboardLayout::new(DashboardLayoutConfig {
///     sidebar: SidebarProps { fixed_top: vec![], groups: vec![], fixed_bottom: vec![] },
///     header: HeaderProps {
///         business_name: "My App".to_string(),
///         notification_count: None,
///         user_name: None,
///         user_avatar: None,
///         logout_url: None,
///         theme_url: None,
///         profile_url: None,
///     },
///     sse_url: None,
/// }));
/// ```
pub struct DashboardLayout {
    /// Layout configuration (sidebar, header, SSE URL).
    pub config: DashboardLayoutConfig,
}

impl DashboardLayout {
    /// Create a new `DashboardLayout` from a `DashboardLayoutConfig`.
    pub fn new(config: DashboardLayoutConfig) -> Self {
        Self { config }
    }
}

impl Layout for DashboardLayout {
    fn render(&self, ctx: &LayoutContext) -> String {
        let sidebar_html = layout_sidebar_html(&self.config.sidebar);
        let header_html = layout_header_html(&self.config.header);
        let wrapper = ferro_wrapper(ctx);

        let body_data = if let Some(ref url) = self.config.sse_url {
            format!("data-sse-url=\"{}\"", html_escape(url))
        } else {
            String::new()
        };

        let runtime_script = format!(
            "<script>\n{}\n</script>",
            crate::runtime::FERRO_RUNTIME_JS.as_str()
        );
        let scripts = if ctx.scripts.is_empty() {
            runtime_script
        } else {
            format!("{}\n{}", ctx.scripts, runtime_script)
        };

        // Header is a sibling of the content column (not inside it) so its
        // border-bottom spans the full viewport width edge-to-edge (Finding 3).
        // The header's own md:pl-64 class mirrors the sidebar offset for content
        // alignment. Main keeps md:pl-64 so its content also clears the sidebar.
        let body_content = format!(
            r#"{sidebar_html}
    <div class="flex flex-col">
        {header_html}
        <main class="flex-1 px-3 py-4 md:p-6 md:pl-64">
            <div class="mx-auto w-full max-w-7xl">
                {wrapper}
            </div>
        </main>
        <div data-toast-container class="fixed top-4 right-4 z-50 flex flex-col gap-2"></div>
    </div>"#,
        );

        let body_class = if ctx.body_class.is_empty() {
            "bg-surface"
        } else {
            ctx.body_class
        };

        base_document_ext(
            ctx.title,
            ctx.head,
            body_class,
            &body_data,
            &body_content,
            &scripts,
        )
    }
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
            body_class: "bg-background",
            view_json: "{\"schema\":\"ferro-json-ui/v2\"}",
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
        assert!(html.contains("class=\"bg-background\""));
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
        assert!(html.contains("<main class=\"flex-1 px-3 py-4 md:p-6\">"));
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

        // Structural centering and max-width are preserved.
        assert!(
            html.contains("min-h-screen flex items-center justify-center"),
            "centering wrapper must remain"
        );
        assert!(
            html.contains("w-full max-w-md"),
            "max-width wrapper must remain"
        );
        assert!(html.contains("<div id=\"ferro-json-ui\""));
        // D-05: layout no longer applies card chrome; the spec's root declares its own Card.
        assert!(
            !html.contains("bg-card rounded-lg shadow-md p-8"),
            "card chrome must be removed from AuthLayout; spec root must declare its own Card"
        );
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
        assert!(html.contains("text-text-muted hover:text-text"));
    }

    #[test]
    fn navigation_marks_active_item() {
        let items = vec![
            NavItem::new("Home", "/").active(),
            NavItem::new("Users", "/users"),
        ];
        let html = navigation(&items);
        assert!(html.contains("text-primary font-medium"));
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

    // ── DashboardLayout tests ───────────────────────────────────────

    fn dashboard_layout() -> DashboardLayout {
        use crate::component::{HeaderProps, SidebarProps};
        DashboardLayout::new(DashboardLayoutConfig {
            sidebar: SidebarProps {
                fixed_top: vec![],
                groups: vec![],
                fixed_bottom: vec![],
            },
            header: HeaderProps {
                business_name: "Acme".to_string(),
                notification_count: None,
                user_name: Some("Alice".to_string()),
                user_avatar: None,
                logout_url: Some("/logout".to_string()),
                theme_url: None,
                profile_url: None,
            },
            sse_url: None,
        })
    }

    #[test]
    fn dashboard_layout_renders_full_html_structure() {
        let ctx = test_ctx();
        let html = dashboard_layout().render(&ctx);

        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test Page</title>"));
        assert!(html.contains("<div id=\"ferro-json-ui\""));
        assert!(html.contains("<p>Hello</p>"));
    }

    #[test]
    fn dashboard_layout_has_persistent_sidebar() {
        let ctx = test_ctx();
        let html = dashboard_layout().render(&ctx);
        assert!(html.contains("<aside data-sidebar"));
    }

    #[test]
    fn dashboard_layout_has_persistent_header() {
        let ctx = test_ctx();
        let html = dashboard_layout().render(&ctx);
        assert!(html.contains("<header"));
        assert!(html.contains("Acme"));
    }

    #[test]
    fn dashboard_layout_has_main_content_area() {
        let ctx = test_ctx();
        let html = dashboard_layout().render(&ctx);
        // md:pl-64 clears the fixed sidebar; added alongside md:p-6 (Finding 3).
        assert!(html.contains("<main class=\"flex-1 px-3 py-4 md:p-6 md:pl-64\">"));
    }

    #[test]
    fn dashboard_layout_has_toast_container() {
        let ctx = test_ctx();
        let html = dashboard_layout().render(&ctx);
        assert!(html.contains("data-toast-container"));
    }

    #[test]
    fn dashboard_layout_injects_runtime_js() {
        let ctx = test_ctx();
        let html = dashboard_layout().render(&ctx);
        // JS runtime is injected as a <script> tag containing the IIFE
        assert!(html.contains("<script>"));
        assert!(html.contains("FERRO_RUNTIME_JS") || html.contains("(function()"));
    }

    #[test]
    fn dashboard_layout_has_mobile_hamburger_toggle() {
        let ctx = test_ctx();
        let html = dashboard_layout().render(&ctx);
        assert!(html.contains("data-sidebar-toggle"));
    }

    #[test]
    fn dashboard_layout_no_sse_url_attribute_on_body_when_not_configured() {
        let ctx = test_ctx();
        let html = dashboard_layout().render(&ctx);
        // data-sse-url appears in the JS runtime source as a string literal,
        // but should NOT appear as a body element attribute when sse_url is None.
        // Check that the body tag does not contain the attribute.
        let body_start = html.find("<body").unwrap_or(0);
        let body_tag_end = html[body_start..].find('>').unwrap_or(0) + body_start;
        let body_tag = &html[body_start..=body_tag_end];
        assert!(!body_tag.contains("data-sse-url="));
    }

    #[test]
    fn dashboard_layout_adds_sse_url_to_body_when_configured() {
        use crate::component::{HeaderProps, SidebarProps};
        let layout = DashboardLayout::new(DashboardLayoutConfig {
            sidebar: SidebarProps {
                fixed_top: vec![],
                groups: vec![],
                fixed_bottom: vec![],
            },
            header: HeaderProps {
                business_name: "App".to_string(),
                notification_count: None,
                user_name: None,
                user_avatar: None,
                logout_url: None,
                theme_url: None,
                profile_url: None,
            },
            sse_url: Some("/events".to_string()),
        });
        let ctx = test_ctx();
        let html = layout.render(&ctx);
        assert!(html.contains("data-sse-url=\"/events\""));
    }

    #[test]
    fn dashboard_layout_escapes_sse_url_xss() {
        use crate::component::{HeaderProps, SidebarProps};
        let layout = DashboardLayout::new(DashboardLayoutConfig {
            sidebar: SidebarProps {
                fixed_top: vec![],
                groups: vec![],
                fixed_bottom: vec![],
            },
            header: HeaderProps {
                business_name: "App".to_string(),
                notification_count: None,
                user_name: None,
                user_avatar: None,
                logout_url: None,
                theme_url: None,
                profile_url: None,
            },
            sse_url: Some("/events?a=1&b=2".to_string()),
        });
        let ctx = test_ctx();
        let html = layout.render(&ctx);
        assert!(html.contains("data-sse-url=\"/events?a=1&amp;b=2\""));
    }

    #[test]
    fn dashboard_layout_notification_toggle_present_with_count() {
        use crate::component::{HeaderProps, SidebarProps};
        let layout = DashboardLayout::new(DashboardLayoutConfig {
            sidebar: SidebarProps {
                fixed_top: vec![],
                groups: vec![],
                fixed_bottom: vec![],
            },
            header: HeaderProps {
                business_name: "App".to_string(),
                notification_count: Some(5),
                user_name: None,
                user_avatar: None,
                logout_url: None,
                theme_url: None,
                profile_url: None,
            },
            sse_url: None,
        });
        let ctx = test_ctx();
        let html = layout.render(&ctx);
        assert!(html.contains("data-notification-toggle"));
    }

    #[test]
    fn dashboard_layout_has_sidebar_backdrop() {
        let ctx = test_ctx();
        let html = dashboard_layout().render(&ctx);
        assert!(html.contains("data-sidebar-backdrop"));
        assert!(html.contains("bg-black/50"));
        assert!(html.contains("md:hidden"));
    }

    #[test]
    fn dashboard_layout_sidebar_mobile_classes() {
        let ctx = test_ctx();
        let html = dashboard_layout().render(&ctx);
        // Sidebar uses responsive classes: hidden on mobile, flex on md+
        assert!(html.contains("hidden md:flex"));
    }

    #[test]
    fn dashboard_layout_uses_default_body_class() {
        let ctx = test_ctx();
        let html = dashboard_layout().render(&ctx);
        // body_class from test_ctx is "bg-background" — should be preserved
        assert!(html.contains("class=\"bg-background\""));
    }

    #[test]
    fn sidebar_nav_item_renders_icon_as_raw_svg() {
        let item = SidebarNavItem {
            label: "Dashboard".to_string(),
            href: "/dashboard".to_string(),
            icon: Some("<svg class=\"h-5 w-5\"><path d=\"M3 12l2-2\"/></svg>".to_string()),
            active: false,
            disabled: None,
        };
        let html = layout_sidebar_nav_item(&item);
        assert!(
            html.contains("<svg"),
            "icon SVG should be rendered raw, not escaped"
        );
        assert!(
            !html.contains("&lt;svg"),
            "icon SVG should NOT be html-escaped"
        );
        assert!(html.contains("Dashboard"), "label should still appear");
    }

    #[test]
    fn sidebar_group_label_uses_fjui_class() {
        // After Plan 09 migration: appearance (11px/500/uppercase) is in the
        // fjui-sidebar__group-label skin rule; Rust only emits the semantic class.
        let group = SidebarGroup {
            label: "Cassa".to_string(),
            collapsed: false,
            items: vec![],
        };
        let html = layout_sidebar_group(&group);
        assert!(html.contains("Cassa"));
        assert!(
            html.contains("fjui-sidebar__group-label"),
            "sidebar group label must emit fjui-sidebar__group-label"
        );
        // No appearance utilities in Rust output (SKIN-01).
        assert!(
            !html.contains("font-semibold"),
            "font-semibold must be removed from Rust emission; handled by skin rule"
        );
    }

    // ── INT-07 (layout): DashboardLayout sidebar nav item focus ring ──────
    // After Plan 09 migration: focus ring and transitions are in the
    // fjui-sidebar__nav-item skin rule (:focus-visible, transition-property).
    // The Rust emission carries only the semantic class — no appearance utilities.

    #[test]
    fn layout_sidebar_nav_focus_ring() {
        let item = SidebarNavItem {
            label: "Dashboard".to_string(),
            href: "/dashboard".to_string(),
            icon: None,
            active: false,
            disabled: None,
        };
        let html = layout_sidebar_nav_item(&item);
        assert!(
            html.contains("fjui-sidebar__nav-item"),
            "layout sidebar nav <a> item must emit fjui-sidebar__nav-item (INT-07 — focus ring in skin rule)"
        );
        // Focus ring and motion are handled by the fjui-sidebar__nav-item skin rule (SKIN-01).
        // They must NOT be inlined as Tailwind utilities in the Rust output.
        assert!(
            !html.contains("focus-visible:ring-ring"),
            "focus-visible:ring-ring must be removed from Rust emission; handled by skin rule"
        );
        assert!(
            !html.contains("duration-fast"),
            "duration-fast must be removed from Rust emission; handled by skin rule"
        );
    }

    // ── Plan 09: fjui-* chrome class migration tests ────────────────────────

    /// Sidebar shell emits fjui-sidebar (T-246-19: swap-target preserved).
    #[test]
    fn layout_sidebar_html_emits_fjui_sidebar_class() {
        use crate::component::SidebarProps;
        let props = SidebarProps { fixed_top: vec![], groups: vec![], fixed_bottom: vec![] };
        let html = layout_sidebar_html(&props);
        assert!(
            html.contains("fjui-sidebar"),
            "layout_sidebar_html must emit fjui-sidebar class; found: {html}"
        );
        // Structural attributes must be preserved (T-246-19).
        assert!(html.contains("data-sidebar"), "data-sidebar attribute must be preserved");
        assert!(html.contains("data-sidebar-backdrop"), "data-sidebar-backdrop must be preserved");
    }

    /// Header shell emits fjui-header.
    #[test]
    fn layout_header_html_emits_fjui_header_class() {
        use crate::component::HeaderProps;
        let props = HeaderProps {
            business_name: "Test".to_string(),
            notification_count: None,
            user_name: None,
            user_avatar: None,
            logout_url: None,
            theme_url: None,
            profile_url: None,
        };
        let html = layout_header_html(&props);
        assert!(
            html.contains("fjui-header"),
            "layout_header_html must emit fjui-header class; found: {html}"
        );
    }

    /// Sidebar nav item emits fjui-sidebar__nav-item (not old appearance utilities).
    #[test]
    fn layout_sidebar_nav_item_emits_fjui_class() {
        let item = SidebarNavItem {
            label: "Dashboard".to_string(),
            href: "/dashboard".to_string(),
            icon: None,
            active: false,
            disabled: None,
        };
        let html = layout_sidebar_nav_item(&item);
        assert!(
            html.contains("fjui-sidebar__nav-item"),
            "inactive nav item must emit fjui-sidebar__nav-item; found: {html}"
        );
    }

    /// Active sidebar nav item emits fjui-sidebar__nav-item--active modifier.
    #[test]
    fn layout_sidebar_nav_item_active_emits_fjui_active_modifier() {
        let item = SidebarNavItem {
            label: "Dashboard".to_string(),
            href: "/dashboard".to_string(),
            icon: None,
            active: true,
            disabled: None,
        };
        let html = layout_sidebar_nav_item(&item);
        assert!(
            html.contains("fjui-sidebar__nav-item--active"),
            "active nav item must emit fjui-sidebar__nav-item--active; found: {html}"
        );
    }

    /// Sidebar group label emits fjui-sidebar__group-label.
    #[test]
    fn layout_sidebar_group_label_emits_fjui_class() {
        let group = SidebarGroup {
            label: "Cassa".to_string(),
            collapsed: false,
            items: vec![],
        };
        let html = layout_sidebar_group(&group);
        assert!(
            html.contains("fjui-sidebar__group-label"),
            "sidebar group label must emit fjui-sidebar__group-label; found: {html}"
        );
    }

    /// DashboardLayout full render emits fjui-sidebar and fjui-header (structural id/attrs preserved).
    #[test]
    fn dashboard_layout_shell_emits_fjui_chrome_classes() {
        let ctx = test_ctx();
        let html = dashboard_layout().render(&ctx);
        assert!(html.contains("fjui-sidebar"), "DashboardLayout must emit fjui-sidebar");
        assert!(html.contains("fjui-header"), "DashboardLayout must emit fjui-header");
        // Structural swap-target preserved (T-246-19).
        assert!(html.contains("id=\"ferro-json-ui\""), "ferro-json-ui swap-target must be preserved");
        assert!(html.contains("data-sidebar"), "data-sidebar must be preserved");
        assert!(html.contains("data-toast-container"), "data-toast-container must be preserved");
        // Grid layout utilities preserved (D-02).
        assert!(html.contains("md:pl-64"), "DashboardLayout grid utility md:pl-64 must be preserved");
        assert!(html.contains("max-w-7xl"), "DashboardLayout grid utility max-w-7xl must be preserved");
    }

    // ── Plan 03 (247): Avatar menu + search affordance tests ────────────────

    #[test]
    fn header_emits_avatar_initials() {
        use crate::component::HeaderProps;
        let props = HeaderProps {
            business_name: "Acme".to_string(),
            notification_count: None,
            user_name: Some("Alice Rossi".into()),
            user_avatar: None,
            logout_url: Some("/logout".into()),
            theme_url: None,
            profile_url: None,
        };
        let html = layout_header_html(&props);
        assert!(html.contains("fjui-avatar"), "avatar button must be present; got: {html}");
        assert!(
            html.contains("popovertarget=\"fjui-avatar-menu\""),
            "avatar button must wire to fjui-avatar-menu popover; got: {html}"
        );
        assert!(
            html.contains("AR"),
            "initials AR must be present from 'Alice Rossi'; got: {html}"
        );
    }

    #[test]
    fn header_no_bare_logout_link() {
        use crate::component::HeaderProps;
        let props = HeaderProps {
            business_name: "Acme".to_string(),
            notification_count: None,
            user_name: None,
            user_avatar: None,
            logout_url: Some("/logout".into()),
            theme_url: None,
            profile_url: None,
        };
        let html = layout_header_html(&props);
        assert!(
            !html.contains(">Logout</a>"),
            "bare Logout link must be gone; got: {html}"
        );
        assert!(
            html.contains("action=\"/logout\""),
            "logout must be moved into Esci POST form; got: {html}"
        );
    }

    #[test]
    fn header_search_button_present() {
        use crate::component::HeaderProps;
        let props = HeaderProps {
            business_name: "Acme".to_string(),
            notification_count: None,
            user_name: None,
            user_avatar: None,
            logout_url: None,
            theme_url: None,
            profile_url: None,
        };
        let html = layout_header_html(&props);
        assert!(
            html.contains("fjui-header__search-btn"),
            "search button must be present; got: {html}"
        );
        assert!(
            html.contains("data-tooltip=\"Cerca\""),
            "search button must carry data-tooltip Cerca (UX-02); got: {html}"
        );
        assert!(
            html.contains("fjui:open-command-palette"),
            "search button must dispatch fjui:open-command-palette (D-06); got: {html}"
        );
        assert!(
            html.contains("fjui-kbd"),
            "search button must include fjui-kbd chip; got: {html}"
        );
    }

    /// Tema item renders only when theme_url is configured, POSTs to that URL,
    /// and toggles the dark class only on a 2xx response (WR-04).
    #[test]
    fn header_theme_toggle_only_when_theme_url_set() {
        use crate::component::HeaderProps;
        let mut props = HeaderProps {
            business_name: "Acme".to_string(),
            notification_count: None,
            user_name: None,
            user_avatar: None,
            logout_url: None,
            theme_url: None,
            profile_url: None,
        };
        let html = layout_header_html(&props);
        assert!(
            !html.contains(">Tema</button>"),
            "Tema item must be omitted when theme_url is None; got: {html}"
        );

        props.theme_url = Some("/dashboard/theme".to_string());
        let html = layout_header_html(&props);
        assert!(
            html.contains("fetch('/dashboard/theme'"),
            "Tema must POST to the configured theme_url; got: {html}"
        );
        assert!(
            html.contains("if(r.ok)document.documentElement.classList.toggle('dark')"),
            "dark-class toggle must be gated on response.ok; got: {html}"
        );
    }
}
