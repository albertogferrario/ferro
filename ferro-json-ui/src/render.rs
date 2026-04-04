//! HTML render engine for JSON-UI views.
//!
//! Walks a `JsonUiView` component tree and produces an HTML fragment using
//! Tailwind CSS utility classes. All 26 built-in component types plus plugin
//! components are supported. Plugin components are dispatched to the plugin
//! registry; their CSS/JS assets are collected and returned separately.

use std::collections::HashSet;

use serde_json::Value;

use crate::action::HttpMethod;
use crate::component::{
    ActionCardProps, ActionCardVariant, AlertProps, AlertVariant, AvatarProps, BadgeProps,
    BadgeVariant, BreadcrumbProps, ButtonGroupProps, ButtonProps, ButtonType, ButtonVariant,
    CalendarCellProps, CardProps, CheckboxProps, ChecklistProps, CollapsibleProps, Component,
    ComponentNode, DataTableProps, DescriptionListProps, DropdownMenuAction, DropdownMenuProps,
    EmptyStateProps, FormMaxWidth, FormProps, FormSectionLayout, FormSectionProps, GapSize,
    GridProps, HeaderProps, IconPosition, InputProps, InputType, KanbanBoardProps, ModalProps,
    NotificationDropdownProps, Orientation, PageHeaderProps, PaginationProps, PluginProps,
    ProductTileProps, ProgressProps, SelectProps, SeparatorProps, SidebarProps, Size, SkeletonProps,
    StatCardProps, SwitchProps, TableProps, TabsProps, TextElement, TextProps, ToastProps,
    ToastVariant,
};
use crate::data::{resolve_path, resolve_path_string};
use crate::plugin::{collect_plugin_assets, Asset};
use crate::view::JsonUiView;

/// Render a JSON-UI view to an HTML fragment.
///
/// Walks the component tree and produces a `<div>` containing all rendered
/// components. This is a fragment, not a full page -- the framework wrapper
/// handles `<html>`, `<head>`, and `<body>`.
///
/// The `data` parameter is used to resolve `data_path` references on form
/// fields and table components.
pub fn render_to_html(view: &JsonUiView, data: &Value) -> String {
    let mut html = String::from(
        "<div class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">",
    );
    for node in &view.components {
        html.push_str(&render_node(node, data));
    }
    html.push_str("</div>");
    html
}

/// Result of rendering a view with plugin support.
///
/// Contains the rendered HTML fragment plus CSS and JS tags collected
/// from plugins used on the page.
pub struct RenderResult {
    /// The rendered HTML fragment (same as `render_to_html` output).
    pub html: String,
    /// CSS `<link>` tags to inject into `<head>`.
    pub css_head: String,
    /// JS `<script>` tags and init scripts to inject before `</body>`.
    pub scripts: String,
}

/// Render a JSON-UI view to HTML and collect plugin assets.
///
/// Scans the component tree for plugin components, renders everything to
/// HTML (including plugin components via the registry), then collects and
/// deduplicates CSS/JS assets from the plugins used on the page.
pub fn render_to_html_with_plugins(view: &JsonUiView, data: &Value) -> RenderResult {
    let html = render_to_html(view, data);

    let plugin_types = collect_plugin_types(view);
    if plugin_types.is_empty() {
        return RenderResult {
            html,
            css_head: String::new(),
            scripts: String::new(),
        };
    }

    let type_names: Vec<String> = plugin_types.into_iter().collect();
    let assets = collect_plugin_assets(&type_names);

    let css_head = render_css_tags(&assets.css);
    let scripts = render_js_tags(&assets.js, &assets.init_scripts);

    RenderResult {
        html,
        css_head,
        scripts,
    }
}

/// Walk the component tree and collect unique plugin type names.
pub(crate) fn collect_plugin_types(view: &JsonUiView) -> HashSet<String> {
    let mut types = HashSet::new();
    for node in &view.components {
        collect_plugin_types_node(node, &mut types);
    }
    types
}

/// Recursively collect plugin type names from a component node.
fn collect_plugin_types_node(node: &ComponentNode, types: &mut HashSet<String>) {
    match &node.component {
        Component::Plugin(props) => {
            types.insert(props.plugin_type.clone());
        }
        Component::Card(props) => {
            for child in &props.children {
                collect_plugin_types_node(child, types);
            }
            for child in &props.footer {
                collect_plugin_types_node(child, types);
            }
        }
        Component::Form(props) => {
            for field in &props.fields {
                collect_plugin_types_node(field, types);
            }
        }
        Component::Modal(props) => {
            for child in &props.children {
                collect_plugin_types_node(child, types);
            }
            for child in &props.footer {
                collect_plugin_types_node(child, types);
            }
        }
        Component::Tabs(props) => {
            for tab in &props.tabs {
                for child in &tab.children {
                    collect_plugin_types_node(child, types);
                }
            }
        }
        Component::Grid(props) => {
            for child in &props.children {
                collect_plugin_types_node(child, types);
            }
        }
        Component::Collapsible(props) => {
            for child in &props.children {
                collect_plugin_types_node(child, types);
            }
        }
        Component::FormSection(props) => {
            for child in &props.children {
                collect_plugin_types_node(child, types);
            }
        }
        Component::PageHeader(props) => {
            for child in &props.actions {
                collect_plugin_types_node(child, types);
            }
        }
        Component::ButtonGroup(props) => {
            for child in &props.buttons {
                collect_plugin_types_node(child, types);
            }
        }
        // Leaf components have no children to recurse into.
        Component::Table(_)
        | Component::Button(_)
        | Component::Input(_)
        | Component::Select(_)
        | Component::Alert(_)
        | Component::Badge(_)
        | Component::Text(_)
        | Component::Checkbox(_)
        | Component::Switch(_)
        | Component::Separator(_)
        | Component::DescriptionList(_)
        | Component::Breadcrumb(_)
        | Component::Pagination(_)
        | Component::Progress(_)
        | Component::Avatar(_)
        | Component::Skeleton(_)
        | Component::StatCard(_)
        | Component::Checklist(_)
        | Component::Toast(_)
        | Component::NotificationDropdown(_)
        | Component::Sidebar(_)
        | Component::Header(_)
        | Component::EmptyState(_)
        | Component::DropdownMenu(_)
        | Component::CalendarCell(_)
        | Component::ActionCard(_)
        | Component::ProductTile(_)
        | Component::DataTable(_) => {}
        Component::KanbanBoard(props) => {
            for col in &props.columns {
                for child in &col.children {
                    collect_plugin_types_node(child, types);
                }
            }
        }
    }
}

/// Render CSS assets as `<link>` tags.
fn render_css_tags(assets: &[Asset]) -> String {
    let mut out = String::new();
    for asset in assets {
        out.push_str("<link rel=\"stylesheet\" href=\"");
        out.push_str(&html_escape(&asset.url));
        out.push('"');
        if let Some(ref integrity) = asset.integrity {
            out.push_str(" integrity=\"");
            out.push_str(&html_escape(integrity));
            out.push('"');
        }
        if let Some(ref co) = asset.crossorigin {
            out.push_str(" crossorigin=\"");
            out.push_str(&html_escape(co));
            out.push('"');
        }
        out.push('>');
    }
    out
}

/// Render JS assets as `<script>` tags followed by inline init scripts.
fn render_js_tags(assets: &[Asset], init_scripts: &[String]) -> String {
    let mut out = String::new();
    for asset in assets {
        out.push_str("<script src=\"");
        out.push_str(&html_escape(&asset.url));
        out.push('"');
        if let Some(ref integrity) = asset.integrity {
            out.push_str(" integrity=\"");
            out.push_str(&html_escape(integrity));
            out.push('"');
        }
        if let Some(ref co) = asset.crossorigin {
            out.push_str(" crossorigin=\"");
            out.push_str(&html_escape(co));
            out.push('"');
        }
        out.push_str("></script>");
    }
    if !init_scripts.is_empty() {
        out.push_str("<script>");
        for script in init_scripts {
            out.push_str(script);
        }
        out.push_str("</script>");
    }
    out
}

/// Render a single component node, optionally wrapping in `<a>` for GET actions.
fn render_node(node: &ComponentNode, data: &Value) -> String {
    let component_html = render_component(&node.component, data);

    // Wrap in <a> if the node has a GET action with a resolved URL.
    if let Some(ref action) = node.action {
        if action.method == HttpMethod::Get {
            if let Some(ref url) = action.url {
                return format!(
                    "<a href=\"{}\" class=\"block\">{}</a>",
                    html_escape(url),
                    component_html
                );
            }
        }
    }

    component_html
}

/// Dispatch to the appropriate per-component renderer.
fn render_component(component: &Component, data: &Value) -> String {
    match component {
        Component::Text(props) => render_text(props),
        Component::Button(props) => render_button(props),
        Component::Badge(props) => render_badge(props),
        Component::Alert(props) => render_alert(props),
        Component::Separator(props) => render_separator(props),
        Component::Progress(props) => render_progress(props),
        Component::Avatar(props) => render_avatar(props),
        Component::Skeleton(props) => render_skeleton(props),
        Component::Breadcrumb(props) => render_breadcrumb(props),
        Component::Pagination(props) => render_pagination(props),
        Component::DescriptionList(props) => render_description_list(props),

        // Container components.
        Component::Card(props) => render_card(props, data),
        Component::Form(props) => render_form(props, data),
        Component::Modal(props) => render_modal(props, data),
        Component::Tabs(props) => render_tabs(props, data),
        Component::Table(props) => render_table(props, data),

        // Form field components.
        Component::Input(props) => render_input(props, data),
        Component::Select(props) => render_select(props, data),
        Component::Checkbox(props) => render_checkbox(props, data),
        Component::Switch(props) => render_switch(props, data),

        // Layout components.
        Component::Grid(props) => render_grid(props, data),
        Component::Collapsible(props) => render_collapsible(props, data),
        Component::FormSection(props) => render_form_section(props, data),

        // Standalone components.
        Component::EmptyState(props) => render_empty_state(props),
        Component::DropdownMenu(props) => render_dropdown_menu(props),

        // Dashboard components.
        Component::StatCard(props) => render_stat_card(props),
        Component::Checklist(props) => render_checklist(props),
        Component::Toast(props) => render_toast(props),
        Component::NotificationDropdown(props) => render_notification_dropdown(props),
        Component::Sidebar(props) => render_sidebar(props),
        Component::Header(props) => render_header(props),

        // Page layout components.
        Component::PageHeader(props) => render_page_header(props, data),
        Component::ButtonGroup(props) => render_button_group(props, data),

        // Standalone leaf components.
        Component::CalendarCell(props) => render_calendar_cell(props),
        Component::ActionCard(props) => render_action_card(props),
        Component::ProductTile(props) => render_product_tile(props),
        Component::DataTable(props) => render_data_table(props, data),

        // Container components (responsive).
        Component::KanbanBoard(props) => render_kanban_board(props, data),

        // Plugin components (rendered via plugin registry).
        Component::Plugin(props) => render_plugin(props, data),
    }
}

// ── CalendarCell renderer ───────────────────────────────────────────────

fn render_calendar_cell(props: &CalendarCellProps) -> String {
    let opacity = if props.is_current_month { "" } else { " opacity-40" };
    let hover = if props.is_current_month {
        " hover:bg-surface/60 transition-colors cursor-pointer"
    } else {
        " cursor-pointer"
    };

    let mut html = format!(
        "<div class=\"flex flex-col min-h-[5rem] p-2 border border-border -mt-px -ml-px{opacity}{hover}\">",
    );

    // Day number — top-left, today gets a small circle
    if props.is_today {
        html.push_str(&format!(
            "<span class=\"w-7 h-7 flex items-center justify-center text-sm font-semibold bg-primary text-primary-foreground rounded-full\">{}</span>",
            props.day
        ));
    } else {
        html.push_str(&format!(
            "<span class=\"text-sm text-text\">{}</span>",
            props.day
        ));
    }

    // Event indicators — dots for 1-3 events, count badge for 4+
    if props.event_count > 0 && props.event_count <= 3 {
        html.push_str("<div class=\"flex gap-1 mt-auto pt-1\">");
        for _ in 0..props.event_count {
            html.push_str("<span class=\"w-1.5 h-1.5 rounded-full bg-primary\"></span>");
        }
        html.push_str("</div>");
    } else if props.event_count > 3 {
        html.push_str(&format!(
            "<span class=\"mt-auto pt-1 text-xs font-medium text-primary\">{} prenot.</span>",
            props.event_count
        ));
    }

    html.push_str("</div>");
    html
}

// ── ActionCard renderer ────────────────────────────────────────────────

fn render_action_card(props: &ActionCardProps) -> String {
    let border_class = match props.variant {
        ActionCardVariant::Default => "border-l-primary",
        ActionCardVariant::Setup => "border-l-warning",
        ActionCardVariant::Danger => "border-l-destructive",
    };

    // A11Y-04: use <a> when href is set, <div> otherwise
    let (open_tag, close_tag) = if let Some(ref href) = props.href {
        (
            format!(
                "<a href=\"{}\" aria-label=\"{}\" class=\"rounded-lg border-l-4 {} border border-border bg-card shadow-sm p-4 flex items-center gap-4 hover:bg-surface transition-colors duration-150 no-underline\">",
                html_escape(href),
                html_escape(&props.title),
                border_class,
            ),
            "</a>".to_string(),
        )
    } else {
        (
            format!(
                "<div class=\"rounded-lg border-l-4 {} border border-border bg-card shadow-sm p-4 flex items-center gap-4 cursor-pointer hover:bg-surface transition-colors duration-150\">",
                border_class,
            ),
            "</div>".to_string(),
        )
    };

    let mut html = open_tag;

    // Optional icon.
    if let Some(ref icon) = props.icon {
        html.push_str(&format!(
            "<div class=\"w-10 h-10 flex-shrink-0 rounded-md bg-surface flex items-center justify-center text-text-muted\">{icon}</div>",
        ));
    }

    // Text block.
    html.push_str(&format!(
        "<div class=\"flex-1 min-w-0\"><p class=\"text-sm font-semibold text-text\">{}</p><p class=\"text-sm text-text-muted mt-0.5\">{}</p></div>",
        html_escape(&props.title),
        html_escape(&props.description)
    ));

    // Chevron.
    html.push_str("<span class=\"text-text-muted flex-shrink-0 text-lg\">&rsaquo;</span>");

    html.push_str(&close_tag);
    html
}

// ── ProductTile renderer ────────────────────────────────────────────────

fn render_product_tile(props: &ProductTileProps) -> String {
    let name = html_escape(&props.name);
    let price = html_escape(&props.price);
    let field = html_escape(&props.field);
    let qty = props.default_quantity.unwrap_or(0);

    format!(
        "<div class=\"rounded-lg border border-border bg-card p-4 flex flex-col gap-3 touch-manipulation\">\
         <div class=\"flex items-start justify-between gap-2\">\
         <span class=\"text-sm font-semibold text-text\">{name}</span>\
         <span class=\"text-sm font-semibold text-text-muted\">{price}</span>\
         </div>\
         <div class=\"flex items-center justify-between gap-2\">\
         <button type=\"button\" data-qty-dec=\"{field}\" \
         class=\"min-h-[44px] min-w-[44px] flex items-center justify-center rounded-md border border-border bg-surface text-text text-lg font-semibold hover:bg-border transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\" \
         aria-label=\"Diminuisci quantit\u{00E0} {name}\">\u{2212}</button>\
         <span data-qty-display=\"{field}\" class=\"text-sm font-semibold text-text min-w-[2ch] text-center\">{qty}</span>\
         <button type=\"button\" data-qty-inc=\"{field}\" \
         class=\"min-h-[44px] min-w-[44px] flex items-center justify-center rounded-md border border-border bg-surface text-text text-lg font-semibold hover:bg-border transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\" \
         aria-label=\"Aumenta quantit\u{00E0} {name}\">+</button>\
         </div>\
         <input type=\"hidden\" name=\"{field}\" data-qty-input=\"{field}\" value=\"{qty}\">\
         </div>"
    )
}

// ── KanbanBoard renderer ────────────────────────────────────────────────

fn render_kanban_board(props: &KanbanBoardProps, data: &Value) -> String {
    if props.columns.is_empty() {
        return String::new();
    }

    let default_id = props
        .mobile_default_column
        .as_deref()
        .unwrap_or_else(|| &props.columns[0].id);

    let mut html = String::new();

    // ── Desktop view: horizontal scrollable columns ──────────────────
    html.push_str("<div class=\"hidden md:block overflow-x-auto\">");
    html.push_str("<div class=\"flex gap-4\" style=\"min-width: min-content;\">");

    for col in &props.columns {
        html.push_str("<div class=\"min-w-[260px] flex-1 flex-shrink-0 rounded-lg border border-border bg-card/50 p-3\">");
        html.push_str("<div class=\"flex items-center justify-between mb-3\">");
        html.push_str(&format!(
            "<h3 class=\"text-sm font-semibold text-text\">{}</h3>",
            html_escape(&col.title),
        ));
        let badge_class = if col.count > 0 {
            "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-semibold bg-primary text-primary-foreground"
        } else {
            "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium text-text-muted bg-surface"
        };
        html.push_str(&format!(
            "<span class=\"{}\">{}</span>",
            badge_class,
            col.count,
        ));
        html.push_str("</div>");
        html.push_str("<div class=\"space-y-2\">");
        for child in &col.children {
            html.push_str("<div data-kanban-card class=\"cursor-pointer\">");
            html.push_str(&render_node(child, data));
            html.push_str("</div>");
        }
        html.push_str("</div>");
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html.push_str("</div>");

    // ── Mobile view: tab-based column switching ──────────────────────
    html.push_str("<div class=\"block md:hidden\" data-tabs>");
    html.push_str("<div class=\"flex border-b border-border mb-4\">");

    for col in &props.columns {
        let is_default = col.id == default_id;
        let (border, text) = if is_default {
            ("border-primary", "text-primary font-semibold")
        } else {
            ("border-transparent", "text-text-muted hover:text-text")
        };
        html.push_str(&format!(
            "<button type=\"button\" data-tab=\"{}\" class=\"flex-1 px-3 py-2 text-sm border-b-2 {} {}\" aria-selected=\"{}\">{} <span class=\"ml-1 text-xs text-text-muted\">({})</span></button>",
            html_escape(&col.id),
            border,
            text,
            is_default,
            html_escape(&col.title),
            col.count,
        ));
    }

    html.push_str("</div>");

    for col in &props.columns {
        let is_default = col.id == default_id;
        let hidden = if is_default { "" } else { " hidden" };
        html.push_str(&format!(
            "<div data-tab-panel=\"{}\" class=\"{}\">",
            html_escape(&col.id),
            format!("space-y-3{hidden}"),
        ));
        for child in &col.children {
            html.push_str("<div data-kanban-card class=\"cursor-pointer\">");
            html.push_str(&render_node(child, data));
            html.push_str("</div>");
        }
        html.push_str("</div>");
    }

    html.push_str("</div>");

    html
}

// ── DropdownMenu renderer ───────────────────────────────────────────────

fn render_dropdown_menu(props: &DropdownMenuProps) -> String {
    let mut html = String::from("<div class=\"relative\">");

    // Trigger button — kebab icon (⋮) with label as aria-label for accessibility
    let trigger_icon = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><circle cx=\"12\" cy=\"5\" r=\"1\"/><circle cx=\"12\" cy=\"12\" r=\"1\"/><circle cx=\"12\" cy=\"19\" r=\"1\"/></svg>";
    html.push_str(&format!(
        "<button type=\"button\" data-dropdown-toggle=\"{}\" aria-label=\"{}\" \
         class=\"inline-flex items-center justify-center rounded-md p-1.5 \
         text-text-muted hover:text-text hover:bg-surface transition-colors duration-150 \
         focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary \
         focus-visible:ring-offset-2\">{}</button>",
        html_escape(&props.menu_id),
        html_escape(&props.trigger_label),
        trigger_icon,
    ));

    // Panel (hidden by default)
    html.push_str(&format!(
        "<div data-dropdown=\"{}\" \
         class=\"absolute right-0 z-50 mt-1 w-48 rounded-md border border-border bg-card shadow-md hidden\">",
        html_escape(&props.menu_id),
    ));

    for item in &props.items {
        let url = item.action.url.as_deref().unwrap_or("#");
        let base_class = if item.destructive {
            "block px-4 py-2 text-sm text-destructive hover:bg-destructive/10 transition-colors duration-150"
        } else {
            "block px-4 py-2 text-sm text-text hover:bg-surface transition-colors duration-150"
        };

        // Confirm dialog data attributes
        let confirm_attrs = if let Some(ref confirm) = item.action.confirm {
            let mut attrs = format!(
                " data-confirm-title=\"{}\"",
                html_escape(&confirm.title)
            );
            if let Some(ref message) = confirm.message {
                attrs.push_str(&format!(
                    " data-confirm-message=\"{}\"",
                    html_escape(message)
                ));
            }
            attrs
        } else {
            String::new()
        };

        let onclick = if item.action.confirm.is_some() {
            " onclick=\"return confirm(this.dataset.confirmTitle || this.dataset.confirmMessage)\""
        } else {
            ""
        };

        match item.action.method {
            HttpMethod::Get => {
                html.push_str(&format!(
                    "<a href=\"{}\" class=\"{}\"{}{}>{}</a>",
                    html_escape(url),
                    base_class,
                    confirm_attrs,
                    onclick,
                    html_escape(&item.label),
                ));
            }
            HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch | HttpMethod::Delete => {
                let (form_method, needs_spoofing) = match item.action.method {
                    HttpMethod::Post => ("post", false),
                    HttpMethod::Put | HttpMethod::Patch | HttpMethod::Delete => ("post", true),
                    _ => unreachable!(),
                };
                html.push_str(&format!(
                    "<form action=\"{}\" method=\"{}\">",
                    html_escape(url),
                    form_method,
                ));
                if needs_spoofing {
                    let method_value = match item.action.method {
                        HttpMethod::Put => "PUT",
                        HttpMethod::Patch => "PATCH",
                        HttpMethod::Delete => "DELETE",
                        _ => unreachable!(),
                    };
                    html.push_str(&format!(
                        "<input type=\"hidden\" name=\"_method\" value=\"{method_value}\">"
                    ));
                }
                html.push_str(&format!(
                    "<button type=\"submit\" class=\"w-full text-left {}\"{}{}>{}</button>",
                    base_class,
                    confirm_attrs,
                    onclick,
                    html_escape(&item.label),
                ));
                html.push_str("</form>");
            }
        }
    }

    html.push_str("</div>"); // close panel
    html.push_str("</div>"); // close wrapper
    html
}

// ── Plugin component renderer ───────────────────────────────────────────

fn render_plugin(props: &PluginProps, data: &Value) -> String {
    crate::plugin::with_plugin(&props.plugin_type, |plugin| {
        plugin.render(&props.props, data)
    })
    .unwrap_or_else(|| {
        format!(
            "<div class=\"p-4 bg-destructive/10 text-destructive rounded-md\">Unknown plugin component: {}</div>",
            html_escape(&props.plugin_type)
        )
    })
}

// ── Page layout component renderers ─────────────────────────────────────

fn render_page_header(props: &PageHeaderProps, data: &Value) -> String {
    let mut html = String::from(
        "<div class=\"flex flex-wrap items-center justify-between gap-3 pb-4\">",
    );

    // Title block — breadcrumb and title fused into one inline flow
    html.push_str("<div class=\"flex items-center gap-2 min-w-0\">");

    if !props.breadcrumb.is_empty() {
        for item in &props.breadcrumb {
            if let Some(ref url) = item.url {
                html.push_str(&format!(
                    "<a href=\"{}\" class=\"text-sm text-text-muted hover:text-text whitespace-nowrap\">{}</a>",
                    html_escape(url),
                    html_escape(&item.label)
                ));
            } else {
                html.push_str(&format!(
                    "<span class=\"text-sm text-text-muted whitespace-nowrap\">{}</span>",
                    html_escape(&item.label)
                ));
            }
            // Chevron separator between breadcrumb and title
            html.push_str(
                "<span aria-hidden=\"true\" class=\"text-text-muted flex-shrink-0\">\
                 <svg class=\"h-4 w-4\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">\
                 <path fill-rule=\"evenodd\" d=\"M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z\" clip-rule=\"evenodd\"/>\
                 </svg></span>"
            );
        }
    }

    html.push_str(&format!(
        "<h2 class=\"text-2xl font-semibold leading-tight tracking-tight text-text truncate\">{}</h2>",
        html_escape(&props.title)
    ));
    html.push_str("</div>");

    // Actions (optional)
    if !props.actions.is_empty() {
        html.push_str("<div class=\"flex flex-wrap items-center gap-2\">");
        for action in &props.actions {
            html.push_str(&render_node(action, data));
        }
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html
}

fn render_button_group(props: &ButtonGroupProps, data: &Value) -> String {
    let mut html = String::from("<div class=\"flex items-center gap-2 flex-wrap\">");
    for button in &props.buttons {
        html.push_str(&render_node(button, data));
    }
    html.push_str("</div>");
    html
}

// ── Container component renderers ───────────────────────────────────────

fn render_card(props: &CardProps, data: &Value) -> String {
    let mut html = String::from(
        "<div class=\"rounded-lg border border-border bg-card shadow-sm overflow-visible\"><div class=\"p-4\">",
    );
    html.push_str(&format!(
        "<h3 class=\"text-base font-semibold leading-snug text-text\">{}</h3>",
        html_escape(&props.title)
    ));
    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"mt-1 text-sm text-text-muted\">{}</p>",
            html_escape(desc)
        ));
    }
    if !props.children.is_empty() {
        html.push_str(
            "<div class=\"mt-3 flex flex-wrap gap-3 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto overflow-visible\">",
        );
        for child in &props.children {
            html.push_str(&render_node(child, data));
        }
        html.push_str("</div>");
    }
    html.push_str("</div>"); // close p-6
    if !props.footer.is_empty() {
        html.push_str("<div class=\"border-t border-border px-6 py-4 flex items-center gap-2\">");
        for child in &props.footer {
            html.push_str(&render_node(child, data));
        }
        html.push_str("</div>");
    }
    html.push_str("</div>"); // close outer card

    match props.max_width.as_ref().unwrap_or(&FormMaxWidth::Default) {
        FormMaxWidth::Default => {}
        FormMaxWidth::Narrow => {
            html = format!("<div class=\"max-w-2xl mx-auto\">{}</div>", html);
        }
        FormMaxWidth::Wide => {
            html = format!("<div class=\"max-w-4xl mx-auto\">{}</div>", html);
        }
    }

    html
}

fn render_modal(props: &ModalProps, data: &Value) -> String {
    let trigger = props.trigger_label.as_deref().unwrap_or("Open");
    let mut html = String::new();
    // Trigger button (sibling of dialog, not inside it)
    html.push_str(&format!(
        "<button type=\"button\" class=\"inline-flex items-center justify-center rounded-md bg-primary text-primary-foreground px-4 py-2 text-sm font-medium cursor-pointer\" data-modal-open=\"{}\">{}</button>",
        html_escape(&props.id),
        html_escape(trigger)
    ));
    // Native <dialog> element — focus trap and Escape key are built-in
    html.push_str(&format!(
        "<dialog id=\"{}\" aria-modal=\"true\" aria-labelledby=\"{}-title\" class=\"bg-card rounded-lg shadow-lg max-w-lg w-full mx-4 p-6 backdrop:bg-black/50\">",
        html_escape(&props.id),
        html_escape(&props.id)
    ));
    // Header row: title + close button
    html.push_str("<div class=\"flex items-center justify-between mb-4\">");
    html.push_str(&format!(
        "<h3 id=\"{}-title\" class=\"text-lg font-semibold leading-snug text-text\">{}</h3>",
        html_escape(&props.id),
        html_escape(&props.title)
    ));
    html.push_str(
        "<button type=\"button\" data-modal-close aria-label=\"Chiudi\" class=\"text-text-muted hover:text-text p-2 rounded transition-colors duration-150\">\u{00d7}</button>",
    );
    html.push_str("</div>");
    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"text-sm text-text-muted mb-4\">{}</p>",
            html_escape(desc)
        ));
    }
    html.push_str(
        "<div class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">",
    );
    for child in &props.children {
        html.push_str(&render_node(child, data));
    }
    html.push_str("</div>");
    if !props.footer.is_empty() {
        html.push_str("<div class=\"mt-6 flex items-center justify-end gap-2\">");
        for child in &props.footer {
            html.push_str(&render_node(child, data));
        }
        html.push_str("</div>");
    }
    html.push_str("</dialog>");
    html
}

fn render_tabs(props: &TabsProps, data: &Value) -> String {
    // Auto-hide tab bar when only one tab.
    if props.tabs.len() == 1 {
        let tab = &props.tabs[0];
        let mut html = String::from(
            "<div class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">",
        );
        for child in &tab.children {
            html.push_str(&render_node(child, data));
        }
        html.push_str("</div>");
        return html;
    }

    // Determine rendering strategy per tab:
    // - Tabs with children render their panel and switch client-side via JS.
    // - Empty tabs (server-driven) render as links with ?tab= for full reload.
    let has_any_content = props.tabs.iter().any(|t| !t.children.is_empty());

    let mut html = String::from("<div data-tabs>");
    html.push_str("<div class=\"border-b border-border\">");
    html.push_str("<nav class=\"flex -mb-px space-x-4\" role=\"tablist\">");

    for tab in &props.tabs {
        let is_active = tab.value == props.default_tab;
        let border = if is_active {
            "border-primary"
        } else {
            "border-transparent"
        };
        let text = if is_active {
            "text-primary font-semibold"
        } else {
            "text-text-muted hover:text-text"
        };

        if has_any_content && (is_active || !tab.children.is_empty()) {
            // Client-side tab trigger
            html.push_str(&format!(
                "<button type=\"button\" role=\"tab\" id=\"tab-btn-{}\" aria-controls=\"tab-panel-{}\" data-tab=\"{}\" \
                 class=\"border-b-2 {} {} px-3 py-2 text-sm font-medium cursor-pointer transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\" \
                 aria-selected=\"{}\">{}</button>",
                html_escape(&tab.value),
                html_escape(&tab.value),
                html_escape(&tab.value),
                border,
                text,
                is_active,
                html_escape(&tab.label),
            ));
        } else {
            // Server-driven tab: link with ?tab= query param
            html.push_str(&format!(
                "<a href=\"?tab={}\" role=\"tab\" id=\"tab-btn-{}\" aria-controls=\"tab-panel-{}\" \
                 class=\"border-b-2 {} {} px-3 py-2 text-sm font-medium transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\" \
                 aria-selected=\"{}\">{}</a>",
                html_escape(&tab.value),
                html_escape(&tab.value),
                html_escape(&tab.value),
                border,
                text,
                is_active,
                html_escape(&tab.label),
            ));
        }
    }

    html.push_str("</nav></div>");

    // Render all tab panels — inactive panels are hidden via CSS.
    for tab in &props.tabs {
        if tab.children.is_empty() && tab.value != props.default_tab {
            continue;
        }
        let hidden = if tab.value != props.default_tab {
            " hidden"
        } else {
            ""
        };
        html.push_str(&format!(
            "<div role=\"tabpanel\" id=\"tab-panel-{}\" aria-labelledby=\"tab-btn-{}\" data-tab-panel=\"{}\" class=\"pt-4 flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto{}\">",
            html_escape(&tab.value),
            html_escape(&tab.value),
            html_escape(&tab.value),
            hidden,
        ));
        for child in &tab.children {
            html.push_str(&render_node(child, data));
        }
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html
}

fn render_form(props: &FormProps, data: &Value) -> String {
    // Determine the effective HTTP method.
    let effective_method = props
        .method
        .as_ref()
        .unwrap_or(&props.action.method)
        .clone();

    // For PUT/PATCH/DELETE, use POST with method spoofing.
    let (form_method, needs_spoofing) = match effective_method {
        HttpMethod::Get => ("get", false),
        HttpMethod::Post => ("post", false),
        HttpMethod::Put | HttpMethod::Patch | HttpMethod::Delete => ("post", true),
    };

    let action_url = props.action.url.as_deref().unwrap_or("#");
    let mut html = match &props.guard {
        Some(g) => format!(
            "<form action=\"{}\" method=\"{}\" data-form-guard=\"{}\" class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">",
            html_escape(action_url),
            form_method,
            html_escape(g)
        ),
        None => format!(
            "<form action=\"{}\" method=\"{}\" class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">",
            html_escape(action_url),
            form_method
        ),
    };

    if needs_spoofing {
        let method_value = match effective_method {
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            _ => unreachable!(),
        };
        html.push_str(&format!(
            "<input type=\"hidden\" name=\"_method\" value=\"{method_value}\">"
        ));
    }

    for field in &props.fields {
        html.push_str(&render_node(field, data));
    }
    html.push_str("</form>");

    // FIX-02: wrap in max-width container when specified
    let html = match props.max_width.as_ref().unwrap_or(&FormMaxWidth::Default) {
        FormMaxWidth::Default => html,
        FormMaxWidth::Narrow => format!("<div class=\"max-w-2xl mx-auto\">{}</div>", html),
        FormMaxWidth::Wide => format!("<div class=\"max-w-4xl mx-auto\">{}</div>", html),
    };
    html
}

fn render_table(props: &TableProps, data: &Value) -> String {
    let mut html = String::from(
        "<div class=\"overflow-x-auto\"><table class=\"min-w-full divide-y divide-border\">",
    );

    // Header.
    html.push_str("<thead class=\"bg-surface\"><tr>");
    for col in &props.columns {
        html.push_str(&format!(
            "<th class=\"px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-text-muted\">{}</th>",
            html_escape(&col.label)
        ));
    }
    if props.row_actions.is_some() {
        html.push_str("<th class=\"px-6 py-3 text-right text-xs font-medium uppercase tracking-wider text-text-muted\">Azioni</th>");
    }
    html.push_str("</tr></thead>");

    // Body.
    html.push_str("<tbody class=\"divide-y divide-border bg-background\">");

    let rows = resolve_path(data, &props.data_path);
    let row_array = rows.and_then(|v| v.as_array());

    if let Some(items) = row_array {
        if items.is_empty() {
            if let Some(ref msg) = props.empty_message {
                let col_count =
                    props.columns.len() + if props.row_actions.is_some() { 1 } else { 0 };
                html.push_str(&format!(
                    "<tr><td colspan=\"{}\" class=\"px-6 py-8 text-center text-sm text-text-muted\">{}</td></tr>",
                    col_count,
                    html_escape(msg)
                ));
            }
        } else {
            for row in items {
                html.push_str("<tr class=\"hover:bg-surface\">");
                for col in &props.columns {
                    let cell_value = row.get(&col.key);
                    let cell_text = match cell_value {
                        Some(Value::String(s)) => s.clone(),
                        Some(Value::Number(n)) => n.to_string(),
                        Some(Value::Bool(b)) => b.to_string(),
                        Some(Value::Null) | None => String::new(),
                        Some(v @ Value::Array(_)) | Some(v @ Value::Object(_)) => {
                            serde_json::to_string(v).unwrap_or_default()
                        }
                    };
                    html.push_str(&format!(
                        "<td class=\"px-6 py-4 text-sm text-text whitespace-nowrap\">{}</td>",
                        html_escape(&cell_text)
                    ));
                }
                if let Some(ref actions) = props.row_actions {
                    html.push_str("<td class=\"px-6 py-4 text-right text-sm space-x-2\">");
                    for action in actions {
                        let url = action.url.as_deref().unwrap_or("#");
                        let label = action
                            .handler
                            .split('.')
                            .next_back()
                            .unwrap_or(&action.handler);
                        html.push_str(&format!(
                            "<a href=\"{}\" class=\"text-primary hover:text-primary/80\">{}</a>",
                            html_escape(url),
                            html_escape(label)
                        ));
                    }
                    html.push_str("</td>");
                }
                html.push_str("</tr>");
            }
        }
    } else if let Some(ref msg) = props.empty_message {
        let col_count = props.columns.len() + if props.row_actions.is_some() { 1 } else { 0 };
        html.push_str(&format!(
            "<tr><td colspan=\"{}\" class=\"px-6 py-8 text-center text-sm text-text-muted\">{}</td></tr>",
            col_count,
            html_escape(msg)
        ));
    }

    html.push_str("</tbody></table></div>");
    html
}

fn render_data_table(props: &DataTableProps, data: &Value) -> String {
    let rows = resolve_path(data, &props.data_path);
    let row_array = rows.and_then(|v| v.as_array().cloned());
    let items = row_array.unwrap_or_default();
    let has_actions = props.row_actions.is_some();
    let col_count = props.columns.len() + if has_actions { 1 } else { 0 };
    let empty_msg = props
        .empty_message
        .as_deref()
        .unwrap_or("Nessun elemento trovato");

    let mut html = String::new();

    // --- Desktop table (hidden on mobile) ---
    html.push_str("<div class=\"hidden md:block rounded-lg border border-border overflow-hidden\">");

    if items.is_empty() {
        html.push_str("<table class=\"w-full\"><tbody>");
        html.push_str(&format!(
            "<tr><td colspan=\"{}\" class=\"px-6 py-8 text-center text-sm text-text-muted\">{}</td></tr>",
            col_count,
            html_escape(empty_msg)
        ));
        html.push_str("</tbody></table>");
    } else {
        html.push_str("<table class=\"w-full\">");

        // Header
        html.push_str("<thead><tr class=\"bg-surface\">");
        for col in &props.columns {
            html.push_str(&format!(
                "<th class=\"px-6 py-4 text-left text-xs font-semibold uppercase tracking-wider text-text-muted\">{}</th>",
                html_escape(&col.label)
            ));
        }
        if has_actions {
            html.push_str(
                "<th class=\"px-6 py-4 text-right text-xs font-semibold uppercase tracking-wider text-text-muted\">Azioni</th>"
            );
        }
        html.push_str("</tr></thead>");

        // Body
        html.push_str("<tbody>");
        for (index, row) in items.iter().enumerate() {
            html.push_str(
                "<tr class=\"even:bg-surface hover:bg-surface/80 transition-colors duration-150 border-t border-border\">"
            );
            for col in &props.columns {
                let cell_value = row.get(&col.key);
                let cell_text = match cell_value {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Number(n)) => n.to_string(),
                    Some(Value::Bool(b)) => b.to_string(),
                    Some(Value::Null) | None => String::new(),
                    Some(v @ Value::Array(_)) | Some(v @ Value::Object(_)) => {
                        serde_json::to_string(v).unwrap_or_default()
                    }
                };
                html.push_str(&format!(
                    "<td class=\"px-6 py-4 text-sm text-text\">{}</td>",
                    html_escape(&cell_text)
                ));
            }
            if let Some(ref actions) = props.row_actions {
                let row_key_value = if let Some(ref rk) = props.row_key {
                    row.get(rk)
                        .and_then(|v| match v {
                            Value::String(s) => Some(s.clone()),
                            Value::Number(n) => Some(n.to_string()),
                            _ => None,
                        })
                        .unwrap_or_else(|| index.to_string())
                } else {
                    index.to_string()
                };
                let templated_items: Vec<DropdownMenuAction> = actions.iter().map(|a| {
                    let mut cloned = a.clone();
                    // Resolve URL from handler if url is None, then apply row_key template
                    let base_url = cloned.action.url.clone()
                        .or_else(|| Some(cloned.action.handler.clone()));
                    if let Some(url) = base_url {
                        cloned.action.url = Some(url.replace("{row_key}", &row_key_value));
                    }
                    cloned
                }).collect();
                let dropdown_props = DropdownMenuProps {
                    menu_id: format!("dt-{}", row_key_value),
                    trigger_label: "\u{22EE}".to_string(),
                    items: templated_items,
                    trigger_variant: None,
                };
                html.push_str("<td class=\"px-6 py-4 text-right\">");
                html.push_str(&render_dropdown_menu(&dropdown_props));
                html.push_str("</td>");
            }
            html.push_str("</tr>");
        }
        html.push_str("</tbody></table>");
    }
    html.push_str("</div>");

    // --- Mobile cards (visible on mobile) ---
    html.push_str("<div class=\"block md:hidden space-y-3\">");
    if items.is_empty() {
        html.push_str(&format!(
            "<div class=\"text-center text-sm text-text-muted py-8\">{}</div>",
            html_escape(empty_msg)
        ));
    } else {
        for (index, row) in items.iter().enumerate() {
            html.push_str("<div class=\"rounded-lg border border-border bg-card p-4 space-y-2\">");
            for col in &props.columns {
                let cell_value = row.get(&col.key);
                let cell_text = match cell_value {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Number(n)) => n.to_string(),
                    Some(Value::Bool(b)) => b.to_string(),
                    Some(Value::Null) | None => String::new(),
                    Some(v @ Value::Array(_)) | Some(v @ Value::Object(_)) => {
                        serde_json::to_string(v).unwrap_or_default()
                    }
                };
                html.push_str(&format!(
                    "<div class=\"flex justify-between\"><span class=\"text-xs font-semibold text-text-muted uppercase\">{}</span><span class=\"text-sm text-text\">{}</span></div>",
                    html_escape(&col.label),
                    html_escape(&cell_text)
                ));
            }
            if let Some(ref actions) = props.row_actions {
                let row_key_value = if let Some(ref rk) = props.row_key {
                    row.get(rk)
                        .and_then(|v| match v {
                            Value::String(s) => Some(s.clone()),
                            Value::Number(n) => Some(n.to_string()),
                            _ => None,
                        })
                        .unwrap_or_else(|| index.to_string())
                } else {
                    index.to_string()
                };
                let templated_items: Vec<DropdownMenuAction> = actions.iter().map(|a| {
                    let mut cloned = a.clone();
                    // Resolve URL from handler if url is None, then apply row_key template
                    let base_url = cloned.action.url.clone()
                        .or_else(|| Some(cloned.action.handler.clone()));
                    if let Some(url) = base_url {
                        cloned.action.url = Some(url.replace("{row_key}", &row_key_value));
                    }
                    cloned
                }).collect();
                let dropdown_props = DropdownMenuProps {
                    menu_id: format!("dt-m-{}", row_key_value),
                    trigger_label: "\u{22EE}".to_string(),
                    items: templated_items,
                    trigger_variant: None,
                };
                html.push_str("<div class=\"pt-2 border-t border-border flex justify-end\">");
                html.push_str(&render_dropdown_menu(&dropdown_props));
                html.push_str("</div>");
            }
            html.push_str("</div>");
        }
    }
    html.push_str("</div>");

    html
}

// ── Form field component renderers ──────────────────────────────────────

fn render_input(props: &InputProps, data: &Value) -> String {
    // Resolve the effective value: default_value wins, else data_path, else empty.
    let resolved_value = if let Some(ref dv) = props.default_value {
        Some(dv.clone())
    } else if let Some(ref dp) = props.data_path {
        resolve_path_string(data, dp)
    } else {
        None
    };

    // A11Y-07: Hidden inputs emit no label or wrapper div.
    if matches!(props.input_type, InputType::Hidden) {
        let val = resolved_value.as_deref().unwrap_or("");
        return format!(
            "<input type=\"hidden\" id=\"{}\" name=\"{}\" value=\"{}\">",
            html_escape(&props.field),
            html_escape(&props.field),
            html_escape(val)
        );
    }

    let has_error = props.error.is_some();
    let border_class = if has_error {
        "border-destructive"
    } else {
        "border-border"
    };
    let focus_ring_class = if has_error {
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive focus-visible:ring-offset-2"
    } else {
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
    };

    let mut html = String::from("<div class=\"space-y-1\">");
    html.push_str(&format!(
        "<label class=\"block text-sm font-medium text-text\" for=\"{}\">{}</label>",
        html_escape(&props.field),
        html_escape(&props.label)
    ));

    match props.input_type {
        InputType::Hidden => unreachable!("handled by early return above"),
        InputType::Textarea => {
            let val = resolved_value.as_deref().unwrap_or("");
            html.push_str(&format!(
                "<textarea id=\"{}\" name=\"{}\" class=\"block w-full rounded-md border {} px-3 py-2 text-sm shadow-sm transition-colors duration-150 motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed {}\"",
                html_escape(&props.field),
                html_escape(&props.field),
                border_class,
                focus_ring_class
            ));
            if let Some(ref placeholder) = props.placeholder {
                html.push_str(&format!(" placeholder=\"{}\"", html_escape(placeholder)));
            }
            if props.required == Some(true) {
                html.push_str(" required");
            }
            if props.disabled == Some(true) {
                html.push_str(" disabled");
            }
            // FIX-03 / A11Y-08: inline validation ARIA for textarea
            if has_error {
                html.push_str(&format!(
                    " aria-invalid=\"true\" aria-describedby=\"err-{}\"",
                    html_escape(&props.field)
                ));
            }
            html.push_str(&format!(">{}</textarea>", html_escape(val)));
        }
        _ => {
            let input_type = match props.input_type {
                InputType::Text => "text",
                InputType::Email => "email",
                InputType::Password => "password",
                InputType::Number => "number",
                InputType::Date => "date",
                InputType::Time => "time",
                InputType::Url => "url",
                InputType::Tel => "tel",
                InputType::Search => "search",
                InputType::Textarea | InputType::Hidden => unreachable!(),
            };
            html.push_str(&format!(
                "<input type=\"{}\" id=\"{}\" name=\"{}\" class=\"block w-full rounded-md border {} px-3 py-2 text-sm shadow-sm transition-colors duration-150 motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed {}\"",
                input_type,
                html_escape(&props.field),
                html_escape(&props.field),
                border_class,
                focus_ring_class
            ));
            if let Some(ref placeholder) = props.placeholder {
                html.push_str(&format!(" placeholder=\"{}\"", html_escape(placeholder)));
            }
            if let Some(ref val) = resolved_value {
                html.push_str(&format!(" value=\"{}\"", html_escape(val)));
            }
            if let Some(ref step) = props.step {
                html.push_str(&format!(" step=\"{}\"", html_escape(step)));
            }
            if let Some(ref list_id) = props.list {
                html.push_str(&format!(" list=\"{}\"", html_escape(list_id)));
            }
            if props.required == Some(true) {
                html.push_str(" required");
            }
            if props.disabled == Some(true) {
                html.push_str(" disabled");
            }
            // FIX-03 / A11Y-08: inline validation ARIA for standard inputs
            if has_error {
                html.push_str(&format!(
                    " aria-invalid=\"true\" aria-describedby=\"err-{}\"",
                    html_escape(&props.field)
                ));
            }
            html.push('>');
            if let Some(ref list_id) = props.list {
                if let Some(arr) = data.get(list_id).and_then(|v| v.as_array()) {
                    html.push_str(&format!("<datalist id=\"{}\">", html_escape(list_id)));
                    for opt in arr {
                        if let Some(s) = opt.as_str() {
                            html.push_str(&format!("<option value=\"{}\">", html_escape(s)));
                        }
                    }
                    html.push_str("</datalist>");
                }
            }
        }
    }

    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"text-sm text-text-muted\">{}</p>",
            html_escape(desc)
        ));
    }

    if let Some(ref error) = props.error {
        // FIX-03 / A11Y-08: id on error paragraph links to aria-describedby on input
        html.push_str(&format!(
            "<p id=\"err-{}\" class=\"text-sm text-destructive\">{}</p>",
            html_escape(&props.field),
            html_escape(error)
        ));
    }
    html.push_str("</div>");
    html
}

fn render_select(props: &SelectProps, data: &Value) -> String {
    // Resolve the effective selected value.
    let selected_value = if let Some(ref dv) = props.default_value {
        Some(dv.clone())
    } else if let Some(ref dp) = props.data_path {
        resolve_path_string(data, dp)
    } else {
        None
    };

    let has_error = props.error.is_some();
    let border_class = if has_error {
        "border-destructive"
    } else {
        "border-border"
    };
    let focus_ring_class = if has_error {
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive focus-visible:ring-offset-2"
    } else {
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
    };

    let mut html = String::from("<div class=\"space-y-1\">");
    html.push_str(&format!(
        "<label class=\"block text-sm font-medium text-text\" for=\"{}\">{}</label>",
        html_escape(&props.field),
        html_escape(&props.label)
    ));

    html.push_str("<div class=\"relative\">");
    html.push_str(&format!(
        "<select id=\"{}\" name=\"{}\" class=\"block w-full appearance-none bg-background rounded-md border {} pr-10 px-3 py-2 text-sm shadow-sm transition-colors duration-150 motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed {}\"",
        html_escape(&props.field),
        html_escape(&props.field),
        border_class,
        focus_ring_class
    ));
    if props.required == Some(true) {
        html.push_str(" required");
    }
    if props.disabled == Some(true) {
        html.push_str(" disabled");
    }
    // FIX-03: inline validation ARIA for select
    if has_error {
        html.push_str(&format!(
            " aria-invalid=\"true\" aria-describedby=\"err-{}\"",
            html_escape(&props.field)
        ));
    }
    html.push('>');

    if let Some(ref placeholder) = props.placeholder {
        html.push_str(&format!(
            "<option value=\"\">{}</option>",
            html_escape(placeholder)
        ));
    }

    for opt in &props.options {
        let is_selected = selected_value.as_deref() == Some(&opt.value);
        let selected_attr = if is_selected { " selected" } else { "" };
        html.push_str(&format!(
            "<option value=\"{}\"{}>{}</option>",
            html_escape(&opt.value),
            selected_attr,
            html_escape(&opt.label)
        ));
    }

    html.push_str("</select>");
    html.push_str(concat!(
        "<span class=\"pointer-events-none absolute inset-y-0 right-3 flex items-center\" aria-hidden=\"true\">",
        "<svg class=\"h-4 w-4 text-text-muted\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
        "<path fill-rule=\"evenodd\" d=\"M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z\" clip-rule=\"evenodd\"/>",
        "</svg></span>"
    ));
    html.push_str("</div>");

    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"text-sm text-text-muted\">{}</p>",
            html_escape(desc)
        ));
    }

    if let Some(ref error) = props.error {
        // FIX-03: id on error paragraph links to aria-describedby on select
        html.push_str(&format!(
            "<p id=\"err-{}\" class=\"text-sm text-destructive\">{}</p>",
            html_escape(&props.field),
            html_escape(error)
        ));
    }
    html.push_str("</div>");
    html
}

fn render_checkbox(props: &CheckboxProps, data: &Value) -> String {
    // Resolve checked state: explicit `checked` prop wins, else data_path truthy.
    let is_checked = if let Some(c) = props.checked {
        c
    } else if let Some(ref dp) = props.data_path {
        resolve_path(data, dp)
            .map(|v| match v {
                Value::Bool(b) => *b,
                Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
                Value::String(s) => !s.is_empty() && s != "false" && s != "0",
                Value::Null => false,
                _ => true,
            })
            .unwrap_or(false)
    } else {
        false
    };

    let value_attr = props.value.as_deref().unwrap_or("1");
    // When value is set, use it as part of the id to make each checkbox unique.
    let checkbox_id = match &props.value {
        Some(v) => format!("{}_{}", props.field, v),
        None => props.field.clone(),
    };

    let mut html = String::from("<div class=\"space-y-1\">");
    html.push_str("<div class=\"flex items-center gap-2\">");
    html.push_str(&format!(
        "<input type=\"checkbox\" id=\"{}\" name=\"{}\" value=\"{}\" class=\"h-4 w-4 rounded-sm border-border text-primary transition-colors duration-150 motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\"",
        html_escape(&checkbox_id),
        html_escape(&props.field),
        html_escape(value_attr)
    ));
    if is_checked {
        html.push_str(" checked");
    }
    if props.required == Some(true) {
        html.push_str(" required");
    }
    if props.disabled == Some(true) {
        html.push_str(" disabled");
    }
    html.push('>');
    html.push_str(&format!(
        "<label class=\"text-sm font-medium text-text\" for=\"{}\">{}</label>",
        html_escape(&checkbox_id),
        html_escape(&props.label)
    ));
    html.push_str("</div>");

    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"ml-6 text-sm text-text-muted\">{}</p>",
            html_escape(desc)
        ));
    }

    if let Some(ref error) = props.error {
        html.push_str(&format!(
            "<p class=\"ml-6 text-sm text-destructive\">{}</p>",
            html_escape(error)
        ));
    }
    html.push_str("</div>");
    html
}

fn render_switch(props: &SwitchProps, data: &Value) -> String {
    // Resolve checked state: explicit `checked` prop wins, else data_path truthy.
    let is_checked = if let Some(c) = props.checked {
        c
    } else if let Some(ref dp) = props.data_path {
        resolve_path(data, dp)
            .map(|v| match v {
                Value::Bool(b) => *b,
                Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
                Value::String(s) => !s.is_empty() && s != "false" && s != "0",
                Value::Null => false,
                _ => true,
            })
            .unwrap_or(false)
    } else {
        false
    };

    let auto_submit = props.action.is_some();
    let onchange = if auto_submit {
        " onchange=\"this.closest('form').submit()\""
    } else {
        ""
    };

    let mut html = String::new();

    // Wrap in a minimal form when an action is provided.
    if let Some(ref action) = props.action {
        let action_url = action.url.as_deref().unwrap_or("#");
        let (form_method, needs_spoofing) = match action.method {
            HttpMethod::Get => ("get", false),
            HttpMethod::Post => ("post", false),
            HttpMethod::Put | HttpMethod::Patch | HttpMethod::Delete => ("post", true),
        };
        html.push_str(&format!(
            "<form action=\"{}\" method=\"{}\">",
            html_escape(action_url),
            form_method
        ));
        if needs_spoofing {
            let method_value = match action.method {
                HttpMethod::Put => "PUT",
                HttpMethod::Patch => "PATCH",
                HttpMethod::Delete => "DELETE",
                _ => unreachable!(),
            };
            html.push_str(&format!(
                "<input type=\"hidden\" name=\"_method\" value=\"{method_value}\">"
            ));
        }
    }

    html.push_str("<div class=\"space-y-1\">");
    html.push_str("<div class=\"flex items-center justify-between\">");

    // Left side: label + description.
    html.push_str("<div>");
    html.push_str(&format!(
        "<label class=\"text-sm font-medium text-text\" for=\"{}\">{}</label>",
        html_escape(&props.field),
        html_escape(&props.label)
    ));
    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"text-sm text-text-muted\">{}</p>",
            html_escape(desc)
        ));
    }
    html.push_str("</div>");

    // Right side: toggle.
    html.push_str("<label class=\"relative inline-flex items-center cursor-pointer\">");
    let aria_checked = if is_checked { "true" } else { "false" };
    html.push_str(&format!(
        "<input type=\"checkbox\" id=\"{}\" name=\"{}\" value=\"1\" role=\"switch\" aria-checked=\"{}\" class=\"sr-only peer\"{}",
        html_escape(&props.field),
        html_escape(&props.field),
        aria_checked,
        onchange,
    ));
    if is_checked {
        html.push_str(" checked");
    }
    if props.required == Some(true) {
        html.push_str(" required");
    }
    if props.disabled == Some(true) {
        html.push_str(" disabled");
    }
    html.push('>');
    html.push_str("<div class=\"w-11 h-6 bg-border rounded-full peer peer-checked:bg-primary peer-focus:ring-2 peer-focus:ring-primary/30 after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-background after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full\"></div>");
    html.push_str("</label>");
    html.push_str("</div>");

    if let Some(ref error) = props.error {
        html.push_str(&format!(
            "<p class=\"text-sm text-destructive\">{}</p>",
            html_escape(error)
        ));
    }
    html.push_str("</div>");

    if props.action.is_some() {
        html.push_str("</form>");
    }

    html
}

// ── Leaf component renderers ────────────────────────────────────────────

fn render_text(props: &TextProps) -> String {
    let content = html_escape(&props.content);
    match props.element {
        TextElement::P => format!("<p class=\"text-base leading-relaxed text-text\">{content}</p>"),
        TextElement::H1 => format!("<h1 class=\"text-3xl font-bold leading-tight tracking-tight text-text\">{content}</h1>"),
        TextElement::H2 => {
            format!("<h2 class=\"text-2xl font-semibold leading-tight tracking-tight text-text\">{content}</h2>")
        }
        TextElement::H3 => {
            format!("<h3 class=\"text-xl font-semibold leading-snug text-text\">{content}</h3>")
        }
        TextElement::Span => format!("<span class=\"text-base text-text\">{content}</span>"),
        TextElement::Div => format!("<div class=\"text-base leading-relaxed text-text\">{content}</div>"),
        TextElement::Section => {
            format!("<section class=\"text-base leading-relaxed text-text\">{content}</section>")
        }
    }
}

fn render_button(props: &ButtonProps) -> String {
    let base = "inline-flex items-center justify-center rounded-md font-medium transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2";

    let variant_classes = match props.variant {
        ButtonVariant::Default => "bg-primary text-primary-foreground hover:bg-primary/90",
        ButtonVariant::Secondary => "bg-secondary text-secondary-foreground hover:bg-secondary/90",
        ButtonVariant::Destructive => {
            "bg-destructive text-primary-foreground hover:bg-destructive/90"
        }
        ButtonVariant::Outline => "border border-border bg-background text-text hover:bg-surface",
        ButtonVariant::Ghost => "text-text hover:bg-surface",
        ButtonVariant::Link => "text-primary underline hover:text-primary/80",
    };

    let size_classes = match props.size {
        Size::Xs => "px-2 py-1 text-xs",
        Size::Sm => "px-3 py-1.5 text-sm",
        Size::Default => "px-4 py-2 text-sm",
        Size::Lg => "px-6 py-3 text-base",
    };

    let disabled_classes = if props.disabled == Some(true) {
        " opacity-50 cursor-not-allowed"
    } else {
        ""
    };

    let disabled_attr = if props.disabled == Some(true) {
        " disabled"
    } else {
        ""
    };

    let label = html_escape(&props.label);

    // Build icon + label content.
    let content = if let Some(ref icon) = props.icon {
        let icon_span = format!(
            "<span class=\"icon\" data-icon=\"{}\">{}</span>",
            html_escape(icon),
            html_escape(icon)
        );
        let position = props.icon_position.as_ref().cloned().unwrap_or_default();
        match position {
            IconPosition::Left => format!("{icon_span} {label}"),
            IconPosition::Right => format!("{label} {icon_span}"),
        }
    } else {
        label
    };

    let type_attr = match props.button_type.as_ref().unwrap_or(&ButtonType::Button) {
        ButtonType::Button => "button",
        ButtonType::Submit => "submit",
    };

    format!(
        "<button type=\"{type_attr}\" class=\"{base} {variant_classes} {size_classes}{disabled_classes}\"{disabled_attr}>{content}</button>"
    )
}

fn render_badge(props: &BadgeProps) -> String {
    let base = "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium";
    let variant_classes = match props.variant {
        BadgeVariant::Default => "bg-primary/10 text-primary",
        BadgeVariant::Secondary => "bg-secondary/10 text-secondary-foreground",
        BadgeVariant::Destructive => "bg-destructive/10 text-destructive",
        BadgeVariant::Outline => "border border-border text-text",
    };
    format!(
        "<span class=\"{} {}\">{}</span>",
        base,
        variant_classes,
        html_escape(&props.label)
    )
}

// ── CMP-01: Alert SVG icons ──────────────────────────────────────────────

const ICON_INFO: &str = concat!(
    "<span aria-hidden=\"true\" class=\"shrink-0\">",
    "<svg class=\"h-5 w-5\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
    "<path fill-rule=\"evenodd\" d=\"M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a.75.75 0 000 1.5h.253a.25.25 0 01.244.304l-.459 2.066A1.75 1.75 0 0010.747 15H11a.75.75 0 000-1.5h-.253a.25.25 0 01-.244-.304l.459-2.066A1.75 1.75 0 009.253 9H9z\" clip-rule=\"evenodd\"/>",
    "</svg></span>"
);

const ICON_SUCCESS: &str = concat!(
    "<span aria-hidden=\"true\" class=\"shrink-0\">",
    "<svg class=\"h-5 w-5\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
    "<path fill-rule=\"evenodd\" d=\"M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 10-1.06 1.061l2.5 2.5a.75.75 0 001.137-.089l4-5.5z\" clip-rule=\"evenodd\"/>",
    "</svg></span>"
);

const ICON_WARNING: &str = concat!(
    "<span aria-hidden=\"true\" class=\"shrink-0\">",
    "<svg class=\"h-5 w-5\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
    "<path fill-rule=\"evenodd\" d=\"M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495zM10 5a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 5zm0 9a1 1 0 100-2 1 1 0 000 2z\" clip-rule=\"evenodd\"/>",
    "</svg></span>"
);

const ICON_ERROR: &str = concat!(
    "<span aria-hidden=\"true\" class=\"shrink-0\">",
    "<svg class=\"h-5 w-5\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
    "<path fill-rule=\"evenodd\" d=\"M10 18a8 8 0 100-16 8 8 0 000 16zM8.28 7.22a.75.75 0 00-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 101.06 1.06L10 11.06l1.72 1.72a.75.75 0 101.06-1.06L11.06 10l1.72-1.72a.75.75 0 00-1.06-1.06L10 8.94 8.28 7.22z\" clip-rule=\"evenodd\"/>",
    "</svg></span>"
);

fn render_alert(props: &AlertProps) -> String {
    let variant_classes = match props.variant {
        AlertVariant::Info => "bg-primary/10 border-primary text-primary",
        AlertVariant::Success => "bg-success/10 border-success text-success",
        AlertVariant::Warning => "bg-warning/10 border-warning text-warning",
        AlertVariant::Error => "bg-destructive/10 border-destructive text-destructive",
    };
    let icon = match props.variant {
        AlertVariant::Info => ICON_INFO,
        AlertVariant::Success => ICON_SUCCESS,
        AlertVariant::Warning => ICON_WARNING,
        AlertVariant::Error => ICON_ERROR,
    };
    let mut html = format!(
        "<div role=\"alert\" class=\"rounded-md border p-4 flex items-start gap-3 {variant_classes}\">"
    );
    html.push_str(icon);
    html.push_str("<div>");
    if let Some(ref title) = props.title {
        html.push_str(&format!(
            "<h4 class=\"font-semibold mb-1\">{}</h4>",
            html_escape(title)
        ));
    }
    html.push_str(&format!("<p>{}</p>", html_escape(&props.message)));
    html.push_str("</div>");
    html.push_str("</div>");
    html
}

fn render_separator(props: &SeparatorProps) -> String {
    let orientation = props.orientation.as_ref().cloned().unwrap_or_default();
    match orientation {
        Orientation::Horizontal => "<hr class=\"my-4 border-border\">".to_string(),
        Orientation::Vertical => "<div class=\"mx-4 h-full w-px bg-border\"></div>".to_string(),
    }
}

fn render_progress(props: &ProgressProps) -> String {
    let max = props.max.unwrap_or(100) as f64;
    let pct = if max > 0.0 {
        ((props.value as f64 * 100.0 / max).round() as u8).min(100)
    } else {
        0
    };

    let mut html = String::from("<div class=\"w-full\">");
    if let Some(ref label) = props.label {
        html.push_str(&format!(
            "<div class=\"mb-1 text-sm text-text-muted\">{}</div>",
            html_escape(label)
        ));
    }
    html.push_str(&format!(
        "<div class=\"w-full rounded-full bg-border h-2.5\"><div class=\"rounded-full bg-primary h-2.5\" style=\"width: {pct}%\"></div></div>"
    ));
    html.push_str("</div>");
    html
}

fn render_avatar(props: &AvatarProps) -> String {
    let size = props.size.as_ref().cloned().unwrap_or_default();
    let size_classes = match size {
        Size::Xs => "h-6 w-6 text-xs",
        Size::Sm => "h-8 w-8 text-sm",
        Size::Default => "h-10 w-10 text-sm",
        Size::Lg => "h-12 w-12 text-base",
    };

    if let Some(ref src) = props.src {
        format!(
            "<img src=\"{}\" alt=\"{}\" class=\"rounded-full object-cover {}\">",
            html_escape(src),
            html_escape(&props.alt),
            size_classes
        )
    } else {
        let fallback_text = props.fallback.as_deref().unwrap_or_else(|| {
            // Use first characters of alt as fallback.
            &props.alt
        });
        // Take first two chars for initials.
        let initials: String = fallback_text.chars().take(2).collect();
        format!(
            "<span class=\"inline-flex items-center justify-center rounded-full bg-card text-text-muted {}\">{}</span>",
            size_classes,
            html_escape(&initials)
        )
    }
}

// ── CMP-02: Skeleton shimmer animation ──────────────────────────────────

const SHIMMER_CSS: &str = concat!(
    "<style>",
    "@keyframes ferro-shimmer{0%{background-position:-200% 0}100%{background-position:200% 0}}",
    ".ferro-shimmer{",
    "background:linear-gradient(90deg,var(--color-card,#f1f5f9) 25%,var(--color-border,#e2e8f0) 50%,var(--color-card,#f1f5f9) 75%);",
    "background-size:200% 100%;",
    "animation:ferro-shimmer 1.5s ease-in-out infinite;",
    "}",
    "</style>"
);

fn render_skeleton(props: &SkeletonProps) -> String {
    let width = props.width.as_deref().unwrap_or("100%");
    let height = props.height.as_deref().unwrap_or("1rem");
    let rounded = if props.rounded == Some(true) {
        "rounded-full"
    } else {
        "rounded-md"
    };
    format!("{SHIMMER_CSS}<div class=\"ferro-shimmer {rounded}\" style=\"width: {width}; height: {height}\"></div>")
}

// ── CMP-03: Breadcrumb SVG chevron separator ─────────────────────────────

const BREADCRUMB_SEP: &str = concat!(
    "<span aria-hidden=\"true\" class=\"text-text-muted\">",
    "<svg class=\"h-4 w-4\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
    "<path fill-rule=\"evenodd\" d=\"M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z\" clip-rule=\"evenodd\"/>",
    "</svg></span>"
);

fn render_breadcrumb(props: &BreadcrumbProps) -> String {
    let mut html =
        String::from("<nav class=\"flex items-center space-x-2 text-sm text-text-muted\">");
    let len = props.items.len();
    for (i, item) in props.items.iter().enumerate() {
        let is_last = i == len - 1;
        if is_last {
            html.push_str(&format!(
                "<span class=\"text-text font-medium\">{}</span>",
                html_escape(&item.label)
            ));
        } else if let Some(ref url) = item.url {
            html.push_str(&format!(
                "<a href=\"{}\" class=\"hover:text-text transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\">{}</a>",
                html_escape(url),
                html_escape(&item.label)
            ));
        } else {
            html.push_str(&format!("<span>{}</span>", html_escape(&item.label)));
        }
        if !is_last {
            html.push_str(BREADCRUMB_SEP);
        }
    }
    html.push_str("</nav>");
    html
}

fn render_pagination(props: &PaginationProps) -> String {
    if props.total == 0 || props.per_page == 0 {
        return String::new();
    }

    let total_pages = props.total.div_ceil(props.per_page);
    if total_pages <= 1 {
        return String::new();
    }

    let base_url = props.base_url.as_deref().unwrap_or("?");
    let current = props.current_page;

    let mut html = String::from("<nav class=\"flex items-center space-x-1\">");

    // Previous button.
    if current > 1 {
        html.push_str(&format!(
            "<a href=\"{}page={}\" class=\"px-3 py-1 rounded-md bg-background text-text hover:bg-surface transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\">&laquo;</a>",
            html_escape(base_url),
            current - 1
        ));
    }

    // Page numbers — show up to 7 with ellipsis.
    let pages = compute_page_range(current, total_pages);
    let mut prev_page = 0u32;
    for page in pages {
        if prev_page > 0 && page > prev_page + 1 {
            html.push_str("<span class=\"px-2 text-text-muted\">&hellip;</span>");
        }
        if page == current {
            html.push_str(&format!(
                "<span class=\"px-3 py-1 rounded-md bg-primary text-primary-foreground\">{page}</span>"
            ));
        } else {
            html.push_str(&format!(
                "<a href=\"{}page={}\" class=\"px-3 py-1 rounded-md bg-background text-text hover:bg-surface transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\">{}</a>",
                html_escape(base_url),
                page,
                page
            ));
        }
        prev_page = page;
    }

    // Next button.
    if current < total_pages {
        html.push_str(&format!(
            "<a href=\"{}page={}\" class=\"px-3 py-1 rounded-md bg-background text-text hover:bg-surface transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\">&raquo;</a>",
            html_escape(base_url),
            current + 1
        ));
    }

    html.push_str("</nav>");
    html
}

/// Compute which page numbers to display (up to 7 entries).
fn compute_page_range(current: u32, total: u32) -> Vec<u32> {
    if total <= 7 {
        return (1..=total).collect();
    }
    let mut pages = Vec::new();
    pages.push(1);
    let start = current.saturating_sub(1).max(2);
    let end = (current + 1).min(total - 1);
    for p in start..=end {
        if !pages.contains(&p) {
            pages.push(p);
        }
    }
    if !pages.contains(&total) {
        pages.push(total);
    }
    pages.sort();
    pages.dedup();
    pages
}

fn render_description_list(props: &DescriptionListProps) -> String {
    let columns = props.columns.unwrap_or(1);
    let mut html = format!("<dl class=\"grid grid-cols-{columns} gap-4\">");
    for item in &props.items {
        html.push_str(&format!(
            "<div><dt class=\"text-sm font-medium text-text-muted\">{}</dt><dd class=\"mt-1 text-sm text-text\">{}</dd></div>",
            html_escape(&item.label),
            html_escape(&item.value)
        ));
    }
    html.push_str("</dl>");
    html
}

// ── Layout component renderers ───────────────────────────────────────────

fn render_grid(props: &GridProps, data: &Value) -> String {
    let gap = match props.gap {
        GapSize::None => "gap-0",
        GapSize::Sm => "gap-2",
        GapSize::Md => "gap-4",
        GapSize::Lg => "gap-6",
        GapSize::Xl => "gap-8",
    };

    if props.scrollable == Some(true) {
        let mut html = format!("<div class=\"overflow-x-auto\"><div class=\"grid grid-flow-col auto-cols-[minmax(280px,1fr)] {gap}\">");
        for child in &props.children {
            html.push_str(&render_node(child, data));
        }
        html.push_str("</div></div>");
        return html;
    }

    let cols = props.columns.clamp(1, 12);
    let col_classes = match props.md_columns {
        Some(md) => format!("grid-cols-{cols} md:grid-cols-{}", md.clamp(1, 12)),
        None => format!("grid-cols-{cols}"),
    };
    let mut html = format!("<div class=\"grid w-full {col_classes} {gap}\">");
    for child in &props.children {
        html.push_str(&render_node(child, data));
    }
    html.push_str("</div>");
    html
}

// ── CMP-06: Collapsible SVG chevron ─────────────────────────────────────

const CHEVRON_DOWN: &str = concat!(
    "<svg class=\"h-4 w-4\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
    "<path fill-rule=\"evenodd\" d=\"M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z\" clip-rule=\"evenodd\"/>",
    "</svg>"
);

fn render_collapsible(props: &CollapsibleProps, data: &Value) -> String {
    let mut html = String::from("<details class=\"group\"");
    if props.expanded {
        html.push_str(" open");
    }
    html.push('>');
    let aria_expanded = if props.expanded { "true" } else { "false" };
    html.push_str(&format!(
        "<summary class=\"flex items-center justify-between cursor-pointer px-4 py-3 text-sm font-medium text-text bg-surface rounded-lg hover:bg-card\" aria-expanded=\"{}\">{}<span class=\"text-text-muted group-open:rotate-180 transition-transform\">{CHEVRON_DOWN}</span></summary>",
        aria_expanded,
        html_escape(&props.title)
    ));
    html.push_str("<div class=\"px-4 py-3 flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">");
    for child in &props.children {
        html.push_str(&render_node(child, data));
    }
    html.push_str("</div></details>");
    html
}

fn render_empty_state(props: &EmptyStateProps) -> String {
    let mut html = String::from(
        "<div class=\"flex flex-col items-center justify-center py-16 px-6 text-center\">"
    );
    html.push_str(&format!(
        "<h3 class=\"text-xl font-semibold text-text\">{}</h3>",
        html_escape(&props.title)
    ));
    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"mt-2 text-sm text-text-muted max-w-sm\">{}</p>",
            html_escape(desc)
        ));
    }
    if let Some(ref action) = props.action {
        let label = props.action_label.as_deref().unwrap_or("Action");
        let url = action.url.as_deref().unwrap_or("#");
        html.push_str(&format!(
            "<a href=\"{}\" class=\"mt-6 inline-flex items-center justify-center rounded-md \
             border border-border bg-card text-text px-4 py-2 text-sm font-medium \
             hover:bg-surface transition-colors\">{}</a>",
            html_escape(url),
            html_escape(label)
        ));
    }
    html.push_str("</div>");
    html
}

fn render_form_section(props: &FormSectionProps, data: &Value) -> String {
    let is_two_column = matches!(props.layout.as_ref(), Some(FormSectionLayout::TwoColumn));

    if is_two_column {
        // FIX-05: two-column layout — description left (2 cols), controls right (3 cols)
        let mut html = String::from("<fieldset class=\"md:grid md:grid-cols-5 md:gap-8\">");
        html.push_str(&format!(
            "<div class=\"md:col-span-2\"><legend class=\"text-base font-semibold text-text\">{}</legend>",
            html_escape(&props.title)
        ));
        if let Some(ref desc) = props.description {
            html.push_str(&format!(
                "<p class=\"text-sm text-text-muted mt-1\">{}</p>",
                html_escape(desc)
            ));
        }
        html.push_str("</div>");
        html.push_str("<div class=\"md:col-span-3 space-y-4 mt-4 md:mt-0\">");
        for child in &props.children {
            html.push_str(&render_node(child, data));
        }
        html.push_str("</div></fieldset>");
        html
    } else {
        // Stacked (default) behavior
        let mut html = String::from(
            "<fieldset class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">",
        );
        html.push_str(&format!(
            "<legend class=\"text-base font-semibold text-text\">{}</legend>",
            html_escape(&props.title)
        ));
        if let Some(ref desc) = props.description {
            html.push_str(&format!(
                "<p class=\"text-sm text-text-muted\">{}</p>",
                html_escape(desc)
            ));
        }
        html.push_str("<div class=\"space-y-4\">");
        for child in &props.children {
            html.push_str(&render_node(child, data));
        }
        html.push_str("</div></fieldset>");
        html
    }
}

// ── Dashboard component renderers ───────────────────────────────────────

fn render_stat_card(props: &StatCardProps) -> String {
    let mut html =
        String::from("<div class=\"bg-card rounded-lg shadow-sm p-4 border border-border\">");
    if let Some(ref icon) = props.icon {
        html.push_str(&format!("<span class=\"inline-block mb-2 w-6 h-6\">{icon}</span>"));
        // raw
    }
    html.push_str(&format!(
        "<p class=\"text-sm text-text-muted\">{}</p>",
        html_escape(&props.label)
    ));
    if let Some(ref sse) = props.sse_target {
        html.push_str(&format!(
            "<p class=\"text-2xl font-bold text-text\" data-sse-target=\"{}\" data-live-value>{}</p>",
            html_escape(sse),
            html_escape(&props.value)
        ));
    } else {
        html.push_str(&format!(
            "<p class=\"text-2xl font-bold text-text\">{}</p>",
            html_escape(&props.value)
        ));
    }
    if let Some(ref subtitle) = props.subtitle {
        html.push_str(&format!(
            "<p class=\"text-xs text-text-muted mt-1\">{}</p>",
            html_escape(subtitle)
        ));
    }
    html.push_str("</div>");
    html
}

fn render_checklist(props: &ChecklistProps) -> String {
    let mut html =
        String::from("<div class=\"bg-card rounded-lg shadow-sm p-4 border border-border\">");
    html.push_str("<div class=\"flex items-center justify-between mb-3\">");
    html.push_str(&format!(
        "<h3 class=\"text-sm font-semibold leading-snug text-text\">{}</h3>",
        html_escape(&props.title)
    ));
    if props.dismissible {
        let dismiss_label = props.dismiss_label.as_deref().unwrap_or("Dismiss");
        html.push_str(&format!(
            "<button type=\"button\" class=\"text-xs font-medium text-text hover:text-primary\" data-dismissible>{}</button>",
            html_escape(dismiss_label)
        ));
    }
    html.push_str("</div>");
    if let Some(ref key) = props.data_key {
        html.push_str(&format!(
            "<div data-checklist-key=\"{}\">",
            html_escape(key)
        ));
    } else {
        html.push_str("<div>");
    }
    if props.dismissible {
        html.push_str("<ul data-dismissible class=\"space-y-2\">");
    } else {
        html.push_str("<ul class=\"space-y-2\">");
    }
    for item in &props.items {
        html.push_str("<li class=\"flex items-center gap-2\">");
        if item.checked {
            html.push_str("<input type=\"checkbox\" checked class=\"h-4 w-4 rounded-sm border-border text-primary\">");
        } else {
            html.push_str(
                "<input type=\"checkbox\" class=\"h-4 w-4 rounded-sm border-border text-primary\">",
            );
        }
        let label_class = if item.checked {
            "text-sm line-through text-text-muted"
        } else {
            "text-sm text-text"
        };
        if let Some(ref href) = item.href {
            html.push_str(&format!(
                "<a href=\"{}\" class=\"{}\">{}</a>",
                html_escape(href),
                label_class,
                html_escape(&item.label)
            ));
        } else {
            html.push_str(&format!(
                "<span class=\"{}\">{}</span>",
                label_class,
                html_escape(&item.label)
            ));
        }
        html.push_str("</li>");
    }
    html.push_str("</ul></div></div>");
    html
}

fn render_toast(props: &ToastProps) -> String {
    let variant_classes = match props.variant {
        ToastVariant::Info => "bg-primary/10 border-primary text-primary",
        ToastVariant::Success => "bg-success/10 border-success text-success",
        ToastVariant::Warning => "bg-warning/10 border-warning text-warning",
        ToastVariant::Error => "bg-destructive/10 border-destructive text-destructive",
    };
    let variant_str = match props.variant {
        ToastVariant::Info => "info",
        ToastVariant::Success => "success",
        ToastVariant::Warning => "warning",
        ToastVariant::Error => "error",
    };
    let timeout = props.timeout.unwrap_or(5);
    let mut html = format!(
        "<div class=\"fixed top-4 right-4 z-50 rounded-md border p-4 shadow-lg {variant_classes}\" data-toast-variant=\"{variant_str}\" data-toast-timeout=\"{timeout}\"",
    );
    if props.dismissible {
        html.push_str(" data-toast-dismissible");
    }
    html.push('>');
    html.push_str("<div class=\"flex items-start gap-3\">");
    html.push_str(&format!(
        "<p class=\"text-sm\">{}</p>",
        html_escape(&props.message)
    ));
    if props.dismissible {
        html.push_str(
            "<button type=\"button\" class=\"ml-auto text-current opacity-70 hover:opacity-100\">&times;</button>",
        );
    }
    html.push_str("</div></div>");
    html
}

// ── CMP-05: Bell SVG icon ────────────────────────────────────────────────

const BELL_SVG: &str = concat!(
    "<svg class=\"h-5 w-5\" fill=\"none\" stroke=\"currentColor\" viewBox=\"0 0 24 24\">",
    "<path stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" ",
    "d=\"M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9\"/>",
    "</svg>"
);

fn render_notification_dropdown(props: &NotificationDropdownProps) -> String {
    let unread_count = props.notifications.iter().filter(|n| !n.read).count();
    let mut html = String::from("<div class=\"relative\" data-notification-dropdown>");
    // Bell icon button with badge.
    html.push_str(&format!(
        "<button type=\"button\" class=\"relative p-2 text-text-muted hover:text-text\" data-notification-count=\"{unread_count}\">"
    ));
    html.push_str(BELL_SVG);
    if unread_count > 0 {
        html.push_str(&format!(
            "<span class=\"absolute top-0 right-0 inline-flex items-center justify-center h-4 w-4 text-xs font-bold text-primary-foreground bg-destructive rounded-full\">{unread_count}</span>"
        ));
    }
    html.push_str("</button>");
    // Dropdown panel.
    html.push_str(
        "<div class=\"hidden absolute right-0 mt-2 w-80 bg-card rounded-lg shadow-lg border border-border z-50\" data-notification-panel>",
    );
    if props.notifications.is_empty() {
        let empty = props.empty_text.as_deref().unwrap_or("No notifications");
        html.push_str(&format!(
            "<p class=\"p-4 text-sm text-text-muted\">{}</p>",
            html_escape(empty)
        ));
    } else {
        html.push_str("<ul class=\"divide-y divide-border\">");
        for item in &props.notifications {
            html.push_str("<li class=\"flex items-start gap-3 p-3\">");
            if let Some(ref icon) = item.icon {
                html.push_str(&format!(
                    "<span class=\"text-lg shrink-0\">{}</span>",
                    html_escape(icon)
                ));
            }
            html.push_str("<div class=\"flex-1 min-w-0\">");
            if let Some(ref url) = item.action_url {
                html.push_str(&format!(
                    "<a href=\"{}\" class=\"text-sm text-text hover:underline\">{}</a>",
                    html_escape(url),
                    html_escape(&item.text)
                ));
            } else {
                html.push_str(&format!(
                    "<p class=\"text-sm text-text\">{}</p>",
                    html_escape(&item.text)
                ));
            }
            if let Some(ref ts) = item.timestamp {
                html.push_str(&format!(
                    "<p class=\"text-xs text-text-muted mt-0.5\">{}</p>",
                    html_escape(ts)
                ));
            }
            html.push_str("</div>");
            if !item.read {
                html.push_str(
                    "<span class=\"h-2 w-2 mt-1 shrink-0 rounded-full bg-primary\"></span>",
                );
            }
            html.push_str("</li>");
        }
        html.push_str("</ul>");
    }
    html.push_str("</div></div>");
    html
}

fn render_sidebar(props: &SidebarProps) -> String {
    let mut html =
        String::from("<aside class=\"flex flex-col h-full bg-background border-r border-border\">");
    // Fixed top items.
    if !props.fixed_top.is_empty() {
        html.push_str("<nav class=\"p-4 space-y-1\">");
        for item in &props.fixed_top {
            html.push_str(&render_sidebar_nav_item(item));
        }
        html.push_str("</nav>");
    }
    // Groups.
    if !props.groups.is_empty() {
        html.push_str("<div class=\"flex-1 overflow-y-auto p-4 flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">");
        for group in &props.groups {
            html.push_str("<div data-sidebar-group");
            if group.collapsed {
                html.push_str(" data-collapsed");
            }
            html.push('>');
            html.push_str(&format!(
                "<p class=\"px-2 py-1 text-xs font-semibold text-text-muted uppercase tracking-wider\">{}</p>",
                html_escape(&group.label)
            ));
            html.push_str("<nav class=\"space-y-1\">");
            for item in &group.items {
                html.push_str(&render_sidebar_nav_item(item));
            }
            html.push_str("</nav></div>");
        }
        html.push_str("</div>");
    }
    // Fixed bottom items.
    if !props.fixed_bottom.is_empty() {
        html.push_str("<nav class=\"p-4 space-y-1 border-t border-border\">");
        for item in &props.fixed_bottom {
            html.push_str(&render_sidebar_nav_item(item));
        }
        html.push_str("</nav>");
    }
    html.push_str("</aside>");
    html
}

fn render_sidebar_nav_item(item: &crate::component::SidebarNavItem) -> String {
    let classes = if item.active {
        "flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium bg-card text-primary transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
    } else {
        "flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium text-text-muted hover:text-text hover:bg-surface transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
    };
    let mut html = format!(
        "<a href=\"{}\" class=\"{}\">",
        html_escape(&item.href),
        classes
    );
    if let Some(ref icon) = item.icon {
        html.push_str(&format!(
            "<span class=\"inline-flex items-center justify-center w-5 h-5 shrink-0\">{icon}</span>" // raw SVG
        ));
    }
    html.push_str(&format!("{}</a>", html_escape(&item.label)));
    html
}

fn render_header(props: &HeaderProps) -> String {
    let mut html = String::from(
        "<header class=\"flex items-center justify-between px-6 py-4 bg-background border-b border-border\">",
    );
    // Business name.
    html.push_str(&format!(
        "<span class=\"text-lg font-semibold text-text\">{}</span>",
        html_escape(&props.business_name)
    ));
    html.push_str("<div class=\"flex items-center gap-4\">");
    // Notification bell with count badge.
    if let Some(count) = props.notification_count {
        if count > 0 {
            html.push_str(&format!(
                "<div class=\"relative\"><span class=\"text-text-muted\">{BELL_SVG}</span><span class=\"absolute top-0 right-0 inline-flex items-center justify-center h-4 w-4 text-xs font-bold text-primary-foreground bg-destructive rounded-full\" data-notification-count=\"{count}\">{count}</span></div>"
            ));
        } else {
            html.push_str(&format!(
                "<span class=\"text-text-muted\" data-notification-count=\"{count}\">{BELL_SVG}</span>"
            ));
        }
    }
    // User section.
    html.push_str("<div class=\"flex items-center gap-2\">");
    if let Some(ref avatar) = props.user_avatar {
        html.push_str(&format!(
            "<img src=\"{}\" alt=\"User avatar\" class=\"h-8 w-8 rounded-full object-cover\">",
            html_escape(avatar)
        ));
    } else if let Some(ref name) = props.user_name {
        let initials: String = name
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect();
        html.push_str(&format!(
            "<span class=\"inline-flex items-center justify-center h-8 w-8 rounded-full bg-card text-text-muted text-sm font-medium\">{}</span>",
            html_escape(&initials)
        ));
        html.push_str(&format!(
            "<span class=\"text-sm text-text\">{}</span>",
            html_escape(name)
        ));
    }
    if let Some(ref logout) = props.logout_url {
        html.push_str(&format!(
            "<a href=\"{}\" class=\"text-sm text-text-muted hover:text-text\">Logout</a>",
            html_escape(logout)
        ));
    }
    html.push_str("</div></div></header>");
    html
}

// ── HTML escaping ───────────────────────────────────────────────────────

/// Escape special HTML characters to prevent XSS.
pub(crate) fn html_escape(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, HttpMethod};
    use crate::component::*;
    use serde_json::json;

    // ── Helpers ─────────────────────────────────────────────────────────

    fn text_node(key: &str, content: &str, element: TextElement) -> ComponentNode {
        ComponentNode {
            key: key.to_string(),
            component: Component::Text(TextProps {
                content: content.to_string(),
                element,
            }),
            action: None,
            visibility: None,
        }
    }

    fn button_node(key: &str, label: &str, variant: ButtonVariant, size: Size) -> ComponentNode {
        ComponentNode {
            key: key.to_string(),
            component: Component::Button(ButtonProps {
                label: label.to_string(),
                variant,
                size,
                disabled: None,
                icon: None,
                icon_position: None,
                button_type: None,
            }),
            action: None,
            visibility: None,
        }
    }

    fn make_action(handler: &str, method: HttpMethod) -> Action {
        Action {
            handler: handler.to_string(),
            url: None,
            method,
            confirm: None,
            on_success: None,
            on_error: None,
        }
    }

    fn make_action_with_url(handler: &str, method: HttpMethod, url: &str) -> Action {
        Action {
            handler: handler.to_string(),
            url: Some(url.to_string()),
            method,
            confirm: None,
            on_success: None,
            on_error: None,
        }
    }

    // ── 1. render_to_html produces wrapper div ──────────────────────────

    #[test]
    fn render_empty_view_produces_wrapper_div() {
        let view = JsonUiView::new();
        let html = render_to_html(&view, &json!({}));
        assert_eq!(html, "<div class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\"></div>");
    }

    #[test]
    fn render_view_with_component_wraps_in_div() {
        let view = JsonUiView::new().component(text_node("t", "Hello", TextElement::P));
        let html = render_to_html(&view, &json!({}));
        assert!(html.starts_with(
            "<div class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">"
        ));
        assert!(html.ends_with("</div>"));
        assert!(html.contains("<p class=\"text-base leading-relaxed text-text\">Hello</p>"));
    }

    // ── 2. Text variants ────────────────────────────────────────────────

    #[test]
    fn text_p_variant() {
        let view = JsonUiView::new().component(text_node("t", "Paragraph", TextElement::P));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<p class=\"text-base leading-relaxed text-text\">Paragraph</p>"));
    }

    #[test]
    fn text_h1_variant() {
        let view = JsonUiView::new().component(text_node("t", "Title", TextElement::H1));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains(
            "<h1 class=\"text-3xl font-bold leading-tight tracking-tight text-text\">Title</h1>"
        ));
    }

    #[test]
    fn text_h2_variant() {
        let view = JsonUiView::new().component(text_node("t", "Subtitle", TextElement::H2));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains(
            "<h2 class=\"text-2xl font-semibold leading-tight tracking-tight text-text\">Subtitle</h2>"
        ));
    }

    #[test]
    fn text_h3_variant() {
        let view = JsonUiView::new().component(text_node("t", "Section", TextElement::H3));
        let html = render_to_html(&view, &json!({}));
        assert!(html
            .contains("<h3 class=\"text-xl font-semibold leading-snug text-text\">Section</h3>"));
    }

    #[test]
    fn text_span_variant() {
        let view = JsonUiView::new().component(text_node("t", "Inline", TextElement::Span));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<span class=\"text-base text-text\">Inline</span>"));
    }

    // ── 3. Button variants ──────────────────────────────────────────────

    #[test]
    fn button_default_variant() {
        let view = JsonUiView::new().component(button_node(
            "b",
            "Click",
            ButtonVariant::Default,
            Size::Default,
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("bg-primary text-primary-foreground hover:bg-primary/90"));
        assert!(html.contains(">Click</button>"));
    }

    #[test]
    fn button_secondary_variant() {
        let view = JsonUiView::new().component(button_node(
            "b",
            "Click",
            ButtonVariant::Secondary,
            Size::Default,
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("bg-secondary text-secondary-foreground hover:bg-secondary/90"));
    }

    #[test]
    fn button_destructive_variant() {
        let view = JsonUiView::new().component(button_node(
            "b",
            "Delete",
            ButtonVariant::Destructive,
            Size::Default,
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("bg-destructive text-primary-foreground hover:bg-destructive/90"));
    }

    #[test]
    fn button_outline_variant() {
        let view = JsonUiView::new().component(button_node(
            "b",
            "Click",
            ButtonVariant::Outline,
            Size::Default,
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("border border-border bg-background text-text hover:bg-surface"));
    }

    #[test]
    fn button_ghost_variant() {
        let view = JsonUiView::new().component(button_node(
            "b",
            "Click",
            ButtonVariant::Ghost,
            Size::Default,
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("text-text hover:bg-surface"));
    }

    #[test]
    fn button_link_variant() {
        let view = JsonUiView::new().component(button_node(
            "b",
            "Click",
            ButtonVariant::Link,
            Size::Default,
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("text-primary underline hover:text-primary/80"));
    }

    #[test]
    fn button_disabled_state() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "b".to_string(),
            component: Component::Button(ButtonProps {
                label: "Disabled".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: Some(true),
                icon: None,
                icon_position: None,
                        button_type: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("opacity-50 cursor-not-allowed"));
        assert!(html.contains(" disabled"));
    }

    #[test]
    fn button_with_icon_left() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "b".to_string(),
            component: Component::Button(ButtonProps {
                label: "Save".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: None,
                icon: Some("save".to_string()),
                icon_position: Some(IconPosition::Left),
                        button_type: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("data-icon=\"save\""));
        // Icon span comes before label.
        let icon_pos = html.find("data-icon").unwrap();
        let label_pos = html.find("Save").unwrap();
        assert!(icon_pos < label_pos);
    }

    #[test]
    fn button_with_icon_right() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "b".to_string(),
            component: Component::Button(ButtonProps {
                label: "Next".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: None,
                icon: Some("arrow-right".to_string()),
                icon_position: Some(IconPosition::Right),
                        button_type: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("data-icon=\"arrow-right\""));
        // Label comes before icon span.
        let label_pos = html.find("Next").unwrap();
        let icon_pos = html.find("data-icon").unwrap();
        assert!(label_pos < icon_pos);
    }

    // ── 4. Button sizes ─────────────────────────────────────────────────

    #[test]
    fn button_size_xs() {
        let view =
            JsonUiView::new().component(button_node("b", "X", ButtonVariant::Default, Size::Xs));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("px-2 py-1 text-xs"));
    }

    #[test]
    fn button_size_sm() {
        let view =
            JsonUiView::new().component(button_node("b", "S", ButtonVariant::Default, Size::Sm));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("px-3 py-1.5 text-sm"));
    }

    #[test]
    fn button_size_default() {
        let view = JsonUiView::new().component(button_node(
            "b",
            "D",
            ButtonVariant::Default,
            Size::Default,
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("px-4 py-2 text-sm"));
    }

    #[test]
    fn button_size_lg() {
        let view =
            JsonUiView::new().component(button_node("b", "L", ButtonVariant::Default, Size::Lg));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("px-6 py-3 text-base"));
    }

    // ── 5. Badge variants ───────────────────────────────────────────────

    #[test]
    fn badge_default_variant() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "bg".to_string(),
            component: Component::Badge(BadgeProps {
                label: "New".to_string(),
                variant: BadgeVariant::Default,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("bg-primary/10 text-primary"));
        assert!(html.contains(">New</span>"));
    }

    #[test]
    fn badge_secondary_variant() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "bg".to_string(),
            component: Component::Badge(BadgeProps {
                label: "Draft".to_string(),
                variant: BadgeVariant::Secondary,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("bg-secondary/10 text-secondary-foreground"));
    }

    #[test]
    fn badge_destructive_variant() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "bg".to_string(),
            component: Component::Badge(BadgeProps {
                label: "Deleted".to_string(),
                variant: BadgeVariant::Destructive,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("bg-destructive/10 text-destructive"));
    }

    #[test]
    fn badge_outline_variant() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "bg".to_string(),
            component: Component::Badge(BadgeProps {
                label: "Info".to_string(),
                variant: BadgeVariant::Outline,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("border border-border text-text"));
    }

    #[test]
    fn badge_has_base_classes() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "bg".to_string(),
            component: Component::Badge(BadgeProps {
                label: "Test".to_string(),
                variant: BadgeVariant::Default,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html
            .contains("inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium"));
    }

    // ── 6. Alert variants ───────────────────────────────────────────────

    #[test]
    fn alert_info_variant() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "a".to_string(),
            component: Component::Alert(AlertProps {
                message: "Info message".to_string(),
                variant: AlertVariant::Info,
                title: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("bg-primary/10 border-primary text-primary"));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("<p>Info message</p>"));
    }

    #[test]
    fn alert_success_variant() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "a".to_string(),
            component: Component::Alert(AlertProps {
                message: "Done".to_string(),
                variant: AlertVariant::Success,
                title: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("bg-success/10 border-success text-success"));
    }

    #[test]
    fn alert_warning_variant() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "a".to_string(),
            component: Component::Alert(AlertProps {
                message: "Careful".to_string(),
                variant: AlertVariant::Warning,
                title: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("bg-warning/10 border-warning text-warning"));
    }

    #[test]
    fn alert_error_variant() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "a".to_string(),
            component: Component::Alert(AlertProps {
                message: "Failed".to_string(),
                variant: AlertVariant::Error,
                title: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("bg-destructive/10 border-destructive text-destructive"));
    }

    #[test]
    fn alert_with_title() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "a".to_string(),
            component: Component::Alert(AlertProps {
                message: "Details here".to_string(),
                variant: AlertVariant::Warning,
                title: Some("Warning".to_string()),
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<h4 class=\"font-semibold mb-1\">Warning</h4>"));
        assert!(html.contains("<p>Details here</p>"));
    }

    #[test]
    fn alert_without_title() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "a".to_string(),
            component: Component::Alert(AlertProps {
                message: "No title".to_string(),
                variant: AlertVariant::Info,
                title: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(!html.contains("<h4"));
    }

    // ── 7. Separator orientations ───────────────────────────────────────

    #[test]
    fn separator_horizontal() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "s".to_string(),
            component: Component::Separator(SeparatorProps {
                orientation: Some(Orientation::Horizontal),
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<hr class=\"my-4 border-border\">"));
    }

    #[test]
    fn separator_vertical() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "s".to_string(),
            component: Component::Separator(SeparatorProps {
                orientation: Some(Orientation::Vertical),
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<div class=\"mx-4 h-full w-px bg-border\"></div>"));
    }

    #[test]
    fn separator_default_is_horizontal() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "s".to_string(),
            component: Component::Separator(SeparatorProps { orientation: None }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<hr"));
    }

    // ── 8. Progress ─────────────────────────────────────────────────────

    #[test]
    fn progress_renders_bar() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "p".to_string(),
            component: Component::Progress(ProgressProps {
                value: 50,
                max: None,
                label: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("style=\"width: 50%\""));
        assert!(html.contains("bg-primary h-2.5"));
    }

    #[test]
    fn progress_with_label() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "p".to_string(),
            component: Component::Progress(ProgressProps {
                value: 75,
                max: None,
                label: Some("Uploading...".to_string()),
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("Uploading..."));
        assert!(html.contains("text-sm text-text-muted"));
    }

    #[test]
    fn progress_with_custom_max() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "p".to_string(),
            component: Component::Progress(ProgressProps {
                value: 25,
                max: Some(50),
                label: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        // 25/50 = 50%
        assert!(html.contains("style=\"width: 50%\""));
    }

    // ── 9. Avatar ───────────────────────────────────────────────────────

    #[test]
    fn avatar_with_src() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "av".to_string(),
            component: Component::Avatar(AvatarProps {
                src: Some("/img/user.jpg".to_string()),
                alt: "User".to_string(),
                fallback: None,
                size: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<img"));
        assert!(html.contains("src=\"/img/user.jpg\""));
        assert!(html.contains("alt=\"User\""));
        assert!(html.contains("rounded-full object-cover"));
    }

    #[test]
    fn avatar_without_src_uses_fallback() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "av".to_string(),
            component: Component::Avatar(AvatarProps {
                src: None,
                alt: "John Doe".to_string(),
                fallback: Some("JD".to_string()),
                size: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(!html.contains("<img"));
        assert!(html.contains("<span"));
        assert!(html.contains("bg-card text-text-muted"));
        assert!(html.contains(">JD</span>"));
    }

    #[test]
    fn avatar_without_src_or_fallback_uses_alt_initials() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "av".to_string(),
            component: Component::Avatar(AvatarProps {
                src: None,
                alt: "Alice".to_string(),
                fallback: None,
                size: Some(Size::Lg),
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains(">Al</span>"));
        assert!(html.contains("h-12 w-12 text-base"));
    }

    // ── 10. Skeleton ────────────────────────────────────────────────────

    #[test]
    fn skeleton_default() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "sk".to_string(),
            component: Component::Skeleton(SkeletonProps {
                width: None,
                height: None,
                rounded: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("ferro-shimmer"));
        assert!(html.contains("rounded-md"));
        assert!(html.contains("width: 100%"));
        assert!(html.contains("height: 1rem"));
    }

    #[test]
    fn skeleton_custom_dimensions() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "sk".to_string(),
            component: Component::Skeleton(SkeletonProps {
                width: Some("200px".to_string()),
                height: Some("40px".to_string()),
                rounded: Some(true),
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("rounded-full"));
        assert!(html.contains("width: 200px"));
        assert!(html.contains("height: 40px"));
    }

    // ── 11. Breadcrumb ──────────────────────────────────────────────────

    #[test]
    fn breadcrumb_items_with_links() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "bc".to_string(),
            component: Component::Breadcrumb(BreadcrumbProps {
                items: vec![
                    BreadcrumbItem {
                        label: "Home".to_string(),
                        url: Some("/".to_string()),
                    },
                    BreadcrumbItem {
                        label: "Users".to_string(),
                        url: Some("/users".to_string()),
                    },
                    BreadcrumbItem {
                        label: "Edit".to_string(),
                        url: None,
                    },
                ],
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<nav"));
        assert!(
            html.contains("<a href=\"/\""),
            "breadcrumb Home link should exist"
        );
        assert!(
            html.contains(">Home</a>"),
            "breadcrumb Home label should exist"
        );
        assert!(
            html.contains("<a href=\"/users\""),
            "breadcrumb Users link should exist"
        );
        assert!(
            html.contains(">Users</a>"),
            "breadcrumb Users label should exist"
        );
        // Last item is plain span, not a link.
        assert!(html.contains("<span class=\"text-text font-medium\">Edit</span>"));
        // Separators between items — SVG chevrons.
        assert!(html.contains("<svg"));
    }

    #[test]
    fn breadcrumb_single_item() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "bc".to_string(),
            component: Component::Breadcrumb(BreadcrumbProps {
                items: vec![BreadcrumbItem {
                    label: "Home".to_string(),
                    url: Some("/".to_string()),
                }],
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        // Single item is the last item, rendered as font-medium span.
        assert!(html.contains("<span class=\"text-text font-medium\">Home</span>"));
        // No separator.
        assert!(!html.contains("<span>/</span>"));
    }

    // ── 12. Pagination ──────────────────────────────────────────────────

    #[test]
    fn pagination_renders_page_links() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "pg".to_string(),
            component: Component::Pagination(PaginationProps {
                current_page: 2,
                per_page: 10,
                total: 50,
                base_url: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<nav"));
        // Current page has active class.
        assert!(html.contains("bg-primary text-primary-foreground\">2</span>"));
        // Other pages are links.
        assert!(html.contains("?page=1"));
        assert!(html.contains("?page=3"));
    }

    #[test]
    fn pagination_single_page_produces_no_output() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "pg".to_string(),
            component: Component::Pagination(PaginationProps {
                current_page: 1,
                per_page: 10,
                total: 5,
                base_url: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        // Single page: no nav rendered.
        assert!(!html.contains("<nav"));
    }

    #[test]
    fn pagination_prev_and_next_buttons() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "pg".to_string(),
            component: Component::Pagination(PaginationProps {
                current_page: 3,
                per_page: 10,
                total: 100,
                base_url: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        // Prev button.
        assert!(html.contains("?page=2"));
        // Next button.
        assert!(html.contains("?page=4"));
    }

    #[test]
    fn pagination_no_prev_on_first_page() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "pg".to_string(),
            component: Component::Pagination(PaginationProps {
                current_page: 1,
                per_page: 10,
                total: 30,
                base_url: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        // Should not have prev link (&laquo;).
        assert!(!html.contains("&laquo;"));
        // Should have next link.
        assert!(html.contains("&raquo;"));
    }

    #[test]
    fn pagination_custom_base_url() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "pg".to_string(),
            component: Component::Pagination(PaginationProps {
                current_page: 1,
                per_page: 10,
                total: 30,
                base_url: Some("/users?sort=name&".to_string()),
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("/users?sort=name&amp;page=2"));
    }

    // ── 13. DescriptionList ─────────────────────────────────────────────

    #[test]
    fn description_list_renders_dl_dt_dd() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "dl".to_string(),
            component: Component::DescriptionList(DescriptionListProps {
                items: vec![
                    DescriptionItem {
                        label: "Name".to_string(),
                        value: "Alice".to_string(),
                        format: None,
                    },
                    DescriptionItem {
                        label: "Email".to_string(),
                        value: "alice@example.com".to_string(),
                        format: None,
                    },
                ],
                columns: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<dl"));
        assert!(html.contains("grid-cols-1"));
        assert!(html.contains("<dt class=\"text-sm font-medium text-text-muted\">Name</dt>"));
        assert!(html.contains("<dd class=\"mt-1 text-sm text-text\">Alice</dd>"));
        assert!(html.contains("<dt class=\"text-sm font-medium text-text-muted\">Email</dt>"));
    }

    #[test]
    fn description_list_with_columns() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "dl".to_string(),
            component: Component::DescriptionList(DescriptionListProps {
                items: vec![DescriptionItem {
                    label: "Status".to_string(),
                    value: "Active".to_string(),
                    format: None,
                }],
                columns: Some(3),
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("grid-cols-3"));
    }

    // ── 14. XSS prevention ──────────────────────────────────────────────

    #[test]
    fn xss_script_tags_escaped_in_text() {
        let view = JsonUiView::new().component(text_node(
            "t",
            "<script>alert('xss')</script>",
            TextElement::P,
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&#x27;"));
    }

    #[test]
    fn xss_quotes_escaped_in_attributes() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "av".to_string(),
            component: Component::Avatar(AvatarProps {
                src: Some("x\" onload=\"alert(1)".to_string()),
                alt: "Test".to_string(),
                fallback: None,
                size: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        // Quotes are escaped so the attacker cannot break out of the attribute.
        assert!(html.contains("&quot;"));
        // The src attribute value stays intact within quotes (no breakout).
        assert!(html.contains("src=\"x&quot; onload=&quot;alert(1)\""));
    }

    #[test]
    fn xss_in_button_label() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "b".to_string(),
            component: Component::Button(ButtonProps {
                label: "<img src=x onerror=alert(1)>".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: None,
                icon: None,
                icon_position: None,
                        button_type: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn xss_ampersand_in_content() {
        let view = JsonUiView::new().component(text_node("t", "Tom & Jerry", TextElement::P));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("Tom &amp; Jerry"));
    }

    #[test]
    fn html_escape_function_covers_all_chars() {
        let result = html_escape("&<>\"'normal");
        assert_eq!(result, "&amp;&lt;&gt;&quot;&#x27;normal");
    }

    // ── 15. Action wrapping ─────────────────────────────────────────────

    #[test]
    fn get_action_wraps_in_anchor() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "b".to_string(),
            component: Component::Button(ButtonProps {
                label: "View".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: None,
                icon: None,
                icon_position: None,
                        button_type: None,
            }),
            action: Some(make_action_with_url(
                "users.show",
                HttpMethod::Get,
                "/users/1",
            )),
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<a href=\"/users/1\" class=\"block\">"));
        assert!(html.contains("</a>"));
        assert!(html.contains("<button"));
    }

    #[test]
    fn post_action_does_not_wrap_in_anchor() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "b".to_string(),
            component: Component::Button(ButtonProps {
                label: "Submit".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: None,
                icon: None,
                icon_position: None,
                        button_type: None,
            }),
            action: Some(make_action_with_url(
                "users.store",
                HttpMethod::Post,
                "/users",
            )),
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(!html.contains("<a href="));
        assert!(html.contains("<button"));
    }

    #[test]
    fn get_action_without_url_does_not_wrap() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "b".to_string(),
            component: Component::Button(ButtonProps {
                label: "View".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: None,
                icon: None,
                icon_position: None,
                        button_type: None,
            }),
            action: Some(make_action("users.show", HttpMethod::Get)),
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(!html.contains("<a href="));
    }

    #[test]
    fn delete_action_does_not_wrap_in_anchor() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "b".to_string(),
            component: Component::Button(ButtonProps {
                label: "Delete".to_string(),
                variant: ButtonVariant::Destructive,
                size: Size::Default,
                disabled: None,
                icon: None,
                icon_position: None,
                        button_type: None,
            }),
            action: Some(make_action_with_url(
                "users.destroy",
                HttpMethod::Delete,
                "/users/1",
            )),
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(!html.contains("<a href="));
    }

    #[test]
    fn action_url_is_html_escaped() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "b".to_string(),
            component: Component::Button(ButtonProps {
                label: "View".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: None,
                icon: None,
                icon_position: None,
                        button_type: None,
            }),
            action: Some(make_action_with_url(
                "users.show",
                HttpMethod::Get,
                "/users?id=1&name=test",
            )),
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("href=\"/users?id=1&amp;name=test\""));
    }

    // ── 16. Card ───────────────────────────────────────────────────────

    #[test]
    fn card_renders_title_and_description() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "c".to_string(),
            component: Component::Card(CardProps {
                title: "My Card".to_string(),
                description: Some("A description".to_string()),
                children: vec![],
                footer: vec![],
                max_width: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("rounded-lg border border-border bg-card shadow-sm"));
        assert!(html
            .contains("<h3 class=\"text-lg font-semibold leading-snug text-text\">My Card</h3>"));
        assert!(html.contains("<p class=\"mt-1 text-sm text-text-muted\">A description</p>"));
    }

    #[test]
    fn card_renders_children_recursively() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "c".to_string(),
            component: Component::Card(CardProps {
                title: "Card".to_string(),
                description: None,
                children: vec![text_node("t", "Child content", TextElement::P)],
                footer: vec![],
                max_width: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(
            html.contains("mt-4 flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto")
        );
        assert!(html.contains("Child content"));
    }

    #[test]
    fn card_renders_footer() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "c".to_string(),
            component: Component::Card(CardProps {
                title: "Card".to_string(),
                description: None,
                children: vec![],
                footer: vec![button_node(
                    "btn",
                    "Save",
                    ButtonVariant::Default,
                    Size::Default,
                )],
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("border-t border-border px-6 py-4 flex items-center gap-2"));
        assert!(html.contains(">Save</button>"));
    }

    // ── 17. Modal ──────────────────────────────────────────────────────

    #[test]
    fn modal_renders_dialog_element() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "m".to_string(),
            component: Component::Modal(ModalProps {
                id: "modal-confirm".to_string(),
                title: "Confirm".to_string(),
                description: Some("Are you sure?".to_string()),
                children: vec![text_node("t", "Body text", TextElement::P)],
                footer: vec![button_node(
                    "ok",
                    "OK",
                    ButtonVariant::Default,
                    Size::Default,
                )],
                trigger_label: Some("Open Modal".to_string()),
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<dialog"), "uses dialog element");
        assert!(html.contains("aria-modal=\"true\""), "has aria-modal");
        assert!(
            html.contains("data-modal-open=\"modal-confirm\""),
            "trigger has data-modal-open"
        );
        assert!(html.contains("data-modal-close"), "has close button");
        assert!(html.contains("Confirm"), "shows title");
        assert!(html.contains("Are you sure?"), "shows description");
        assert!(html.contains("Body text"), "shows children");
        assert!(html.contains(">OK</button>"), "shows footer");
        assert!(!html.contains("<details"), "no details element");
        assert!(!html.contains("<summary"), "no summary element");
    }

    #[test]
    fn modal_default_trigger_label() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "m".to_string(),
            component: Component::Modal(ModalProps {
                id: "modal-dialog".to_string(),
                title: "Dialog".to_string(),
                description: None,
                children: vec![],
                footer: vec![],
                max_width: None,
                trigger_label: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("Open"), "default trigger label");
        assert!(html.contains("<dialog"), "uses dialog element");
    }

    // ── 18. Tabs ───────────────────────────────────────────────────────

    #[test]
    fn tabs_renders_only_default_tab_content() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "tabs".to_string(),
            component: Component::Tabs(TabsProps {
                default_tab: "general".to_string(),
                tabs: vec![
                    Tab {
                        value: "general".to_string(),
                        label: "General".to_string(),
                        children: vec![text_node("t1", "General content", TextElement::P)],
                    },
                    Tab {
                        value: "security".to_string(),
                        label: "Security".to_string(),
                        children: vec![text_node("t2", "Security content", TextElement::P)],
                    },
                ],
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        // Active tab styling.
        assert!(html.contains("border-b-2 border-primary text-primary"));
        assert!(html.contains(">General</button>"));
        // Inactive tab styling.
        assert!(html.contains("border-transparent text-text-muted"));
        assert!(html.contains(">Security</button>"));
        // Default tab panel visible, inactive panel hidden.
        assert!(html.contains("General content"));
        assert!(html.contains("Security content"));
        // Active panel has no hidden class; inactive panel is hidden.
        assert!(html.contains("data-tab-panel=\"general\" class=\"pt-4 flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\""));
        assert!(html.contains("data-tab-panel=\"security\" class=\"pt-4 flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto hidden\""));
    }

    // ── 19. Form ───────────────────────────────────────────────────────

    #[test]
    fn form_renders_action_url_and_method() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "f".to_string(),
            component: Component::Form(FormProps {
                action: Action {
                    handler: "users.store".to_string(),
                    url: Some("/users".to_string()),
                    method: HttpMethod::Post,
                    confirm: None,
                    on_success: None,
                    on_error: None,
                },
                fields: vec![],
                method: None,
                guard: None,
                max_width: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("action=\"/users\""));
        assert!(html.contains("method=\"post\""));
        assert!(html.contains(
            "class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\""
        ));
    }

    #[test]
    fn form_method_spoofing_for_delete() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "f".to_string(),
            component: Component::Form(FormProps {
                action: Action {
                    handler: "users.destroy".to_string(),
                    url: Some("/users/1".to_string()),
                    method: HttpMethod::Delete,
                    confirm: None,
                    on_success: None,
                    on_error: None,
                },
                fields: vec![],
                method: None,
                guard: None,
                max_width: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("method=\"post\""));
        assert!(html.contains("<input type=\"hidden\" name=\"_method\" value=\"DELETE\">"));
    }

    #[test]
    fn form_method_spoofing_for_put() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "f".to_string(),
            component: Component::Form(FormProps {
                action: Action {
                    handler: "users.update".to_string(),
                    url: Some("/users/1".to_string()),
                    method: HttpMethod::Put,
                    confirm: None,
                    on_success: None,
                    on_error: None,
                },
                fields: vec![],
                method: Some(HttpMethod::Put),
                guard: None,
                max_width: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("method=\"post\""));
        assert!(html.contains("name=\"_method\" value=\"PUT\""));
    }

    #[test]
    fn form_get_method_no_spoofing() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "f".to_string(),
            component: Component::Form(FormProps {
                action: Action {
                    handler: "users.index".to_string(),
                    url: Some("/users".to_string()),
                    method: HttpMethod::Get,
                    confirm: None,
                    on_success: None,
                    on_error: None,
                },
                fields: vec![],
                method: None,
                guard: None,
                max_width: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("method=\"get\""));
        assert!(!html.contains("_method"));
    }

    // ── 20. Input ──────────────────────────────────────────────────────

    #[test]
    fn input_renders_label_and_field() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "i".to_string(),
            component: Component::Input(InputProps {
                field: "email".to_string(),
                label: "Email".to_string(),
                input_type: InputType::Email,
                placeholder: Some("user@example.com".to_string()),
                required: Some(true),
                disabled: None,
                error: None,
                description: Some("Your work email".to_string()),
                default_value: None,
                data_path: None,
                step: None,
                list: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("for=\"email\""));
        assert!(html.contains(">Email</label>"));
        assert!(html.contains("Your work email"));
        assert!(html.contains("type=\"email\""));
        assert!(html.contains("id=\"email\""));
        assert!(html.contains("name=\"email\""));
        assert!(html.contains("placeholder=\"user@example.com\""));
        assert!(html.contains(" required"));
        assert!(html.contains("border-border"));
    }

    #[test]
    fn input_renders_error_with_red_border() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "i".to_string(),
            component: Component::Input(InputProps {
                field: "name".to_string(),
                label: "Name".to_string(),
                input_type: InputType::Text,
                placeholder: None,
                required: None,
                disabled: None,
                error: Some("Name is required".to_string()),
                description: None,
                default_value: None,
                data_path: None,
                step: None,
                list: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("border-destructive"));
        assert!(html.contains("text-destructive") && html.contains("Name is required"));
        assert!(
            html.contains("ring-destructive"),
            "error input should have destructive ring"
        );
    }

    #[test]
    fn input_resolves_data_path_for_value() {
        let data = json!({"user": {"name": "Alice"}});
        let view = JsonUiView::new().component(ComponentNode {
            key: "i".to_string(),
            component: Component::Input(InputProps {
                field: "name".to_string(),
                label: "Name".to_string(),
                input_type: InputType::Text,
                placeholder: None,
                required: None,
                disabled: None,
                error: None,
                description: None,
                default_value: None,
                data_path: Some("/user/name".to_string()),
                step: None,
                list: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &data);
        assert!(html.contains("value=\"Alice\""));
    }

    #[test]
    fn input_default_value_overrides_data_path() {
        let data = json!({"user": {"name": "Alice"}});
        let view = JsonUiView::new().component(ComponentNode {
            key: "i".to_string(),
            component: Component::Input(InputProps {
                field: "name".to_string(),
                label: "Name".to_string(),
                input_type: InputType::Text,
                placeholder: None,
                required: None,
                disabled: None,
                error: None,
                description: None,
                default_value: Some("Bob".to_string()),
                data_path: Some("/user/name".to_string()),
                step: None,
                list: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &data);
        assert!(html.contains("value=\"Bob\""));
        assert!(!html.contains("Alice"));
    }

    #[test]
    fn input_textarea_renders_textarea_element() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "i".to_string(),
            component: Component::Input(InputProps {
                field: "bio".to_string(),
                label: "Bio".to_string(),
                input_type: InputType::Textarea,
                placeholder: Some("Tell us about yourself".to_string()),
                required: None,
                disabled: None,
                error: None,
                description: None,
                default_value: Some("Hello world".to_string()),
                data_path: None,
                step: None,
                list: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<textarea"));
        assert!(html.contains(">Hello world</textarea>"));
        assert!(html.contains("placeholder=\"Tell us about yourself\""));
    }

    #[test]
    fn input_hidden_renders_hidden_field() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "i".to_string(),
            component: Component::Input(InputProps {
                field: "token".to_string(),
                label: "Token".to_string(),
                input_type: InputType::Hidden,
                placeholder: None,
                required: None,
                disabled: None,
                error: None,
                description: None,
                default_value: Some("abc123".to_string()),
                data_path: None,
                step: None,
                list: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("type=\"hidden\""));
        assert!(html.contains("value=\"abc123\""));
    }

    #[test]
    fn input_renders_datalist() {
        let props = InputProps {
            field: "category".to_string(),
            label: "Category".to_string(),
            input_type: InputType::Text,
            placeholder: None,
            required: None,
            disabled: None,
            error: None,
            description: None,
            default_value: None,
            data_path: None,
            step: None,
            list: Some("cat-suggestions".to_string()),
        };
        let data = serde_json::json!({
            "cat-suggestions": ["Pizza", "Pasta", "Bevande"]
        });
        let html = render_input(&props, &data);
        assert!(
            html.contains("list=\"cat-suggestions\""),
            "input should have list attribute"
        );
        assert!(
            html.contains("<datalist id=\"cat-suggestions\">"),
            "should render datalist element"
        );
        assert!(
            html.contains("<option value=\"Pizza\">"),
            "should render option for Pizza"
        );
        assert!(
            html.contains("<option value=\"Pasta\">"),
            "should render option for Pasta"
        );
        assert!(
            html.contains("<option value=\"Bevande\">"),
            "should render option for Bevande"
        );
        assert!(html.contains("</datalist>"), "should close datalist");
    }

    #[test]
    fn input_no_datalist_without_data() {
        let props = InputProps {
            field: "category".to_string(),
            label: "Category".to_string(),
            input_type: InputType::Text,
            placeholder: None,
            required: None,
            disabled: None,
            error: None,
            description: None,
            default_value: None,
            data_path: None,
            step: None,
            list: Some("missing-key".to_string()),
        };
        let data = serde_json::json!({});
        let html = render_input(&props, &data);
        assert!(
            html.contains("list=\"missing-key\""),
            "input should still have list attribute"
        );
        assert!(
            !html.contains("<datalist"),
            "should NOT render datalist when data key missing"
        );
    }

    // ── 21. Select ─────────────────────────────────────────────────────

    #[test]
    fn select_renders_options_with_selected() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "s".to_string(),
            component: Component::Select(SelectProps {
                field: "role".to_string(),
                label: "Role".to_string(),
                options: vec![
                    SelectOption {
                        value: "admin".to_string(),
                        label: "Admin".to_string(),
                    },
                    SelectOption {
                        value: "user".to_string(),
                        label: "User".to_string(),
                    },
                ],
                placeholder: Some("Select a role".to_string()),
                required: Some(true),
                disabled: None,
                error: None,
                description: None,
                default_value: Some("admin".to_string()),
                data_path: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("for=\"role\""));
        assert!(html.contains("id=\"role\""));
        assert!(html.contains("name=\"role\""));
        assert!(html.contains("<option value=\"\">Select a role</option>"));
        assert!(html.contains("<option value=\"admin\" selected>Admin</option>"));
        assert!(html.contains("<option value=\"user\">User</option>"));
        assert!(html.contains(" required"));
    }

    #[test]
    fn select_resolves_data_path_for_selected() {
        let data = json!({"user": {"role": "user"}});
        let view = JsonUiView::new().component(ComponentNode {
            key: "s".to_string(),
            component: Component::Select(SelectProps {
                field: "role".to_string(),
                label: "Role".to_string(),
                options: vec![
                    SelectOption {
                        value: "admin".to_string(),
                        label: "Admin".to_string(),
                    },
                    SelectOption {
                        value: "user".to_string(),
                        label: "User".to_string(),
                    },
                ],
                placeholder: None,
                required: None,
                disabled: None,
                error: None,
                description: None,
                default_value: None,
                data_path: Some("/user/role".to_string()),
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &data);
        assert!(html.contains("<option value=\"user\" selected>User</option>"));
        assert!(!html.contains("<option value=\"admin\" selected>"));
    }

    #[test]
    fn select_renders_error() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "s".to_string(),
            component: Component::Select(SelectProps {
                field: "role".to_string(),
                label: "Role".to_string(),
                options: vec![],
                placeholder: None,
                required: None,
                disabled: None,
                error: Some("Role is required".to_string()),
                description: None,
                default_value: None,
                data_path: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("border-destructive"));
        assert!(html.contains("Role is required"));
        assert!(
            html.contains("ring-destructive"),
            "error select should have destructive ring"
        );
    }

    // ── 22. Checkbox ───────────────────────────────────────────────────

    #[test]
    fn checkbox_renders_checked_state() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "cb".to_string(),
            component: Component::Checkbox(CheckboxProps {
                field: "terms".to_string(),
                value: None,
                label: "Accept Terms".to_string(),
                description: Some("You must accept".to_string()),
                checked: Some(true),
                data_path: None,
                required: Some(true),
                disabled: None,
                error: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("type=\"checkbox\""));
        assert!(html.contains("id=\"terms\""));
        assert!(html.contains("name=\"terms\""));
        assert!(html.contains("value=\"1\""));
        assert!(html.contains(" checked"));
        assert!(html.contains(" required"));
        assert!(html.contains("for=\"terms\""));
        assert!(html.contains(">Accept Terms</label>"));
        assert!(html.contains("ml-6 text-sm text-text-muted"));
        assert!(html.contains("You must accept"));
    }

    #[test]
    fn checkbox_resolves_data_path_for_checked() {
        let data = json!({"user": {"accepted": true}});
        let view = JsonUiView::new().component(ComponentNode {
            key: "cb".to_string(),
            component: Component::Checkbox(CheckboxProps {
                field: "accepted".to_string(),
                value: None,
                label: "Accepted".to_string(),
                description: None,
                checked: None,
                data_path: Some("/user/accepted".to_string()),
                required: None,
                disabled: None,
                error: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &data);
        assert!(html.contains(" checked"));
    }

    #[test]
    fn checkbox_renders_error() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "cb".to_string(),
            component: Component::Checkbox(CheckboxProps {
                field: "terms".to_string(),
                value: None,
                label: "Terms".to_string(),
                description: None,
                checked: None,
                data_path: None,
                required: None,
                disabled: None,
                error: Some("Must accept".to_string()),
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("ml-6 text-sm text-destructive"));
        assert!(html.contains("Must accept"));
    }

    // ── 23. Switch ─────────────────────────────────────────────────────

    #[test]
    fn switch_renders_toggle_structure() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "sw".to_string(),
            component: Component::Switch(SwitchProps {
                field: "notifications".to_string(),
                label: "Notifications".to_string(),
                description: Some("Get email updates".to_string()),
                checked: Some(true),
                data_path: None,
                required: None,
                disabled: None,
                error: None,
                action: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("sr-only peer"));
        assert!(html.contains("id=\"notifications\""));
        assert!(html.contains("name=\"notifications\""));
        assert!(html.contains("value=\"1\""));
        assert!(html.contains(" checked"));
        assert!(html.contains("peer-checked:bg-primary"));
        assert!(html.contains("for=\"notifications\""));
        assert!(html.contains(">Notifications</label>"));
        assert!(html.contains("Get email updates"));
    }

    #[test]
    fn switch_renders_error() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "sw".to_string(),
            component: Component::Switch(SwitchProps {
                field: "agree".to_string(),
                label: "Agree".to_string(),
                description: None,
                checked: None,
                data_path: None,
                required: None,
                disabled: None,
                error: Some("Required".to_string()),
                action: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("text-sm text-destructive"));
        assert!(html.contains("Required"));
    }

    // ── 24. Table ──────────────────────────────────────────────────────

    #[test]
    fn table_renders_headers_and_data_rows() {
        let data = json!({
            "users": [
                {"name": "Alice", "email": "alice@example.com"},
                {"name": "Bob", "email": "bob@example.com"}
            ]
        });
        let view = JsonUiView::new().component(ComponentNode {
            key: "t".to_string(),
            component: Component::Table(TableProps {
                columns: vec![
                    Column {
                        key: "name".to_string(),
                        label: "Name".to_string(),
                        format: None,
                    },
                    Column {
                        key: "email".to_string(),
                        label: "Email".to_string(),
                        format: None,
                    },
                ],
                data_path: "/users".to_string(),
                row_actions: None,
                empty_message: Some("No users".to_string()),
                sortable: None,
                sort_column: None,
                sort_direction: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &data);
        // Headers.
        assert!(html.contains("tracking-wider text-text-muted\">Name</th>"));
        assert!(html.contains("tracking-wider text-text-muted\">Email</th>"));
        // Data rows.
        assert!(html.contains(">Alice</td>"));
        assert!(html.contains(">alice@example.com</td>"));
        assert!(html.contains(">Bob</td>"));
        assert!(html.contains(">bob@example.com</td>"));
        // Wrapped in overflow container.
        assert!(html.contains("overflow-x-auto"));
    }

    #[test]
    fn table_renders_empty_message() {
        let data = json!({"users": []});
        let view = JsonUiView::new().component(ComponentNode {
            key: "t".to_string(),
            component: Component::Table(TableProps {
                columns: vec![Column {
                    key: "name".to_string(),
                    label: "Name".to_string(),
                    format: None,
                }],
                data_path: "/users".to_string(),
                row_actions: None,
                empty_message: Some("No users found".to_string()),
                sortable: None,
                sort_column: None,
                sort_direction: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &data);
        assert!(html.contains("No users found"));
        assert!(html.contains("text-center text-sm text-text-muted"));
    }

    #[test]
    fn table_renders_empty_message_when_path_missing() {
        let data = json!({});
        let view = JsonUiView::new().component(ComponentNode {
            key: "t".to_string(),
            component: Component::Table(TableProps {
                columns: vec![Column {
                    key: "name".to_string(),
                    label: "Name".to_string(),
                    format: None,
                }],
                data_path: "/users".to_string(),
                row_actions: None,
                empty_message: Some("No data".to_string()),
                sortable: None,
                sort_column: None,
                sort_direction: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &data);
        assert!(html.contains("No data"));
    }

    #[test]
    fn table_renders_row_actions() {
        let data = json!({"items": [{"name": "Item 1"}]});
        let view = JsonUiView::new().component(ComponentNode {
            key: "t".to_string(),
            component: Component::Table(TableProps {
                columns: vec![Column {
                    key: "name".to_string(),
                    label: "Name".to_string(),
                    format: None,
                }],
                data_path: "/items".to_string(),
                row_actions: Some(vec![
                    make_action_with_url("items.edit", HttpMethod::Get, "/items/1/edit"),
                    make_action_with_url("items.destroy", HttpMethod::Delete, "/items/1"),
                ]),
                empty_message: None,
                sortable: None,
                sort_column: None,
                sort_direction: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &data);
        // Actions header.
        assert!(html.contains(">Azioni</th>"));
        // Action links.
        assert!(html.contains("href=\"/items/1/edit\""));
        assert!(html.contains(">edit</a>"));
        assert!(html.contains("href=\"/items/1\""));
        assert!(html.contains(">destroy</a>"));
    }

    #[test]
    fn table_handles_numeric_and_bool_cells() {
        let data = json!({"rows": [{"count": 42, "active": true}]});
        let view = JsonUiView::new().component(ComponentNode {
            key: "t".to_string(),
            component: Component::Table(TableProps {
                columns: vec![
                    Column {
                        key: "count".to_string(),
                        label: "Count".to_string(),
                        format: None,
                    },
                    Column {
                        key: "active".to_string(),
                        label: "Active".to_string(),
                        format: None,
                    },
                ],
                data_path: "/rows".to_string(),
                row_actions: None,
                empty_message: None,
                sortable: None,
                sort_column: None,
                sort_direction: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &data);
        assert!(html.contains(">42</td>"));
        assert!(html.contains(">true</td>"));
    }

    // ── Plugin rendering tests ────────────────────────────────────────

    #[test]
    fn plugin_renders_error_div_when_not_registered() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "map-1".to_string(),
            component: Component::Plugin(PluginProps {
                plugin_type: "UnknownPluginXyz".to_string(),
                props: json!({"lat": 0}),
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("Unknown plugin component: UnknownPluginXyz"));
        assert!(html.contains("bg-destructive/10"));
    }

    #[test]
    fn collect_plugin_types_finds_top_level_plugins() {
        let view = JsonUiView::new()
            .component(ComponentNode {
                key: "map".to_string(),
                component: Component::Plugin(PluginProps {
                    plugin_type: "Map".to_string(),
                    props: json!({}),
                }),
                action: None,
                visibility: None,
            })
            .component(ComponentNode {
                key: "text".to_string(),
                component: Component::Text(TextProps {
                    content: "Hello".to_string(),
                    element: TextElement::P,
                }),
                action: None,
                visibility: None,
            });
        let types = collect_plugin_types(&view);
        assert_eq!(types.len(), 1);
        assert!(types.contains("Map"));
    }

    #[test]
    fn collect_plugin_types_finds_nested_in_card() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "card".to_string(),
            component: Component::Card(CardProps {
                title: "Test".to_string(),
                description: None,
                children: vec![ComponentNode {
                    key: "chart".to_string(),
                    component: Component::Plugin(PluginProps {
                        plugin_type: "Chart".to_string(),
                        props: json!({}),
                    }),
                    action: None,
                    visibility: None,
                }],
                footer: vec![],
                max_width: None,
            }),
            action: None,
            visibility: None,
        });
        let types = collect_plugin_types(&view);
        assert!(types.contains("Chart"));
    }

    #[test]
    fn collect_plugin_types_deduplicates() {
        let view = JsonUiView::new()
            .component(ComponentNode {
                key: "map1".to_string(),
                component: Component::Plugin(PluginProps {
                    plugin_type: "Map".to_string(),
                    props: json!({}),
                }),
                action: None,
                visibility: None,
            })
            .component(ComponentNode {
                key: "map2".to_string(),
                component: Component::Plugin(PluginProps {
                    plugin_type: "Map".to_string(),
                    props: json!({"zoom": 5}),
                }),
                action: None,
                visibility: None,
            });
        let types = collect_plugin_types(&view);
        assert_eq!(types.len(), 1);
    }

    #[test]
    fn collect_plugin_types_empty_for_builtin_only() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "text".to_string(),
            component: Component::Text(TextProps {
                content: "Hello".to_string(),
                element: TextElement::P,
            }),
            action: None,
            visibility: None,
        });
        let types = collect_plugin_types(&view);
        assert!(types.is_empty());
    }

    #[test]
    fn render_to_html_with_plugins_returns_empty_assets_for_builtin_only() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "text".to_string(),
            component: Component::Text(TextProps {
                content: "Hello".to_string(),
                element: TextElement::P,
            }),
            action: None,
            visibility: None,
        });
        let result = render_to_html_with_plugins(&view, &json!({}));
        assert!(result.css_head.is_empty());
        assert!(result.scripts.is_empty());
        assert!(result.html.contains("Hello"));
    }

    #[test]
    fn render_css_tags_generates_link_elements() {
        let assets = vec![Asset::new("https://cdn.example.com/style.css")
            .integrity("sha256-abc")
            .crossorigin("")];
        let tags = render_css_tags(&assets);
        assert!(tags.contains("rel=\"stylesheet\""));
        assert!(tags.contains("href=\"https://cdn.example.com/style.css\""));
        assert!(tags.contains("integrity=\"sha256-abc\""));
        assert!(tags.contains("crossorigin=\"\""));
    }

    #[test]
    fn render_js_tags_generates_script_elements() {
        let assets = vec![Asset::new("https://cdn.example.com/lib.js")];
        let init = vec!["initLib();".to_string()];
        let tags = render_js_tags(&assets, &init);
        assert!(tags.contains("src=\"https://cdn.example.com/lib.js\""));
        assert!(tags.contains("<script>initLib();</script>"));
    }

    // ── StatCard ─────────────────────────────────────────────────────────

    #[test]
    fn stat_card_renders_label_and_value() {
        let view = JsonUiView::new().component(ComponentNode::stat_card(
            "rev",
            StatCardProps {
                label: "Revenue".to_string(),
                value: "$1,234".to_string(),
                icon: None,
                subtitle: None,
                sse_target: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("Revenue"));
        assert!(html.contains("$1,234"));
        assert!(html.contains("bg-card rounded-lg shadow-sm"));
    }

    #[test]
    fn stat_card_renders_icon_and_subtitle() {
        let view = JsonUiView::new().component(ComponentNode::stat_card(
            "users",
            StatCardProps {
                label: "Users".to_string(),
                value: "42".to_string(),
                icon: Some("👤".to_string()),
                subtitle: Some("active today".to_string()),
                sse_target: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("👤"));
        assert!(html.contains("active today"));
    }

    #[test]
    fn stat_card_renders_svg_icon_without_escaping() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="8" y="2" width="8" height="4" rx="1" ry="1"/></svg>"#;
        let view = JsonUiView::new().component(ComponentNode::stat_card(
            "svg-icon",
            StatCardProps {
                label: "Test".to_string(),
                value: "0".to_string(),
                icon: Some(svg.to_string()),
                subtitle: None,
                sse_target: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(
            html.contains("<svg"),
            "SVG should render as markup, not escaped text"
        );
        assert!(!html.contains("&lt;svg"), "SVG should NOT be HTML-escaped");
    }

    #[test]
    fn stat_card_renders_sse_target_data_attributes() {
        let view = JsonUiView::new().component(ComponentNode::stat_card(
            "live",
            StatCardProps {
                label: "Live count".to_string(),
                value: "100".to_string(),
                icon: None,
                subtitle: None,
                sse_target: Some("visitor_count".to_string()),
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("data-sse-target=\"visitor_count\""));
        assert!(html.contains("data-live-value"));
    }

    #[test]
    fn stat_card_no_sse_target_omits_data_attributes() {
        let view = JsonUiView::new().component(ComponentNode::stat_card(
            "static",
            StatCardProps {
                label: "Label".to_string(),
                value: "99".to_string(),
                icon: None,
                subtitle: None,
                sse_target: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(!html.contains("data-sse-target"));
        assert!(!html.contains("data-live-value"));
    }

    // ── Checklist ────────────────────────────────────────────────────────

    #[test]
    fn checklist_renders_title_and_items() {
        let view = JsonUiView::new().component(ComponentNode::checklist(
            "tasks",
            ChecklistProps {
                title: "Setup Tasks".to_string(),
                items: vec![
                    ChecklistItem {
                        label: "Create account".to_string(),
                        checked: true,
                        href: None,
                    },
                    ChecklistItem {
                        label: "Add team member".to_string(),
                        checked: false,
                        href: None,
                    },
                ],
                dismissible: true,
                dismiss_label: None,
                data_key: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("Setup Tasks"));
        assert!(html.contains("Create account"));
        assert!(html.contains("Add team member"));
    }

    #[test]
    fn checklist_checked_item_has_strikethrough() {
        let view = JsonUiView::new().component(ComponentNode::checklist(
            "tasks",
            ChecklistProps {
                title: "Tasks".to_string(),
                items: vec![ChecklistItem {
                    label: "Done".to_string(),
                    checked: true,
                    href: None,
                }],
                dismissible: false,
                dismiss_label: None,
                data_key: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("line-through"));
        assert!(html.contains("checked"));
    }

    #[test]
    fn checklist_dismissible_renders_dismiss_button() {
        let view = JsonUiView::new().component(ComponentNode::checklist(
            "tasks",
            ChecklistProps {
                title: "Tasks".to_string(),
                items: vec![],
                dismissible: true,
                dismiss_label: Some("Close".to_string()),
                data_key: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("Close"));
        assert!(html.contains("data-dismissible"));
        assert!(html.contains("font-medium"));
        assert!(html.contains("text-text"));
        assert!(html.contains("hover:text-primary"));
    }

    #[test]
    fn checklist_data_key_added_to_container() {
        let view = JsonUiView::new().component(ComponentNode::checklist(
            "tasks",
            ChecklistProps {
                title: "Tasks".to_string(),
                items: vec![],
                dismissible: false,
                dismiss_label: None,
                data_key: Some("onboarding_checklist".to_string()),
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("data-checklist-key=\"onboarding_checklist\""));
    }

    #[test]
    fn checklist_item_with_href_renders_link() {
        let view = JsonUiView::new().component(ComponentNode::checklist(
            "tasks",
            ChecklistProps {
                title: "Tasks".to_string(),
                items: vec![ChecklistItem {
                    label: "Visit docs".to_string(),
                    checked: false,
                    href: Some("/docs".to_string()),
                }],
                dismissible: false,
                dismiss_label: None,
                data_key: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("href=\"/docs\""));
        assert!(html.contains("Visit docs"));
    }

    // ── Toast ────────────────────────────────────────────────────────────

    #[test]
    fn toast_renders_message_and_variant() {
        let view = JsonUiView::new().component(ComponentNode::toast(
            "t",
            ToastProps {
                message: "Saved successfully!".to_string(),
                variant: ToastVariant::Success,
                timeout: None,
                dismissible: true,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("Saved successfully!"));
        assert!(html.contains("data-toast-variant=\"success\""));
    }

    #[test]
    fn toast_renders_timeout_attribute() {
        let view = JsonUiView::new().component(ComponentNode::toast(
            "t",
            ToastProps {
                message: "Warning!".to_string(),
                variant: ToastVariant::Warning,
                timeout: Some(10),
                dismissible: false,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("data-toast-timeout=\"10\""));
        assert!(!html.contains("data-toast-dismissible"));
    }

    #[test]
    fn toast_default_timeout_is_five_seconds() {
        let view = JsonUiView::new().component(ComponentNode::toast(
            "t",
            ToastProps {
                message: "Hello".to_string(),
                variant: ToastVariant::Info,
                timeout: None,
                dismissible: false,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("data-toast-timeout=\"5\""));
    }

    #[test]
    fn toast_dismissible_renders_dismiss_button() {
        let view = JsonUiView::new().component(ComponentNode::toast(
            "t",
            ToastProps {
                message: "Error occurred".to_string(),
                variant: ToastVariant::Error,
                timeout: None,
                dismissible: true,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("data-toast-dismissible"));
        assert!(html.contains("&times;"));
    }

    #[test]
    fn toast_info_variant_uses_blue_classes() {
        let view = JsonUiView::new().component(ComponentNode::toast(
            "t",
            ToastProps {
                message: "Info".to_string(),
                variant: ToastVariant::Info,
                timeout: None,
                dismissible: false,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("bg-primary/10"));
        assert!(html.contains("data-toast-variant=\"info\""));
    }

    #[test]
    fn toast_has_fixed_position_classes() {
        let view = JsonUiView::new().component(ComponentNode::toast(
            "t",
            ToastProps {
                message: "msg".to_string(),
                variant: ToastVariant::Info,
                timeout: None,
                dismissible: false,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("fixed top-4 right-4 z-50"));
    }

    // ── NotificationDropdown ─────────────────────────────────────────────

    #[test]
    fn notification_dropdown_renders_bell_icon() {
        let view = JsonUiView::new().component(ComponentNode::notification_dropdown(
            "notifs",
            NotificationDropdownProps {
                notifications: vec![],
                empty_text: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("data-notification-dropdown"));
        assert!(html.contains("data-notification-count=\"0\""));
    }

    #[test]
    fn notification_dropdown_shows_unread_count_badge() {
        let view = JsonUiView::new().component(ComponentNode::notification_dropdown(
            "notifs",
            NotificationDropdownProps {
                notifications: vec![
                    NotificationItem {
                        icon: None,
                        text: "New message".to_string(),
                        timestamp: None,
                        read: false,
                        action_url: None,
                    },
                    NotificationItem {
                        icon: None,
                        text: "Old message".to_string(),
                        timestamp: None,
                        read: true,
                        action_url: None,
                    },
                ],
                empty_text: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("data-notification-count=\"1\""));
        assert!(html.contains("New message"));
        assert!(html.contains("Old message"));
    }

    #[test]
    fn notification_dropdown_shows_empty_text_when_no_notifications() {
        let view = JsonUiView::new().component(ComponentNode::notification_dropdown(
            "notifs",
            NotificationDropdownProps {
                notifications: vec![],
                empty_text: Some("All caught up!".to_string()),
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("All caught up!"));
    }

    #[test]
    fn notification_dropdown_unread_indicator_for_unread_items() {
        let view = JsonUiView::new().component(ComponentNode::notification_dropdown(
            "notifs",
            NotificationDropdownProps {
                notifications: vec![NotificationItem {
                    icon: None,
                    text: "Unread".to_string(),
                    timestamp: None,
                    read: false,
                    action_url: None,
                }],
                empty_text: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("bg-primary"));
    }

    // ── Sidebar ──────────────────────────────────────────────────────────

    #[test]
    fn sidebar_renders_aside_element() {
        let view = JsonUiView::new().component(ComponentNode::sidebar(
            "nav",
            SidebarProps {
                fixed_top: vec![],
                groups: vec![],
                fixed_bottom: vec![],
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<aside"));
        assert!(html.contains("</aside>"));
    }

    #[test]
    fn sidebar_renders_fixed_top_items() {
        let view = JsonUiView::new().component(ComponentNode::sidebar(
            "nav",
            SidebarProps {
                fixed_top: vec![SidebarNavItem {
                    label: "Dashboard".to_string(),
                    href: "/dashboard".to_string(),
                    icon: None,
                    active: true,
                }],
                groups: vec![],
                fixed_bottom: vec![],
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("href=\"/dashboard\""));
        assert!(html.contains("Dashboard"));
        assert!(html.contains("bg-card text-primary"));
    }

    #[test]
    fn sidebar_renders_groups_with_data_attribute() {
        let view = JsonUiView::new().component(ComponentNode::sidebar(
            "nav",
            SidebarProps {
                fixed_top: vec![],
                groups: vec![SidebarGroup {
                    label: "Management".to_string(),
                    collapsed: false,
                    items: vec![SidebarNavItem {
                        label: "Users".to_string(),
                        href: "/users".to_string(),
                        icon: None,
                        active: false,
                    }],
                }],
                fixed_bottom: vec![],
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("data-sidebar-group"));
        assert!(html.contains("Management"));
        assert!(html.contains("Users"));
        assert!(!html.contains("data-collapsed"));
    }

    #[test]
    fn sidebar_collapsed_group_has_data_collapsed() {
        let view = JsonUiView::new().component(ComponentNode::sidebar(
            "nav",
            SidebarProps {
                fixed_top: vec![],
                groups: vec![SidebarGroup {
                    label: "Advanced".to_string(),
                    collapsed: true,
                    items: vec![],
                }],
                fixed_bottom: vec![],
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("data-collapsed"));
    }

    #[test]
    fn sidebar_inactive_item_uses_gray_classes() {
        let view = JsonUiView::new().component(ComponentNode::sidebar(
            "nav",
            SidebarProps {
                fixed_top: vec![SidebarNavItem {
                    label: "Settings".to_string(),
                    href: "/settings".to_string(),
                    icon: None,
                    active: false,
                }],
                groups: vec![],
                fixed_bottom: vec![],
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("text-text-muted"));
        assert!(!html.contains("text-primary"));
    }

    // ── Header ───────────────────────────────────────────────────────────

    #[test]
    fn header_renders_business_name() {
        let view = JsonUiView::new().component(ComponentNode::header(
            "hdr",
            HeaderProps {
                business_name: "Acme Corp".to_string(),
                notification_count: None,
                user_name: None,
                user_avatar: None,
                logout_url: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<header"));
        assert!(html.contains("Acme Corp"));
    }

    #[test]
    fn header_renders_notification_count_badge() {
        let view = JsonUiView::new().component(ComponentNode::header(
            "hdr",
            HeaderProps {
                business_name: "Acme".to_string(),
                notification_count: Some(3),
                user_name: None,
                user_avatar: None,
                logout_url: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("data-notification-count=\"3\""));
    }

    #[test]
    fn header_no_badge_when_count_is_zero() {
        let view = JsonUiView::new().component(ComponentNode::header(
            "hdr",
            HeaderProps {
                business_name: "Acme".to_string(),
                notification_count: Some(0),
                user_name: None,
                user_avatar: None,
                logout_url: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("data-notification-count=\"0\""));
        // No red badge when count is zero.
        assert!(!html.contains("bg-destructive"));
    }

    #[test]
    fn header_renders_user_name_initials() {
        let view = JsonUiView::new().component(ComponentNode::header(
            "hdr",
            HeaderProps {
                business_name: "Acme".to_string(),
                notification_count: None,
                user_name: Some("John Doe".to_string()),
                user_avatar: None,
                logout_url: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("JD"));
        assert!(html.contains("John Doe"));
    }

    #[test]
    fn header_renders_avatar_image_when_provided() {
        let view = JsonUiView::new().component(ComponentNode::header(
            "hdr",
            HeaderProps {
                business_name: "Acme".to_string(),
                notification_count: None,
                user_name: None,
                user_avatar: Some("/avatar.jpg".to_string()),
                logout_url: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("src=\"/avatar.jpg\""));
        assert!(html.contains("rounded-full"));
    }

    #[test]
    fn header_renders_logout_link() {
        let view = JsonUiView::new().component(ComponentNode::header(
            "hdr",
            HeaderProps {
                business_name: "Acme".to_string(),
                notification_count: None,
                user_name: None,
                user_avatar: None,
                logout_url: Some("/logout".to_string()),
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("href=\"/logout\""));
        assert!(html.contains("Logout"));
    }

    #[test]
    fn header_escapes_business_name_xss() {
        let view = JsonUiView::new().component(ComponentNode::header(
            "hdr",
            HeaderProps {
                business_name: "<script>alert(1)</script>".to_string(),
                notification_count: None,
                user_name: None,
                user_avatar: None,
                logout_url: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    // ── Edge case integration tests ───────────────────────────────────────

    #[test]
    fn test_render_deeply_nested_components() {
        // Card -> Card -> Text (three levels deep)
        let inner_card = ComponentNode::card(
            "inner-card",
            CardProps {
                title: "Inner Card".to_string(),
                description: None,
                children: vec![ComponentNode {
                    key: "inner-text".to_string(),
                    component: Component::Text(TextProps {
                        content: "Deep content".to_string(),
                        element: TextElement::P,
                    }),
                    action: None,
                    visibility: None,
                }],
                footer: vec![],
                max_width: None,
            },
        );
        let outer_card = ComponentNode::card(
            "outer-card",
            CardProps {
                title: "Outer Card".to_string(),
                description: None,
                children: vec![inner_card],
                footer: vec![],
                max_width: None,
            },
        );
        let view = JsonUiView::new().component(outer_card);
        let html = render_to_html(&view, &json!({}));

        assert!(
            html.contains("Outer Card"),
            "outer card title should be rendered"
        );
        assert!(
            html.contains("Inner Card"),
            "inner card title should be rendered"
        );
        assert!(
            html.contains("Deep content"),
            "nested text content should be rendered"
        );
    }

    #[test]
    fn test_render_empty_view() {
        let view = JsonUiView::new();
        let html = render_to_html(&view, &json!({}));
        assert_eq!(html, "<div class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\"></div>", "empty view renders empty div");
    }

    #[test]
    fn test_render_component_with_visibility_and_action() {
        use crate::action::{Action, HttpMethod};
        use crate::visibility::{Visibility, VisibilityCondition, VisibilityOperator};

        // A ComponentNode with GET action + URL wraps in <a href="...">.
        let node = ComponentNode {
            key: "admin-link".to_string(),
            component: Component::Button(ButtonProps {
                label: "View Reports".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: None,
                icon: None,
                icon_position: None,
                        button_type: None,
            }),
            action: Some(Action {
                handler: "reports.index".to_string(),
                url: Some("/reports".to_string()),
                method: HttpMethod::Get,
                confirm: None,
                on_success: None,
                on_error: None,
            }),
            visibility: Some(Visibility::Condition(VisibilityCondition {
                path: "/auth/user/role".to_string(),
                operator: VisibilityOperator::Eq,
                value: Some(serde_json::Value::String("admin".to_string())),
            })),
        };
        let view = JsonUiView::new().component(node);
        let html = render_to_html(&view, &json!({}));

        // GET action with URL wraps the component in <a href="...">
        assert!(
            html.contains("View Reports"),
            "button label should be rendered"
        );
        assert!(
            html.contains("href=\"/reports\""),
            "GET action with URL should produce anchor href"
        );
        assert!(
            html.contains("<a "),
            "GET action should wrap component in anchor tag"
        );
    }

    // ── Grid ──────────────────────────────────────────────────────────

    #[test]
    fn grid_renders_columns_and_gap() {
        let view = JsonUiView::new().component(ComponentNode::grid(
            "g",
            crate::component::GridProps {
                columns: 4,
                md_columns: None,
                gap: crate::component::GapSize::Lg,
                scrollable: None,
                children: vec![text_node("c1", "Cell 1", TextElement::P)],
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("grid grid-cols-4 gap-6"));
        assert!(html.contains("Cell 1"));
    }

    #[test]
    fn grid_clamps_columns() {
        let view = JsonUiView::new().component(ComponentNode::grid(
            "g",
            crate::component::GridProps {
                columns: 20,
                md_columns: None,
                gap: crate::component::GapSize::default(),
                scrollable: None,
                children: vec![],
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("grid-cols-12"));
    }

    #[test]
    fn grid_responsive_md_columns() {
        let view = JsonUiView::new().component(ComponentNode::grid(
            "g",
            crate::component::GridProps {
                columns: 1,
                md_columns: Some(3),
                gap: crate::component::GapSize::Md,
                scrollable: None,
                children: vec![text_node("c1", "Cell 1", TextElement::P)],
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("grid-cols-1 md:grid-cols-3"));
    }

    #[test]
    fn grid_scrollable_renders_overflow() {
        let view = JsonUiView::new().component(ComponentNode::grid(
            "g",
            crate::component::GridProps {
                columns: 3,
                md_columns: None,
                gap: crate::component::GapSize::Md,
                scrollable: Some(true),
                children: vec![
                    text_node("c1", "Col 1", TextElement::P),
                    text_node("c2", "Col 2", TextElement::P),
                ],
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(
            html.contains("overflow-x-auto"),
            "should wrap with overflow-x-auto"
        );
        assert!(
            html.contains("grid-flow-col"),
            "should use grid-flow-col for scrollable"
        );
        assert!(
            html.contains("auto-cols-[minmax(280px,1fr)]"),
            "should use auto-cols"
        );
        assert!(html.contains("Col 1"));
        assert!(html.contains("Col 2"));
    }

    #[test]
    fn grid_non_scrollable_unchanged() {
        let view = JsonUiView::new().component(ComponentNode::grid(
            "g",
            crate::component::GridProps {
                columns: 3,
                md_columns: None,
                gap: crate::component::GapSize::Md,
                scrollable: None,
                children: vec![text_node("c1", "Cell 1", TextElement::P)],
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(
            html.contains("grid grid-cols-3 gap-4"),
            "non-scrollable should use grid-cols-N"
        );
        assert!(
            !html.contains("overflow-x-auto"),
            "non-scrollable should not have overflow-x-auto"
        );
        assert!(
            !html.contains("grid-flow-col"),
            "non-scrollable should not have grid-flow-col"
        );
    }

    #[test]
    fn form_guard_renders_data_attribute() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "f".to_string(),
            component: Component::Form(crate::component::FormProps {
                action: Action {
                    handler: "orders.create".to_string(),
                    url: Some("/orders".to_string()),
                    method: HttpMethod::Post,
                    confirm: None,
                    on_success: None,
                    on_error: None,
                },
                fields: vec![],
                method: None,
                guard: Some("number-gt-0".to_string()),
                max_width: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(
            html.contains("data-form-guard=\"number-gt-0\""),
            "form with guard should render data-form-guard attribute"
        );
    }

    #[test]
    fn form_without_guard_unchanged() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "f".to_string(),
            component: Component::Form(crate::component::FormProps {
                action: Action {
                    handler: "orders.create".to_string(),
                    url: Some("/orders".to_string()),
                    method: HttpMethod::Post,
                    confirm: None,
                    on_success: None,
                    on_error: None,
                },
                fields: vec![],
                method: None,
                guard: None,
                max_width: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(
            !html.contains("data-form-guard"),
            "form without guard should not render data-form-guard attribute"
        );
    }

    // ── Collapsible ───────────────────────────────────────────────────

    #[test]
    fn collapsible_renders_details_summary() {
        let view = JsonUiView::new().component(ComponentNode::collapsible(
            "c",
            crate::component::CollapsibleProps {
                title: "More info".into(),
                expanded: false,
                children: vec![text_node("t", "Hidden text", TextElement::P)],
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<details"));
        assert!(!html.contains(" open"));
        assert!(html.contains("<summary"));
        assert!(html.contains("More info"));
        assert!(html.contains("Hidden text"));
    }

    #[test]
    fn collapsible_expanded_has_open() {
        let view = JsonUiView::new().component(ComponentNode::collapsible(
            "c",
            crate::component::CollapsibleProps {
                title: "Open".into(),
                expanded: true,
                children: vec![],
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<details") && html.contains(" open"));
    }

    // ── EmptyState ────────────────────────────────────────────────────

    #[test]
    fn empty_state_renders_title_and_description() {
        let view = JsonUiView::new().component(ComponentNode::empty_state(
            "es",
            crate::component::EmptyStateProps {
                title: "No orders".into(),
                description: Some("Create your first order".into()),
                action: None,
                action_label: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("No orders"));
        assert!(html.contains("Create your first order"));
        assert!(!html.contains("<a "));
    }

    #[test]
    fn empty_state_renders_action_link() {
        let view = JsonUiView::new().component(ComponentNode::empty_state(
            "es",
            crate::component::EmptyStateProps {
                title: "Empty".into(),
                description: None,
                action: Some(Action {
                    handler: "orders.new".into(),
                    url: Some("/orders/new".into()),
                    method: HttpMethod::Get,
                    confirm: None,
                    on_success: None,
                    on_error: None,
                }),
                action_label: Some("New order".into()),
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("href=\"/orders/new\""));
        assert!(html.contains("New order"));
    }

    // ── FormSection ───────────────────────────────────────────────────

    #[test]
    fn form_section_renders_fieldset() {
        let view = JsonUiView::new().component(ComponentNode::form_section(
            "fs",
            crate::component::FormSectionProps {
                title: "Contact".into(),
                description: Some("Enter details".into()),
                children: vec![text_node("n", "Name field", TextElement::P)],
                layout: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<fieldset"));
        assert!(html.contains("<legend"));
        assert!(html.contains("Contact"));
        assert!(html.contains("Enter details"));
        assert!(html.contains("Name field"));
    }

    // ── Switch auto-submit ────────────────────────────────────────────

    #[test]
    fn switch_with_action_renders_form() {
        let view = JsonUiView::new().component(ComponentNode::switch(
            "sw",
            SwitchProps {
                field: "active".into(),
                label: "Active".into(),
                description: None,
                checked: Some(true),
                data_path: None,
                required: None,
                disabled: None,
                error: None,
                action: Some(Action {
                    handler: "settings.toggle".into(),
                    url: Some("/settings/toggle".into()),
                    method: HttpMethod::Post,
                    confirm: None,
                    on_success: None,
                    on_error: None,
                }),
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<form action=\"/settings/toggle\" method=\"post\">"));
        assert!(html.contains("onchange=\"this.closest('form').submit()\""));
        assert!(html.contains("</form>"));
    }

    #[test]
    fn switch_without_action_no_form() {
        let view = JsonUiView::new().component(ComponentNode::switch(
            "sw",
            SwitchProps {
                field: "f".into(),
                label: "L".into(),
                description: None,
                checked: None,
                data_path: None,
                required: None,
                disabled: None,
                error: None,
                action: None,
            },
        ));
        let html = render_to_html(&view, &json!({}));
        assert!(!html.contains("<form"));
        assert!(!html.contains("onchange"));
    }

    // ── PageHeader ──────────────────────────────────────────────────────

    #[test]
    fn test_render_page_header_title_only() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "ph".to_string(),
            component: Component::PageHeader(PageHeaderProps {
                title: "My Page".to_string(),
                breadcrumb: vec![],
                actions: vec![],
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("pb-4"), "flex container with pb-4");
        assert!(html.contains(
            "<h2 class=\"text-2xl font-semibold leading-tight tracking-tight text-text\">My Page</h2>"
        ));
        assert!(!html.contains("<nav"), "no breadcrumb nav when empty");
        assert!(!html.contains("flex-shrink-0"), "no actions div when empty");
    }

    #[test]
    fn test_render_page_header_with_breadcrumb() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "ph".to_string(),
            component: Component::PageHeader(PageHeaderProps {
                title: "Users".to_string(),
                breadcrumb: vec![
                    BreadcrumbItem {
                        label: "Home".to_string(),
                        url: Some("/".to_string()),
                    },
                    BreadcrumbItem {
                        label: "Users".to_string(),
                        url: None,
                    },
                ],
                actions: vec![],
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html
            .contains("<nav class=\"flex items-center space-x-2 text-sm text-text-muted mb-1\">"));
        assert!(html.contains("<a href=\"/\" class=\"hover:text-text\">Home</a>"));
        assert!(html.contains("<span>Users</span>"));
        assert!(
            html.contains("<svg"),
            "SVG chevron separator between breadcrumb items"
        );
    }

    #[test]
    fn test_render_page_header_with_actions() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "ph".to_string(),
            component: Component::PageHeader(PageHeaderProps {
                title: "Dashboard".to_string(),
                breadcrumb: vec![],
                actions: vec![ComponentNode {
                    key: "add-btn".to_string(),
                    component: Component::Button(ButtonProps {
                        label: "Add New".to_string(),
                        variant: ButtonVariant::Default,
                        size: Size::Default,
                        disabled: None,
                        icon: None,
                        icon_position: None,
                                        button_type: None,
                    }),
                    action: None,
                    visibility: None,
                }],
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(
            html.contains("flex-shrink-0"),
            "actions wrapper with flex-shrink-0"
        );
        assert!(
            html.contains(">Add New</button>"),
            "action button rendered inside"
        );
    }

    // ── ButtonGroup ─────────────────────────────────────────────────────

    #[test]
    fn test_render_button_group() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "bg".to_string(),
            component: Component::ButtonGroup(ButtonGroupProps {
                buttons: vec![
                    ComponentNode {
                        key: "save".to_string(),
                        component: Component::Button(ButtonProps {
                            label: "Save".to_string(),
                            variant: ButtonVariant::Default,
                            size: Size::Default,
                            disabled: None,
                            icon: None,
                            icon_position: None,
                                                button_type: None,
                        }),
                        action: None,
                        visibility: None,
                    },
                    ComponentNode {
                        key: "cancel".to_string(),
                        component: Component::Button(ButtonProps {
                            label: "Cancel".to_string(),
                            variant: ButtonVariant::Outline,
                            size: Size::Default,
                            disabled: None,
                            icon: None,
                            icon_position: None,
                                                button_type: None,
                        }),
                        action: None,
                        visibility: None,
                    },
                ],
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(
            html.contains("flex items-center gap-2 flex-wrap"),
            "horizontal flex container"
        );
        assert!(html.contains(">Save</button>"));
        assert!(html.contains(">Cancel</button>"));
    }

    #[test]
    fn test_render_button_group_empty() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "bg".to_string(),
            component: Component::ButtonGroup(ButtonGroupProps { buttons: vec![] }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<div class=\"flex items-center gap-2 flex-wrap\"></div>"));
    }

    // ── Select appearance-none fix ──────────────────────────────────────

    #[test]
    fn test_render_select_appearance_none() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "sel".to_string(),
            component: Component::Select(SelectProps {
                field: "role".to_string(),
                label: "Role".to_string(),
                options: vec![SelectOption {
                    value: "admin".to_string(),
                    label: "Admin".to_string(),
                }],
                placeholder: None,
                required: None,
                disabled: None,
                error: None,
                description: None,
                default_value: None,
                data_path: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(
            html.contains("appearance-none"),
            "select must have appearance-none class"
        );
        assert!(
            html.contains("bg-background"),
            "select must have bg-background class"
        );
    }

    // ── Single-tab auto-hide ────────────────────────────────────────────

    #[test]
    fn test_render_tabs_single_tab() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "tabs".to_string(),
            component: Component::Tabs(TabsProps {
                default_tab: "only".to_string(),
                tabs: vec![Tab {
                    value: "only".to_string(),
                    label: "Only Tab".to_string(),
                    children: vec![ComponentNode {
                        key: "txt".to_string(),
                        component: Component::Text(TextProps {
                            content: "Content here".to_string(),
                            element: TextElement::P,
                        }),
                        action: None,
                        visibility: None,
                    }],
                }],
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(
            !html.contains("data-tabs"),
            "no data-tabs wrapper for single tab"
        );
        assert!(
            !html.contains("role=\"tablist\""),
            "no tab nav for single tab"
        );
        assert!(html.contains("Content here"), "tab content still rendered");
    }

    #[test]
    fn test_render_tabs_multi_still_works() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "tabs".to_string(),
            component: Component::Tabs(TabsProps {
                default_tab: "tab1".to_string(),
                tabs: vec![
                    Tab {
                        value: "tab1".to_string(),
                        label: "Tab One".to_string(),
                        children: vec![ComponentNode {
                            key: "t1".to_string(),
                            component: Component::Text(TextProps {
                                content: "Tab 1 content".to_string(),
                                element: TextElement::P,
                            }),
                            action: None,
                            visibility: None,
                        }],
                    },
                    Tab {
                        value: "tab2".to_string(),
                        label: "Tab Two".to_string(),
                        children: vec![ComponentNode {
                            key: "t2".to_string(),
                            component: Component::Text(TextProps {
                                content: "Tab 2 content".to_string(),
                                element: TextElement::P,
                            }),
                            action: None,
                            visibility: None,
                        }],
                    },
                ],
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(
            html.contains("data-tabs"),
            "multi-tab still has data-tabs wrapper"
        );
        assert!(
            html.contains("role=\"tablist\""),
            "multi-tab still has nav with role=tablist"
        );
        assert!(html.contains("Tab One"), "tab label rendered");
        assert!(html.contains("Tab Two"), "tab label rendered");
    }

    // ── Structural test helpers ───────────────────────────────────────────

    /// Check that rendered HTML contains an element with a specific CSS class.
    /// Checks class as: sole class, first class, middle class, or last class.
    /// More resilient than full class string matching — survives class additions.
    fn has_class(html: &str, class: &str) -> bool {
        html.contains(&format!("class=\"{class}\""))
            || html.contains(&format!("class=\"{class} "))
            || html.contains(&format!(" {class}\""))
            || html.contains(&format!(" {class} "))
    }

    /// Assert HTML contains a specific element tag and content string.
    fn assert_element(html: &str, tag: &str, content: &str) {
        assert!(
            html.contains(&format!("<{tag} ")) || html.contains(&format!("<{tag}>")),
            "expected <{tag}> element in HTML"
        );
        assert!(
            html.contains(content),
            "expected content '{content}' in HTML"
        );
    }

    // ── Structural tests — survive cosmetic class additions ───────────────
    //
    // These tests verify element type, text content, and semantic token classes
    // without matching the full class attribute string. Adding classes like
    // `leading-tight` or `bg-card` will not break these tests.
    //
    // The existing full-string tests above remain as documentation of current
    // class output and will be updated in Phases 103-107 as classes change.

    mod structural_tests {
        use super::*;
        use serde_json::json;

        // ── Text H1 (Phase 104 adds leading-tight tracking-tight) ─────────

        #[test]
        fn h1_structural_element_and_semantic_class() {
            let view = JsonUiView::new().component(text_node("t", "Page Title", TextElement::H1));
            let html = render_to_html(&view, &json!({}));
            assert_element(&html, "h1", "Page Title");
            assert!(
                has_class(&html, "text-text"),
                "h1 should have text-text class"
            );
        }

        // ── Text H2 (Phase 104 adds leading-tight tracking-tight) ─────────

        #[test]
        fn h2_structural_element_and_semantic_class() {
            let view =
                JsonUiView::new().component(text_node("t", "Section Title", TextElement::H2));
            let html = render_to_html(&view, &json!({}));
            assert_element(&html, "h2", "Section Title");
            assert!(
                has_class(&html, "text-text"),
                "h2 should have text-text class"
            );
        }

        // ── Text H3 (Phase 104 adds leading-snug) ─────────────────────────

        #[test]
        fn h3_structural_element_and_semantic_class() {
            let view = JsonUiView::new().component(text_node("t", "Subsection", TextElement::H3));
            let html = render_to_html(&view, &json!({}));
            assert_element(&html, "h3", "Subsection");
            assert!(
                has_class(&html, "text-text"),
                "h3 should have text-text class"
            );
        }

        // ── Text P (Phase 104 adds leading-relaxed) ───────────────────────

        #[test]
        fn p_structural_element_and_semantic_class() {
            let view = JsonUiView::new().component(text_node("t", "Body text", TextElement::P));
            let html = render_to_html(&view, &json!({}));
            assert_element(&html, "p", "Body text");
            assert!(
                has_class(&html, "text-text"),
                "p should have text-text class"
            );
        }

        // ── Card (Phase 103 adds bg-card) ─────────────────────────────────

        #[test]
        fn card_structural_title_and_description() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "c".to_string(),
                component: Component::Card(CardProps {
                    title: "Card Title".to_string(),
                    description: Some("Card description".to_string()),
                    children: vec![],
                    footer: vec![],
                max_width: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(html.contains("<div"), "card should render a div container");
            assert!(html.contains("Card Title"), "card title should be present");
            assert!(
                html.contains("Card description"),
                "card description should be present"
            );
            assert!(
                has_class(&html, "border-border"),
                "card container should have border-border"
            );
        }

        // ── Alert (Phase 107 adds SVG icon) ──────────────────────────────

        #[test]
        fn alert_structural_container_and_message() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "a".to_string(),
                component: Component::Alert(AlertProps {
                    message: "Something went wrong".to_string(),
                    variant: AlertVariant::Warning,
                    title: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(
                html.contains("role=\"alert\""),
                "alert should have role=alert"
            );
            assert!(
                html.contains("Something went wrong"),
                "alert message should be present"
            );
            assert!(
                has_class(&html, "text-warning"),
                "warning alert should have text-warning class"
            );
        }

        // ── Input (Phase 105 adds transitions, disabled states) ───────────

        #[test]
        fn input_structural_element_and_type() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "i".to_string(),
                component: Component::Input(InputProps {
                    field: "username".to_string(),
                    label: "Username".to_string(),
                    input_type: InputType::Text,
                    placeholder: None,
                    required: None,
                    disabled: None,
                    error: None,
                    description: None,
                    default_value: None,
                    data_path: None,
                    step: None,
                    list: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(
                html.contains("<input"),
                "input should render an <input element"
            );
            assert!(html.contains("type=\"text\""), "input type should be text");
            assert!(
                html.contains("name=\"username\""),
                "input name should match field"
            );
        }

        // ── Select (Phase 105 adds custom arrow styling) ──────────────────

        #[test]
        fn select_structural_element_and_options() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "s".to_string(),
                component: Component::Select(SelectProps {
                    field: "status".to_string(),
                    label: "Status".to_string(),
                    options: vec![
                        SelectOption {
                            value: "active".to_string(),
                            label: "Active".to_string(),
                        },
                        SelectOption {
                            value: "inactive".to_string(),
                            label: "Inactive".to_string(),
                        },
                    ],
                    placeholder: None,
                    required: None,
                    disabled: None,
                    error: None,
                    description: None,
                    default_value: None,
                    data_path: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(
                html.contains("<select"),
                "select should render a <select element"
            );
            assert!(
                html.contains("Active"),
                "select should render option labels"
            );
            assert!(
                html.contains("Inactive"),
                "select should render all options"
            );
        }

        // ── Table (Phase 106 adds hover:bg-surface to rows) ───────────────

        #[test]
        fn table_structural_headers_and_body() {
            let data = json!({
                "items": [{"name": "Widget", "price": "9.99"}]
            });
            let view = JsonUiView::new().component(ComponentNode {
                key: "t".to_string(),
                component: Component::Table(TableProps {
                    columns: vec![
                        Column {
                            key: "name".to_string(),
                            label: "Name".to_string(),
                            format: None,
                        },
                        Column {
                            key: "price".to_string(),
                            label: "Price".to_string(),
                            format: None,
                        },
                    ],
                    data_path: "/items".to_string(),
                    row_actions: None,
                    empty_message: None,
                    sortable: None,
                    sort_column: None,
                    sort_direction: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &data);
            assert!(
                html.contains("<table"),
                "table should render a <table element"
            );
            assert!(html.contains("<th"), "table should render header cells");
            assert!(
                html.contains("Name"),
                "table should render Name column header"
            );
            assert!(
                html.contains("Price"),
                "table should render Price column header"
            );
            assert!(html.contains("Widget"), "table should render row data");
        }

        // ── Breadcrumb (Phase 107 changes separator) ──────────────────────

        #[test]
        fn breadcrumb_structural_nav_and_links() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "bc".to_string(),
                component: Component::Breadcrumb(BreadcrumbProps {
                    items: vec![
                        BreadcrumbItem {
                            label: "Home".to_string(),
                            url: Some("/".to_string()),
                        },
                        BreadcrumbItem {
                            label: "Products".to_string(),
                            url: Some("/products".to_string()),
                        },
                        BreadcrumbItem {
                            label: "Widget".to_string(),
                            url: None,
                        },
                    ],
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(
                html.contains("<nav"),
                "breadcrumb should render a <nav element"
            );
            assert!(
                html.contains("href=\"/\""),
                "breadcrumb should render Home link"
            );
            assert!(
                html.contains("href=\"/products\""),
                "breadcrumb should render Products link"
            );
            assert!(
                html.contains("Widget"),
                "breadcrumb should render last item text"
            );
        }

        // ── Tabs (Phase 107 adds font-semibold to active tab) ─────────────

        #[test]
        fn tabs_structural_buttons_and_content() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "tabs".to_string(),
                component: Component::Tabs(TabsProps {
                    default_tab: "overview".to_string(),
                    tabs: vec![
                        Tab {
                            value: "overview".to_string(),
                            label: "Overview".to_string(),
                            children: vec![text_node("t1", "Overview content", TextElement::P)],
                        },
                        Tab {
                            value: "details".to_string(),
                            label: "Details".to_string(),
                            children: vec![text_node("t2", "Details content", TextElement::P)],
                        },
                    ],
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(
                html.contains("<button"),
                "tabs should render button elements"
            );
            assert!(html.contains("Overview"), "tabs should render tab labels");
            assert!(
                html.contains("Details"),
                "tabs should render all tab labels"
            );
            assert!(
                html.contains("Overview content"),
                "tabs should render active tab content"
            );
        }

        // ── StatCard (Phase 103 adds bg-card) ────────────────────────────

        #[test]
        fn stat_card_structural_value_and_label() {
            let view = JsonUiView::new().component(ComponentNode::stat_card(
                "sales",
                StatCardProps {
                    label: "Total Sales".to_string(),
                    value: "1,024".to_string(),
                    icon: None,
                    subtitle: None,
                    sse_target: None,
                },
            ));
            let html = render_to_html(&view, &json!({}));
            assert!(
                html.contains("Total Sales"),
                "stat card should render label"
            );
            assert!(html.contains("1,024"), "stat card should render value");
            assert!(
                has_class(&html, "rounded-lg"),
                "stat card should have rounded-lg class"
            );
        }

        // ── Skeleton (Phase 107 changes animation class) ──────────────────

        #[test]
        fn skeleton_structural_animate_class() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "sk".to_string(),
                component: Component::Skeleton(SkeletonProps {
                    width: None,
                    height: None,
                    rounded: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(html.contains("<div"), "skeleton should render a div");
            assert!(
                html.contains("ferro-shimmer"),
                "skeleton should have ferro-shimmer class"
            );
        }

        // ── Collapsible (Phase 107 adds SVG chevron) ──────────────────────

        #[test]
        fn collapsible_structural_details_element() {
            let view = JsonUiView::new().component(ComponentNode::collapsible(
                "col",
                crate::component::CollapsibleProps {
                    title: "Show more".into(),
                    expanded: false,
                    children: vec![text_node("t", "Collapsed content", TextElement::P)],
                },
            ));
            let html = render_to_html(&view, &json!({}));
            assert!(
                html.contains("<details"),
                "collapsible should render a <details element"
            );
            assert!(
                html.contains("Show more"),
                "collapsible should render the title"
            );
            assert!(
                html.contains("Collapsed content"),
                "collapsible should render children"
            );
        }

        // ── CMP-01: Alert SVG icon per variant (Phase 107) ───────────────────

        #[test]
        fn alert_svg_icon_per_variant() {
            use crate::component::AlertVariant;
            let variants = [
                AlertVariant::Info,
                AlertVariant::Success,
                AlertVariant::Warning,
                AlertVariant::Error,
            ];
            for variant in variants {
                let view = JsonUiView::new().component(ComponentNode {
                    key: "a".to_string(),
                    component: Component::Alert(AlertProps {
                        variant,
                        title: None,
                        message: "Test message".to_string(),
                    }),
                    action: None,
                    visibility: None,
                });
                let html = render_to_html(&view, &json!({}));
                assert!(html.contains("<svg"), "alert should contain SVG icon");
                assert!(
                    html.contains("role=\"alert\""),
                    "alert should preserve accessibility role"
                );
                assert!(
                    has_class(&html, "flex"),
                    "alert container should have flex class"
                );
            }
        }

        // ── CMP-02: Skeleton shimmer class (Phase 107) ────────────────────

        #[test]
        fn skeleton_shimmer_class() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "sk".to_string(),
                component: Component::Skeleton(SkeletonProps {
                    width: None,
                    height: None,
                    rounded: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(
                html.contains("ferro-shimmer"),
                "shimmer class should be present"
            );
            assert!(
                !html.contains("animate-pulse"),
                "old pulse class should be removed"
            );
            assert!(
                html.contains("@keyframes ferro-shimmer"),
                "CSS keyframe should be injected"
            );
        }

        // ── CMP-03: Breadcrumb SVG separator (Phase 107) ─────────────────

        #[test]
        fn breadcrumb_svg_separator() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "bc".to_string(),
                component: Component::Breadcrumb(BreadcrumbProps {
                    items: vec![
                        BreadcrumbItem {
                            label: "Home".to_string(),
                            url: Some("/".to_string()),
                        },
                        BreadcrumbItem {
                            label: "Products".to_string(),
                            url: Some("/products".to_string()),
                        },
                        BreadcrumbItem {
                            label: "Detail".to_string(),
                            url: None,
                        },
                    ],
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(html.contains("<svg"), "SVG separator should be present");
            assert!(
                !html.contains("<span>/</span>"),
                "old text separator should be removed"
            );
            assert!(
                html.contains("aria-hidden"),
                "separator should be decorative (aria-hidden)"
            );
        }

        // ── CMP-04: Active tab font-semibold (Phase 107) ─────────────────

        #[test]
        fn tab_active_font_semibold() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "tabs".to_string(),
                component: Component::Tabs(TabsProps {
                    default_tab: "tab1".to_string(),
                    tabs: vec![
                        Tab {
                            value: "tab1".to_string(),
                            label: "First Tab".to_string(),
                            children: vec![text_node("t1", "Content one", TextElement::P)],
                        },
                        Tab {
                            value: "tab2".to_string(),
                            label: "Second Tab".to_string(),
                            children: vec![text_node("t2", "Content two", TextElement::P)],
                        },
                    ],
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(
                has_class(&html, "font-semibold"),
                "active tab should have font-semibold class"
            );
            let count = html.matches("font-semibold").count();
            assert_eq!(count, 1, "only the active tab should have font-semibold");
        }

        // ── CMP-05: Notification bell SVG (Phase 107) ─────────────────────

        #[test]
        fn notification_bell_svg() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "nd".to_string(),
                component: Component::NotificationDropdown(NotificationDropdownProps {
                    notifications: vec![crate::component::NotificationItem {
                        text: "New message".to_string(),
                        read: false,
                        icon: None,
                        action_url: None,
                        timestamp: None,
                    }],
                    empty_text: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(html.contains("<svg"), "SVG bell should be present");
            assert!(
                !html.contains("&#x1F514;"),
                "bell emoji entity should be removed"
            );
        }

        // ── CMP-06: Collapsible SVG chevron (Phase 107) ───────────────────

        #[test]
        fn collapsible_svg_chevron() {
            let view = JsonUiView::new().component(ComponentNode::collapsible(
                "col",
                crate::component::CollapsibleProps {
                    title: "Section".into(),
                    expanded: false,
                    children: vec![text_node("t", "Body text", TextElement::P)],
                },
            ));
            let html = render_to_html(&view, &json!({}));
            assert!(html.contains("<svg"), "SVG chevron should be present");
            assert!(
                !html.contains("&#9660;"),
                "old down-arrow entity should be removed"
            );
            assert!(
                has_class(&html, "group-open:rotate-180"),
                "rotation class should be preserved"
            );
            assert!(
                has_class(&html, "transition-transform"),
                "transition class should be preserved"
            );
        }

        // ── Form polish (Phase 105) ────────────────────────────────────────

        #[test]
        fn select_renders_chevron_wrapper() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "s".to_string(),
                component: Component::Select(SelectProps {
                    field: "role".to_string(),
                    label: "Role".to_string(),
                    options: vec![],
                    placeholder: None,
                    required: None,
                    disabled: None,
                    error: None,
                    description: None,
                    default_value: None,
                    data_path: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(
                html.contains("<div class=\"relative\">"),
                "select should be wrapped in relative div"
            );
            assert!(
                html.contains("aria-hidden=\"true\""),
                "SVG span should have aria-hidden"
            );
            assert!(
                html.contains("<svg"),
                "inline SVG chevron should be present"
            );
            assert!(
                html.contains("pointer-events-none"),
                "SVG span should be non-interactive"
            );
            assert!(has_class(&html, "pr-10"), "select should have pr-10 class");
        }

        #[test]
        fn input_renders_transition_classes() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "i".to_string(),
                component: Component::Input(InputProps {
                    field: "name".to_string(),
                    label: "Name".to_string(),
                    input_type: InputType::Text,
                    placeholder: None,
                    required: None,
                    disabled: None,
                    error: None,
                    description: None,
                    default_value: None,
                    data_path: None,
                    step: None,
                    list: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(
                has_class(&html, "transition-colors"),
                "input should have transition-colors"
            );
            assert!(
                has_class(&html, "duration-150"),
                "input should have duration-150"
            );
            assert!(
                html.contains("motion-reduce:transition-none"),
                "input should support reduced motion"
            );
        }

        #[test]
        fn input_disabled_renders_disabled_classes() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "i".to_string(),
                component: Component::Input(InputProps {
                    field: "name".to_string(),
                    label: "Name".to_string(),
                    input_type: InputType::Text,
                    placeholder: None,
                    required: None,
                    disabled: Some(true),
                    error: None,
                    description: None,
                    default_value: None,
                    data_path: None,
                    step: None,
                    list: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(
                html.contains("disabled:opacity-50"),
                "input should have disabled:opacity-50"
            );
            assert!(
                html.contains("disabled:cursor-not-allowed"),
                "input should have disabled:cursor-not-allowed"
            );
            assert!(
                html.contains(" disabled"),
                "input should have disabled HTML attribute"
            );
        }

        #[test]
        fn textarea_renders_error_focus_ring() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "i".to_string(),
                component: Component::Input(InputProps {
                    field: "bio".to_string(),
                    label: "Bio".to_string(),
                    input_type: InputType::Textarea,
                    placeholder: None,
                    required: None,
                    disabled: None,
                    error: Some("Required".to_string()),
                    description: None,
                    default_value: None,
                    data_path: None,
                    step: None,
                    list: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(
                html.contains("ring-destructive"),
                "textarea with error should have ring-destructive"
            );
        }

        #[test]
        fn input_description_order() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "i".to_string(),
                component: Component::Input(InputProps {
                    field: "name".to_string(),
                    label: "Name".to_string(),
                    input_type: InputType::Text,
                    placeholder: None,
                    required: None,
                    disabled: None,
                    error: None,
                    description: Some("Help text".to_string()),
                    default_value: None,
                    data_path: None,
                    step: None,
                    list: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            let input_pos = html.find("<input").expect("input element should exist");
            let desc_pos = html.find("Help text").expect("description should exist");
            assert!(
                input_pos < desc_pos,
                "input should appear before description in DOM"
            );
        }

        #[test]
        fn select_description_order() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "s".to_string(),
                component: Component::Select(SelectProps {
                    field: "role".to_string(),
                    label: "Role".to_string(),
                    options: vec![],
                    placeholder: None,
                    required: None,
                    disabled: None,
                    error: None,
                    description: Some("Pick one".to_string()),
                    default_value: None,
                    data_path: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            let select_close_pos = html.find("</select>").expect("select close should exist");
            let desc_pos = html.find("Pick one").expect("description should exist");
            assert!(
                select_close_pos < desc_pos,
                "select close should appear before description in DOM"
            );
        }

        // ── Button (Phase 106 adds focus-visible ring) ────────────────────

        #[test]
        fn button_structural_element_and_text() {
            let view = JsonUiView::new().component(button_node(
                "btn",
                "Submit",
                ButtonVariant::Default,
                Size::Default,
            ));
            let html = render_to_html(&view, &json!({}));
            assert!(
                html.contains("<button"),
                "button should render a <button element"
            );
            assert!(html.contains("Submit"), "button should render label text");
            assert!(
                has_class(&html, "bg-primary"),
                "default button should have bg-primary class"
            );
        }

        // ── INT-01: Button focus ring and transition ───────────────────────

        #[test]
        fn button_focus_ring() {
            let view = JsonUiView::new().component(button_node(
                "btn",
                "Click me",
                ButtonVariant::Default,
                Size::Default,
            ));
            let html = render_to_html(&view, &json!({}));
            assert!(
                has_class(&html, "focus-visible:ring-primary"),
                "button should have focus-visible:ring-primary class (INT-01)"
            );
            assert!(
                has_class(&html, "duration-150"),
                "button should have duration-150 class (INT-07)"
            );
            assert!(
                html.contains("motion-reduce:transition-none"),
                "button should have motion-reduce:transition-none (INT-07)"
            );
        }

        // ── INT-02: Tabs focus ring and transition ─────────────────────────

        #[test]
        fn tabs_focus_ring() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "tabs".to_string(),
                component: Component::Tabs(TabsProps {
                    default_tab: "tab1".to_string(),
                    tabs: vec![
                        Tab {
                            value: "tab1".to_string(),
                            label: "Tab One".to_string(),
                            children: vec![text_node("t1", "Content one", TextElement::P)],
                        },
                        Tab {
                            value: "tab2".to_string(),
                            label: "Tab Two".to_string(),
                            children: vec![text_node("t2", "Content two", TextElement::P)],
                        },
                    ],
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(
                has_class(&html, "focus-visible:ring-primary"),
                "tab button/link should have focus-visible:ring-primary class (INT-02)"
            );
            assert!(
                has_class(&html, "duration-150"),
                "tab button/link should have duration-150 class (INT-07)"
            );
        }

        // ── INT-03: Pagination focus ring and transition ───────────────────

        #[test]
        fn pagination_focus_ring() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "pg".to_string(),
                component: Component::Pagination(PaginationProps {
                    total: 30,
                    per_page: 10,
                    current_page: 2,
                    base_url: Some("?".to_string()),
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(
                has_class(&html, "focus-visible:ring-primary"),
                "pagination <a> links should have focus-visible:ring-primary class (INT-03)"
            );
            assert!(
                has_class(&html, "duration-150"),
                "pagination <a> links should have duration-150 class (INT-07)"
            );
        }

        // ── INT-04: Breadcrumb focus ring and transition ───────────────────

        #[test]
        fn breadcrumb_focus_ring() {
            let view = JsonUiView::new().component(ComponentNode {
                key: "bc".to_string(),
                component: Component::Breadcrumb(BreadcrumbProps {
                    items: vec![
                        BreadcrumbItem {
                            label: "Home".to_string(),
                            url: Some("/".to_string()),
                        },
                        BreadcrumbItem {
                            label: "Current".to_string(),
                            url: None,
                        },
                    ],
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &json!({}));
            assert!(
                has_class(&html, "focus-visible:ring-primary"),
                "breadcrumb <a> link should have focus-visible:ring-primary class (INT-04)"
            );
            assert!(
                has_class(&html, "duration-150"),
                "breadcrumb <a> link should have duration-150 class (INT-07)"
            );
        }

        // ── INT-05: Sidebar nav item focus ring and transition ─────────────

        #[test]
        fn sidebar_nav_focus_ring() {
            let view = JsonUiView::new().component(ComponentNode::sidebar(
                "nav",
                SidebarProps {
                    fixed_top: vec![SidebarNavItem {
                        label: "Dashboard".to_string(),
                        href: "/dashboard".to_string(),
                        icon: None,
                        active: false,
                    }],
                    groups: vec![],
                    fixed_bottom: vec![],
                },
            ));
            let html = render_to_html(&view, &json!({}));
            assert!(
                has_class(&html, "focus-visible:ring-primary"),
                "sidebar nav <a> item should have focus-visible:ring-primary class (INT-05)"
            );
            assert!(
                has_class(&html, "duration-150"),
                "sidebar nav <a> item should have duration-150 class (INT-07)"
            );
        }

        // ── INT-06: Table body row hover ───────────────────────────────────

        #[test]
        fn table_row_hover() {
            let data = json!({"items": [{"name": "Alice"}]});
            let view = JsonUiView::new().component(ComponentNode {
                key: "t".to_string(),
                component: Component::Table(TableProps {
                    columns: vec![Column {
                        key: "name".to_string(),
                        label: "Name".to_string(),
                        format: None,
                    }],
                    data_path: "/items".to_string(),
                    row_actions: None,
                    empty_message: None,
                    sortable: None,
                    sort_column: None,
                    sort_direction: None,
                }),
                action: None,
                visibility: None,
            });
            let html = render_to_html(&view, &data);
            assert!(
                html.contains("<tr class=\"hover:bg-surface\">"),
                "table body row should have hover:bg-surface class (INT-06)"
            );
        }
    }

    // ── DropdownMenu tests ───────────────────────────────────────────────

    #[test]
    fn test_render_dropdown_menu() {
        use crate::action::ConfirmDialog;

        let props = DropdownMenuProps {
            menu_id: "actions-1".to_string(),
            trigger_label: "Azioni".to_string(),
            items: vec![
                DropdownMenuAction {
                    label: "Modifica".to_string(),
                    action: Action {
                        handler: "items.edit".to_string(),
                        url: Some("/items/1/edit".to_string()),
                        method: HttpMethod::Get,
                        confirm: None,
                        on_success: None,
                        on_error: None,
                    },
                    destructive: false,
                },
                DropdownMenuAction {
                    label: "Elimina".to_string(),
                    action: Action {
                        handler: "items.destroy".to_string(),
                        url: Some("/items/1".to_string()),
                        method: HttpMethod::Delete,
                        confirm: None,
                        on_success: None,
                        on_error: None,
                    },
                    destructive: true,
                },
            ],
            trigger_variant: None,
        };

        let view = JsonUiView::new().component(ComponentNode::dropdown_menu("menu", props));
        let html = render_to_html(&view, &json!({}));

        assert!(html.contains("data-dropdown-toggle=\"actions-1\""), "trigger has data-dropdown-toggle");
        assert!(html.contains("data-dropdown=\"actions-1\""), "panel has data-dropdown");
        assert!(html.contains("hidden"), "panel starts hidden");
        assert!(html.contains("text-destructive"), "destructive item has text-destructive class");
        assert!(html.contains("type=\"button\""), "trigger is type=button");
        assert!(html.contains("aria-label=\"Azioni\""), "trigger has aria-label");
        assert!(html.contains("Modifica"), "normal item label present");
        assert!(html.contains("Elimina"), "destructive item label present");
        // GET action renders as <a>, DELETE renders as <form>
        assert!(html.contains("<a href=\"/items/1/edit\""), "GET action renders as link");
        assert!(html.contains("<form action=\"/items/1\" method=\"post\">"), "DELETE action renders as form");
        assert!(html.contains("name=\"_method\" value=\"DELETE\""), "DELETE method spoofing");
    }

    #[test]
    fn test_render_dropdown_menu_confirm() {
        use crate::action::{ConfirmDialog, DialogVariant};

        let props = DropdownMenuProps {
            menu_id: "confirm-menu".to_string(),
            trigger_label: "Menu".to_string(),
            items: vec![DropdownMenuAction {
                label: "Elimina".to_string(),
                action: Action {
                    handler: "items.destroy".to_string(),
                    url: Some("/items/1".to_string()),
                    method: HttpMethod::Delete,
                    confirm: Some(ConfirmDialog {
                        title: "Conferma eliminazione".to_string(),
                        message: Some("Sei sicuro?".to_string()),
                        variant: DialogVariant::Danger,
                    }),
                    on_success: None,
                    on_error: None,
                },
                destructive: true,
            }],
            trigger_variant: None,
        };

        let view = JsonUiView::new().component(ComponentNode::dropdown_menu("cm", props));
        let html = render_to_html(&view, &json!({}));

        assert!(html.contains("data-confirm-title=\"Conferma eliminazione\""), "confirm title attribute");
        assert!(html.contains("data-confirm-message=\"Sei sicuro?\""), "confirm message attribute");
        assert!(html.contains("data-confirm"), "has data-confirm attribute");
    }

    // ── KanbanBoard tests ───────────────────────────────────────────────

    fn make_kanban_props() -> KanbanBoardProps {
        use crate::component::{KanbanBoardProps, KanbanColumnProps, CardProps};

        KanbanBoardProps {
            columns: vec![
                KanbanColumnProps {
                    id: "new".to_string(),
                    title: "Nuovi".to_string(),
                    count: 3,
                    children: vec![ComponentNode::card(
                        "card-1",
                        CardProps {
                            title: "Ordine #1".to_string(),
                            description: None,
                            children: vec![],
                            footer: vec![],
                max_width: None,
                        },
                    )],
                },
                KanbanColumnProps {
                    id: "progress".to_string(),
                    title: "In corso".to_string(),
                    count: 1,
                    children: vec![ComponentNode::card(
                        "card-2",
                        CardProps {
                            title: "Ordine #2".to_string(),
                            description: None,
                            children: vec![],
                            footer: vec![],
                max_width: None,
                        },
                    )],
                },
            ],
            mobile_default_column: None,
        }
    }

    #[test]
    fn test_render_kanban_board_desktop() {
        let props = make_kanban_props();
        let view = JsonUiView::new().component(ComponentNode::kanban_board("kb", props));
        let html = render_to_html(&view, &json!({}));

        assert!(html.contains("hidden md:block"), "desktop wrapper present");
        assert!(html.contains("min-w-[280px]"), "column min width");
        assert!(html.contains("overflow-x-auto"), "scrollable container");
        assert!(html.contains("Nuovi"), "first column title");
        assert!(html.contains("In corso"), "second column title");
        assert!(html.contains("bg-primary text-primary-foreground"), "count badge styling");
        assert!(html.contains(">3<"), "first column count");
        assert!(html.contains(">1<"), "second column count");
    }

    #[test]
    fn test_render_kanban_board_mobile() {
        let props = make_kanban_props();
        let view = JsonUiView::new().component(ComponentNode::kanban_board("kb", props));
        let html = render_to_html(&view, &json!({}));

        assert!(html.contains("block md:hidden"), "mobile wrapper present");
        assert!(html.contains("data-tabs"), "tab container attribute");
        assert!(html.contains("data-tab=\"new\""), "first tab button");
        assert!(html.contains("data-tab=\"progress\""), "second tab button");
        assert!(html.contains("data-tab-panel=\"new\""), "first tab panel");
        assert!(html.contains("data-tab-panel=\"progress\""), "second tab panel");
        // Default tab (first) is active
        assert!(html.contains("aria-selected=\"true\""), "default tab selected");
        assert!(html.contains("aria-selected=\"false\""), "non-default tab not selected");
    }

    #[test]
    fn test_render_kanban_board_custom_default_column() {
        use crate::component::KanbanBoardProps;

        let mut props = make_kanban_props();
        props.mobile_default_column = Some("progress".to_string());
        let view = JsonUiView::new().component(ComponentNode::kanban_board("kb", props));
        let html = render_to_html(&view, &json!({}));

        // The "progress" tab should be selected, "new" should not
        // Check that the progress panel is NOT hidden
        assert!(!html.contains("data-tab-panel=\"progress\" class=\"space-y-3 hidden\""), "progress panel visible");
    }

    // ── CalendarCell tests ──────────────────────────────────────────────

    #[test]
    fn test_render_calendar_cell_today() {
        let props = CalendarCellProps {
            day: 15,
            is_today: true,
            is_current_month: true,
            event_count: 0,
        };
        let html = render_calendar_cell(&props);
        assert!(html.contains("bg-primary"), "today has bg-primary");
        assert!(html.contains("text-primary-foreground"), "today has foreground color");
        assert!(html.contains("font-semibold"), "today is bold");
        assert!(html.contains("15"), "shows day number");
    }

    #[test]
    fn test_render_calendar_cell_out_of_month() {
        let props = CalendarCellProps {
            day: 30,
            is_today: false,
            is_current_month: false,
            event_count: 0,
        };
        let html = render_calendar_cell(&props);
        assert!(html.contains("text-text-muted"), "out-of-month is muted");
        assert!(html.contains("opacity-50"), "out-of-month has opacity");
    }

    #[test]
    fn test_render_calendar_cell_events() {
        let props = CalendarCellProps {
            day: 5,
            is_today: false,
            is_current_month: true,
            event_count: 3,
        };
        let html = render_calendar_cell(&props);
        assert!(html.contains(">3</span>"), "shows event count");
        assert!(html.contains("text-primary"), "count styled as primary");
    }

    #[test]
    fn test_render_calendar_cell_single_event_dot() {
        let props = CalendarCellProps {
            day: 5,
            is_today: false,
            is_current_month: true,
            event_count: 1,
        };
        let html = render_calendar_cell(&props);
        assert!(html.contains("w-1.5 h-1.5 rounded-full bg-primary"), "single event shows dot");
    }

    // ── ActionCard tests ────────────────────────────────────────────────

    #[test]
    fn test_render_action_card_default() {
        let props = ActionCardProps {
            title: "Nuovo ordine".into(),
            description: "Crea un ordine".into(),
            icon: Some("📦".into()),
            variant: ActionCardVariant::Default,
            href: None,
        };
        let html = render_action_card(&props);
        assert!(html.contains("border-l-primary"), "default variant has primary border");
        assert!(html.contains("Nuovo ordine"), "shows title");
        assert!(html.contains("Crea un ordine"), "shows description");
        assert!(html.contains("rsaquo"), "shows chevron");
    }

    #[test]
    fn test_render_action_card_setup() {
        let props = ActionCardProps {
            title: "Configura".into(),
            description: "Completa la configurazione".into(),
            icon: None,
            variant: ActionCardVariant::Setup,
            href: None,
        };
        let html = render_action_card(&props);
        assert!(html.contains("border-l-warning"), "setup variant has warning border");
    }

    #[test]
    fn test_render_action_card_danger() {
        let props = ActionCardProps {
            title: "Elimina".into(),
            description: "Elimina questo elemento".into(),
            icon: None,
            variant: ActionCardVariant::Danger,
            href: None,
        };
        let html = render_action_card(&props);
        assert!(html.contains("border-l-destructive"), "danger variant has destructive border");
    }

    // ─── ProductTile tests ────────────────────────────────────────────

    #[test]
    fn test_render_product_tile() {
        let props = ProductTileProps {
            product_id: "p1".into(),
            name: "Margherita".into(),
            price: "\u{20AC}8,50".into(),
            field: "qty_p1".into(),
            default_quantity: None,
        };
        let html = render_product_tile(&props);
        assert!(html.contains("Margherita"), "shows product name");
        assert!(html.contains("\u{20AC}8,50"), "shows price");
        assert!(html.contains("data-qty-inc=\"qty_p1\""), "inc button has data attr");
        assert!(html.contains("data-qty-dec=\"qty_p1\""), "dec button has data attr");
        assert!(html.contains("data-qty-display=\"qty_p1\""), "display span has data attr");
        assert!(html.contains("data-qty-input=\"qty_p1\""), "hidden input has data attr");
        assert!(html.contains("type=\"button\""), "buttons use type=button");
        assert!(html.contains("type=\"hidden\""), "hidden input present");
        assert!(html.contains("min-h-[44px]"), "44px touch target height");
        assert!(html.contains("min-w-[44px]"), "44px touch target width");
        assert!(html.contains("touch-manipulation"), "touch-manipulation on container");
        assert!(html.contains("value=\"0\""), "default quantity is 0");
    }

    #[test]
    fn test_render_product_tile_default_qty() {
        let props = ProductTileProps {
            product_id: "p2".into(),
            name: "Diavola".into(),
            price: "\u{20AC}10,00".into(),
            field: "qty_p2".into(),
            default_quantity: Some(2),
        };
        let html = render_product_tile(&props);
        assert!(html.contains("value=\"2\""), "default quantity is 2");
        assert!(html.contains(">2<"), "display shows 2");
    }

    #[test]
    fn test_render_data_table_rows() {
        let props = DataTableProps {
            columns: vec![
                Column { key: "name".into(), label: "Nome".into(), format: None },
                Column { key: "price".into(), label: "Prezzo".into(), format: None },
            ],
            data_path: "items".into(),
            row_actions: None,
            empty_message: None,
            row_key: None,
        };
        let data = json!({
            "items": [
                {"name": "Margherita", "price": "8.50"},
                {"name": "Diavola", "price": "10.00"}
            ]
        });
        let html = render_data_table(&props, &data);
        assert!(html.contains("hidden md:block"), "desktop wrapper");
        assert!(html.contains("even:bg-surface"), "alternating rows");
        assert!(html.contains("block md:hidden"), "mobile wrapper");
        assert!(html.contains("uppercase"), "column header style");
        assert!(html.contains("Margherita"), "first row value");
        assert!(html.contains("Diavola"), "second row value");
    }

    #[test]
    fn test_render_data_table_with_actions() {
        let props = DataTableProps {
            columns: vec![
                Column { key: "name".into(), label: "Nome".into(), format: None },
            ],
            data_path: "items".into(),
            row_actions: Some(vec![
                DropdownMenuAction {
                    label: "Modifica".into(),
                    action: Action {
                        handler: "edit".into(),
                        method: HttpMethod::Get,
                        url: Some("/edit".into()),
                        confirm: None,
                        on_success: None,
                        on_error: None,
                    },
                    destructive: false,
                },
                DropdownMenuAction {
                    label: "Elimina".into(),
                    action: Action {
                        handler: "delete".into(),
                        method: HttpMethod::Delete,
                        url: Some("/delete".into()),
                        confirm: None,
                        on_success: None,
                        on_error: None,
                    },
                    destructive: true,
                },
            ]),
            empty_message: None,
            row_key: Some("id".into()),
        };
        let data = json!({
            "items": [{"id": "p1", "name": "Margherita"}]
        });
        let html = render_data_table(&props, &data);
        assert!(html.contains("data-dropdown-toggle"), "DropdownMenu trigger present");
        assert!(html.contains("text-destructive"), "destructive action in menu");
    }

    #[test]
    fn test_render_data_table_empty() {
        let props = DataTableProps {
            columns: vec![
                Column { key: "name".into(), label: "Nome".into(), format: None },
            ],
            data_path: "items".into(),
            row_actions: None,
            empty_message: None,
            row_key: None,
        };
        let data = json!({"items": []});
        let html = render_data_table(&props, &data);
        assert!(html.contains("Nessun elemento trovato"), "default empty message");
    }

    #[test]
    fn test_render_data_table_mobile_cards() {
        let props = DataTableProps {
            columns: vec![
                Column { key: "name".into(), label: "Nome".into(), format: None },
                Column { key: "price".into(), label: "Prezzo".into(), format: None },
            ],
            data_path: "items".into(),
            row_actions: None,
            empty_message: None,
            row_key: None,
        };
        let data = json!({
            "items": [
                {"name": "Margherita", "price": "8.50"},
                {"name": "Diavola", "price": "10.00"}
            ]
        });
        let html = render_data_table(&props, &data);
        assert!(html.contains("block md:hidden"), "mobile cards shown");
        assert!(
            html.contains("text-xs font-semibold text-text-muted uppercase"),
            "label styling"
        );
    }

    // ── Modal dialog tests ─────────────────────────────────────────────────

    #[test]
    fn test_render_modal_dialog() {
        let props = ModalProps {
            id: "modal-test".into(),
            title: "Test Title".into(),
            description: None,
            children: vec![],
            footer: vec![],
                max_width: None,
            trigger_label: Some("Open".into()),
        };
        let html = render_modal(&props, &serde_json::Value::Null);
        assert!(html.contains("<dialog"), "uses dialog element");
        assert!(html.contains("aria-modal=\"true\""), "has aria-modal");
        assert!(
            html.contains("aria-labelledby=\"modal-test-title\""),
            "has aria-labelledby"
        );
        assert!(
            html.contains("data-modal-open=\"modal-test\""),
            "trigger has data-modal-open"
        );
        assert!(html.contains("data-modal-close"), "has close button");
        assert!(html.contains("Chiudi"), "close button has Italian aria-label");
        assert!(!html.contains("<details"), "no details element");
        assert!(!html.contains("<summary"), "no summary element");
    }

    #[test]
    fn test_render_modal_with_description() {
        let props = ModalProps {
            id: "modal-desc".into(),
            title: "Title".into(),
            description: Some("A description".into()),
            children: vec![],
            footer: vec![],
                max_width: None,
            trigger_label: None,
        };
        let html = render_modal(&props, &serde_json::Value::Null);
        assert!(html.contains("A description"), "shows description");
    }

    // ── Form layout tests ──────────────────────────────────────────────────

    #[test]
    fn test_render_form_max_width_narrow() {
        let props = FormProps {
            action: Action::new("save"),
            fields: vec![],
            method: None,
            guard: None,
            max_width: Some(FormMaxWidth::Narrow),
        };
        let html = render_form(&props, &serde_json::Value::Null);
        assert!(html.contains("max-w-2xl"), "narrow form has max-w-2xl");
        assert!(html.contains("mx-auto"), "narrow form is centered");
    }

    #[test]
    fn test_render_form_max_width_default() {
        let props = FormProps {
            action: Action::new("save"),
            fields: vec![],
            method: None,
            guard: None,
            max_width: None,
        };
        let html = render_form(&props, &serde_json::Value::Null);
        assert!(!html.contains("max-w-2xl"), "default form has no max-width wrapper");
    }

    #[test]
    fn test_render_form_section_two_column() {
        let props = FormSectionProps {
            title: "Section".into(),
            description: Some("Desc".into()),
            children: vec![],
            layout: Some(FormSectionLayout::TwoColumn),
        };
        let html = render_form_section(&props, &serde_json::Value::Null);
        assert!(html.contains("md:grid"), "two-column uses grid");
        assert!(html.contains("md:grid-cols-5"), "two-column uses 5-col grid");
        assert!(html.contains("md:col-span-2"), "description takes 2 cols");
        assert!(html.contains("md:col-span-3"), "controls take 3 cols");
    }

    // ── Input ARIA tests ───────────────────────────────────────────────────

    #[test]
    fn test_render_input_with_error() {
        let props = InputProps {
            field: "email".into(),
            label: "Email".into(),
            input_type: InputType::Email,
            error: Some("Campo obbligatorio".into()),
            placeholder: None,
            default_value: None,
            data_path: None,
            required: None,
            disabled: None,
            step: None,
            description: None,
            list: None,
        };
        let html = render_input(&props, &serde_json::Value::Null);
        assert!(html.contains("aria-invalid=\"true\""), "input has aria-invalid");
        assert!(html.contains("aria-describedby=\"err-email\""), "input has aria-describedby");
        assert!(html.contains("id=\"err-email\""), "error paragraph has matching id");
        assert!(html.contains("Campo obbligatorio"), "error message rendered");
    }

    #[test]
    fn test_render_input_hidden_no_label() {
        let props = InputProps {
            field: "csrf".into(),
            label: "".into(),
            input_type: InputType::Hidden,
            default_value: Some("token123".into()),
            error: None,
            placeholder: None,
            data_path: None,
            required: None,
            disabled: None,
            step: None,
            description: None,
            list: None,
        };
        let html = render_input(&props, &serde_json::Value::Null);
        assert!(!html.contains("<label"), "hidden input has no label");
        assert!(!html.contains("space-y-1"), "hidden input has no wrapper div");
        assert!(html.contains("type=\"hidden\""), "hidden input present");
    }

    // ── Switch ARIA tests ──────────────────────────────────────────────────

    #[test]
    fn test_render_switch_role_switch() {
        let props = SwitchProps {
            field: "active".into(),
            label: "Attivo".into(),
            description: None,
            checked: Some(true),
            data_path: None,
            required: None,
            disabled: None,
            error: None,
            action: None,
        };
        let html = render_switch(&props, &serde_json::Value::Null);
        assert!(html.contains("role=\"switch\""), "switch has role=switch");
        assert!(html.contains("aria-checked=\"true\""), "checked switch has aria-checked=true");
    }

    // ── Tabs ARIA tests ────────────────────────────────────────────────────

    #[test]
    fn test_render_tabs_aria_attributes() {
        let props = TabsProps {
            default_tab: "general".into(),
            tabs: vec![
                Tab { value: "general".into(), label: "Generale".into(), children: vec![] },
                Tab { value: "advanced".into(), label: "Avanzate".into(), children: vec![] },
            ],
        };
        let html = render_tabs(&props, &serde_json::Value::Null);
        assert!(html.contains("id=\"tab-btn-general\""), "tab button has id");
        assert!(html.contains("aria-controls=\"tab-panel-general\""), "tab button has aria-controls");
        assert!(html.contains("id=\"tab-panel-general\""), "tab panel has id");
        assert!(html.contains("aria-labelledby=\"tab-btn-general\""), "tab panel has aria-labelledby");
    }

    // ── Collapsible ARIA tests ─────────────────────────────────────────────

    #[test]
    fn test_render_collapsible_aria_expanded() {
        let props = CollapsibleProps {
            title: "Details".into(),
            expanded: false,
            children: vec![],
        };
        let html = render_collapsible(&props, &serde_json::Value::Null);
        assert!(html.contains("aria-expanded=\"false\""), "closed collapsible has aria-expanded=false");
    }

    // ── ActionCard href tests ──────────────────────────────────────────────

    #[test]
    fn test_render_action_card_with_href() {
        let props = ActionCardProps {
            title: "Nuovo ordine".into(),
            description: "Crea un ordine".into(),
            icon: None,
            variant: ActionCardVariant::Default,
            href: Some("/ordini/nuovo".into()),
        };
        let html = render_action_card(&props);
        assert!(html.contains("<a href=\"/ordini/nuovo\""), "card wraps in <a> with href");
        assert!(html.contains("aria-label=\"Nuovo ordine\""), "card link has aria-label");
        assert!(!html.contains("<div class=\"rounded"), "no div wrapper when href present");
    }

    #[test]
    fn test_render_action_card_without_href() {
        let props = ActionCardProps {
            title: "Test".into(),
            description: "Desc".into(),
            icon: None,
            variant: ActionCardVariant::Default,
            href: None,
        };
        let html = render_action_card(&props);
        assert!(html.contains("<div class=\"rounded"), "uses div when no href");
        assert!(!html.contains("<a "), "no anchor when no href");
    }

    #[test]
    fn test_render_button_type_button_default() {
        let props = ButtonProps {
            label: "Click".into(),
            variant: ButtonVariant::Default,
            size: Size::Default,
            disabled: None,
            icon: None,
            icon_position: None,
            button_type: None,
        };
        let html = render_button(&props);
        assert!(html.contains("type=\"button\""), "default button type is button");
        assert!(!html.contains("type=\"submit\""), "default is not submit");
    }

    #[test]
    fn test_render_button_type_submit() {
        let props = ButtonProps {
            label: "Salva".into(),
            variant: ButtonVariant::Default,
            size: Size::Default,
            disabled: None,
            icon: None,
            icon_position: None,
            button_type: Some(ButtonType::Submit),
        };
        let html = render_button(&props);
        assert!(html.contains("type=\"submit\""), "submit button type is submit");
    }

    #[test]
    fn data_table_row_actions_url_templating() {
        use crate::component::*;
        use crate::action::*;
        let props = DataTableProps {
            columns: vec![Column { key: "name".into(), label: "Name".into(), format: None }],
            data_path: "items".into(),
            row_actions: Some(vec![DropdownMenuAction {
                label: "Delete".into(),
                action: {
                    let mut a = Action::new("items.delete");
                    a.url = Some("/items/{row_key}/delete".into());
                    a.method = HttpMethod::Delete;
                    a
                },
                destructive: true,
            }]),
            empty_message: None,
            row_key: Some("id".into()),
        };
        let data = serde_json::json!({ "items": [{"id": "42", "name": "Widget"}] });
        let html = render_data_table(&props, &data);
        assert!(html.contains("/items/42/delete"), "URL must have {{row_key}} replaced with actual row key value '42'");
        assert!(!html.contains("{row_key}"), "No unreplaced {{row_key}} placeholders should remain");
    }
}
