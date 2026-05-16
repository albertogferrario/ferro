//! Phase 116: form-control renderers ported from v1 `render.rs`.
//!
//! Per CONTEXT D-21 v1 HTML emission is the canonical contract. This module
//! changes the per-function signature to `(el, spec, data, depth) -> String`,
//! replaces `props.fields` (removed in Phase 115-02) with `Element.children`
//! for Form (D-05), and applies D-15/D-16 action URL handling.
//!
//! Non-obvious v1 behaviors preserved verbatim (per CONTEXT §"Non-obvious v1
//! behaviors to preserve"):
//! - **Switch auto-form wrap** — when `SwitchProps.action` is `Some`, the
//!   switch is wrapped inside a `<form>` that submits on change via
//!   `onchange="this.closest('form').submit()"` (v1 render.rs:1603–1711).
//! - **Input / Select / Checkbox / Switch `data_path` pre-fill** — resolves
//!   the path against the render-time `data` via
//!   [`crate::data::resolve_path`] / [`crate::data::resolve_path_string`] to
//!   populate `value=""` or `checked` attributes. Per v1 precedence,
//!   `default_value` wins over `data_path` for Input and Select.
//! - **Form method spoofing** — PUT/PATCH/DELETE rewrite to POST + hidden
//!   `_method` input (v1 render.rs:961–1015).

use serde_json::Value;

use crate::action::HttpMethod;
use crate::component::{
    CheckboxListProps, CheckboxProps, FormMaxWidth, FormProps, InputProps, InputType, SelectOption,
    SelectProps, SwitchProps,
};
use crate::data::{resolve_path, resolve_path_string};
use crate::spec::{Element, Spec};

use super::{html_escape, render_element};

/// Port of v1 `render_form` (render.rs:961–1015).
///
/// Differences from v1:
/// - Child fields come from `el.children` (list of IDs) instead of the
///   removed `FormProps.fields: Vec<ComponentNode>` per D-05.
/// - `action.url = None` falls back to `action="#"` AND emits the D-16
///   diagnostic comment (v1 silently used `"#"`).
pub(crate) fn render_form(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    let props: FormProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode Form props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    // Effective method — `props.method` overrides `props.action.method`.
    let effective_method = props
        .method
        .as_ref()
        .unwrap_or(&props.action.method)
        .clone();

    let (form_method, needs_spoofing) = match effective_method {
        HttpMethod::Get => ("get", false),
        HttpMethod::Post => ("post", false),
        HttpMethod::Put | HttpMethod::Patch | HttpMethod::Delete => ("post", true),
    };

    // D-15/D-16 action URL handling. Emit diagnostic comment when None.
    let (action_url, diagnostic) = match props.action.url.as_deref() {
        Some(u) => (u.to_string(), String::new()),
        None => (
            "#".to_string(),
            format!(
                "<!-- ferro-json-ui: action '{}' has no resolved url -->",
                html_escape(&props.action.handler)
            ),
        ),
    };

    let mut html = match &props.guard {
        Some(g) => format!(
            "<form action=\"{}\" method=\"{}\" data-form-guard=\"{}\" class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">",
            html_escape(&action_url),
            form_method,
            html_escape(g)
        ),
        None => format!(
            "<form action=\"{}\" method=\"{}\" class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">",
            html_escape(&action_url),
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

    // Fields come from el.children (D-05). Each child ID is looked up and
    // rendered via the walker, which applies visibility and depth guards.
    for child_id in &el.children {
        html.push_str(&render_element(child_id, spec, data, depth + 1));
    }

    html.push_str("</form>");

    // v1 FIX-02: max-width wrapper when specified.
    let html = match props.max_width.as_ref().unwrap_or(&FormMaxWidth::Default) {
        FormMaxWidth::Default => html,
        FormMaxWidth::Narrow => format!("<div class=\"max-w-2xl mx-auto\">{html}</div>"),
        FormMaxWidth::Wide => format!("<div class=\"max-w-4xl mx-auto\">{html}</div>"),
    };

    format!("{diagnostic}{html}")
}

/// Port of v1 `render_input` (render.rs:1289–1436).
///
/// `default_value` wins over `data_path` per v1 precedence — if an explicit
/// default is provided in the spec it overrides the resolved data path.
/// Hidden inputs skip the label/wrapper per A11Y-07.
pub(crate) fn render_input(el: &Element, _spec: &Spec, data: &Value, _depth: usize) -> String {
    let props: InputProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode Input props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    // Effective value: default_value wins, else data_path, else none.
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
                "<textarea id=\"{}\" name=\"{}\" class=\"block w-full rounded-md border {} px-3 py-2 text-base shadow-sm transition-colors duration-150 motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed {}\"",
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
                "<input type=\"{}\" id=\"{}\" name=\"{}\" class=\"block w-full rounded-md border {} px-3 py-2 text-base shadow-sm transition-colors duration-150 motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed {}\"",
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
            if has_error {
                html.push_str(&format!(
                    " aria-invalid=\"true\" aria-describedby=\"err-{}\"",
                    html_escape(&props.field)
                ));
            }
            html.push('>');

            // Optional datalist companion. v1 looks up a flat `list_id` key on
            // the root data object; we preserve that lookup semantics.
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
        html.push_str(&format!(
            "<p id=\"err-{}\" class=\"text-sm text-destructive\">{}</p>",
            html_escape(&props.field),
            html_escape(error)
        ));
    }
    html.push_str("</div>");
    html
}

/// Port of v1 `render_select` (render.rs:1438–1534).
///
/// Same `default_value` > `data_path` precedence as `render_input`. The
/// resolved value marks the matching `<option>` with the `selected`
/// attribute.
pub(crate) fn render_select(el: &Element, _spec: &Spec, data: &Value, _depth: usize) -> String {
    let props: SelectProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode Select props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

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
        "<select id=\"{}\" name=\"{}\" class=\"block w-full appearance-none bg-background rounded-md border {} pr-10 px-3 py-2 text-base shadow-sm transition-colors duration-150 motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed {}\"",
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
        html.push_str(&format!(
            "<p id=\"err-{}\" class=\"text-sm text-destructive\">{}</p>",
            html_escape(&props.field),
            html_escape(error)
        ));
    }
    html.push_str("</div>");
    html
}

/// Port of v1 `render_checkbox` (render.rs:1536–1601).
///
/// `checked` prop wins over `data_path` truthy resolution. When `props.value`
/// is set, the checkbox id is scoped as `"{field}_{value}"` so multiple
/// checkboxes in a group can share the same name without id collisions.
pub(crate) fn render_checkbox(el: &Element, _spec: &Spec, data: &Value, _depth: usize) -> String {
    let props: CheckboxProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode Checkbox props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    let is_checked = resolve_checked(props.checked, props.data_path.as_deref(), data);

    let value_attr = props.value.as_deref().unwrap_or("1");
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

/// Renders a multi-select checkbox group from static `options` or a data-driven
/// `options_path`. Pre-checked options are resolved from `selected_path` as a
/// `Vec<String>`. All string interpolations pass through `html_escape`.
pub(crate) fn render_checkbox_list(
    el: &Element,
    _spec: &Spec,
    data: &Value,
    _depth: usize,
) -> String {
    let props: CheckboxListProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode CheckboxList props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    // Resolve options: data-driven path wins when static `options` is empty.
    let options: Vec<SelectOption> = if props.options.is_empty() {
        props
            .options_path
            .as_deref()
            .and_then(|path| resolve_path(data, path))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value::<SelectOption>(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        props.options.clone()
    };

    // Resolve pre-selected values.
    let selected: Vec<String> = props
        .selected_path
        .as_deref()
        .and_then(|path| resolve_path(data, path))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let mut html = String::from("<fieldset class=\"space-y-2\">");
    if let Some(ref label) = props.label {
        html.push_str(&format!(
            "<legend class=\"text-sm font-medium text-text\">{}</legend>",
            html_escape(label)
        ));
    }
    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"text-sm text-muted-foreground mb-2\">{}</p>",
            html_escape(desc)
        ));
    }
    for option in &options {
        let is_checked = selected.contains(&option.value);
        let checkbox_id = format!("{}_{}", props.field, option.value);
        html.push_str("<div class=\"flex items-center gap-2\">");
        html.push_str(&format!(
            "<input type=\"checkbox\" id=\"{}\" name=\"{}\" value=\"{}\" \
             class=\"h-4 w-4 rounded-sm border-border text-primary\"",
            html_escape(&checkbox_id),
            html_escape(&props.field),
            html_escape(&option.value)
        ));
        if is_checked {
            html.push_str(" checked");
        }
        if props.disabled == Some(true) {
            html.push_str(" disabled");
        }
        html.push('>');
        html.push_str(&format!(
            "<label class=\"text-sm font-medium text-text\" for=\"{}\">{}</label>",
            html_escape(&checkbox_id),
            html_escape(&option.label)
        ));
        html.push_str("</div>");
    }
    if let Some(ref err) = props.error {
        html.push_str(&format!(
            "<p class=\"text-sm text-destructive mt-1\">{}</p>",
            html_escape(err)
        ));
    }
    html.push_str("</fieldset>");
    html
}

/// Port of v1 `render_switch` (render.rs:1603–1711).
///
/// Preserves v1's auto-form wrap when `props.action` is `Some`: the switch
/// renders inside a minimal `<form>` and fires an `onchange` submit. When
/// `action.url = None` the form falls back to `action="#"` with the D-16
/// diagnostic comment preceding the rendered markup.
pub(crate) fn render_switch(el: &Element, _spec: &Spec, data: &Value, _depth: usize) -> String {
    let props: SwitchProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode Switch props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    let is_checked = resolve_checked(props.checked, props.data_path.as_deref(), data);

    let compact_class = if props.compact == Some(true) {
        " scale-75 origin-left"
    } else {
        ""
    };

    let auto_submit = props.action.is_some();
    let onchange = if auto_submit {
        " onchange=\"this.closest('form').submit()\""
    } else {
        ""
    };

    let mut diagnostic = String::new();
    let mut html = String::new();

    if let Some(ref action) = props.action {
        let (action_url, diag) = match action.url.as_deref() {
            Some(u) => (u.to_string(), String::new()),
            None => (
                "#".to_string(),
                format!(
                    "<!-- ferro-json-ui: action '{}' has no resolved url -->",
                    html_escape(&action.handler)
                ),
            ),
        };
        diagnostic = diag;

        let (form_method, needs_spoofing) = match action.method {
            HttpMethod::Get => ("get", false),
            HttpMethod::Post => ("post", false),
            HttpMethod::Put | HttpMethod::Patch | HttpMethod::Delete => ("post", true),
        };
        html.push_str(&format!(
            "<form action=\"{}\" method=\"{}\">",
            html_escape(&action_url),
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

    html.push_str(&format!("<div class=\"space-y-1{compact_class}\">"));
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

    format!("{diagnostic}{html}")
}

/// Shared truthiness rule for `Checkbox.data_path` and `Switch.data_path`.
/// Matches v1's explicit match on JSON `Value` variants (render.rs:1538–1552,
/// 1605–1619): `Bool` is its own value; `Number` is truthy iff nonzero;
/// `String` is truthy iff non-empty AND not the literal `"false"` / `"0"`;
/// `Null` is always falsy; arrays/objects are always truthy.
fn resolve_checked(explicit: Option<bool>, data_path: Option<&str>, data: &Value) -> bool {
    if let Some(c) = explicit {
        return c;
    }
    match data_path {
        Some(dp) => resolve_path(data, dp)
            .map(|v| match v {
                Value::Bool(b) => *b,
                Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
                Value::String(s) => !s.is_empty() && s != "false" && s != "0",
                Value::Null => false,
                _ => true,
            })
            .unwrap_or(false),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, HttpMethod};
    use crate::spec::{Element, Spec};
    use serde_json::json;

    fn mk_spec(root: &str, el: Element) -> Spec {
        let mut spec = Spec::builder()
            .element("__tmp__", Element::new("Text"))
            .build()
            .expect("builder accepts trivial spec");
        spec.root = root.to_string();
        spec.elements.clear();
        spec.elements.insert(root.to_string(), el);
        spec
    }

    fn mk_element(type_name: &str, props: Value) -> Element {
        Element {
            type_name: type_name.to_string(),
            props,
            children: Vec::new(),
            action: None,
            visible: None,
            each: None,
        }
    }

    // ── Input ────────────────────────────────────────────────────────────

    #[test]
    fn input_data_path_prefills_value() {
        let el = mk_element(
            "Input",
            json!({"field": "name", "label": "Name", "data_path": "/user/name"}),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"user": {"name": "Alice"}});
        let html = render_input(&el, &spec, &data, 1);
        assert!(html.contains("value=\"Alice\""), "got: {html}");
    }

    #[test]
    fn input_default_value_wins_over_data_path() {
        // v1 precedence: default_value > data_path.
        let el = mk_element(
            "Input",
            json!({
                "field": "name",
                "label": "Name",
                "default_value": "explicit",
                "data_path": "/user/name",
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"user": {"name": "Alice"}});
        let html = render_input(&el, &spec, &data, 1);
        assert!(html.contains("value=\"explicit\""), "got: {html}");
        assert!(!html.contains("value=\"Alice\""), "got: {html}");
    }

    #[test]
    fn input_hidden_emits_no_label() {
        let el = mk_element(
            "Input",
            json!({"field": "token", "label": "t", "input_type": "hidden", "default_value": "abc"}),
        );
        let spec = mk_spec("root", el.clone());
        let html = render_input(&el, &spec, &json!({}), 1);
        assert!(html.starts_with("<input type=\"hidden\""), "got: {html}");
        assert!(!html.contains("<label"), "got: {html}");
        assert!(html.contains("value=\"abc\""), "got: {html}");
    }

    #[test]
    fn input_xss_in_value_is_escaped() {
        // Attribute-breakout via data_path — the resolved value must be
        // html-escaped before interpolation into `value=""`.
        let el = mk_element(
            "Input",
            json!({"field": "name", "label": "n", "data_path": "/payload"}),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"payload": "\"><script>x</script>"});
        let html = render_input(&el, &spec, &data, 1);
        assert!(!html.contains("<script>x</script>"), "got: {html}");
        assert!(html.contains("&quot;"), "got: {html}");
    }

    #[test]
    fn input_error_emits_aria_describedby() {
        let el = mk_element(
            "Input",
            json!({"field": "email", "label": "Email", "error": "required"}),
        );
        let spec = mk_spec("root", el.clone());
        let html = render_input(&el, &spec, &json!({}), 1);
        assert!(html.contains("aria-invalid=\"true\""), "got: {html}");
        assert!(
            html.contains("aria-describedby=\"err-email\""),
            "got: {html}"
        );
        assert!(
            html.contains("<p id=\"err-email\" class=\"text-sm text-destructive\">required</p>"),
            "got: {html}"
        );
    }

    // ── Select ───────────────────────────────────────────────────────────

    #[test]
    fn select_data_path_marks_option_selected() {
        let el = mk_element(
            "Select",
            json!({
                "field": "role",
                "label": "Role",
                "data_path": "/role",
                "options": [
                    {"value": "a", "label": "A"},
                    {"value": "b", "label": "B"},
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"role": "b"});
        let html = render_select(&el, &spec, &data, 1);
        assert!(
            html.contains("<option value=\"b\" selected>B</option>"),
            "got: {html}"
        );
        assert!(
            html.contains("<option value=\"a\">A</option>"),
            "got: {html}"
        );
    }

    #[test]
    fn select_default_value_wins_over_data_path() {
        let el = mk_element(
            "Select",
            json!({
                "field": "role",
                "label": "Role",
                "default_value": "a",
                "data_path": "/role",
                "options": [
                    {"value": "a", "label": "A"},
                    {"value": "b", "label": "B"},
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"role": "b"});
        let html = render_select(&el, &spec, &data, 1);
        assert!(
            html.contains("<option value=\"a\" selected>A</option>"),
            "got: {html}"
        );
    }

    // ── Checkbox ─────────────────────────────────────────────────────────

    #[test]
    fn checkbox_data_path_truthy_renders_checked() {
        let el = mk_element(
            "Checkbox",
            json!({"field": "agreed", "label": "Agree", "data_path": "/agreed"}),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"agreed": true});
        let html = render_checkbox(&el, &spec, &data, 1);
        assert!(html.contains(" checked"), "got: {html}");
    }

    #[test]
    fn checkbox_data_path_false_omits_checked() {
        let el = mk_element(
            "Checkbox",
            json!({"field": "agreed", "label": "Agree", "data_path": "/agreed"}),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"agreed": false});
        let html = render_checkbox(&el, &spec, &data, 1);
        assert!(!html.contains(" checked"), "got: {html}");
    }

    #[test]
    fn checkbox_with_value_scopes_id() {
        let el = mk_element(
            "Checkbox",
            json!({"field": "topic", "value": "news", "label": "News"}),
        );
        let spec = mk_spec("root", el.clone());
        let html = render_checkbox(&el, &spec, &json!({}), 1);
        assert!(html.contains("id=\"topic_news\""), "got: {html}");
        assert!(html.contains("name=\"topic\""), "got: {html}");
        assert!(html.contains("value=\"news\""), "got: {html}");
    }

    // ── Switch ───────────────────────────────────────────────────────────

    #[test]
    fn switch_with_action_wraps_in_form() {
        let action = Action {
            handler: "users.toggle".into(),
            url: Some("/users/1/toggle".into()),
            method: HttpMethod::Post,
            confirm: None,
            on_success: None,
            on_error: None,
            target: None,
        };
        let el = mk_element(
            "Switch",
            json!({
                "field": "active",
                "label": "Active",
                "action": action,
            }),
        );
        let spec = mk_spec("root", el.clone());
        let html = render_switch(&el, &spec, &json!({}), 1);
        assert!(
            html.contains("<form action=\"/users/1/toggle\" method=\"post\">"),
            "got: {html}"
        );
        assert!(
            html.contains("onchange=\"this.closest('form').submit()\""),
            "got: {html}"
        );
        assert!(html.contains("</form>"), "got: {html}");
    }

    #[test]
    fn switch_without_action_no_form_wrap() {
        let el = mk_element("Switch", json!({"field": "active", "label": "Active"}));
        let spec = mk_spec("root", el.clone());
        let html = render_switch(&el, &spec, &json!({}), 1);
        assert!(
            !html.contains("<form"),
            "switch without action must not wrap in form; got: {html}"
        );
        assert!(
            !html.contains("onchange"),
            "switch without action must not have onchange; got: {html}"
        );
    }

    #[test]
    fn switch_action_without_url_emits_diagnostic() {
        let action = Action {
            handler: "users.toggle".into(),
            url: None,
            method: HttpMethod::Post,
            confirm: None,
            on_success: None,
            on_error: None,
            target: None,
        };
        let el = mk_element(
            "Switch",
            json!({"field": "active", "label": "Active", "action": action}),
        );
        let spec = mk_spec("root", el.clone());
        let html = render_switch(&el, &spec, &json!({}), 1);
        assert!(
            html.contains("<!-- ferro-json-ui: action 'users.toggle' has no resolved url -->"),
            "got: {html}"
        );
        assert!(html.contains("<form action=\"#\""), "got: {html}");
    }

    #[test]
    fn switch_put_method_spoofs_post() {
        let action = Action {
            handler: "users.update".into(),
            url: Some("/users/1".into()),
            method: HttpMethod::Put,
            confirm: None,
            on_success: None,
            on_error: None,
            target: None,
        };
        let el = mk_element(
            "Switch",
            json!({"field": "x", "label": "X", "action": action}),
        );
        let spec = mk_spec("root", el.clone());
        let html = render_switch(&el, &spec, &json!({}), 1);
        assert!(
            html.contains("<form action=\"/users/1\" method=\"post\">"),
            "got: {html}"
        );
        assert!(
            html.contains("<input type=\"hidden\" name=\"_method\" value=\"PUT\">"),
            "got: {html}"
        );
    }

    // ── Form ─────────────────────────────────────────────────────────────

    #[test]
    fn form_recurses_children_as_fields() {
        // Build with the regular builder so the walker can resolve children.
        let spec = Spec::builder()
            .element(
                "root",
                Element::new("Form")
                    .prop(
                        "action",
                        json!({"handler": "save", "method": "POST", "url": "/save"}),
                    )
                    .child("i1")
                    .child("i2"),
            )
            .element(
                "i1",
                Element::new("Input").prop("field", "a").prop("label", "A"),
            )
            .element(
                "i2",
                Element::new("Input").prop("field", "b").prop("label", "B"),
            )
            .build()
            .expect("spec builds");

        let el = spec.elements.get("root").unwrap();
        let html = render_form(el, &spec, &json!({}), 1);
        assert!(html.contains("<form"), "got: {html}");
        // Both labels appear — confirms recursion into render_element which
        // dispatches to render_input for each child.
        assert!(html.contains(">A</label>"), "got: {html}");
        assert!(html.contains(">B</label>"), "got: {html}");
    }

    #[test]
    fn form_action_url_resolved_in_action_attr() {
        let el = mk_element(
            "Form",
            json!({"action": {"handler": "save", "method": "POST", "url": "/submit"}}),
        );
        let spec = mk_spec("root", el.clone());
        let html = render_form(&el, &spec, &json!({}), 1);
        assert!(html.contains("action=\"/submit\""), "got: {html}");
        assert!(!html.contains("<!-- ferro-json-ui: action"), "got: {html}");
    }

    #[test]
    fn form_action_url_unresolved_falls_back_with_diagnostic() {
        let el = mk_element(
            "Form",
            json!({"action": {"handler": "save", "method": "POST"}}),
        );
        let spec = mk_spec("root", el.clone());
        let html = render_form(&el, &spec, &json!({}), 1);
        assert!(html.contains("action=\"#\""), "got: {html}");
        assert!(
            html.contains("<!-- ferro-json-ui: action 'save' has no resolved url -->"),
            "got: {html}"
        );
    }

    #[test]
    fn form_put_method_spoofs_post_with_hidden_input() {
        let el = mk_element(
            "Form",
            json!({"action": {"handler": "update", "method": "PUT", "url": "/r/1"}}),
        );
        let spec = mk_spec("root", el.clone());
        let html = render_form(&el, &spec, &json!({}), 1);
        assert!(
            html.contains("method=\"post\""),
            "PUT must spoof to POST; got: {html}"
        );
        assert!(
            html.contains("<input type=\"hidden\" name=\"_method\" value=\"PUT\">"),
            "got: {html}"
        );
    }

    // ── Decode-failure diagnostics ───────────────────────────────────────

    #[test]
    fn input_props_decode_failure_emits_diagnostic() {
        let el = mk_element("Input", json!(42));
        let spec = mk_spec("root", el.clone());
        let html = render_input(&el, &spec, &json!({}), 1);
        assert!(
            html.contains("<!-- ferro-json-ui: failed to decode Input props"),
            "got: {html}"
        );
    }

    #[test]
    fn form_props_decode_failure_emits_diagnostic() {
        let el = mk_element("Form", json!(42));
        let spec = mk_spec("root", el.clone());
        let html = render_form(&el, &spec, &json!({}), 1);
        assert!(
            html.contains("<!-- ferro-json-ui: failed to decode Form props"),
            "got: {html}"
        );
    }

    // ── CheckboxList ─────────────────────────────────────────────────────

    #[test]
    fn checkbox_list_renders_one_checkbox_per_option() {
        let el = mk_element(
            "CheckboxList",
            json!({
                "field": "services",
                "options": [{"value": "a", "label": "A"}, {"value": "b", "label": "B"}]
            }),
        );
        let spec = mk_spec("root", el.clone());
        let html = render_checkbox_list(&el, &spec, &json!({}), 1);
        assert_eq!(
            html.matches("<input type=\"checkbox\"").count(),
            2,
            "expected 2 checkboxes; got: {html}"
        );
        assert!(html.contains("name=\"services\""), "got: {html}");
        assert!(html.contains("value=\"a\""), "got: {html}");
        assert!(html.contains("value=\"b\""), "got: {html}");
    }

    #[test]
    fn checkbox_list_selected_path_prechecks_matching_options() {
        let el = mk_element(
            "CheckboxList",
            json!({
                "field": "services",
                "options": [{"value": "a", "label": "A"}, {"value": "b", "label": "B"}],
                "selected_path": "/sel"
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"sel": ["a"]});
        let html = render_checkbox_list(&el, &spec, &data, 1);
        // Only option "a" should be checked
        let checked_count = html.matches(" checked").count();
        assert_eq!(checked_count, 1, "expected 1 checked; got: {html}");
        // The checked input must be the one with value="a"
        let a_pos = html.find("value=\"a\"").expect("value=a not found");
        let checked_pos = html.find(" checked").expect("checked not found");
        assert!(
            checked_pos > a_pos,
            "checked attribute should follow value=a; got: {html}"
        );
    }

    #[test]
    fn checkbox_list_options_path_resolves_dynamic_options() {
        let el = mk_element(
            "CheckboxList",
            json!({
                "field": "services",
                "options_path": "/opts"
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"opts": [{"value": "x", "label": "X"}]});
        let html = render_checkbox_list(&el, &spec, &data, 1);
        assert!(html.contains("value=\"x\""), "got: {html}");
        assert_eq!(
            html.matches("<input type=\"checkbox\"").count(),
            1,
            "got: {html}"
        );
    }

    #[test]
    fn checkbox_list_escapes_html_in_option_label() {
        let el = mk_element(
            "CheckboxList",
            json!({
                "field": "services",
                "options": [{"value": "a", "label": "<script>alert(1)</script>"}]
            }),
        );
        let spec = mk_spec("root", el.clone());
        let html = render_checkbox_list(&el, &spec, &json!({}), 1);
        assert!(
            html.contains("&lt;script&gt;"),
            "label must be HTML-escaped; got: {html}"
        );
        assert!(
            !html.contains("<script>"),
            "raw <script> must not appear; got: {html}"
        );
    }

    #[test]
    fn checkbox_list_disabled_propagates_to_each_input() {
        let el = mk_element(
            "CheckboxList",
            json!({
                "field": "services",
                "options": [{"value": "a", "label": "A"}, {"value": "b", "label": "B"}],
                "disabled": true
            }),
        );
        let spec = mk_spec("root", el.clone());
        let html = render_checkbox_list(&el, &spec, &json!({}), 1);
        let disabled_count = html.matches(" disabled").count();
        assert_eq!(
            disabled_count, 2,
            "every input must carry disabled; got: {html}"
        );
    }

    // ── Switch.compact ───────────────────────────────────────────────────

    #[test]
    fn switch_compact_adds_scale_class() {
        // compact=true must produce scale-75 and origin-left in the output
        let el = mk_element(
            "Switch",
            json!({"field": "x", "label": "L", "compact": true}),
        );
        let spec = mk_spec("root", el.clone());
        let html = render_switch(&el, &spec, &json!({}), 1);
        assert!(
            html.contains("scale-75"),
            "compact=true must emit scale-75; got: {html}"
        );
        assert!(
            html.contains("origin-left"),
            "compact=true must emit origin-left; got: {html}"
        );

        // compact=false must NOT produce the compact classes
        let el_false = mk_element(
            "Switch",
            json!({"field": "x", "label": "L", "compact": false}),
        );
        let html_false = render_switch(&el_false, &spec, &json!({}), 1);
        assert!(
            !html_false.contains("scale-75"),
            "compact=false must not emit scale-75; got: {html_false}"
        );

        // compact absent must NOT produce the compact classes
        let el_none = mk_element("Switch", json!({"field": "x", "label": "L"}));
        let html_none = render_switch(&el_none, &spec, &json!({}), 1);
        assert!(
            !html_none.contains("scale-75"),
            "compact=None must not emit scale-75; got: {html_none}"
        );
    }
}
