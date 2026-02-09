//! HTML render engine for JSON-UI views.
//!
//! Walks a `JsonUiView` component tree and produces an HTML fragment using
//! Tailwind CSS utility classes. All 20 component types are supported:
//! 12 leaf components, 5 container/layout components, and 3 form field components.

use serde_json::Value;

use crate::action::HttpMethod;
use crate::component::{
    AlertProps, AlertVariant, AvatarProps, BadgeProps, BadgeVariant, BreadcrumbProps, ButtonProps,
    ButtonVariant, CardProps, CheckboxProps, Component, ComponentNode, DescriptionListProps,
    FormProps, IconPosition, InputProps, InputType, ModalProps, Orientation, PaginationProps,
    ProgressProps, SelectProps, SeparatorProps, Size, SkeletonProps, SwitchProps, TableProps,
    TabsProps, TextElement, TextProps,
};
use crate::data::{resolve_path, resolve_path_string};
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
    let mut html = String::from("<div>");
    for node in &view.components {
        html.push_str(&render_node(node, data));
    }
    html.push_str("</div>");
    html
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
    }
}

// ── Container component renderers ───────────────────────────────────────

fn render_card(props: &CardProps, data: &Value) -> String {
    let mut html = String::from(
        "<div class=\"rounded-lg border border-gray-200 bg-white shadow-sm\"><div class=\"p-6\">",
    );
    html.push_str(&format!(
        "<h3 class=\"text-lg font-semibold text-gray-900\">{}</h3>",
        html_escape(&props.title)
    ));
    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"mt-1 text-sm text-gray-500\">{}</p>",
            html_escape(desc)
        ));
    }
    if !props.children.is_empty() {
        html.push_str("<div class=\"mt-4 space-y-4\">");
        for child in &props.children {
            html.push_str(&render_node(child, data));
        }
        html.push_str("</div>");
    }
    html.push_str("</div>"); // close p-6
    if !props.footer.is_empty() {
        html.push_str("<div class=\"border-t border-gray-200 px-6 py-4 flex items-center gap-2\">");
        for child in &props.footer {
            html.push_str(&render_node(child, data));
        }
        html.push_str("</div>");
    }
    html.push_str("</div>"); // close outer card
    html
}

fn render_modal(props: &ModalProps, data: &Value) -> String {
    let trigger = props.trigger_label.as_deref().unwrap_or("Open");
    let mut html = String::from("<details class=\"group\">");
    html.push_str(&format!(
        "<summary class=\"inline-flex items-center justify-center rounded-md bg-blue-600 text-white px-4 py-2 text-sm font-medium cursor-pointer\">{}</summary>",
        html_escape(trigger)
    ));
    html.push_str("<div class=\"fixed inset-0 z-50 flex items-center justify-center bg-black/50 group-open:block hidden\">");
    html.push_str(
        "<div class=\"relative bg-white rounded-lg shadow-lg max-w-lg w-full mx-4 p-6\">",
    );
    html.push_str(&format!(
        "<h3 class=\"text-lg font-semibold text-gray-900\">{}</h3>",
        html_escape(&props.title)
    ));
    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"mt-1 text-sm text-gray-500\">{}</p>",
            html_escape(desc)
        ));
    }
    html.push_str("<div class=\"mt-4 space-y-4\">");
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
    html.push_str("</div></div></details>");
    html
}

fn render_tabs(props: &TabsProps, data: &Value) -> String {
    let mut html = String::from("<div>");
    html.push_str("<div class=\"border-b border-gray-200\">");
    html.push_str("<nav class=\"flex -mb-px space-x-4\">");
    for tab in &props.tabs {
        if tab.value == props.default_tab {
            html.push_str(&format!(
                "<span class=\"border-b-2 border-blue-600 text-blue-600 px-3 py-2 text-sm font-medium\">{}</span>",
                html_escape(&tab.label)
            ));
        } else {
            html.push_str(&format!(
                "<span class=\"border-b-2 border-transparent text-gray-500 px-3 py-2 text-sm font-medium\">{}</span>",
                html_escape(&tab.label)
            ));
        }
    }
    html.push_str("</nav></div>");
    // Render only the default tab's children.
    for tab in &props.tabs {
        if tab.value == props.default_tab {
            html.push_str("<div class=\"pt-4 space-y-4\">");
            for child in &tab.children {
                html.push_str(&render_node(child, data));
            }
            html.push_str("</div>");
            break;
        }
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
    let mut html = format!(
        "<form action=\"{}\" method=\"{}\" class=\"space-y-4\">",
        html_escape(action_url),
        form_method
    );

    if needs_spoofing {
        let method_value = match effective_method {
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            _ => unreachable!(),
        };
        html.push_str(&format!(
            "<input type=\"hidden\" name=\"_method\" value=\"{}\">",
            method_value
        ));
    }

    for field in &props.fields {
        html.push_str(&render_node(field, data));
    }
    html.push_str("</form>");
    html
}

fn render_table(props: &TableProps, data: &Value) -> String {
    let mut html = String::from(
        "<div class=\"overflow-x-auto\"><table class=\"min-w-full divide-y divide-gray-200\">",
    );

    // Header.
    html.push_str("<thead class=\"bg-gray-50\"><tr>");
    for col in &props.columns {
        html.push_str(&format!(
            "<th class=\"px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500\">{}</th>",
            html_escape(&col.label)
        ));
    }
    if props.row_actions.is_some() {
        html.push_str("<th class=\"px-6 py-3 text-right text-xs font-medium uppercase tracking-wider text-gray-500\">Actions</th>");
    }
    html.push_str("</tr></thead>");

    // Body.
    html.push_str("<tbody class=\"divide-y divide-gray-200 bg-white\">");

    let rows = resolve_path(data, &props.data_path);
    let row_array = rows.and_then(|v| v.as_array());

    if let Some(items) = row_array {
        if items.is_empty() {
            if let Some(ref msg) = props.empty_message {
                let col_count =
                    props.columns.len() + if props.row_actions.is_some() { 1 } else { 0 };
                html.push_str(&format!(
                    "<tr><td colspan=\"{}\" class=\"px-6 py-8 text-center text-sm text-gray-500\">{}</td></tr>",
                    col_count,
                    html_escape(msg)
                ));
            }
        } else {
            for row in items {
                html.push_str("<tr>");
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
                        "<td class=\"px-6 py-4 text-sm text-gray-900 whitespace-nowrap\">{}</td>",
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
                            "<a href=\"{}\" class=\"text-blue-600 hover:text-blue-800\">{}</a>",
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
            "<tr><td colspan=\"{}\" class=\"px-6 py-8 text-center text-sm text-gray-500\">{}</td></tr>",
            col_count,
            html_escape(msg)
        ));
    }

    html.push_str("</tbody></table></div>");
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

    let has_error = props.error.is_some();
    let border_class = if has_error {
        "border-red-500"
    } else {
        "border-gray-300"
    };

    let mut html = String::from("<div class=\"space-y-1\">");
    html.push_str(&format!(
        "<label class=\"block text-sm font-medium text-gray-700\" for=\"{}\">{}</label>",
        html_escape(&props.field),
        html_escape(&props.label)
    ));

    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"text-sm text-gray-500\">{}</p>",
            html_escape(desc)
        ));
    }

    match props.input_type {
        InputType::Hidden => {
            let val = resolved_value.as_deref().unwrap_or("");
            html.push_str(&format!(
                "<input type=\"hidden\" id=\"{}\" name=\"{}\" value=\"{}\">",
                html_escape(&props.field),
                html_escape(&props.field),
                html_escape(val)
            ));
        }
        InputType::Textarea => {
            let val = resolved_value.as_deref().unwrap_or("");
            html.push_str(&format!(
                "<textarea id=\"{}\" name=\"{}\" class=\"block w-full rounded-md border {} px-3 py-2 text-sm shadow-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500\"",
                html_escape(&props.field),
                html_escape(&props.field),
                border_class
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
                "<input type=\"{}\" id=\"{}\" name=\"{}\" class=\"block w-full rounded-md border {} px-3 py-2 text-sm shadow-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500\"",
                input_type,
                html_escape(&props.field),
                html_escape(&props.field),
                border_class
            ));
            if let Some(ref placeholder) = props.placeholder {
                html.push_str(&format!(" placeholder=\"{}\"", html_escape(placeholder)));
            }
            if let Some(ref val) = resolved_value {
                html.push_str(&format!(" value=\"{}\"", html_escape(val)));
            }
            if props.required == Some(true) {
                html.push_str(" required");
            }
            if props.disabled == Some(true) {
                html.push_str(" disabled");
            }
            html.push('>');
        }
    }

    if let Some(ref error) = props.error {
        html.push_str(&format!(
            "<p class=\"text-sm text-red-600\">{}</p>",
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
        "border-red-500"
    } else {
        "border-gray-300"
    };

    let mut html = String::from("<div class=\"space-y-1\">");
    html.push_str(&format!(
        "<label class=\"block text-sm font-medium text-gray-700\" for=\"{}\">{}</label>",
        html_escape(&props.field),
        html_escape(&props.label)
    ));

    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"text-sm text-gray-500\">{}</p>",
            html_escape(desc)
        ));
    }

    html.push_str(&format!(
        "<select id=\"{}\" name=\"{}\" class=\"block w-full rounded-md border {} px-3 py-2 text-sm shadow-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500\"",
        html_escape(&props.field),
        html_escape(&props.field),
        border_class
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

    if let Some(ref error) = props.error {
        html.push_str(&format!(
            "<p class=\"text-sm text-red-600\">{}</p>",
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

    let mut html = String::from("<div class=\"space-y-1\">");
    html.push_str("<div class=\"flex items-center gap-2\">");
    html.push_str(&format!(
        "<input type=\"checkbox\" id=\"{}\" name=\"{}\" value=\"1\" class=\"h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500\"",
        html_escape(&props.field),
        html_escape(&props.field)
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
        "<label class=\"text-sm font-medium text-gray-700\" for=\"{}\">{}</label>",
        html_escape(&props.field),
        html_escape(&props.label)
    ));
    html.push_str("</div>");

    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"ml-6 text-sm text-gray-500\">{}</p>",
            html_escape(desc)
        ));
    }

    if let Some(ref error) = props.error {
        html.push_str(&format!(
            "<p class=\"ml-6 text-sm text-red-600\">{}</p>",
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

    let mut html = String::from("<div class=\"space-y-1\">");
    html.push_str("<div class=\"flex items-center justify-between\">");

    // Left side: label + description.
    html.push_str("<div>");
    html.push_str(&format!(
        "<label class=\"text-sm font-medium text-gray-700\" for=\"{}\">{}</label>",
        html_escape(&props.field),
        html_escape(&props.label)
    ));
    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"text-sm text-gray-500\">{}</p>",
            html_escape(desc)
        ));
    }
    html.push_str("</div>");

    // Right side: toggle.
    html.push_str("<label class=\"relative inline-flex items-center cursor-pointer\">");
    html.push_str(&format!(
        "<input type=\"checkbox\" id=\"{}\" name=\"{}\" value=\"1\" class=\"sr-only peer\"",
        html_escape(&props.field),
        html_escape(&props.field)
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
    html.push_str("<div class=\"w-11 h-6 bg-gray-200 rounded-full peer peer-checked:bg-blue-600 peer-focus:ring-2 peer-focus:ring-blue-300 after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full\"></div>");
    html.push_str("</label>");
    html.push_str("</div>");

    if let Some(ref error) = props.error {
        html.push_str(&format!(
            "<p class=\"text-sm text-red-600\">{}</p>",
            html_escape(error)
        ));
    }
    html.push_str("</div>");
    html
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
        assert_eq!(html, "<div></div>");
    }

    #[test]
    fn render_view_with_component_wraps_in_div() {
        let view = JsonUiView::new().component(text_node("t", "Hello", TextElement::P));
        let html = render_to_html(&view, &json!({}));
        assert!(html.starts_with("<div>"));
        assert!(html.ends_with("</div>"));
        assert!(html.contains("<p class=\"text-base text-gray-700\">Hello</p>"));
    }

    // ── 2. Text variants ────────────────────────────────────────────────

    #[test]
    fn text_p_variant() {
        let view = JsonUiView::new().component(text_node("t", "Paragraph", TextElement::P));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<p class=\"text-base text-gray-700\">Paragraph</p>"));
    }

    #[test]
    fn text_h1_variant() {
        let view = JsonUiView::new().component(text_node("t", "Title", TextElement::H1));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<h1 class=\"text-3xl font-bold text-gray-900\">Title</h1>"));
    }

    #[test]
    fn text_h2_variant() {
        let view = JsonUiView::new().component(text_node("t", "Subtitle", TextElement::H2));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<h2 class=\"text-2xl font-semibold text-gray-900\">Subtitle</h2>"));
    }

    #[test]
    fn text_h3_variant() {
        let view = JsonUiView::new().component(text_node("t", "Section", TextElement::H3));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<h3 class=\"text-xl font-semibold text-gray-900\">Section</h3>"));
    }

    #[test]
    fn text_span_variant() {
        let view = JsonUiView::new().component(text_node("t", "Inline", TextElement::Span));
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("<span class=\"text-base text-gray-700\">Inline</span>"));
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
        assert!(html.contains("bg-blue-600 text-white hover:bg-blue-700"));
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
        assert!(html.contains("bg-gray-100 text-gray-900 hover:bg-gray-200"));
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
        assert!(html.contains("bg-red-600 text-white hover:bg-red-700"));
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
        assert!(html.contains("border border-gray-300 bg-white text-gray-700"));
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
        assert!(html.contains("text-gray-700 hover:bg-gray-100"));
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
        assert!(html.contains("text-blue-600 underline hover:text-blue-700"));
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
        assert!(html.contains("bg-blue-100 text-blue-800"));
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
        assert!(html.contains("bg-gray-100 text-gray-800"));
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
        assert!(html.contains("bg-red-100 text-red-800"));
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
        assert!(html.contains("border border-gray-300 text-gray-700"));
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
        assert!(html.contains("bg-blue-50 border-blue-200 text-blue-800"));
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
        assert!(html.contains("bg-green-50 border-green-200 text-green-800"));
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
        assert!(html.contains("bg-yellow-50 border-yellow-200 text-yellow-800"));
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
        assert!(html.contains("bg-red-50 border-red-200 text-red-800"));
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
        assert!(html.contains("<hr class=\"my-4 border-gray-200\">"));
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
        assert!(html.contains("<div class=\"mx-4 h-full w-px bg-gray-200\"></div>"));
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
        assert!(html.contains("bg-blue-600 h-2.5"));
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
        assert!(html.contains("text-sm text-gray-600"));
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
        assert!(html.contains("bg-gray-200 text-gray-600"));
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
        assert!(html.contains("animate-pulse bg-gray-200"));
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
        assert!(html.contains("<a href=\"/\" class=\"hover:text-gray-700\">Home</a>"));
        assert!(html.contains("<a href=\"/users\" class=\"hover:text-gray-700\">Users</a>"));
        // Last item is plain span, not a link.
        assert!(html.contains("<span class=\"text-gray-900 font-medium\">Edit</span>"));
        // Separators between items.
        assert!(html.contains("<span>/</span>"));
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
        assert!(html.contains("<span class=\"text-gray-900 font-medium\">Home</span>"));
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
        assert!(html.contains("bg-blue-600 text-white\">2</span>"));
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
        assert!(html.contains("<dt class=\"text-sm font-medium text-gray-500\">Name</dt>"));
        assert!(html.contains("<dd class=\"mt-1 text-sm text-gray-900\">Alice</dd>"));
        assert!(html.contains("<dt class=\"text-sm font-medium text-gray-500\">Email</dt>"));
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
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("rounded-lg border border-gray-200 bg-white shadow-sm"));
        assert!(html.contains("<h3 class=\"text-lg font-semibold text-gray-900\">My Card</h3>"));
        assert!(html.contains("<p class=\"mt-1 text-sm text-gray-500\">A description</p>"));
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
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("mt-4 space-y-4"));
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
        assert!(html.contains("border-t border-gray-200 px-6 py-4 flex items-center gap-2"));
        assert!(html.contains(">Save</button>"));
    }

    // ── 17. Modal ──────────────────────────────────────────────────────

    #[test]
    fn modal_renders_details_summary() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "m".to_string(),
            component: Component::Modal(ModalProps {
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
        assert!(html.contains("<details class=\"group\">"));
        assert!(html.contains("<summary"));
        assert!(html.contains("Open Modal</summary>"));
        assert!(html.contains("<h3 class=\"text-lg font-semibold text-gray-900\">Confirm</h3>"));
        assert!(html.contains("Are you sure?"));
        assert!(html.contains("Body text"));
        assert!(html.contains(">OK</button>"));
        assert!(html.contains("</details>"));
    }

    #[test]
    fn modal_default_trigger_label() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "m".to_string(),
            component: Component::Modal(ModalProps {
                title: "Dialog".to_string(),
                description: None,
                children: vec![],
                footer: vec![],
                trigger_label: None,
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("Open</summary>"));
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
        assert!(html.contains("border-b-2 border-blue-600 text-blue-600"));
        assert!(html.contains(">General</span>"));
        // Inactive tab styling.
        assert!(html.contains("border-transparent text-gray-500"));
        assert!(html.contains(">Security</span>"));
        // Only default tab content rendered.
        assert!(html.contains("General content"));
        assert!(!html.contains("Security content"));
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
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("action=\"/users\""));
        assert!(html.contains("method=\"post\""));
        assert!(html.contains("class=\"space-y-4\""));
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
        assert!(html.contains("border-gray-300"));
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
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("border-red-500"));
        assert!(html.contains("<p class=\"text-sm text-red-600\">Name is required</p>"));
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
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("type=\"hidden\""));
        assert!(html.contains("value=\"abc123\""));
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
        assert!(html.contains("border-red-500"));
        assert!(html.contains("Role is required"));
    }

    // ── 22. Checkbox ───────────────────────────────────────────────────

    #[test]
    fn checkbox_renders_checked_state() {
        let view = JsonUiView::new().component(ComponentNode {
            key: "cb".to_string(),
            component: Component::Checkbox(CheckboxProps {
                field: "terms".to_string(),
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
        assert!(html.contains("ml-6 text-sm text-gray-500"));
        assert!(html.contains("You must accept"));
    }

    #[test]
    fn checkbox_resolves_data_path_for_checked() {
        let data = json!({"user": {"accepted": true}});
        let view = JsonUiView::new().component(ComponentNode {
            key: "cb".to_string(),
            component: Component::Checkbox(CheckboxProps {
                field: "accepted".to_string(),
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
        assert!(html.contains("ml-6 text-sm text-red-600"));
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
        assert!(html.contains("peer-checked:bg-blue-600"));
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
            }),
            action: None,
            visibility: None,
        });
        let html = render_to_html(&view, &json!({}));
        assert!(html.contains("text-sm text-red-600"));
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
        assert!(html.contains("tracking-wider text-gray-500\">Name</th>"));
        assert!(html.contains("tracking-wider text-gray-500\">Email</th>"));
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
        assert!(html.contains("text-center text-sm text-gray-500"));
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
        assert!(html.contains(">Actions</th>"));
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
}
