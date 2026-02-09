//! HTML render engine for JSON-UI views.
//!
//! Walks a `JsonUiView` component tree and produces an HTML fragment using
//! Tailwind CSS utility classes. Container components (Card, Form, Modal, Tabs,
//! Table) are handled in a follow-up plan; this module covers the tree walker
//! and all 12 leaf component renderers.

use serde_json::Value;

use crate::action::HttpMethod;
use crate::component::{
    AlertProps, AlertVariant, AvatarProps, BadgeProps, BadgeVariant, BreadcrumbProps, ButtonProps,
    ButtonVariant, Component, ComponentNode, DescriptionListProps, IconPosition, Orientation,
    PaginationProps, ProgressProps, SeparatorProps, Size, SkeletonProps, TextElement, TextProps,
};
use crate::view::JsonUiView;

/// Render a JSON-UI view to an HTML fragment.
///
/// Walks the component tree and produces a `<div>` containing all rendered
/// components. This is a fragment, not a full page — the framework wrapper
/// handles `<html>`, `<head>`, and `<body>`.
pub fn render_to_html(view: &JsonUiView, _data: &Value) -> String {
    let mut html = String::from("<div>");
    for node in &view.components {
        html.push_str(&render_node(node));
    }
    html.push_str("</div>");
    html
}

/// Render a single component node, optionally wrapping in `<a>` for GET actions.
fn render_node(node: &ComponentNode) -> String {
    let component_html = render_component(&node.component);

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
fn render_component(component: &Component) -> String {
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

        // Container components — rendered as placeholders until Plan 02.
        Component::Card(props) => {
            let mut html = format!(
                "<div class=\"rounded-lg border bg-white p-6 shadow-sm\"><h3 class=\"text-lg font-semibold\">{}</h3>",
                html_escape(&props.title)
            );
            if let Some(ref desc) = props.description {
                html.push_str(&format!(
                    "<p class=\"text-sm text-gray-500\">{}</p>",
                    html_escape(desc)
                ));
            }
            for child in &props.children {
                html.push_str(&render_node(child));
            }
            if !props.footer.is_empty() {
                html.push_str("<div class=\"mt-4 flex gap-2\">");
                for child in &props.footer {
                    html.push_str(&render_node(child));
                }
                html.push_str("</div>");
            }
            html.push_str("</div>");
            html
        }
        Component::Form(props) => {
            let method_attr = match props.action.method {
                HttpMethod::Get => "get",
                _ => "post",
            };
            let action_url = props.action.url.as_deref().unwrap_or("");
            let mut html = format!(
                "<form method=\"{}\" action=\"{}\">",
                method_attr,
                html_escape(action_url)
            );
            for field in &props.fields {
                html.push_str(&render_node(field));
            }
            html.push_str("</form>");
            html
        }
        Component::Modal(props) => {
            let mut html = format!(
                "<div class=\"modal\"><h3 class=\"text-lg font-semibold\">{}</h3>",
                html_escape(&props.title)
            );
            if let Some(ref desc) = props.description {
                html.push_str(&format!(
                    "<p class=\"text-sm text-gray-500\">{}</p>",
                    html_escape(desc)
                ));
            }
            for child in &props.children {
                html.push_str(&render_node(child));
            }
            if !props.footer.is_empty() {
                html.push_str("<div class=\"mt-4 flex gap-2\">");
                for child in &props.footer {
                    html.push_str(&render_node(child));
                }
                html.push_str("</div>");
            }
            html.push_str("</div>");
            html
        }
        Component::Tabs(props) => {
            let mut html = String::from("<div class=\"tabs\">");
            html.push_str("<div class=\"flex border-b\">");
            for tab in &props.tabs {
                let active = if tab.value == props.default_tab {
                    " border-b-2 border-blue-600 text-blue-600"
                } else {
                    ""
                };
                html.push_str(&format!(
                    "<button class=\"px-4 py-2 text-sm font-medium{}\">{}</button>",
                    active,
                    html_escape(&tab.label)
                ));
            }
            html.push_str("</div>");
            for tab in &props.tabs {
                let hidden = if tab.value != props.default_tab {
                    " hidden"
                } else {
                    ""
                };
                html.push_str(&format!("<div class=\"p-4{}\">", hidden));
                for child in &tab.children {
                    html.push_str(&render_node(child));
                }
                html.push_str("</div>");
            }
            html.push_str("</div>");
            html
        }
        Component::Table(props) => {
            let mut html =
                String::from("<table class=\"min-w-full divide-y divide-gray-200\"><thead><tr>");
            for col in &props.columns {
                html.push_str(&format!(
                    "<th class=\"px-4 py-3 text-left text-xs font-medium uppercase text-gray-500\">{}</th>",
                    html_escape(&col.label)
                ));
            }
            html.push_str("</tr></thead><tbody></tbody></table>");
            html
        }

        // Form field components — basic SSR rendering.
        Component::Input(props) => {
            let mut html = format!(
                "<div class=\"mb-4\"><label class=\"block text-sm font-medium text-gray-700 mb-1\">{}</label>",
                html_escape(&props.label)
            );
            let input_type = match props.input_type {
                crate::component::InputType::Text => "text",
                crate::component::InputType::Email => "email",
                crate::component::InputType::Password => "password",
                crate::component::InputType::Number => "number",
                crate::component::InputType::Textarea => "textarea",
                crate::component::InputType::Hidden => "hidden",
                crate::component::InputType::Date => "date",
                crate::component::InputType::Time => "time",
                crate::component::InputType::Url => "url",
                crate::component::InputType::Tel => "tel",
                crate::component::InputType::Search => "search",
            };
            html.push_str(&format!(
                "<input type=\"{}\" name=\"{}\" class=\"w-full rounded-md border border-gray-300 px-3 py-2 text-sm\"",
                input_type,
                html_escape(&props.field)
            ));
            if let Some(ref placeholder) = props.placeholder {
                html.push_str(&format!(" placeholder=\"{}\"", html_escape(placeholder)));
            }
            if let Some(ref dv) = props.default_value {
                html.push_str(&format!(" value=\"{}\"", html_escape(dv)));
            }
            if props.required == Some(true) {
                html.push_str(" required");
            }
            if props.disabled == Some(true) {
                html.push_str(" disabled");
            }
            html.push('>');
            if let Some(ref error) = props.error {
                html.push_str(&format!(
                    "<p class=\"mt-1 text-sm text-red-600\">{}</p>",
                    html_escape(error)
                ));
            }
            html.push_str("</div>");
            html
        }
        Component::Select(props) => {
            let mut html = format!(
                "<div class=\"mb-4\"><label class=\"block text-sm font-medium text-gray-700 mb-1\">{}</label>",
                html_escape(&props.label)
            );
            html.push_str(&format!(
                "<select name=\"{}\" class=\"w-full rounded-md border border-gray-300 px-3 py-2 text-sm\"",
                html_escape(&props.field)
            ));
            if props.required == Some(true) {
                html.push_str(" required");
            }
            if props.disabled == Some(true) {
                html.push_str(" disabled");
            }
            html.push('>');
            if let Some(ref placeholder) = props.placeholder {
                html.push_str(&format!(
                    "<option value=\"\">{}</option>",
                    html_escape(placeholder)
                ));
            }
            for opt in &props.options {
                let selected = if props.default_value.as_deref() == Some(&opt.value) {
                    " selected"
                } else {
                    ""
                };
                html.push_str(&format!(
                    "<option value=\"{}\"{}>{}</option>",
                    html_escape(&opt.value),
                    selected,
                    html_escape(&opt.label)
                ));
            }
            html.push_str("</select>");
            if let Some(ref error) = props.error {
                html.push_str(&format!(
                    "<p class=\"mt-1 text-sm text-red-600\">{}</p>",
                    html_escape(error)
                ));
            }
            html.push_str("</div>");
            html
        }
        Component::Checkbox(props) => {
            let mut html = String::from("<div class=\"mb-4 flex items-start gap-2\">");
            html.push_str(&format!(
                "<input type=\"checkbox\" name=\"{}\" class=\"mt-0.5 rounded border-gray-300\"",
                html_escape(&props.field)
            ));
            if props.checked == Some(true) {
                html.push_str(" checked");
            }
            if props.disabled == Some(true) {
                html.push_str(" disabled");
            }
            html.push_str(&format!(
                "><label class=\"text-sm text-gray-700\">{}</label>",
                html_escape(&props.label)
            ));
            html.push_str("</div>");
            html
        }
        Component::Switch(props) => {
            let mut html = String::from("<div class=\"mb-4 flex items-center gap-2\">");
            html.push_str(&format!(
                "<input type=\"checkbox\" name=\"{}\" role=\"switch\" class=\"rounded-full\"",
                html_escape(&props.field)
            ));
            if props.checked == Some(true) {
                html.push_str(" checked");
            }
            if props.disabled == Some(true) {
                html.push_str(" disabled");
            }
            html.push_str(&format!(
                "><label class=\"text-sm text-gray-700\">{}</label>",
                html_escape(&props.label)
            ));
            html.push_str("</div>");
            html
        }
    }
}

// ── Leaf component renderers ────────────────────────────────────────────

fn render_text(props: &TextProps) -> String {
    let content = html_escape(&props.content);
    match props.element {
        TextElement::P => format!("<p class=\"text-base text-gray-700\">{}</p>", content),
        TextElement::H1 => format!(
            "<h1 class=\"text-3xl font-bold text-gray-900\">{}</h1>",
            content
        ),
        TextElement::H2 => format!(
            "<h2 class=\"text-2xl font-semibold text-gray-900\">{}</h2>",
            content
        ),
        TextElement::H3 => format!(
            "<h3 class=\"text-xl font-semibold text-gray-900\">{}</h3>",
            content
        ),
        TextElement::Span => format!("<span class=\"text-base text-gray-700\">{}</span>", content),
    }
}

fn render_button(props: &ButtonProps) -> String {
    let base = "inline-flex items-center justify-center rounded-md font-medium transition-colors";

    let variant_classes = match props.variant {
        ButtonVariant::Default => "bg-blue-600 text-white hover:bg-blue-700",
        ButtonVariant::Secondary => "bg-gray-100 text-gray-900 hover:bg-gray-200",
        ButtonVariant::Destructive => "bg-red-600 text-white hover:bg-red-700",
        ButtonVariant::Outline => "border border-gray-300 bg-white text-gray-700 hover:bg-gray-50",
        ButtonVariant::Ghost => "text-gray-700 hover:bg-gray-100",
        ButtonVariant::Link => "text-blue-600 underline hover:text-blue-700",
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
            IconPosition::Left => format!("{} {}", icon_span, label),
            IconPosition::Right => format!("{} {}", label, icon_span),
        }
    } else {
        label
    };

    format!(
        "<button class=\"{} {} {}{}\"{}>{}</button>",
        base, variant_classes, size_classes, disabled_classes, disabled_attr, content
    )
}

fn render_badge(props: &BadgeProps) -> String {
    let base = "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium";
    let variant_classes = match props.variant {
        BadgeVariant::Default => "bg-blue-100 text-blue-800",
        BadgeVariant::Secondary => "bg-gray-100 text-gray-800",
        BadgeVariant::Destructive => "bg-red-100 text-red-800",
        BadgeVariant::Outline => "border border-gray-300 text-gray-700",
    };
    format!(
        "<span class=\"{} {}\">{}</span>",
        base,
        variant_classes,
        html_escape(&props.label)
    )
}

fn render_alert(props: &AlertProps) -> String {
    let variant_classes = match props.variant {
        AlertVariant::Info => "bg-blue-50 border-blue-200 text-blue-800",
        AlertVariant::Success => "bg-green-50 border-green-200 text-green-800",
        AlertVariant::Warning => "bg-yellow-50 border-yellow-200 text-yellow-800",
        AlertVariant::Error => "bg-red-50 border-red-200 text-red-800",
    };
    let mut html = format!(
        "<div role=\"alert\" class=\"rounded-md border p-4 {}\">",
        variant_classes
    );
    if let Some(ref title) = props.title {
        html.push_str(&format!(
            "<h4 class=\"font-semibold mb-1\">{}</h4>",
            html_escape(title)
        ));
    }
    html.push_str(&format!("<p>{}</p>", html_escape(&props.message)));
    html.push_str("</div>");
    html
}

fn render_separator(props: &SeparatorProps) -> String {
    let orientation = props.orientation.as_ref().cloned().unwrap_or_default();
    match orientation {
        Orientation::Horizontal => "<hr class=\"my-4 border-gray-200\">".to_string(),
        Orientation::Vertical => "<div class=\"mx-4 h-full w-px bg-gray-200\"></div>".to_string(),
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
            "<div class=\"mb-1 text-sm text-gray-600\">{}</div>",
            html_escape(label)
        ));
    }
    html.push_str(&format!(
        "<div class=\"w-full rounded-full bg-gray-200 h-2.5\"><div class=\"rounded-full bg-blue-600 h-2.5\" style=\"width: {}%\"></div></div>",
        pct
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
            "<span class=\"inline-flex items-center justify-center rounded-full bg-gray-200 text-gray-600 {}\">{}</span>",
            size_classes,
            html_escape(&initials)
        )
    }
}

fn render_skeleton(props: &SkeletonProps) -> String {
    let width = props.width.as_deref().unwrap_or("100%");
    let height = props.height.as_deref().unwrap_or("1rem");
    let rounded = if props.rounded == Some(true) {
        "rounded-full"
    } else {
        "rounded-md"
    };
    format!(
        "<div class=\"animate-pulse bg-gray-200 {}\" style=\"width: {}; height: {}\"></div>",
        rounded, width, height
    )
}

fn render_breadcrumb(props: &BreadcrumbProps) -> String {
    let mut html =
        String::from("<nav class=\"flex items-center space-x-2 text-sm text-gray-500\">");
    let len = props.items.len();
    for (i, item) in props.items.iter().enumerate() {
        let is_last = i == len - 1;
        if is_last {
            html.push_str(&format!(
                "<span class=\"text-gray-900 font-medium\">{}</span>",
                html_escape(&item.label)
            ));
        } else if let Some(ref url) = item.url {
            html.push_str(&format!(
                "<a href=\"{}\" class=\"hover:text-gray-700\">{}</a>",
                html_escape(url),
                html_escape(&item.label)
            ));
        } else {
            html.push_str(&format!("<span>{}</span>", html_escape(&item.label)));
        }
        if !is_last {
            html.push_str("<span>/</span>");
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
            "<a href=\"{}page={}\" class=\"px-3 py-1 rounded-md bg-white text-gray-700 hover:bg-gray-50\">&laquo;</a>",
            html_escape(base_url),
            current - 1
        ));
    }

    // Page numbers — show up to 7 with ellipsis.
    let pages = compute_page_range(current, total_pages);
    let mut prev_page = 0u32;
    for page in pages {
        if prev_page > 0 && page > prev_page + 1 {
            html.push_str("<span class=\"px-2 text-gray-400\">&hellip;</span>");
        }
        if page == current {
            html.push_str(&format!(
                "<span class=\"px-3 py-1 rounded-md bg-blue-600 text-white\">{}</span>",
                page
            ));
        } else {
            html.push_str(&format!(
                "<a href=\"{}page={}\" class=\"px-3 py-1 rounded-md bg-white text-gray-700 hover:bg-gray-50\">{}</a>",
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
            "<a href=\"{}page={}\" class=\"px-3 py-1 rounded-md bg-white text-gray-700 hover:bg-gray-50\">&raquo;</a>",
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
    let mut html = format!("<dl class=\"grid grid-cols-{} gap-4\">", columns);
    for item in &props.items {
        html.push_str(&format!(
            "<div><dt class=\"text-sm font-medium text-gray-500\">{}</dt><dd class=\"mt-1 text-sm text-gray-900\">{}</dd></div>",
            html_escape(&item.label),
            html_escape(&item.value)
        ));
    }
    html.push_str("</dl>");
    html
}

// ── HTML escaping ───────────────────────────────────────────────────────

/// Escape special HTML characters to prevent XSS.
fn html_escape(s: &str) -> String {
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
