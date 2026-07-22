//! Leaf-element renderers.
//!
//! Each public function in this module takes `(el, spec, data, depth)` and
//! returns an HTML fragment. Atoms have no recursive children — they decode
//! their typed `*Props`, escape and substitute text, and emit a self-contained
//! HTML element. Decode failures surface via [`decode_diagnostic`] as an HTML
//! comment rather than a panic.

use serde_json::Value;

use crate::action::HttpMethod;
use crate::component::{
    ActionCardProps, AlertProps, AvatarProps, BadgeProps, BreadcrumbProps, ButtonProps, ButtonType,
    CalendarCellProps, ChecklistProps, DescriptionListProps, DropdownMenuAction, EmptyStateProps,
    HeaderProps, IconPosition, ImageProps, NotificationDropdownProps, Orientation, PaginationProps,
    ProgressProps, RawHtmlProps, SeparatorProps, SidebarNavItem, SidebarProps, Size, SkeletonProps,
    StatCardProps, StreamTextProps, TextElement, TextProps, TileProps, ToastProps, Tone, Variant,
};
use crate::spec::{Element, Spec};

use super::classes::{
    DISABLED_BASE, FOCUS_RING, HIT_TARGET_MIN, HIT_TARGET_NUMPAD, INTERACTIVE_BASE, MOTION_BASE,
    MOTION_FAST, PRESS_ACTIVE, TAP_HIGHLIGHT, TOUCH_ACTION,
};
// TOAST_TONE_* constants are defined in classes.rs for the runtime lockstep test
// (toast_tone_classes_match_ssr). render_toast now emits fjui-toast--{tone} literals
// (Plan 06 migration); the constants are no longer needed here.
use super::html_escape;

// ── Prop-decode diagnostic helper ────────────────────────────────────────

/// Emits a `<!-- ferro-json-ui: failed to decode TYPE props: MSG -->` comment.
/// Used as the fallback on serde_json::from_value decode errors across every
/// atom renderer.
fn decode_diagnostic(type_name: &str, err: impl std::fmt::Display) -> String {
    format!(
        "<!-- ferro-json-ui: failed to decode {} props: {} -->",
        type_name,
        html_escape(&err.to_string())
    )
}

/// Decode `el.props` into a typed `TProps` struct.
///
/// Treats `Value::Null` as an empty object: `Spec` serialization omits the
/// `props` field when it's null (via `skip_serializing_if`), so elements with
/// no user-provided props arrive here with `props = Null`. Structs whose fields
/// are all optional (e.g. `SeparatorProps`, `SkeletonProps`) serde-decode fine
/// from `{}` but not from `null` — this helper bridges that gap so
/// `Element::new("Separator")` (with no `.prop()` calls) renders correctly.
fn decode_props<TProps: serde::de::DeserializeOwned>(
    props: &Value,
) -> Result<TProps, serde_json::Error> {
    if props.is_null() {
        serde_json::from_value(Value::Object(serde_json::Map::new()))
    } else {
        serde_json::from_value(props.clone())
    }
}

// ── SVG icon constants ───────────────────────────────────────────────────

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

const BREADCRUMB_SEP: &str = concat!(
    "<span aria-hidden=\"true\" class=\"text-text-muted\">",
    "<svg class=\"h-4 w-4\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
    "<path fill-rule=\"evenodd\" d=\"M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z\" clip-rule=\"evenodd\"/>",
    "</svg></span>"
);

const BELL_SVG: &str = concat!(
    "<svg class=\"h-5 w-5\" fill=\"none\" stroke=\"currentColor\" viewBox=\"0 0 24 24\">",
    "<path stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" ",
    "d=\"M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9\"/>",
    "</svg>"
);

// ── 1. Text ──────────────────────────────────────────────────────────────

pub(crate) fn render_text(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: TextProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Text", e),
    };
    let content = html_escape(&props.content);
    // D-01: full string literals for fjui-text--* size roles.
    // Appearance (font-size, font-weight, color) owned by skin layer via fjui-text--* rules.
    // h1/display use the display scale (28px/600); h2/h3 use section (15px/600);
    // p/div/section/span use body (14px/400).
    match props.element {
        TextElement::P => format!("<p class=\"fjui-text--body\">{content}</p>"),
        TextElement::H1 => format!("<h1 class=\"fjui-text--display\">{content}</h1>"),
        TextElement::H2 => format!("<h2 class=\"fjui-text--section\">{content}</h2>"),
        TextElement::H3 => format!("<h3 class=\"fjui-text--section\">{content}</h3>"),
        TextElement::Span => format!("<span class=\"fjui-text--body\">{content}</span>"),
        TextElement::Div => format!("<div class=\"fjui-text--body\">{content}</div>"),
        TextElement::Section => format!("<section class=\"fjui-text--body\">{content}</section>"),
    }
}

// ── 2. Button ────────────────────────────────────────────────────────────

/// Emits the inner `<button>` markup for a `Button` element. Reads
/// `ButtonProps` (variant, size, icon, label, disabled) and produces the
/// styled button without any wrapping anchor — the anchor wrap is applied by
/// [`render_button`] when the element carries a GET action.
///
/// Appearance (color, radius, padding, font-size) is owned by the skin layer
/// via fjui-btn / fjui-btn--{variant} / fjui-btn--{size} classes (Plan 05).
/// Layout utilities (inline-flex, items-center, gap-2) remain inline per D-02.
fn render_button_inner(props: &ButtonProps) -> String {
    // Appearance classes are in ferro-skin.css @layer components. Rust emits
    // only semantic identifiers (D-01 full literals) + D-02 layout utilities.
    let variant_class = match props.variant {
        Variant::Primary     => "fjui-btn--primary",
        Variant::Secondary   => "fjui-btn--secondary",
        Variant::Outline     => "fjui-btn--outline",
        Variant::Ghost       => "fjui-btn--ghost",
        Variant::Destructive => "fjui-btn--destructive",
    };

    let size_class = match props.size {
        Size::Sm => "fjui-btn--sm",
        Size::Md => "fjui-btn--md",
        Size::Lg => "fjui-btn--lg",
    };

    // Disabled contract (D-16): the native `disabled` attr drives the skin's
    // :disabled state; the aria-disabled + literal pointer-events-none/opacity-50
    // pair keeps the treatment intact where markup is not a native control.
    let disabled_classes = if props.disabled == Some(true) {
        " pointer-events-none opacity-50"
    } else {
        ""
    };
    let disabled_attr = if props.disabled == Some(true) {
        " disabled aria-disabled=\"true\""
    } else {
        ""
    };

    let label = html_escape(&props.label);

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

    let type_attr = match props.button_type.as_ref() {
        Some(ButtonType::Button) => " type=\"button\"",
        Some(ButtonType::Submit) => " type=\"submit\"",
        None => "",
    };

    // HTML5 `form="<id>"` so a submit button outside its target form (e.g.
    // when rendered in a PageHeader actions slot) still submits that form.
    let form_attr = match props.form.as_deref() {
        Some(f) => format!(" form=\"{}\"", html_escape(f)),
        None => String::new(),
    };

    // Double-submit guard marker (D-16): the runtime setupFormGuards hook picks up
    // this attribute and disables the button after the first submission (bfcache-safe).
    let disable_on_submit_attr = if props.disable_on_submit == Some(true) {
        " data-disable-on-submit"
    } else {
        ""
    };

    // D-02 layout utilities (inline-flex, items-center, gap-2) stay inline.
    // INTERACTIVE_BASE + DISABLED_BASE stay inline for motion/focus/disabled
    // behavior utilities — they coexist with the skin's :focus-visible/:disabled rules.
    format!(
        "<button{type_attr}{form_attr}{disable_on_submit_attr} class=\"fjui-btn {variant_class} {size_class} inline-flex items-center gap-2 {INTERACTIVE_BASE} {DISABLED_BASE}{disabled_classes}\"{disabled_attr}>{content}</button>"
    )
}

pub(crate) fn render_button(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: ButtonProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Button", e),
    };

    // GET actions wrap the inner button in an `<a>` for href-driven navigation.
    // When the action has no resolved url, fall back to `href="#"` and emit a
    // diagnostic HTML comment.
    if let Some(action) = &el.action {
        if matches!(action.method, HttpMethod::Get) {
            // A disabled Button is never anchor-wrapped: `disabled:` variants do
            // not fire on `<a>`, so the wrap would still navigate (D-16). The
            // inner button carries aria-disabled + pointer-events-none opacity-50.
            if props.disabled == Some(true) {
                return render_button_inner(&props);
            }
            // Force the inner button to `type="button"` for the anchor-wrapped
            // path. Without it, a `<button>` nested inside any ambient `<form>`
            // (e.g. an edit-form cancel button) defaults to `type="submit"`
            // per HTML spec and submits the form instead of letting the
            // anchor's href fire — yielding a 404 on the form's POST target.
            let mut anchored_props = props.clone();
            anchored_props.button_type = Some(ButtonType::Button);
            let inner_html = render_button_inner(&anchored_props);

            let target_attr = match action.target.as_deref() {
                Some(t) => format!(" target=\"{}\" rel=\"noopener noreferrer\"", html_escape(t)),
                None => String::new(),
            };
            let (href, diagnostic) = match &action.url {
                Some(u) => (html_escape(u), String::new()),
                None => (
                    "#".to_string(),
                    format!(
                        "<!-- ferro-json-ui: action '{}' has no resolved url -->",
                        html_escape(action.handler.as_str())
                    ),
                ),
            };
            return format!(
                "{diagnostic}<a href=\"{href}\" class=\"block\"{target_attr}>{inner_html}</a>"
            );
        }
        // Non-GET actions: the inner button is returned as-is. The client
        // runtime reads `data-*` attributes added by the form/button wrapper
        // elsewhere.
    }
    render_button_inner(&props)
}

// ── 3. Badge ─────────────────────────────────────────────────────────────

pub(crate) fn render_badge(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: BadgeProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Badge", e),
    };
    badge_inline_html(props.tone, &props.label)
}

/// Render a Badge `<span>` from `(tone, label)`. Shared by `render_badge`, the
/// DataTable `ColumnFormat::Badge` cell renderer, and the MediaCardGrid footer
/// badge so all surfaces stay in lockstep.
///
/// Appearance (radius, padding, font-size, color) is owned by the skin layer
/// via fjui-badge / fjui-badge--{tone} (Plan 05). The skin rule sets
/// `width: max-content` which replaces the former inline justify-self workaround.
pub(crate) fn badge_inline_html(tone: Tone, label: &str) -> String {
    // D-01: full string literals — never format!("fjui-badge--{}", tone)
    let tone_class = match tone {
        Tone::Neutral     => "fjui-badge--neutral",
        Tone::Success     => "fjui-badge--success",
        Tone::Warning     => "fjui-badge--warning",
        Tone::Destructive => "fjui-badge--destructive",
    };
    // No inline style — skin rule `.fjui-badge { width: max-content }` handles chip-hug.
    format!("<span class=\"fjui-badge {tone_class}\">{}</span>", html_escape(label))
}

// ── 4. Alert ─────────────────────────────────────────────────────────────

pub(crate) fn render_alert(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: AlertProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Alert", e),
    };
    // D-01: full string literals — appearance owned by skin layer (Plan 05).
    // Neutral is a muted surface tint. Success/Warning/Destructive match Badge vocabulary.
    let tone_class = match props.tone {
        Tone::Neutral     => "fjui-alert--neutral",
        Tone::Success     => "fjui-alert--success",
        Tone::Warning     => "fjui-alert--warning",
        Tone::Destructive => "fjui-alert--destructive",
    };
    let icon = match props.tone {
        Tone::Neutral => ICON_INFO,
        Tone::Success => ICON_SUCCESS,
        Tone::Warning => ICON_WARNING,
        Tone::Destructive => ICON_ERROR,
    };
    // Layout utilities (flex, items-start, gap-3) stay inline per D-02.
    let mut html = format!(
        "<div role=\"alert\" class=\"fjui-alert {tone_class} flex items-start gap-3\">"
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

// ── 5. Separator ─────────────────────────────────────────────────────────

pub(crate) fn render_separator(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: SeparatorProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Separator", e),
    };
    let orientation = props.orientation.as_ref().cloned().unwrap_or_default();
    // D-01: fjui-separator--* full literals; layout utilities (my-4, mx-4, h-full) stay inline (D-02).
    // Appearance (border-color, background-color) owned by skin layer.
    match orientation {
        Orientation::Horizontal => "<hr class=\"fjui-separator--horizontal my-4\">".to_string(),
        Orientation::Vertical => "<div class=\"fjui-separator--vertical mx-4 h-full w-px\"></div>".to_string(),
    }
}

// ── 6. Progress ──────────────────────────────────────────────────────────

pub(crate) fn render_progress(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: ProgressProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Progress", e),
    };
    let max = props.max.unwrap_or(100) as f64;
    let pct = if max > 0.0 {
        ((props.value as f64 * 100.0 / max).round() as u8).min(100)
    } else {
        0
    };

    // D-01: fjui-progress* full literals; layout utility (w-full) stays inline (D-02).
    // Appearance (bg-color, border-radius) owned by skin layer.
    let mut html = String::from("<div class=\"fjui-progress w-full\" role=\"progressbar\">");
    if let Some(ref label) = props.label {
        html.push_str(&format!(
            "<div class=\"fjui-progress__label mb-1\">{}</div>",
            html_escape(label)
        ));
    }
    html.push_str(&format!(
        "<div class=\"fjui-progress__track w-full\"><div class=\"fjui-progress__fill\" style=\"width: {pct}%\"></div></div>"
    ));
    html.push_str("</div>");
    html
}

// ── 7. Avatar ────────────────────────────────────────────────────────────

pub(crate) fn render_avatar(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: AvatarProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Avatar", e),
    };
    let size = props.size.unwrap_or_default();
    // D-01: full literal fjui-avatar size classes. Layout (inline-flex, items-center,
    // justify-center) stays inline (D-02). Appearance (bg, color, radius) in skin.
    let size_class = match size {
        Size::Sm => "fjui-avatar--sm",
        Size::Md => "fjui-avatar--md",
        Size::Lg => "fjui-avatar--lg",
    };

    if let Some(ref src) = props.src {
        format!(
            "<img src=\"{}\" alt=\"{}\" class=\"fjui-avatar {size_class} object-cover\">",
            html_escape(src),
            html_escape(&props.alt),
        )
    } else {
        let fallback_text = props.fallback.as_deref().unwrap_or(&props.alt);
        let initials: String = fallback_text.chars().take(2).collect();
        format!(
            "<span class=\"fjui-avatar {size_class} inline-flex items-center justify-center\">{}</span>",
            html_escape(&initials)
        )
    }
}

// ── 8. Image ─────────────────────────────────────────────────────────────

pub(crate) fn render_image(el: &Element, _spec: &Spec, data: &Value, _depth: usize) -> String {
    let props: ImageProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Image", e),
    };

    // D-17: inline SVG branch — emit verbatim, no <img> tag.
    // Server-only; content is NOT sanitized; alt is HTML-escaped for the aria-label.
    if let Some(ref svg) = props.inline_svg {
        return format!(
            "<div aria-label=\"{}\">{}</div>",
            html_escape(&props.alt),
            svg // verbatim — intentionally not escaped (server-only trust)
        );
    }

    // D-15: data_path takes precedence over static src.
    let resolved_src = props
        .data_path
        .as_deref()
        .and_then(|p| crate::data::resolve_path(data, p))
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| props.src.clone());

    let container_style = match &props.aspect_ratio {
        Some(ratio) => format!(" style=\"aspect-ratio: {}\"", html_escape(ratio)),
        None => String::new(),
    };

    // D-01: fjui-image* full literals; layout utilities (absolute, inset-0, flex,
    // items-center, justify-center, relative, w-full, h-full) stay inline (D-02).
    // Appearance (bg-color, border-radius, text-size, text-color) owned by skin.
    let placeholder = match &props.placeholder_label {
        Some(label) => format!(
            "<div class=\"fjui-image__placeholder absolute inset-0 flex items-center justify-center\">{}</div>",
            html_escape(label)
        ),
        None => String::from("<div class=\"fjui-image__placeholder absolute inset-0\"></div>"),
    };

    format!(
        "<div class=\"fjui-image relative w-full\"{container_style}>\
            {placeholder}\
            <img src=\"{src}\" alt=\"{alt}\" \
                 class=\"fjui-image__img relative w-full h-full object-cover object-top\" \
                 loading=\"lazy\" onerror=\"this.style.display='none'\">\
         </div>",
        src = html_escape(&resolved_src),
        alt = html_escape(&props.alt),
    )
}

// ── 9. Skeleton ──────────────────────────────────────────────────────────

pub(crate) fn render_skeleton(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: SkeletonProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Skeleton", e),
    };
    let width = props.width.as_deref().unwrap_or("100%");
    let height = props.height.as_deref().unwrap_or("1rem");
    // D-01: fjui-skeleton full literal for the structural class.
    // SHIMMER_CSS keeps its own <style> block (pre-skin behavior animation — not appearance-token).
    // fjui-skeleton--rounded modifier selects pill vs rect radius via skin rule.
    let modifier = if props.rounded == Some(true) {
        "fjui-skeleton--rounded"
    } else {
        "fjui-skeleton--rect"
    };
    // Width and height are passed through `html_escape` before interpolation
    // into the `style` attribute to preserve the XSS discipline that every
    // interpolated string is escaped.
    format!(
        "{SHIMMER_CSS}<div class=\"fjui-skeleton {modifier} ferro-shimmer\" style=\"width: {}; height: {}\"></div>",
        html_escape(width),
        html_escape(height)
    )
}

// ── 10. Breadcrumb ───────────────────────────────────────────────────────

pub(crate) fn render_breadcrumb(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    let props: BreadcrumbProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Breadcrumb", e),
    };
    // D-01: fjui-breadcrumb* full literals; layout utilities (flex, items-center,
    // space-x-2) stay inline (D-02). Appearance (text-size, color) owned by skin.
    let mut html =
        String::from("<nav class=\"fjui-breadcrumb flex items-center space-x-2\">");
    let len = props.items.len();
    for (i, item) in props.items.iter().enumerate() {
        let is_last = i == len - 1;
        if is_last {
            html.push_str(&format!(
                "<span class=\"fjui-breadcrumb__item fjui-breadcrumb__item--current\">{}</span>",
                html_escape(&item.label)
            ));
        } else if let Some(ref url) = item.url {
            html.push_str(&format!(
                "<a href=\"{}\" class=\"fjui-breadcrumb__item {INTERACTIVE_BASE}\">{}</a>",
                html_escape(url),
                html_escape(&item.label)
            ));
        } else {
            html.push_str(&format!(
                "<span class=\"fjui-breadcrumb__item\">{}</span>",
                html_escape(&item.label)
            ));
        }
        if !is_last {
            html.push_str(BREADCRUMB_SEP);
        }
    }
    html.push_str("</nav>");
    html
}

// ── 11. Pagination ───────────────────────────────────────────────────────

pub(crate) fn render_pagination(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    let props: PaginationProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Pagination", e),
    };
    if props.total == 0 || props.per_page == 0 {
        return String::new();
    }

    let total_pages = props.total.div_ceil(props.per_page);
    if total_pages <= 1 {
        return String::new();
    }

    let base_url = props.base_url.as_deref().unwrap_or("?");
    // Clamp current_page to [1, total_pages] — defensive for callers that
    // didn't sanitize query-string input.
    let current = props.current_page.clamp(1, total_pages);

    let base_url_escaped = html_escape(base_url);

    // D-01: fjui-pagination* full literals; layout utility (flex, items-center,
    // space-x-1) stays inline (D-02). Appearance (bg, color, radius) owned by skin.
    let mut html = String::from("<nav class=\"fjui-pagination flex items-center space-x-1\">");

    // Previous button.
    if current > 1 {
        let prev = current - 1;
        html.push_str(&format!(
            "<a href=\"{base_url_escaped}page={prev}\" class=\"fjui-pagination__btn {INTERACTIVE_BASE}\">&laquo;</a>"
        ));
    }

    // Page numbers — show up to 7 with ellipsis.
    let pages = compute_page_range(current, total_pages);
    let mut prev_page = 0u32;
    for page in pages {
        if prev_page > 0 && page > prev_page + 1 {
            html.push_str("<span class=\"fjui-pagination__ellipsis px-2\">&hellip;</span>");
        }
        if page == current {
            html.push_str(&format!(
                "<span class=\"fjui-pagination__btn fjui-pagination__btn--active\">{page}</span>"
            ));
        } else {
            html.push_str(&format!(
                "<a href=\"{base_url_escaped}page={page}\" class=\"fjui-pagination__btn {INTERACTIVE_BASE}\">{page}</a>"
            ));
        }
        prev_page = page;
    }

    // Next button.
    if current < total_pages {
        let next = current + 1;
        html.push_str(&format!(
            "<a href=\"{base_url_escaped}page={next}\" class=\"fjui-pagination__btn {INTERACTIVE_BASE}\">&raquo;</a>"
        ));
    }

    html.push_str("</nav>");
    html
}

/// Computes the sequence of page numbers to display (up to 7 entries) given
/// the current page and the total page count. Always includes page 1 and the
/// last page, plus a window around the current page.
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

// ── 12. DescriptionList ──────────────────────────────────────────────────

pub(crate) fn render_description_list(
    el: &Element,
    _spec: &Spec,
    data: &Value,
    _depth: usize,
) -> String {
    let props: DescriptionListProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("DescriptionList", e),
    };

    // `data_path` takes precedence over static `items` when present.
    let items: Vec<crate::component::DescriptionItem> = props
        .data_path
        .as_deref()
        .and_then(|p| crate::data::resolve_path(data, p))
        .and_then(|v| v.as_array().cloned())
        .map(|arr| {
            arr.into_iter()
                .filter_map(|v| serde_json::from_value(v).ok())
                .collect()
        })
        .unwrap_or_else(|| props.items.clone());

    let columns = props.columns.unwrap_or(1);
    // D-01: fjui-description-list* full literals; layout utilities (grid, gap-4, mt-1)
    // stay inline (D-02). Appearance (text-size, color) owned by skin.
    // RSK-03: columns>=2 are responsive — grid-cols-1 on mobile, md:grid-cols-2 at >=768px.
    let grid_cols_class = match columns {
        1 => "grid-cols-1",
        2 => "grid-cols-1 md:grid-cols-2",
        _ => "grid-cols-1 md:grid-cols-2 lg:grid-cols-3",
    };
    let mut html = format!("<dl class=\"fjui-description-list grid {grid_cols_class} gap-4\">");
    for item in &items {
        if let Some(ref field) = item.inline_edit_field {
            let endpoint = item.inline_edit_endpoint.as_deref().unwrap_or("");
            let kind = item.inline_edit_kind.as_deref().unwrap_or("text");
            html.push_str(&format!(
                "<div>                 <dt class=\"fjui-description-list__term\">{label}</dt>                 <dd class=\"fjui-description-list__detail mt-1 fjui-inline-edit\"                     data-inline-edit-field=\"{field}\"                     data-inline-edit-endpoint=\"{endpoint}\"                     data-inline-edit-kind=\"{kind}\">                   <span class=\"fjui-inline-edit__value\">{value}</span>                   <button class=\"fjui-inline-edit__pencil\" type=\"button\" aria-label=\"Modifica {label}\">                     <svg width=\"14\" height=\"14\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7\"/><path d=\"M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z\"/></svg>                   </button>                 </dd>                 </div>",
                label = html_escape(&item.label),
                field = html_escape(field),
                endpoint = html_escape(endpoint),
                kind = html_escape(kind),
                value = html_escape(&item.value),
            ));
        } else {
            html.push_str(&format!(
                "<div><dt class=\"fjui-description-list__term\">{}</dt><dd class=\"fjui-description-list__detail mt-1\">{}</dd></div>",
                html_escape(&item.label),
                html_escape(&item.value)
            ));
        }
    }
    html.push_str("</dl>");
    html
}

// ── 13. EmptyState ───────────────────────────────────────────────────────

pub(crate) fn render_empty_state(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    let props: EmptyStateProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("EmptyState", e),
    };
    // Centered placeholder. One pattern only (RSK-04): icon + title + body + one CTA.
    // Layout utilities (flex, items-center, justify-center, min-h-40, py-8, px-6,
    // text-center, max-w-md) stay inline per D-02.
    // Appearance (background, border, radius, typography) is in the skin layer.
    let mut html = String::from(
        "<div class=\"fjui-empty-state flex items-center justify-center min-h-40 py-8 px-6\">\
         <div class=\"text-center max-w-md\">",
    );
    if !props.title.is_empty() {
        html.push_str(&format!(
            "<h3 class=\"text-base font-semibold text-text mb-2\">{}</h3>",
            html_escape(&props.title)
        ));
    }
    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"text-sm text-text-muted\">{}</p>",
            html_escape(desc)
        ));
    }
    if let Some(ref action) = props.action {
        let label = props.action_label.as_deref().unwrap_or("Action");
        let url = action.url.as_deref().unwrap_or("#");
        // CTA uses fjui-btn--primary fjui-btn--md (D-01 literals); layout
        // utilities inline-flex + items-center stay per D-02.
        html.push_str(&format!(
            "<a href=\"{}\" class=\"mt-4 fjui-btn fjui-btn--primary fjui-btn--md inline-flex items-center gap-2 {INTERACTIVE_BASE}\">{}</a>",
            html_escape(url),
            html_escape(label)
        ));
    }
    html.push_str("</div></div>");
    html
}

// ── 14. StatCard ─────────────────────────────────────────────────────────

pub(crate) fn render_stat_card(el: &Element, _spec: &Spec, data: &Value, _depth: usize) -> String {
    let props: StatCardProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("StatCard", e),
    };
    // Resolve the displayed value: value_path wins when it resolves against
    // handler data; falls back to the static `props.value` otherwise.
    let display_value = props
        .value_path
        .as_deref()
        .and_then(|p| crate::data::resolve_path_string(data, p))
        .unwrap_or_else(|| props.value.clone());

    // D-01: full string literals for tone modifier — appearance in skin layer (Plan 05).
    // Neutral maps to --default (border-only, no colour accent per UI-SPEC §StatCard).
    let tone_class = match props.tone {
        Tone::Neutral     => "fjui-stat-card--default",
        Tone::Success     => "fjui-stat-card--success",
        Tone::Warning     => "fjui-stat-card--warning",
        Tone::Destructive => "fjui-stat-card--destructive",
    };

    // Layout utilities (p-4) stay inline per D-02. Icon class stays as a
    // layout/color helper until the icon system migrates in a later plan.
    let icon_color_class = match props.tone {
        Tone::Neutral     => "",
        Tone::Success     => " text-success",
        Tone::Warning     => " text-warning",
        Tone::Destructive => " text-destructive",
    };

    let mut html = format!("<div class=\"fjui-stat-card {tone_class} p-4\">");
    if let Some(ref icon) = props.icon {
        // Icon string is emitted raw — trusted SVG supplied by server-side
        // authoring; not user input.
        html.push_str(&format!(
            "<span class=\"inline-block mb-2 w-6 h-6{icon_color_class}\">{icon}</span>"
        ));
    }
    // Label: skin rule handles font-size/color via .fjui-stat-card p.
    // Using an explicit meta-size class keeps the label readable if skin is absent.
    html.push_str(&format!(
        "<p class=\"text-sm text-text-muted\">{}</p>",
        html_escape(&props.label)
    ));
    // Value: fjui-stat-card__value — skin rule applies 24px/600/tabular-nums.
    if let Some(ref sse) = props.sse_target {
        html.push_str(&format!(
            "<p class=\"fjui-stat-card__value\" data-sse-target=\"{}\" data-live-value>{}</p>",
            html_escape(sse),
            html_escape(&display_value)
        ));
    } else {
        html.push_str(&format!(
            "<p class=\"fjui-stat-card__value\">{}</p>",
            html_escape(&display_value)
        ));
    }
    if let Some(ref subtitle) = props.subtitle {
        html.push_str(&format!(
            "<p class=\"text-xs text-text-muted mt-1\">{}</p>",
            html_escape(subtitle)
        ));
    }
    // Sparkline: sibling of the value element, aria-hidden (decorative trend, INS-03).
    // The SVG is Rust-generated from DB integers — no user input in path; safe to emit raw.
    if let Some(ref svg) = props.sparkline_svg {
        html.push_str("<div class=\"fjui-sparkline\" aria-hidden=\"true\">");
        html.push_str(svg); // trusted server-generated SVG; not html_escape'd (would corrupt SVG)
        html.push_str("</div>");
    }
    html.push_str("</div>");
    html
}

// ── 15. Checklist ────────────────────────────────────────────────────────

pub(crate) fn render_checklist(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: ChecklistProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Checklist", e),
    };
    // D-01: fjui-checklist full literal; layout utilities (flex, items-center,
    // justify-between, mb-3, p-4, space-y-2) stay inline (D-02).
    // Appearance (bg, border, radius, shadow, text-size, color) owned by skin.
    let mut html = String::from("<div class=\"fjui-checklist p-4\">");
    html.push_str("<div class=\"flex items-center justify-between mb-3\">");
    html.push_str(&format!(
        "<h3 class=\"fjui-checklist__title\">{}</h3>",
        html_escape(&props.title)
    ));
    if props.dismissible {
        let dismiss_label = props.dismiss_label.as_deref().unwrap_or("Dismiss");
        html.push_str(&format!(
            "<button type=\"button\" class=\"text-xs font-medium text-text hover:text-primary {INTERACTIVE_BASE}\" data-dismissible>{}</button>",
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
            html.push_str(&format!(
                "<input type=\"checkbox\" checked class=\"h-4 w-4 rounded-sm border-border text-primary {MOTION_FAST} {FOCUS_RING}\">"
            ));
        } else {
            html.push_str(&format!(
                "<input type=\"checkbox\" class=\"h-4 w-4 rounded-sm border-border text-primary {MOTION_FAST} {FOCUS_RING}\">"
            ));
        }
        let label_class = if item.checked {
            "text-sm line-through text-text-muted"
        } else {
            "text-sm text-text"
        };
        if let Some(ref href) = item.href {
            html.push_str(&format!(
                "<a href=\"{}\" class=\"{} {INTERACTIVE_BASE}\">{}</a>",
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

// ── 16. Toast ────────────────────────────────────────────────────────────

pub(crate) fn render_toast(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: ToastProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Toast", e),
    };
    // D-01: fjui-toast + fjui-toast--{tone} full literals. Overlay elevation:
    // shadow only (no border) per LANG-04. Appearance (bg, color, shadow, radius)
    // owned by skin. Layout utilities (fixed, top-4, right-4, z-50, flex,
    // items-start, gap-3, max-w-sm) stay inline (D-02).
    // The tone class is the semantic identifier; skin owns the actual color.
    let tone_class = match props.tone {
        Tone::Neutral     => "fjui-toast--neutral",
        Tone::Success     => "fjui-toast--success",
        Tone::Warning     => "fjui-toast--warning",
        Tone::Destructive => "fjui-toast--destructive",
    };
    // strum guarantees the wire string (see component.rs strum_tests).
    let tone_str = props.tone.as_ref();
    // `dismissible` (default true) emits the same [data-toast-close] button
    // the JS showToast() path creates; setupServerToasts() wires the click.
    // When NOT dismissible, clamp timeout to >= 1s so the auto-dismiss timer
    // is always scheduled — a `timeout: 0` toast without a close button would
    // otherwise cover the viewport corner with no way out.
    let timeout = props.timeout.unwrap_or(5);
    let timeout = if props.dismissible {
        timeout
    } else {
        timeout.max(1)
    };
    let close_button = if props.dismissible {
        "<button class=\"fjui-toast__close text-current opacity-70 hover:opacity-100 text-lg leading-none\" \
         data-toast-close>&times;</button>"
    } else {
        ""
    };
    format!(
        "<div class=\"fjui-toast {tone_class} fixed top-4 right-4 z-50 flex items-start gap-3 max-w-sm {MOTION_BASE}\" \
         data-toast-tone=\"{tone_str}\" data-toast-timeout=\"{timeout}\">\
         <span class=\"fjui-toast__message flex-1\">{}</span>\
         {close_button}\
         </div>",
        html_escape(&props.message)
    )
}

// ── 17. NotificationDropdown ─────────────────────────────────────────────

pub(crate) fn render_notification_dropdown(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    let props: NotificationDropdownProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("NotificationDropdown", e),
    };
    // D-01: fjui-notification-dropdown* full literals; layout utilities (relative,
    // absolute, right-0, mt-2, w-80, z-50, flex, items-start, gap-3, p-2, p-3, p-4,
    // min-w-0, flex-1, hidden, divide-y) stay inline (D-02).
    // Appearance (bg, border, shadow, radius, text-size, color) owned by skin.
    let unread_count = props.notifications.iter().filter(|n| !n.read).count();
    let mut html = String::from("<div class=\"relative\" data-notification-dropdown>");
    html.push_str(&format!(
        "<button type=\"button\" class=\"fjui-notification-dropdown__trigger relative p-2 {INTERACTIVE_BASE}\" data-notification-count=\"{unread_count}\">"
    ));
    html.push_str(BELL_SVG);
    if unread_count > 0 {
        html.push_str(&format!(
            "<span class=\"fjui-notification-dropdown__badge absolute top-0 right-0 inline-flex items-center justify-center h-4 w-4\">{unread_count}</span>"
        ));
    }
    html.push_str("</button>");
    html.push_str(
        "<div class=\"fjui-notification-dropdown__panel hidden absolute right-0 mt-2 w-80 z-50\" data-notification-panel>",
    );
    if props.notifications.is_empty() {
        let empty = props.empty_text.as_deref().unwrap_or("No notifications");
        html.push_str(&format!(
            "<p class=\"fjui-notification-dropdown__empty p-4\">{}</p>",
            html_escape(empty)
        ));
    } else {
        html.push_str("<ul class=\"divide-y divide-border\">");
        for item in &props.notifications {
            html.push_str("<li class=\"fjui-notification-dropdown__item flex items-start gap-3 p-3\">");
            if let Some(ref icon) = item.icon {
                html.push_str(&format!(
                    "<span class=\"text-lg shrink-0\">{}</span>",
                    html_escape(icon)
                ));
            }
            html.push_str("<div class=\"flex-1 min-w-0\">");
            if let Some(ref url) = item.action_url {
                html.push_str(&format!(
                    "<a href=\"{}\" class=\"fjui-notification-dropdown__item-link {INTERACTIVE_BASE}\">{}</a>",
                    html_escape(url),
                    html_escape(&item.text)
                ));
            } else {
                html.push_str(&format!(
                    "<p class=\"fjui-notification-dropdown__item-text\">{}</p>",
                    html_escape(&item.text)
                ));
            }
            if let Some(ref ts) = item.timestamp {
                html.push_str(&format!(
                    "<p class=\"fjui-notification-dropdown__timestamp mt-0.5\">{}</p>",
                    html_escape(ts)
                ));
            }
            html.push_str("</div>");
            if !item.read {
                html.push_str(
                    "<span class=\"fjui-notification-dropdown__unread-dot h-2 w-2 mt-1 shrink-0\"></span>",
                );
            }
            html.push_str("</li>");
        }
        html.push_str("</ul>");
    }
    html.push_str("</div></div>");
    html
}

// ── 18. Sidebar ──────────────────────────────────────────────────────────

pub(crate) fn render_sidebar(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: SidebarProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Sidebar", e),
    };
    // D-01: fjui-sidebar full literal. Layout utilities (flex, flex-col, h-full,
    // flex-1, overflow-y-auto, space-y-1, p-4) stay inline (D-02).
    // Appearance (bg, border) owned by skin.
    let mut html =
        String::from("<aside class=\"fjui-sidebar flex flex-col h-full\">");
    if !props.fixed_top.is_empty() {
        html.push_str("<nav class=\"p-4 space-y-1\">");
        for item in &props.fixed_top {
            html.push_str(&render_sidebar_nav_item(item));
        }
        html.push_str("</nav>");
    }
    if !props.groups.is_empty() {
        html.push_str("<div class=\"flex-1 overflow-y-auto p-4 flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">");
        for group in &props.groups {
            html.push_str("<div data-sidebar-group");
            if group.collapsed {
                html.push_str(" data-collapsed");
            }
            html.push('>');
            // D-01: fjui-sidebar__group-label full literal.
            // Appearance (font-size 11px/500, uppercase, muted color) owned by skin.
            html.push_str(&format!(
                "<p class=\"fjui-sidebar__group-label px-2 py-1\">{}</p>",
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

/// Renders a single sidebar nav item as an `<a>` element. `item.icon` is
/// emitted raw — trusted SVG supplied by server-side authoring; not user
/// input.
fn render_sidebar_nav_item(item: &SidebarNavItem) -> String {
    let disabled = item.disabled.unwrap_or(false);
    // D-01: fjui-sidebar__nav-item + fjui-sidebar__nav-item--active full literals.
    // Layout utilities (flex, items-center, gap-*, inline-flex, w-5, h-5, shrink-0)
    // stay inline (D-02). Appearance (bg, color, border-left, radius, font) in skin.
    let (tag, classes) = if disabled {
        (
            "span",
            "fjui-sidebar__nav-item flex items-center gap-2 opacity-50 pointer-events-none select-none".to_string(),
        )
    } else if item.active {
        (
            "a",
            format!("fjui-sidebar__nav-item fjui-sidebar__nav-item--active flex items-center gap-2 {INTERACTIVE_BASE}"),
        )
    } else {
        (
            "a",
            format!("fjui-sidebar__nav-item flex items-center gap-2 {INTERACTIVE_BASE}"),
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
            "<span class=\"inline-flex items-center justify-center w-5 h-5 shrink-0\">{icon}</span>"
        ));
    }
    html.push_str(&format!("{}</{tag}>", html_escape(&item.label)));
    html
}

// ── 19. Header ───────────────────────────────────────────────────────────

pub(crate) fn render_header(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: HeaderProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Header", e),
    };
    // D-01: fjui-header full literal. Layout utilities (flex, justify-between,
    // px-6, relative) are layout-boundary keepers (D-02). Appearance
    // (bg-*, border-*, py-*) moved to skin layer.
    let mut html = String::from(
        "<header class=\"fjui-header flex items-center justify-between px-6 relative\">",
    );
    html.push_str("<div></div>");
    html.push_str(&format!(
        "<span class=\"absolute left-1/2 -translate-x-1/2 text-lg font-semibold text-text pointer-events-none\">{}</span>",
        html_escape(&props.business_name)
    ));
    html.push_str("<div class=\"flex items-center gap-4\">");
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
            "<a href=\"{}\" class=\"text-sm text-text-muted hover:text-text {INTERACTIVE_BASE}\">Logout</a>",
            html_escape(logout)
        ));
    }
    html.push_str("</div></div></header>");
    html
}

// ── 20. DropdownMenu ─────────────────────────────────────────────────────

/// Render a single menu item shared by `DropdownMenu` and the inline
/// dropdown used by `DataTable.row_actions`. A `GET` action emits an
/// `<a href>`. `POST`/`PUT`/`PATCH`/`DELETE` emit a `<form method="post">`
/// with a `<button type="submit">`; `PUT`/`PATCH`/`DELETE` add a
/// `<input type="hidden" name="_method">` for HTTP-method spoofing.
/// `action.confirm` adds `data-confirm-*` attributes plus an
/// `onclick="return confirm(...)"` guard.
///
/// `role_attr` is inserted as a raw attribute fragment (e.g. ` role="menuitem"`)
/// or empty. Callers pass full Tailwind class strings — no additive
/// composition — so each surface keeps its own visual treatment.
pub(crate) fn render_menu_item(
    item: &DropdownMenuAction,
    normal_class: &str,
    destructive_class: &str,
    role_attr: &str,
) -> String {
    // `action.url` is populated by the spec-tree resolver for actions wired
    // into static elements. When items arrive via a `$data` binding (e.g.
    // `DropdownMenu.items: {"$data": "/orders/0/actions"}` inside `$each`),
    // they bypass that resolver — fall back to the handler literal so the
    // emitted `<a href>` / `<form action>` still points somewhere real.
    let url = match item.action.url.as_deref().filter(|s| !s.is_empty()) {
        Some(u) => u,
        None => {
            let h = item.action.handler.as_str();
            if h.is_empty() {
                "#"
            } else {
                h
            }
        }
    };
    let class_attr = if item.destructive {
        destructive_class
    } else {
        normal_class
    };

    let (confirm_attrs, onclick) = if let Some(ref confirm) = item.action.confirm {
        let mut attrs = format!(" data-confirm-title=\"{}\"", html_escape(&confirm.title));
        if let Some(ref message) = confirm.message {
            attrs.push_str(&format!(
                " data-confirm-message=\"{}\"",
                html_escape(message)
            ));
        }
        (
            attrs,
            " onclick=\"return confirm(this.dataset.confirmTitle || this.dataset.confirmMessage)\"",
        )
    } else {
        (String::new(), "")
    };

    match item.action.method {
        HttpMethod::Get => format!(
            "<a href=\"{}\"{} class=\"{}\"{}{}>{}</a>",
            html_escape(url),
            role_attr,
            class_attr,
            confirm_attrs,
            onclick,
            html_escape(&item.label),
        ),
        HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch | HttpMethod::Delete => {
            let method_spoof = match item.action.method {
                HttpMethod::Put => Some("PUT"),
                HttpMethod::Patch => Some("PATCH"),
                HttpMethod::Delete => Some("DELETE"),
                _ => None,
            };
            let mut html = format!("<form action=\"{}\" method=\"post\">", html_escape(url));
            if let Some(m) = method_spoof {
                html.push_str(&format!(
                    "<input type=\"hidden\" name=\"_method\" value=\"{m}\">"
                ));
            }
            html.push_str(&format!(
                "<button type=\"submit\"{} class=\"w-full text-left {}\"{}{}>{}</button>",
                role_attr,
                class_attr,
                confirm_attrs,
                onclick,
                html_escape(&item.label),
            ));
            html.push_str("</form>");
            html
        }
    }
}

// ── 21. CalendarCell ─────────────────────────────────────────────────────

pub(crate) fn render_calendar_cell(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    let props: CalendarCellProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("CalendarCell", e),
    };
    // D-01: fjui-calendar-cell* full literals; layout utilities (flex, flex-col,
    // min-h-[5rem], p-2, -mt-px, -ml-px, w-7, h-7, items-center, justify-center,
    // relative, absolute, inset-0, flex, gap-1, mt-auto, pt-1) stay inline (D-02).
    // Appearance (border, bg, text-size, color, radius) owned by skin.
    let out_of_month = if props.is_current_month { "" } else { " fjui-calendar-cell--other-month" };
    let closed_mod = if props.closed { " fjui-calendar-cell--closed relative" } else { "" };
    let hover_class = if props.is_current_month {
        format!(" {MOTION_FAST} cursor-pointer")
    } else {
        " cursor-pointer".to_string()
    };

    let mut html = format!(
        "<div class=\"fjui-calendar-cell flex flex-col min-h-[5rem] p-2 -mt-px -ml-px{out_of_month}{closed_mod}{hover_class}\">",
    );

    if props.is_today {
        html.push_str(&format!(
            "<span class=\"fjui-calendar-cell__day fjui-calendar-cell__day--today w-7 h-7 flex items-center justify-center\">{}</span>",
            props.day
        ));
    } else {
        html.push_str(&format!(
            "<span class=\"fjui-calendar-cell__day\">{}</span>",
            props.day
        ));
    }

    let total = if !props.dot_colors.is_empty() {
        props.dot_colors.len() as u32
    } else {
        props.event_count
    };
    if total > 0 && total <= 3 {
        html.push_str("<div class=\"flex gap-1 mt-auto pt-1\">");
        if props.dot_colors.is_empty() {
            for _ in 0..total {
                html.push_str("<span class=\"fjui-calendar-cell__dot w-1.5 h-1.5\"></span>");
            }
        } else {
            for color in props.dot_colors.iter().take(3) {
                // dot_colors are Tailwind bg-* utilities supplied by the caller — kept as-is
                // since they are caller-controlled appearance tokens (not skin-migrated).
                html.push_str(&format!(
                    "<span class=\"fjui-calendar-cell__dot w-1.5 h-1.5 rounded-full {}\"></span>",
                    html_escape(color)
                ));
            }
        }
        html.push_str("</div>");
    } else if total > 3 {
        html.push_str(&format!(
            "<span class=\"fjui-calendar-cell__overflow mt-auto pt-1\">{total} prenot.</span>"
        ));
    }

    // Closed-day marker: a neutral diagonal hatch (repeating stripes) — the
    // availability-calendar convention for "unavailable". `currentColor` keeps
    // it theme-aware (muted tone); `relative` on the cell anchors the overlay.
    if props.closed {
        html.push_str(
            "<div class=\"fjui-calendar-cell__closed-overlay pointer-events-none absolute inset-0\" \
             style=\"opacity:0.14;background-image:repeating-linear-gradient(45deg,transparent 0,transparent 5px,currentColor 5px,currentColor 6px)\" \
             aria-hidden=\"true\"></div>",
        );
    }

    html.push_str("</div>");

    // GET actions wrap the cell in an `<a>` so the click navigates. Mirrors
    // the Button render path; ignored for POST/PUT/PATCH/DELETE (CalendarCell
    // is a navigation-only surface, so non-GET actions are not exposed here).
    if let Some(action) = &el.action {
        if matches!(action.method, HttpMethod::Get) {
            let (href, diagnostic) = match &action.url {
                Some(u) => (html_escape(u), String::new()),
                None => (
                    "#".to_string(),
                    format!(
                        "<!-- ferro-json-ui: CalendarCell action '{}' has no resolved url -->",
                        html_escape(action.handler.as_str())
                    ),
                ),
            };
            return format!(
                "{diagnostic}<a href=\"{href}\" class=\"block {FOCUS_RING}\">{html}</a>"
            );
        }
    }

    html
}

// ── 22. ActionCard ───────────────────────────────────────────────────────

pub(crate) fn render_action_card(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    let props: ActionCardProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("ActionCard", e),
    };
    // D-01: fjui-action-card + fjui-action-card--{tone} full literals.
    // Layout utilities (flex, items-center, gap-4, p-4, no-underline) stay inline (D-02).
    // Appearance (border, bg, shadow, radius, hover) owned by skin.
    let tone_class = match props.tone {
        Tone::Neutral     => "fjui-action-card--neutral",
        Tone::Success     => "fjui-action-card--success",
        Tone::Warning     => "fjui-action-card--warning",
        Tone::Destructive => "fjui-action-card--destructive",
    };

    let (open_tag, close_tag) = if let Some(ref href) = props.href {
        (
            format!(
                "<a href=\"{}\" aria-label=\"{}\" class=\"fjui-action-card {tone_class} flex items-center gap-4 {INTERACTIVE_BASE} no-underline\">",
                html_escape(href),
                html_escape(&props.title),
            ),
            "</a>".to_string(),
        )
    } else {
        (
            format!(
                "<div class=\"fjui-action-card {tone_class} flex items-center gap-4 cursor-pointer {INTERACTIVE_BASE}\">"
            ),
            "</div>".to_string(),
        )
    };

    let mut html = open_tag;

    if let Some(ref icon) = props.icon {
        html.push_str(&format!(
            "<div class=\"fjui-action-card__icon w-10 h-10 flex-shrink-0 flex items-center justify-center\">{icon}</div>",
        ));
    }

    html.push_str(&format!(
        "<div class=\"flex-1 min-w-0\"><p class=\"fjui-action-card__title\">{}</p><p class=\"fjui-action-card__description mt-0.5\">{}</p></div>",
        html_escape(&props.title),
        html_escape(&props.description)
    ));

    html.push_str("<span class=\"fjui-action-card__chevron flex-shrink-0\">&rsaquo;</span>");

    html.push_str(&close_tag);
    html
}

// ── 23. Tile ──────────────────────────────────────────────────────

pub(crate) fn render_tile(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    use crate::component::Tone;

    let props: TileProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Tile", e),
    };
    let name = html_escape(&props.name);
    let price = html_escape(&props.price);
    let field = html_escape(&props.field);
    let qty = props.default_quantity.unwrap_or(0);

    // Token-list contract: the attribute value is space-separated, so a
    // space inside a category name would split it into two tokens.
    // Normalize spaces to hyphens; the Phase 255 filter runtime applies
    // the same normalization to category labels before matching.
    let categories_attr = if props.categories.is_empty() {
        String::new()
    } else {
        format!(
            " data-filter-tokens=\"{}\"",
            html_escape(
                &props
                    .categories
                    .iter()
                    .map(|c| c.replace(' ', "-"))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        )
    };

    // Emit data-unit-price only when price_cents is set (D-04).
    let unit_price_attr = match props.price_cents {
        Some(cents) => format!(" data-unit-price=\"{cents}\""),
        None => String::new(),
    };

    // D-01: fjui-tile + fjui-tile--{tone} full literals for tone modifier.
    // Appearance (border-color, bg, radius) owned by skin. Layout (touch-action,
    // w-full, flex, flex-col, gap-2, p-3) stays inline (D-02).
    let tone_class = match props.color {
        None | Some(Tone::Neutral)     => "fjui-tile--neutral",
        Some(Tone::Success)            => "fjui-tile--success",
        Some(Tone::Warning)            => "fjui-tile--warning",
        Some(Tone::Destructive)        => "fjui-tile--destructive",
    };

    // Optional image area (D-03): lazy-loaded when image_url is set.
    let image_html = match &props.image_url {
        Some(url) => {
            let escaped_url = html_escape(url);
            format!(
                "<img src=\"{escaped_url}\" alt=\"{name}\" loading=\"lazy\" \
                 class=\"w-full aspect-square object-cover rounded-md\">"
            )
        }
        None => String::new(),
    };

    // Optional stock badge chip (D-03) — uses fjui-badge skin class (already migrated in Plan 05).
    let badge_html = match &props.stock_badge {
        Some(badge) => {
            let escaped_badge = html_escape(badge);
            format!("<span class=\"fjui-badge fjui-badge--neutral\">{escaped_badge}</span>")
        }
        None => String::new(),
    };

    // Tile structure (D-01/D-02):
    // - Outer <div> wrapper carries filter/unit-price data attrs + fjui-tile tone class.
    // - Inner <button> is the entire tap surface (data-qty-inc).
    // - Hidden input is a SIBLING of the button (inputs inside <button> are
    //   invalid HTML — D-01 note).
    format!(
        "<div class=\"fjui-tile {tone_class} {TOUCH_ACTION}\" \
         data-filter-text=\"{name}\"{categories_attr}{unit_price_attr}>\
         <button type=\"button\" data-qty-inc=\"{field}\" \
         class=\"{HIT_TARGET_MIN} {TOUCH_ACTION} {PRESS_ACTIVE} {TAP_HIGHLIGHT} {INTERACTIVE_BASE} \
         w-full flex flex-col gap-2 p-3 text-left\" \
         aria-label=\"Add {name}\">\
         {image_html}\
         <span class=\"fjui-tile__name\">{name}</span>\
         <span class=\"fjui-tile__price\">{price}</span>\
         {badge_html}\
         </button>\
         <input type=\"hidden\" name=\"{field}\" data-qty-input=\"{field}\" value=\"{qty}\">\
         </div>"
    )
}

// ── Filter tab strip shared helper ──────────────────────────────────────

/// Renders the shared filter-tab strip markup used by both [`render_filter_tabs`]
/// (standalone) and `render_tile_grid` (integrated category strip).
///
/// Emits a `<div role="tablist">` with an All tab (active-initial, empty token)
/// followed by one button-tab per `items` entry (inactive-initial,
/// token = space→hyphen normalization matching `render_tile`'s token emission).
///
/// Inactive classes (`border-transparent text-text-muted hover:text-text`) are
/// exactly what `updateFilterTabClasses` in `runtime/filters.rs` toggles,
/// creating the D-12 lockstep contract.
pub(crate) fn render_filter_tab_strip(items: &[String], all_label: &str) -> String {
    let tab_base = format!(
        "{HIT_TARGET_MIN} {TOUCH_ACTION} {INTERACTIVE_BASE} px-4 text-sm font-medium border-b-2"
    );
    // `type="button"` is required: a filter-tab strip often lives inside a
    // <form> (e.g. a POS TileGrid). Without it the button defaults to `submit`,
    // so a category click submits the enclosing form → page reload → the
    // client-side filter resets.
    let all_tab = format!(
        "<button type=\"button\" role=\"tab\" data-filter-tab=\"\" \
         class=\"{tab_base} border-primary text-primary font-semibold\" \
         aria-selected=\"true\">{}</button>",
        html_escape(all_label)
    );
    let item_tabs: String = items
        .iter()
        .map(|item| {
            let token = html_escape(&item.replace(' ', "-"));
            let label = html_escape(item);
            format!(
                "<button type=\"button\" role=\"tab\" data-filter-tab=\"{token}\" \
                 class=\"{tab_base} border-transparent text-text-muted hover:text-text\" \
                 aria-selected=\"false\">{label}</button>"
            )
        })
        .collect();
    format!("<div class=\"flex overflow-x-auto\" role=\"tablist\">{all_tab}{item_tabs}</div>")
}

// ── FilterTabs — standalone touch filter-tab strip ───────────────────────

/// Renders a standalone `FilterTabs` component.
///
/// Wraps [`render_filter_tab_strip`] in a `data-filter-scope` root div so that
/// `setupFilters` binds the tab-strip to the nearest enclosing scope. Placing a
/// `FilterTabs` inside an existing `data-filter-scope` also works — `setupFilters`
/// resolves the scope via nearest-ancestor semantics (D-18).
pub(crate) fn render_filter_tabs(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    use crate::component::FilterTabsProps;

    let props: FilterTabsProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("FilterTabs", e),
    };
    let all_label = props.all_label.as_deref().unwrap_or("All");
    let strip = render_filter_tab_strip(&props.items, all_label);
    format!("<div data-filter-scope class=\"w-full\">{strip}</div>")
}

// ── RawHtml — server-injected HTML island ────────────────────────────────

pub(crate) fn render_raw_html(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: RawHtmlProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("RawHtml", e),
    };
    // Verbatim emission — intentionally NOT escaped (server-only trust).
    // See RawHtmlProps rustdoc for the trust boundary.
    format!("<div data-ferro-raw-html>{}</div>", props.html)
}

// ── StreamText — SSE token stream renderer ───────────────────────────────

pub(crate) fn render_streamtext(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    let props: StreamTextProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("StreamText", e),
    };
    let escaped_url = html_escape(&props.sse_url);
    let placeholder_html = props
        .placeholder
        .as_deref()
        .map(|t| {
            format!(
                "<span data-ferro-stream-placeholder class=\"text-sm text-text-muted\">{}</span>",
                html_escape(t)
            )
        })
        .unwrap_or_default();
    let loading_html = props
        .loading_text
        .as_deref()
        .map(|t| {
            format!(
                "<span data-ferro-stream-loading class=\"text-sm text-text-muted\">{}</span>",
                html_escape(t)
            )
        })
        .unwrap_or_default();
    format!("<div data-ferro-stream-url=\"{escaped_url}\">{placeholder_html}{loading_html}</div>")
}

// ── 24. QuantityStepper ─────────────────────────────────────────────────

/// Renders a self-contained `QuantityStepper` — dec button, qty display span,
/// inc button, and a hidden input on the `data-qty-*` contract.
///
/// When `min`/`max`/`step` are set, emits `data-qty-min`/`data-qty-max`/
/// `data-qty-step` on BOTH the inc and dec buttons so the `initQtyButton`
/// runtime can enforce bounds (D-22). Initial displayed value = `min.unwrap_or(0)`.
///
/// The visual class composition on inc/dec buttons is identical to the
/// SelectionPanel line steppers (D-22 shared-visual requirement).
pub(crate) fn render_quantity_stepper(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    use crate::component::QuantityStepperProps;

    let props: QuantityStepperProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("QuantityStepper", e),
    };

    let field = html_escape(&props.field);
    let initial = props.min.unwrap_or(0);

    // Optional bounds attributes emitted on BOTH inc and dec buttons (D-22).
    let min_attr = props
        .min
        .map(|v| format!(" data-qty-min=\"{v}\""))
        .unwrap_or_default();
    let max_attr = props
        .max
        .map(|v| format!(" data-qty-max=\"{v}\""))
        .unwrap_or_default();
    let step_attr = props
        .step
        .map(|v| format!(" data-qty-step=\"{v}\""))
        .unwrap_or_default();

    // Visual class composition — identical literal used in SelectionPanel line steppers.
    let btn_classes = format!(
        "{HIT_TARGET_MIN} {TOUCH_ACTION} {PRESS_ACTIVE} flex items-center justify-center \
         rounded-md border border-border bg-surface text-text text-lg font-semibold \
         hover:bg-border {INTERACTIVE_BASE}"
    );

    format!(
        "<div class=\"flex items-center gap-2\">\
         <button type=\"button\" data-qty-dec=\"{field}\"{min_attr}{max_attr}{step_attr} \
         class=\"{btn_classes}\" aria-label=\"Decrease {field}\">\u{2212}</button>\
         <span data-qty-display=\"{field}\" \
         class=\"text-sm font-semibold text-text min-w-[2ch] text-center\">{initial}</span>\
         <button type=\"button\" data-qty-inc=\"{field}\"{min_attr}{max_attr}{step_attr} \
         class=\"{btn_classes}\" aria-label=\"Increase {field}\">+</button>\
         <input type=\"hidden\" name=\"{field}\" data-qty-input=\"{field}\" value=\"{initial}\">\
         </div>"
    )
}

// ── 25. Numpad ────────────────────────────────────────────────────────────

/// Renders a `Numpad` — a tap-surface 3×4 keypad that writes to a hidden
/// field and NEVER renders a native input (so the mobile software keyboard
/// is never triggered).
///
/// Emits the exact Phase-255 attribute contract (D-23):
/// - Container with `data-numpad` + `data-numpad-target="{field}"`, plus
///   `data-numpad-mode="price"` only when `mode: Price` (absent in Quantity mode).
/// - `data-numpad-display` element the runtime overwrites on each key tap.
/// - 12-key 3×4 grid (1–9, clear, 0, backspace); each button carries
///   `data-numpad-key` and `HIT_TARGET_NUMPAD` (≥56px).
/// - `data-numpad-input` hidden field adjacent to the container.
///
/// The running total in T-256-10 is display-only — the server must
/// re-validate the submitted cents value.
pub(crate) fn render_numpad(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    use crate::component::{NumpadMode, NumpadProps};

    let props: NumpadProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Numpad", e),
    };

    let field = html_escape(&props.target_field);
    let mode_attr = match props.mode {
        NumpadMode::Price => " data-numpad-mode=\"price\"",
        NumpadMode::Quantity => "",
    };

    let key_classes = format!(
        "{HIT_TARGET_NUMPAD} {TOUCH_ACTION} {PRESS_ACTIVE} {TAP_HIGHLIGHT} flex items-center \
         justify-center rounded-md border border-border bg-surface text-text text-lg \
         font-semibold hover:bg-border {INTERACTIVE_BASE}"
    );

    // Key order: 1–9, then clear / 0 / backspace (3×4 grid, left-to-right, top-to-bottom).
    // Values are server-authored constants — no user input reaches data-numpad-key (T-256-09).
    let key_buttons: String = [
        ("1", "1"),
        ("2", "2"),
        ("3", "3"),
        ("4", "4"),
        ("5", "5"),
        ("6", "6"),
        ("7", "7"),
        ("8", "8"),
        ("9", "9"),
        ("clear", "Clear"),
        ("0", "0"),
        ("backspace", "Backspace"),
    ]
    .iter()
    .map(|(key_val, aria_label)| {
        let glyph = match *key_val {
            "backspace" => "\u{232B}",
            "clear" => "C",
            _ => key_val,
        };
        format!(
            "<button type=\"button\" data-numpad-key=\"{key_val}\" \
             class=\"{key_classes}\" aria-label=\"{aria_label}\">{glyph}</button>"
        )
    })
    .collect();

    format!(
        "<div class=\"flex flex-col gap-2\">\
         <div data-numpad data-numpad-target=\"{field}\"{mode_attr} \
         class=\"flex flex-col gap-2\">\
         <div data-numpad-display \
         class=\"text-2xl font-semibold text-text text-right px-2\">0</div>\
         <div class=\"grid grid-cols-3 gap-1\">{key_buttons}</div>\
         </div>\
         <input type=\"hidden\" name=\"{field}\" data-numpad-input=\"{field}\" value=\"\">\
         </div>"
    )
}

// ────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, HttpMethod};
    use crate::spec::{Element, Spec};
    use serde_json::json;

    /// Build a trivial Spec wrapping the given Element as `root`.
    fn spec_with_root(el: crate::spec::ElementBuilder) -> Spec {
        Spec::builder()
            .element("root", el)
            .build()
            .expect("trivial spec builds")
    }

    // ── 1. Text ─────────────────────────────────────────────────────────

    #[test]
    fn text_emits_paragraph_by_default() {
        let spec = spec_with_root(Element::new("Text").prop("content", "Hello"));
        let el = spec.elements.get("root").unwrap();
        let html = render_text(el, &spec, &json!({}), 1);
        assert!(html.contains("Hello"), "got: {html}");
        assert!(html.contains("<p"), "got: {html}");
        assert!(html.contains("</p>"), "got: {html}");
    }

    #[test]
    fn text_h1_variant() {
        let spec = spec_with_root(
            Element::new("Text")
                .prop("content", "Title")
                .prop("element", "h1"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_text(el, &spec, &json!({}), 1);
        assert!(html.contains("<h1"), "got: {html}");
        assert!(html.contains("Title"));
    }

    #[test]
    fn text_html_escaping_in_content() {
        let spec =
            spec_with_root(Element::new("Text").prop("content", "<script>alert(1)</script>"));
        let el = spec.elements.get("root").unwrap();
        let html = render_text(el, &spec, &json!({}), 1);
        assert!(html.contains("&lt;script&gt;"), "got: {html}");
        assert!(
            !html.contains("<script>"),
            "raw script tag must not appear; got: {html}"
        );
    }

    // ── 2. Button ───────────────────────────────────────────────────────

    #[test]
    fn button_bare_without_action_emits_button_tag() {
        let spec = spec_with_root(Element::new("Button").prop("label", "Go"));
        let el = spec.elements.get("root").unwrap();
        let html = render_button(el, &spec, &json!({}), 1);
        assert!(html.contains("<button"), "got: {html}");
        assert!(html.contains("Go"));
        assert!(
            !html.contains("<a href"),
            "no wrapping <a> without action; got: {html}"
        );
    }

    #[test]
    fn button_get_action_wraps_in_anchor() {
        let el_builder = Element::new("Button").prop("label", "Go").action(Action {
            handler: "users.list".into(),
            url: Some("/users".into()),
            method: HttpMethod::Get,
            confirm: None,
            on_success: None,
            on_error: None,
            target: None,
        });
        let spec = spec_with_root(el_builder);
        let el = spec.elements.get("root").unwrap();
        let html = render_button(el, &spec, &json!({}), 1);
        assert!(html.contains("<a href=\"/users\""), "got: {html}");
        assert!(
            html.contains("<button"),
            "inner button still emitted; got: {html}"
        );
    }

    /// Regression — a GET-action button must force `type="button"` on the
    /// inner `<button>`. Without it, if the rendered anchor lands inside
    /// any ambient `<form>` (e.g. an edit form's Cancel button), the inner
    /// button defaults to `type="submit"` per HTML spec and submits the
    /// form instead of letting the anchor's href fire.
    #[test]
    fn button_get_action_inner_button_has_type_button() {
        let el_builder = Element::new("Button")
            .prop("label", "Annulla")
            .action(Action {
                handler: "products.show".into(),
                url: Some("/dashboard/cassa/prodotti/1".into()),
                method: HttpMethod::Get,
                confirm: None,
                on_success: None,
                on_error: None,
                target: None,
            });
        let spec = spec_with_root(el_builder);
        let el = spec.elements.get("root").unwrap();
        let html = render_button(el, &spec, &json!({}), 1);
        assert!(html.contains("<a href=\"/dashboard/cassa/prodotti/1\""));
        assert!(
            html.contains("<button type=\"button\""),
            "GET-action inner button must carry type=\"button\"; got: {html}"
        );
    }

    #[test]
    fn button_action_url_none_uses_href_hash_with_diagnostic() {
        let el_builder = Element::new("Button").prop("label", "Go").action(Action {
            handler: "users.list".into(),
            url: None,
            method: HttpMethod::Get,
            confirm: None,
            on_success: None,
            on_error: None,
            target: None,
        });
        let spec = spec_with_root(el_builder);
        let el = spec.elements.get("root").unwrap();
        let html = render_button(el, &spec, &json!({}), 1);
        assert!(html.contains("href=\"#\""), "got: {html}");
        assert!(
            html.contains("<!-- ferro-json-ui: action 'users.list' has no resolved url -->"),
            "got: {html}"
        );
    }

    /// D-16 disabled contract: the native button carries the uniform
    /// disabled classes plus aria-disabled, and a disabled GET-action Button
    /// is never anchor-wrapped (an `<a>` would still navigate).
    #[test]
    fn button_disabled_skips_anchor_wrap_and_carries_aria_disabled() {
        let el_builder = Element::new("Button")
            .prop("label", "Go")
            .prop("disabled", true)
            .action(Action {
                handler: "users.list".into(),
                url: Some("/users".into()),
                method: HttpMethod::Get,
                confirm: None,
                on_success: None,
                on_error: None,
                target: None,
            });
        let spec = spec_with_root(el_builder);
        let el = spec.elements.get("root").unwrap();
        let html = render_button(el, &spec, &json!({}), 1);
        assert!(
            !html.contains("<a href"),
            "disabled Button must not be anchor-wrapped; got: {html}"
        );
        assert!(
            html.contains("aria-disabled=\"true\""),
            "aria-disabled on the disabled button; got: {html}"
        );
        assert!(
            html.contains("pointer-events-none opacity-50"),
            "literal disabled classes for non-native contexts; got: {html}"
        );
        assert!(
            html.contains("disabled:pointer-events-none"),
            "uniform DISABLED_BASE on the button base; got: {html}"
        );
    }

    /// INT-pass (251-02): the Button base sources the `--color-ring` token
    /// and the fast motion tier from the shared constants.
    #[test]
    fn button_carries_token_focus_ring_and_fast_motion() {
        let spec = spec_with_root(Element::new("Button").prop("label", "Go"));
        let el = spec.elements.get("root").unwrap();
        let html = render_button(el, &spec, &json!({}), 1);
        assert!(html.contains("focus-visible:ring-ring"), "got: {html}");
        assert!(html.contains("duration-fast"), "got: {html}");
        assert!(html.contains("ease-base"), "got: {html}");
    }

    #[test]
    fn button_post_action_does_not_wrap_in_anchor() {
        let el_builder = Element::new("Button").prop("label", "Save").action(Action {
            handler: "users.store".into(),
            url: Some("/users".into()),
            method: HttpMethod::Post,
            confirm: None,
            on_success: None,
            on_error: None,
            target: None,
        });
        let spec = spec_with_root(el_builder);
        let el = spec.elements.get("root").unwrap();
        let html = render_button(el, &spec, &json!({}), 1);
        assert!(!html.contains("<a href"), "POST must not wrap; got: {html}");
        assert!(html.contains("<button"));
    }

    #[test]
    fn button_get_action_target_blank_includes_rel_noopener() {
        let el_builder = Element::new("Button")
            .prop("label", "External")
            .action(Action {
                handler: "ext".into(),
                url: Some("https://example.com".into()),
                method: HttpMethod::Get,
                confirm: None,
                on_success: None,
                on_error: None,
                target: Some("_blank".into()),
            });
        let spec = spec_with_root(el_builder);
        let el = spec.elements.get("root").unwrap();
        let html = render_button(el, &spec, &json!({}), 1);
        assert!(html.contains("target=\"_blank\""), "got: {html}");
        assert!(html.contains("rel=\"noopener noreferrer\""), "got: {html}");
    }

    #[test]
    fn render_button_emits_disable_on_submit() {
        let spec = spec_with_root(
            Element::new("Button")
                .prop("label", "Confirm")
                .prop("disable_on_submit", true),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_button(el, &spec, &json!({}), 1);
        assert!(
            html.contains("data-disable-on-submit"),
            "Button with disable_on_submit:true must carry data-disable-on-submit; got: {html}"
        );
    }

    #[test]
    fn render_button_omits_disable_on_submit_by_default() {
        let spec = spec_with_root(Element::new("Button").prop("label", "Go"));
        let el = spec.elements.get("root").unwrap();
        let html = render_button(el, &spec, &json!({}), 1);
        assert!(
            !html.contains("data-disable-on-submit"),
            "Button without disable_on_submit must NOT carry data-disable-on-submit; got: {html}"
        );
    }

    // ── 3. Badge ────────────────────────────────────────────────────────

    #[test]
    fn badge_emits_label() {
        let spec = spec_with_root(Element::new("Badge").prop("label", "New"));
        let el = spec.elements.get("root").unwrap();
        let html = render_badge(el, &spec, &json!({}), 1);
        assert!(html.contains("New"), "got: {html}");
        assert!(html.contains("<span"), "got: {html}");
    }

    // ── 4. Alert ────────────────────────────────────────────────────────

    #[test]
    fn alert_emits_message_and_role() {
        let spec = spec_with_root(
            Element::new("Alert")
                .prop("message", "OK")
                .prop("variant", "info"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_alert(el, &spec, &json!({}), 1);
        assert!(html.contains("OK"), "got: {html}");
        assert!(html.contains("role=\"alert\""), "got: {html}");
    }

    // ── 5. Separator ────────────────────────────────────────────────────

    #[test]
    fn separator_default_is_horizontal_hr() {
        let spec = spec_with_root(Element::new("Separator"));
        let el = spec.elements.get("root").unwrap();
        let html = render_separator(el, &spec, &json!({}), 1);
        assert!(html.contains("<hr"), "got: {html}");
    }

    #[test]
    fn separator_vertical_is_div() {
        let spec = spec_with_root(Element::new("Separator").prop("orientation", "vertical"));
        let el = spec.elements.get("root").unwrap();
        let html = render_separator(el, &spec, &json!({}), 1);
        assert!(html.contains("<div"), "got: {html}");
        assert!(html.contains("w-px"), "got: {html}");
    }

    // ── 6. Progress ─────────────────────────────────────────────────────

    #[test]
    fn progress_emits_progressbar_role() {
        let spec = spec_with_root(Element::new("Progress").prop("value", 50));
        let el = spec.elements.get("root").unwrap();
        let html = render_progress(el, &spec, &json!({}), 1);
        assert!(html.contains("role=\"progressbar\""), "got: {html}");
        assert!(html.contains("50%"), "got: {html}");
    }

    // ── 7. Avatar ───────────────────────────────────────────────────────

    #[test]
    fn avatar_with_src_emits_img() {
        let spec = spec_with_root(
            Element::new("Avatar")
                .prop("src", "/img/a.png")
                .prop("alt", "Alice"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_avatar(el, &spec, &json!({}), 1);
        assert!(html.contains("<img"), "got: {html}");
        assert!(html.contains("src=\"/img/a.png\""), "got: {html}");
        assert!(html.contains("alt=\"Alice\""), "got: {html}");
    }

    #[test]
    fn avatar_with_no_src_emits_fallback_initials() {
        let spec = spec_with_root(
            Element::new("Avatar")
                .prop("alt", "Alice")
                .prop("fallback", "AL"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_avatar(el, &spec, &json!({}), 1);
        assert!(html.contains("AL"), "got: {html}");
        assert!(!html.contains("<img"), "no img without src; got: {html}");
    }

    // ── 8. Image ────────────────────────────────────────────────────────

    #[test]
    fn image_emits_img_tag() {
        let spec = spec_with_root(
            Element::new("Image")
                .prop("src", "/foo.png")
                .prop("alt", "hero"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_image(el, &spec, &json!({}), 1);
        assert!(html.contains("<img"), "got: {html}");
        assert!(html.contains("src=\"/foo.png\""));
        assert!(html.contains("alt=\"hero\""));
    }

    #[test]
    fn image_xss_src_escaped() {
        let spec = spec_with_root(
            Element::new("Image")
                .prop("src", "javascript:alert('xss')")
                .prop("alt", "test"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_image(el, &spec, &json!({}), 1);
        assert!(
            !html.contains("alert('xss')"),
            "single quote must be escaped; got: {html}"
        );
        assert!(html.contains("&#x27;"), "got: {html}");
    }

    #[test]
    fn image_xss_quote_break_out_escaped() {
        // Classic attribute-breakout attempt: `src` must be escaped before
        // interpolation into the `src=""` attribute.
        let spec = spec_with_root(
            Element::new("Image")
                .prop("src", "x\" onerror=\"alert(1)")
                .prop("alt", "Test"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_image(el, &spec, &json!({}), 1);
        assert!(
            html.contains("src=\"x&quot; onerror=&quot;alert(1)\""),
            "got: {html}"
        );
    }

    // ── 8b. Image inline_svg ─────────────────────────────────────────────

    #[test]
    fn image_inline_svg_renders_without_img_tag() {
        let spec = spec_with_root(
            Element::new("Image")
                .prop("src", "")
                .prop("alt", "chart")
                .prop("inline_svg", "<svg><rect/></svg>"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_image(el, &spec, &json!({}), 1);
        assert!(
            html.contains("<svg><rect/></svg>"),
            "inline SVG must be emitted verbatim; got: {html}"
        );
        assert!(
            html.contains("aria-label=\"chart\""),
            "aria-label must contain alt text; got: {html}"
        );
        assert!(
            !html.contains("<img"),
            "inline_svg path must not emit <img; got: {html}"
        );
    }

    #[test]
    fn image_inline_svg_escapes_alt_text() {
        let spec = spec_with_root(
            Element::new("Image")
                .prop("src", "")
                .prop("alt", "<script>x</script>")
                .prop("inline_svg", "<svg/>"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_image(el, &spec, &json!({}), 1);
        assert!(
            html.contains("aria-label=\"&lt;script&gt;x&lt;/script&gt;\""),
            "alt must be HTML-escaped in aria-label; got: {html}"
        );
        assert!(
            !html.contains("<script>"),
            "raw <script> must not appear; got: {html}"
        );
        // The SVG body itself is intentionally unaffected.
        assert!(
            html.contains("<svg/>"),
            "SVG body must still be verbatim; got: {html}"
        );
    }

    // ── 9. Skeleton ─────────────────────────────────────────────────────

    #[test]
    fn skeleton_emits_shimmer_div() {
        let spec = spec_with_root(Element::new("Skeleton"));
        let el = spec.elements.get("root").unwrap();
        let html = render_skeleton(el, &spec, &json!({}), 1);
        assert!(html.contains("ferro-shimmer"), "got: {html}");
    }

    // ── 10. Breadcrumb ──────────────────────────────────────────────────

    #[test]
    fn breadcrumb_emits_nav_with_items() {
        let spec = spec_with_root(Element::new("Breadcrumb").prop(
            "items",
            json!([{"label": "Home", "url": "/"}, {"label": "Dashboard"}]),
        ));
        let el = spec.elements.get("root").unwrap();
        let html = render_breadcrumb(el, &spec, &json!({}), 1);
        assert!(html.contains("<nav"), "got: {html}");
        assert!(html.contains("Home"));
        assert!(html.contains("Dashboard"));
    }

    // ── 11. Pagination ──────────────────────────────────────────────────

    #[test]
    fn pagination_emits_page_links() {
        let spec = spec_with_root(
            Element::new("Pagination")
                .prop("current_page", 1)
                .prop("per_page", 10)
                .prop("total", 50)
                .prop("base_url", "/list?"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_pagination(el, &spec, &json!({}), 1);
        assert!(html.contains("<nav"), "got: {html}");
        assert!(html.contains("page=2"), "got: {html}");
    }

    #[test]
    fn pagination_single_page_returns_empty() {
        let spec = spec_with_root(
            Element::new("Pagination")
                .prop("current_page", 1)
                .prop("per_page", 10)
                .prop("total", 5)
                .prop("base_url", "/list?"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_pagination(el, &spec, &json!({}), 1);
        assert!(html.is_empty(), "got: {html}");
    }

    #[test]
    fn pagination_clamps_current_page() {
        // current_page=99 with total_pages=5 should clamp to 5; page 99 must
        // not appear as a link. The active page must be 5.
        let spec = spec_with_root(
            Element::new("Pagination")
                .prop("current_page", 99)
                .prop("per_page", 10)
                .prop("total", 50)
                .prop("base_url", "/list?"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_pagination(el, &spec, &json!({}), 1);
        assert!(
            !html.contains(">99<"),
            "page 99 must be clamped away; got: {html}"
        );
        // Active page marker: a <span> (not <a>) with fjui-pagination__btn--active containing the page number.
        assert!(
            html.contains("fjui-pagination__btn--active\">5<"),
            "active page 5 missing; got: {html}"
        );
    }

    // ── Prop-decode diagnostic ─────────────────────────────────────────

    #[test]
    fn props_decode_failure_emits_diagnostic() {
        // Force a decode error by stashing a non-object props payload.
        let mut el = Element {
            type_name: "Text".into(),
            props: json!(42),
            children: Vec::new(),
            action: None,
            visible: None,
            each: None,
            if_: None,
        };
        // minimal spec just so we can call the renderer
        let spec = Spec::builder()
            .element("__dummy__", Element::new("Text").prop("content", "x"))
            .build()
            .expect("ok");
        // Call the renderer directly on the hand-built element.
        let html = render_text(&el, &spec, &json!({}), 1);
        assert!(
            html.contains("<!-- ferro-json-ui: failed to decode Text props"),
            "got: {html}"
        );
        el.type_name = "Badge".into();
        let html2 = render_badge(&el, &spec, &json!({}), 1);
        assert!(
            html2.contains("<!-- ferro-json-ui: failed to decode Badge props"),
            "got: {html2}"
        );
    }

    // ── 12. DescriptionList ─────────────────────────────────────────────

    #[test]
    fn description_list_emits_dl_dt_dd() {
        let spec = spec_with_root(
            Element::new("DescriptionList")
                .prop("items", json!([{"label": "Name", "value": "Alice"}])),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_description_list(el, &spec, &json!({}), 1);
        assert!(html.contains("<dl"), "got: {html}");
        assert!(html.contains("<dt"), "got: {html}");
        assert!(html.contains("<dd"), "got: {html}");
        assert!(html.contains("Name"));
        assert!(html.contains("Alice"));
    }

    // ── 13. EmptyState ─────────────────────────────────────────────────

    #[test]
    fn empty_state_emits_title() {
        let spec = spec_with_root(
            Element::new("EmptyState")
                .prop("title", "No data")
                .prop("description", "Try again"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_empty_state(el, &spec, &json!({}), 1);
        assert!(html.contains("No data"));
        assert!(html.contains("Try again"));
    }

    /// INT-pass (251-02): the EmptyState CTA (previously hover-only) carries
    /// the token focus ring and fast motion tier.
    #[test]
    fn empty_state_cta_carries_token_focus_ring() {
        let spec = spec_with_root(
            Element::new("EmptyState")
                .prop("title", "No data")
                .prop(
                    "action",
                    json!({"handler": "items.create", "url": "/items/new", "method": "GET"}),
                )
                .prop("action_label", "Create"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_empty_state(el, &spec, &json!({}), 1);
        assert!(html.contains("Create"), "got: {html}");
        assert!(
            html.contains("focus-visible:ring-ring"),
            "CTA must carry the token focus ring; got: {html}"
        );
        assert!(html.contains("duration-fast"), "got: {html}");
    }

    // ── 14. StatCard ───────────────────────────────────────────────────

    #[test]
    fn stat_card_emits_value() {
        let spec = spec_with_root(
            Element::new("StatCard")
                .prop("label", "Users")
                .prop("value", "42"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_stat_card(el, &spec, &json!({}), 1);
        assert!(html.contains("42"));
        assert!(html.contains("Users"));
    }

    /// OQ-2 (251-01 handoff): StatCard `tone` accents via fjui-stat-card--{tone} class;
    /// `neutral` (the default) maps to fjui-stat-card--default.
    /// Plan 05: value color is skin-owned; this test checks the tone class is present.
    #[test]
    fn stat_card_tone_accents_value() {
        let spec = spec_with_root(
            Element::new("StatCard")
                .prop("label", "Errors")
                .prop("value", "3")
                .prop("tone", "destructive"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_stat_card(el, &spec, &json!({}), 1);
        assert!(
            html.contains("fjui-stat-card--destructive"),
            "destructive tone must emit fjui-stat-card--destructive; got: {html}"
        );
        assert!(
            html.contains("fjui-stat-card__value"),
            "value element must carry fjui-stat-card__value; got: {html}"
        );
    }

    #[test]
    fn stat_card_neutral_tone_keeps_default_value_color() {
        let spec = spec_with_root(
            Element::new("StatCard")
                .prop("label", "Users")
                .prop("value", "42"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_stat_card(el, &spec, &json!({}), 1);
        // Plan 05: neutral tone maps to fjui-stat-card--default; skin owns color.
        assert!(
            html.contains("fjui-stat-card--default"),
            "neutral tone must emit fjui-stat-card--default; got: {html}"
        );
    }

    // ── 15. Checklist ──────────────────────────────────────────────────

    #[test]
    fn checklist_emits_items() {
        let spec = spec_with_root(
            Element::new("Checklist")
                .prop("title", "Todo")
                .prop("items", json!([{"label": "Buy milk", "checked": false}])),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_checklist(el, &spec, &json!({}), 1);
        assert!(html.contains("Buy milk"));
        assert!(html.contains("Todo"));
    }

    // ── 16. Toast ──────────────────────────────────────────────────────

    #[test]
    fn toast_emits_message() {
        let spec = spec_with_root(
            Element::new("Toast")
                .prop("message", "Saved")
                .prop("tone", "success"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_toast(el, &spec, &json!({}), 1);
        assert!(html.contains("Saved"));
        assert!(html.contains("data-toast-tone=\"success\""));
    }

    /// `dismissible` (default true) emits the [data-toast-close] button the
    /// runtime wires in setupServerToasts(); `dismissible: false` omits it
    /// and clamps `timeout: 0` to 1s so the toast can always be removed.
    #[test]
    fn toast_dismissible_emits_close_button_and_clamps_timeout() {
        // Default dismissible: close button present, timeout 0 preserved
        // (persistent until manually dismissed).
        let spec = spec_with_root(
            Element::new("Toast")
                .prop("message", "Hi")
                .prop("timeout", 0),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_toast(el, &spec, &json!({}), 1);
        assert!(html.contains("data-toast-close"), "got: {html}");
        assert!(html.contains("data-toast-timeout=\"0\""), "got: {html}");

        // Not dismissible: no close button, timeout clamped to >= 1.
        let spec = spec_with_root(
            Element::new("Toast")
                .prop("message", "Hi")
                .prop("timeout", 0)
                .prop("dismissible", false),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_toast(el, &spec, &json!({}), 1);
        assert!(!html.contains("data-toast-close"), "got: {html}");
        assert!(html.contains("data-toast-timeout=\"1\""), "got: {html}");
    }

    /// D-03 base tier: the Toast fade uses the duration-base token (in
    /// lockstep with runtime/toasts.rs), never a raw duration.
    #[test]
    fn toast_uses_base_motion_tier() {
        let spec = spec_with_root(Element::new("Toast").prop("message", "Saved"));
        let el = spec.elements.get("root").unwrap();
        let html = render_toast(el, &spec, &json!({}), 1);
        assert!(
            html.contains("transition-opacity duration-base ease-base"),
            "Toast must use the base motion tier; got: {html}"
        );
        assert!(!html.contains("duration-300"), "got: {html}");
    }

    // ── 17. NotificationDropdown ───────────────────────────────────────

    #[test]
    fn notification_dropdown_emits_items() {
        let spec = spec_with_root(
            Element::new("NotificationDropdown")
                .prop("notifications", json!([{"text": "Welcome", "read": false}])),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_notification_dropdown(el, &spec, &json!({}), 1);
        assert!(html.contains("Welcome"), "got: {html}");
    }

    // ── 18. Sidebar ────────────────────────────────────────────────────

    #[test]
    fn sidebar_emits_nav_items() {
        let spec = spec_with_root(
            Element::new("Sidebar").prop("fixed_top", json!([{"label": "Home", "href": "/"}])),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_sidebar(el, &spec, &json!({}), 1);
        assert!(html.contains("<aside"), "got: {html}");
        assert!(html.contains("Home"));
    }

    // ── 19. Header ─────────────────────────────────────────────────────

    #[test]
    fn header_emits_business_name() {
        let spec = spec_with_root(Element::new("Header").prop("business_name", "My Co"));
        let el = spec.elements.get("root").unwrap();
        let html = render_header(el, &spec, &json!({}), 1);
        assert!(html.contains("<header"), "got: {html}");
        assert!(html.contains("My Co"));
    }

    // ── 21. CalendarCell ───────────────────────────────────────────────

    #[test]
    fn calendar_cell_emits_day() {
        let spec = spec_with_root(Element::new("CalendarCell").prop("day", 15));
        let el = spec.elements.get("root").unwrap();
        let html = render_calendar_cell(el, &spec, &json!({}), 1);
        assert!(html.contains(">15<"), "got: {html}");
    }

    #[test]
    fn calendar_cell_closed_draws_hatch() {
        let spec = spec_with_root(
            Element::new("CalendarCell")
                .prop("day", 9)
                .prop("closed", true),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_calendar_cell(el, &spec, &json!({}), 1);
        // Neutral diagonal hatch overlay — no destructive tone, no text label.
        assert!(
            html.contains("repeating-linear-gradient"),
            "diagonal hatch missing: {html}"
        );
        assert!(html.contains(" relative"), "overlay anchor missing: {html}");
        assert!(
            !html.contains("destructive"),
            "closed marker should be neutral, not destructive: {html}"
        );
    }

    #[test]
    fn calendar_cell_open_has_no_hatch() {
        let spec = spec_with_root(Element::new("CalendarCell").prop("day", 9));
        let el = spec.elements.get("root").unwrap();
        let html = render_calendar_cell(el, &spec, &json!({}), 1);
        assert!(
            !html.contains("repeating-linear-gradient"),
            "unexpected hatch: {html}"
        );
    }

    // ── 22. ActionCard ─────────────────────────────────────────────────

    #[test]
    fn action_card_emits_title_and_description() {
        let spec = spec_with_root(
            Element::new("ActionCard")
                .prop("title", "Click me")
                .prop("description", "Go somewhere"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_action_card(el, &spec, &json!({}), 1);
        assert!(html.contains("Click me"));
        assert!(html.contains("Go somewhere"));
    }

    #[test]
    fn action_card_with_href_renders_anchor() {
        let spec = spec_with_root(
            Element::new("ActionCard")
                .prop("title", "Go")
                .prop("description", "Link")
                .prop("href", "/go"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_action_card(el, &spec, &json!({}), 1);
        assert!(html.contains("<a href=\"/go\""), "got: {html}");
    }

    // ── 23. Tile ────────────────────────────────────────────────

    /// INT-pass (251-02): Tile +/- buttons migrated to the ring token.
    #[test]
    fn tile_buttons_carry_token_focus_ring() {
        let spec = spec_with_root(
            Element::new("Tile")
                .prop("item_id", "1")
                .prop("name", "Caffè")
                .prop("price", "€1,20")
                .prop("field", "qty_1"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_tile(el, &spec, &json!({}), 1);
        assert!(
            html.contains("focus-visible:ring-ring"),
            "+/- buttons must carry the token ring; got: {html}"
        );
        assert!(
            !html.contains("focus-visible:ring-primary"),
            "retired ring-primary must not survive; got: {html}"
        );
    }

    #[test]
    fn tile_emits_name_and_price() {
        let spec = spec_with_root(
            Element::new("Tile")
                .prop("item_id", "p1")
                .prop("name", "Widget")
                .prop("price", "9.99")
                .prop("field", "qty_p1"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_tile(el, &spec, &json!({}), 1);
        assert!(html.contains("Widget"));
        assert!(html.contains("9.99"));
        assert!(html.contains("data-qty-inc=\"qty_p1\""), "got: {html}");
    }

    // ── 8c. Image data_path (D-15) ──────────────────────────────────────

    #[test]
    fn image_data_path_resolves_src_from_data() {
        let spec = spec_with_root(
            Element::new("Image")
                .prop("src", "fallback.png")
                .prop("alt", "x")
                .prop("data_path", "/avatar"),
        );
        let el = spec.elements.get("root").unwrap();
        let data = json!({"avatar": "real.png"});
        let html = render_image(el, &spec, &data, 1);
        assert!(
            html.contains(r#"src="real.png""#),
            "data_path override; got: {html}"
        );
        assert!(
            !html.contains("fallback.png"),
            "static src must be overridden; got: {html}"
        );
    }

    #[test]
    fn image_data_path_none_uses_static_src() {
        let spec = spec_with_root(
            Element::new("Image")
                .prop("src", "static.png")
                .prop("alt", "x"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_image(el, &spec, &json!({}), 1);
        assert!(
            html.contains(r#"src="static.png""#),
            "static src when no data_path; got: {html}"
        );
    }

    #[test]
    fn image_data_path_missing_in_data_falls_back_to_src() {
        let spec = spec_with_root(
            Element::new("Image")
                .prop("src", "fallback.png")
                .prop("alt", "x")
                .prop("data_path", "/missing"),
        );
        let el = spec.elements.get("root").unwrap();
        let data = json!({"other": "something"});
        let html = render_image(el, &spec, &data, 1);
        assert!(
            html.contains(r#"src="fallback.png""#),
            "must fall back to static src; got: {html}"
        );
    }

    // ── 12b. DescriptionList data_path (D-15) ───────────────────────────

    #[test]
    fn description_list_data_path_overrides_static_items() {
        let spec = spec_with_root(
            Element::new("DescriptionList")
                .prop("items", json!([{"label": "Static", "value": "X"}]))
                .prop("data_path", "/items"),
        );
        let el = spec.elements.get("root").unwrap();
        let data = json!({"items": [{"label": "Dynamic", "value": "Y"}]});
        let html = render_description_list(el, &spec, &data, 1);
        assert!(html.contains("Dynamic"), "data_path items; got: {html}");
        assert!(
            !html.contains("Static"),
            "static items must be overridden; got: {html}"
        );
    }

    #[test]
    fn description_list_data_path_serde_roundtrip() {
        use crate::component::DescriptionListProps;
        let p = DescriptionListProps {
            items: vec![],
            columns: None,
            data_path: Some("/items".to_string()),
        };
        let j = serde_json::to_value(&p).unwrap();
        let back: DescriptionListProps = serde_json::from_value(j).unwrap();
        assert_eq!(p, back);
    }

    // ── RawHtml (D-17a) ─────────────────────────────────────────────────

    #[test]
    fn raw_html_props_serde_roundtrip() {
        use crate::component::RawHtmlProps;
        let p = RawHtmlProps {
            html: "<span>x</span>".into(),
        };
        let j = serde_json::to_value(&p).unwrap();
        let back: RawHtmlProps = serde_json::from_value(j).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn render_raw_html_emits_verbatim() {
        let spec = spec_with_root(Element::new("RawHtml").prop("html", "<b>hi</b>"));
        let el = spec.elements.get("root").unwrap();
        let html = render_raw_html(el, &spec, &json!(null), 1);
        assert_eq!(html, "<div data-ferro-raw-html><b>hi</b></div>");
    }

    #[test]
    fn render_raw_html_null_props_emits_diagnostic() {
        let spec = Spec::builder()
            .element("root", Element::new("RawHtml"))
            .build()
            .unwrap();
        // Build element with non-object props to force decode failure
        let el = crate::spec::Element {
            type_name: "RawHtml".into(),
            props: json!(42),
            children: vec![],
            action: None,
            visible: None,
            each: None,
            if_: None,
        };
        let html = render_raw_html(&el, &spec, &json!(null), 1);
        assert!(
            html.contains("<!-- ferro-json-ui: failed to decode RawHtml props"),
            "got: {html}"
        );
    }

    #[test]
    fn builtin_types_includes_raw_html() {
        assert!(crate::render::BUILTIN_TYPES.contains(&"RawHtml"));
    }

    // ── StreamText (D-04 / AISSE-02 SC#1, SC#2a, SC#2b) ──────────────────────

    #[test]
    fn stream_text_props_serde_roundtrip() {
        use crate::component::StreamTextProps;
        let p = StreamTextProps {
            sse_url: "/ai/stream".to_string(),
            placeholder: Some("Response will appear here\u{2026}".to_string()),
            loading_text: Some("Generating\u{2026}".to_string()),
        };
        let j = serde_json::to_value(&p).unwrap();
        let back: StreamTextProps = serde_json::from_value(j).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn stream_text_props_minimal_serde_roundtrip() {
        use crate::component::StreamTextProps;
        let p = StreamTextProps {
            sse_url: "/stream".to_string(),
            placeholder: None,
            loading_text: None,
        };
        let j = serde_json::to_value(&p).unwrap();
        assert!(
            j.get("placeholder").is_none(),
            "placeholder must be absent when None"
        );
        assert!(
            j.get("loading_text").is_none(),
            "loading_text must be absent when None"
        );
        let back: StreamTextProps = serde_json::from_value(j).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn render_streamtext_emits_data_attribute() {
        let spec = spec_with_root(Element::new("StreamText").prop("sse_url", "/api/stream"));
        let el = spec.elements.get("root").unwrap();
        let html = render_streamtext(el, &spec, &json!(null), 1);
        assert!(
            html.contains("data-ferro-stream-url=\"/api/stream\""),
            "got: {html}"
        );
    }

    #[test]
    fn render_streamtext_escapes_url() {
        // T-169-01: a url containing "><script> must not break out of the attribute.
        let spec =
            spec_with_root(Element::new("StreamText").prop("sse_url", "/stream?q=a&b=\"><script>"));
        let el = spec.elements.get("root").unwrap();
        let html = render_streamtext(el, &spec, &json!(null), 1);
        // Raw ampersand and raw angle bracket from the url must not survive.
        assert!(!html.contains("&b="), "raw & must be escaped; got: {html}");
        assert!(
            !html.contains("<script>"),
            "raw <script> must be escaped; got: {html}"
        );
    }

    // ── 14. StatCard value_path resolution ──────────────────────────────

    #[test]
    fn stat_card_value_path_resolves_from_data() {
        // Gap C: when value_path is set and resolves against handler data,
        // the resolved string is displayed instead of the static `value`.
        let spec = spec_with_root(
            Element::new("StatCard")
                .prop("label", "Revenue")
                .prop("value", "")
                .prop("value_path", "/data/statistics/total_revenue"),
        );
        let el = spec.elements.get("root").unwrap();
        let data = json!({"data": {"statistics": {"total_revenue": "€12,450"}}});
        let html = render_stat_card(el, &spec, &data, 1);
        assert!(
            html.contains("€12,450"),
            "resolved value must appear in HTML; got: {html}"
        );
    }

    #[test]
    fn stat_card_value_path_fallback_to_static_value() {
        // Gap C fallback: when value_path is None, the static `value` is displayed.
        let spec = spec_with_root(
            Element::new("StatCard")
                .prop("label", "Revenue")
                .prop("value", "static"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_stat_card(el, &spec, &json!({}), 1);
        assert!(
            html.contains("static"),
            "static value must appear when value_path is absent; got: {html}"
        );
    }

    // ── 23. Tile ─────────────────────────────────────────────────

    fn make_tile(categories: Vec<&str>) -> (crate::spec::Element, Spec) {
        use crate::spec::Element as SpecElement;
        let mut el_builder = SpecElement::new("Tile")
            .prop("item_id", "p1")
            .prop("name", "Espresso")
            .prop("price", "€2,50")
            .prop("field", "qty_espresso");
        if !categories.is_empty() {
            el_builder = el_builder.prop("categories", categories);
        }
        let spec = spec_with_root(el_builder);
        let el = spec.elements.get("root").unwrap().clone();
        (el, spec)
    }

    /// Tap-to-add redesign (D-01/D-02): tile root is a wrapper <div>; inner
    /// <button> carries data-qty-inc; hidden input is a sibling of the button.
    /// No on-tile qty display, no dec button, no Italian aria-labels.
    #[test]
    fn tile_tap_to_add_emits_qty_inc_button() {
        use crate::spec::Element as SpecElement;
        let spec = spec_with_root(
            SpecElement::new("Tile")
                .prop("item_id", "p1")
                .prop("name", "Coffee")
                .prop("price", "€2,00")
                .prop("field", "p1"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_tile(el, &spec, &json!({}), 1);

        assert!(
            html.contains("data-qty-inc=\"p1\""),
            "must emit data-qty-inc; got: {html}"
        );
        assert!(
            html.contains("data-qty-input=\"p1\""),
            "must emit hidden input; got: {html}"
        );
        assert!(
            !html.contains("data-qty-display"),
            "tap-to-add: no on-tile qty display; got: {html}"
        );
        assert!(
            !html.contains("data-qty-dec"),
            "tap-to-add: no dec button on tile; got: {html}"
        );
        assert!(
            html.contains("Add Coffee"),
            "neutral English aria-label must say 'Add <name>'; got: {html}"
        );
        // data-filter-text always emitted (D-09 / universal tile marker).
        assert!(
            html.contains("data-filter-text=\"Coffee\""),
            "data-filter-text must always be emitted; got: {html}"
        );
        // Input must NOT be a descendant of <button> (invalid HTML).
        let btn_end = html.find("</button>").expect("must have </button>");
        let input_pos = html
            .find("data-qty-input")
            .expect("must have data-qty-input");
        assert!(
            input_pos > btn_end,
            "hidden input must be a sibling of <button>, not inside it; got: {html}"
        );
    }

    /// Text-only tile (no image_url) must not emit an <img> element.
    #[test]
    fn tile_text_only_when_no_image() {
        let (el, spec) = make_tile(vec![]);
        let html = render_tile(&el, &spec, &json!({}), 1);
        assert!(
            !html.contains("<img"),
            "no image_url → no <img> in output; got: {html}"
        );
    }

    /// Tile with image_url emits a lazy-loaded <img>; stock_badge renders a chip.
    #[test]
    fn tile_renders_image_and_badge() {
        use crate::spec::Element as SpecElement;
        let spec = spec_with_root(
            SpecElement::new("Tile")
                .prop("item_id", "p2")
                .prop("name", "Latte")
                .prop("price", "€3,50")
                .prop("field", "qty_latte")
                .prop("image_url", "https://example.com/latte.jpg")
                .prop("stock_badge", "Low"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_tile(el, &spec, &json!({}), 1);
        assert!(
            html.contains("<img"),
            "image_url set → must emit <img>; got: {html}"
        );
        assert!(
            html.contains("loading=\"lazy\""),
            "image must be lazy-loaded; got: {html}"
        );
        assert!(
            html.contains("Low"),
            "stock_badge must render badge chip; got: {html}"
        );
    }

    #[test]
    fn tile_emits_data_filter_tokens() {
        let (el, spec) = make_tile(vec!["drinks", "food"]);
        let html = render_tile(&el, &spec, &json!({}), 1);
        assert_eq!(
            html.matches("data-filter-tokens").count(),
            1,
            "expected exactly one data-filter-tokens attribute; got: {html}"
        );
        assert!(
            html.contains("data-filter-tokens=\"drinks food\""),
            "expected space-separated tokens; got: {html}"
        );
    }

    #[test]
    fn tile_normalizes_spaces_in_category_names() {
        // Token-list contract: a space inside a category name would split it
        // into two tokens, so render normalizes spaces to hyphens (WR-01).
        let (el, spec) = make_tile(vec!["Bevande calde", "food"]);
        let html = render_tile(&el, &spec, &json!({}), 1);
        assert!(
            html.contains("data-filter-tokens=\"Bevande-calde food\""),
            "expected space-in-name normalized to hyphen; got: {html}"
        );
    }

    #[test]
    fn tile_escapes_categories() {
        // T-255-01: XSS regression guard — html_escape must be applied to
        // category values emitted in data-filter-tokens.
        let (el, spec) = make_tile(vec!["caf\u{00E9}", "\"special\""]);
        let html = render_tile(&el, &spec, &json!({}), 1);
        assert!(
            !html.contains("\"special\""),
            "raw double-quote must not appear in attribute value; got: {html}"
        );
        assert!(
            html.contains("&quot;") || html.contains("&#34;"),
            "double-quote must be HTML-escaped; got: {html}"
        );
    }

    #[test]
    fn tile_escapes_filter_text() {
        // T-255-07: XSS guard — html_escape applied to props.name in the
        // unconditional data-filter-text attribute (D-08).
        use crate::spec::Element as SpecElement;
        let el_builder = SpecElement::new("Tile")
            .prop("item_id", "p1")
            .prop("name", "Ba\"r <item>")
            .prop("price", "\u{20AC}1,00")
            .prop("field", "qty_test");
        let spec = spec_with_root(el_builder);
        let el = spec.elements.get("root").unwrap().clone();
        let html = render_tile(&el, &spec, &json!({}), 1);
        assert!(
            html.contains("data-filter-text="),
            "data-filter-text must always be emitted; got: {html}"
        );
        // Raw double-quote must not break the attribute boundary.
        assert!(
            !html.contains("data-filter-text=\"Ba\""),
            "raw double-quote must not break attribute boundary in data-filter-text; got: {html}"
        );
        assert!(
            html.contains("&quot;") || html.contains("&#34;"),
            "double-quote in name must be HTML-escaped in data-filter-text; got: {html}"
        );
        assert!(
            html.contains("&lt;"),
            "less-than in name must be HTML-escaped in data-filter-text; got: {html}"
        );
    }

    // ── FilterTabs ──────────────────────────────────────────────────────

    #[test]
    fn filter_tabs_all_label_defaults_to_all() {
        let spec = spec_with_root(Element::new("FilterTabs"));
        let el = spec.elements.get("root").unwrap();
        let html = render_filter_tabs(el, &spec, &json!({}), 1);
        assert!(
            html.contains(">All<"),
            "default all_label must be 'All'; got: {html}"
        );
        assert!(
            !html.contains("Tutte"),
            "must not contain locale string 'Tutte'; got: {html}"
        );
    }

    #[test]
    fn filter_tabs_emits_scope_and_tabs() {
        let spec = spec_with_root(
            Element::new("FilterTabs")
                .prop("items", vec!["Drinks", "Food"])
                .prop("all_label", "All"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_filter_tabs(el, &spec, &json!({}), 1);
        assert!(
            html.contains("data-filter-scope"),
            "must carry data-filter-scope; got: {html}"
        );
        assert!(
            html.contains("data-filter-tab=\"\""),
            "All tab must carry empty data-filter-tab; got: {html}"
        );
        assert!(
            html.contains("data-filter-tab=\"Drinks\""),
            "must have Drinks tab; got: {html}"
        );
        assert!(
            html.contains("data-filter-tab=\"Food\""),
            "must have Food tab; got: {html}"
        );
    }

    #[test]
    fn filter_tabs_buttons_are_type_button() {
        // Regression: a filter-tab strip often lives inside a <form> (e.g. a POS
        // TileGrid register). A <button> with no explicit `type` defaults to
        // `submit`, so a category click submits the enclosing form → full page
        // reload → the client-side filter resets. Every tab button must be an
        // explicit non-submit button.
        let spec = spec_with_root(
            Element::new("FilterTabs")
                .prop("items", vec!["Drinks"])
                .prop("all_label", "All"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_filter_tabs(el, &spec, &json!({}), 1);
        assert!(
            !html.contains("<button role="),
            "filter-tab buttons must not omit type (would default to submit); got: {html}"
        );
        assert!(
            html.contains("<button type=\"button\" role=\"tab\""),
            "filter-tab buttons must be explicit type=\"button\"; got: {html}"
        );
    }

    #[test]
    fn filter_tabs_inactive_classes_match_runtime() {
        // Exact class set that updateFilterTabClasses in runtime/filters.rs
        // adds/removes on inactive tabs (D-12 lockstep).
        let spec = spec_with_root(Element::new("FilterTabs").prop("items", vec!["Drinks"]));
        let el = spec.elements.get("root").unwrap();
        let html = render_filter_tabs(el, &spec, &json!({}), 1);
        assert!(
            html.contains("border-transparent"),
            "inactive tab must have border-transparent; got: {html}"
        );
        assert!(
            html.contains("text-text-muted"),
            "inactive tab must have text-text-muted; got: {html}"
        );
        assert!(
            html.contains("hover:text-text"),
            "inactive tab must have hover:text-text; got: {html}"
        );
    }

    #[test]
    fn filter_tabs_targets_are_44px() {
        let spec = spec_with_root(Element::new("FilterTabs").prop("items", vec!["Coffee"]));
        let el = spec.elements.get("root").unwrap();
        let html = render_filter_tabs(el, &spec, &json!({}), 1);
        assert!(
            html.contains("min-h-[44px]"),
            "tabs must carry HIT_TARGET_MIN (min-h-[44px]); got: {html}"
        );
    }

    #[test]
    fn filter_tabs_tokens_match_tile_tokens() {
        // SC-5 pairing: FilterTabs items["Drinks","Food"] emits data-filter-tab="Drinks".
        // Tile with categories:["Drinks"] emits data-filter-tokens="Drinks" (space→hyphen).
        let spec = spec_with_root(Element::new("FilterTabs").prop("items", vec!["Drinks", "Food"]));
        let el = spec.elements.get("root").unwrap();
        let html = render_filter_tabs(el, &spec, &json!({}), 1);
        assert!(
            html.contains("data-filter-tab=\"Drinks\""),
            "FilterTabs emits tab token 'Drinks'; got: {html}"
        );
        // Tile side: render_tile emits data-filter-tokens="Drinks" for categories:["Drinks"]
        let tile_spec = spec_with_root(
            Element::new("Tile")
                .prop("item_id", "p1")
                .prop("name", "Espresso")
                .prop("price", "€2,50")
                .prop("field", "qty_espresso")
                .prop("categories", vec!["Drinks"]),
        );
        let tile_el = tile_spec.elements.get("root").unwrap();
        let tile_html = render_tile(tile_el, &tile_spec, &json!({}), 1);
        assert!(
            tile_html.contains("data-filter-tokens=\"Drinks\""),
            "Tile with categories:['Drinks'] must carry data-filter-tokens=\"Drinks\"; got: {tile_html}"
        );
    }

    // ── 24. QuantityStepper ─────────────────────────────────────────────

    #[test]
    fn quantity_stepper_self_contained() {
        let spec = spec_with_root(Element::new("QuantityStepper").prop("field", "qty_coffee"));
        let el = spec.elements.get("root").unwrap();
        let html = render_quantity_stepper(el, &spec, &json!({}), 1);
        assert!(
            html.contains("data-qty-dec=\"qty_coffee\""),
            "dec button missing; got: {html}"
        );
        assert!(
            html.contains("data-qty-display=\"qty_coffee\""),
            "display span missing; got: {html}"
        );
        assert!(
            html.contains("data-qty-inc=\"qty_coffee\""),
            "inc button missing; got: {html}"
        );
        assert!(
            html.contains("data-qty-input=\"qty_coffee\""),
            "own hidden input missing; got: {html}"
        );
        assert!(
            html.contains("name=\"qty_coffee\""),
            "input must carry field name; got: {html}"
        );
    }

    #[test]
    fn quantity_stepper_targets_44px() {
        let spec = spec_with_root(Element::new("QuantityStepper").prop("field", "qty_coffee"));
        let el = spec.elements.get("root").unwrap();
        let html = render_quantity_stepper(el, &spec, &json!({}), 1);
        // Both dec and inc buttons must carry HIT_TARGET_MIN
        let count = html.matches(crate::render::classes::HIT_TARGET_MIN).count();
        assert!(
            count >= 2,
            "both buttons must be >=44px (HIT_TARGET_MIN x2); count={count}; got: {html}"
        );
    }

    #[test]
    fn quantity_stepper_emits_bounds() {
        let spec = spec_with_root(
            Element::new("QuantityStepper")
                .prop("field", "qty_coffee")
                .prop("min", 1u32)
                .prop("max", 10u32)
                .prop("step", 2u32),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_quantity_stepper(el, &spec, &json!({}), 1);
        assert!(html.contains("data-qty-min=\"1\""), "min attr; got: {html}");
        assert!(
            html.contains("data-qty-max=\"10\""),
            "max attr; got: {html}"
        );
        assert!(
            html.contains("data-qty-step=\"2\""),
            "step attr; got: {html}"
        );
        // Absent when not set
        let spec_no_bounds =
            spec_with_root(Element::new("QuantityStepper").prop("field", "qty_coffee"));
        let el2 = spec_no_bounds.elements.get("root").unwrap();
        let html2 = render_quantity_stepper(el2, &spec_no_bounds, &json!({}), 1);
        assert!(
            !html2.contains("data-qty-min"),
            "no min when unset; got: {html2}"
        );
        assert!(
            !html2.contains("data-qty-max"),
            "no max when unset; got: {html2}"
        );
        assert!(
            !html2.contains("data-qty-step"),
            "no step when unset; got: {html2}"
        );
    }

    // ── 25. Numpad ──────��───────────────────────────────────────────────────

    #[test]
    fn numpad_emits_contract() {
        let spec = spec_with_root(Element::new("Numpad").prop("target_field", "total_cents"));
        let el = spec.elements.get("root").unwrap();
        let html = render_numpad(el, &spec, &json!({}), 1);
        assert!(html.contains("data-numpad"), "container attr; got: {html}");
        assert!(
            html.contains("data-numpad-target=\"total_cents\""),
            "target attr; got: {html}"
        );
        assert!(
            html.contains("data-numpad-display"),
            "display attr; got: {html}"
        );
        // All 12 key values must be present
        for k in &[
            "1",
            "2",
            "3",
            "4",
            "5",
            "6",
            "7",
            "8",
            "9",
            "0",
            "clear",
            "backspace",
        ] {
            let attr = format!("data-numpad-key=\"{k}\"");
            assert!(html.contains(&attr), "missing key {k}; got: {html}");
        }
        assert!(
            html.contains("data-numpad-input=\"total_cents\""),
            "hidden input attr; got: {html}"
        );
        // Must NOT render a native text or number input
        assert!(
            !html.contains("type=\"text\""),
            "no native text input allowed; got: {html}"
        );
        assert!(
            !html.contains("type=\"number\""),
            "no native number input allowed; got: {html}"
        );
    }

    #[test]
    fn numpad_price_mode() {
        // Price mode → data-numpad-mode="price"
        let spec_price = spec_with_root(
            Element::new("Numpad")
                .prop("target_field", "total_cents")
                .prop("mode", "price"),
        );
        let el = spec_price.elements.get("root").unwrap();
        let html = render_numpad(el, &spec_price, &json!({}), 1);
        assert!(
            html.contains("data-numpad-mode=\"price\""),
            "price mode attr missing; got: {html}"
        );
        // Quantity mode (default) → attribute absent
        let spec_qty = spec_with_root(Element::new("Numpad").prop("target_field", "total_cents"));
        let el2 = spec_qty.elements.get("root").unwrap();
        let html2 = render_numpad(el2, &spec_qty, &json!({}), 1);
        assert!(
            !html2.contains("data-numpad-mode"),
            "quantity mode must not emit mode attr; got: {html2}"
        );
    }

    #[test]
    fn numpad_keys_56px() {
        let spec = spec_with_root(Element::new("Numpad").prop("target_field", "total_cents"));
        let el = spec.elements.get("root").unwrap();
        let html = render_numpad(el, &spec, &json!({}), 1);
        let count = html
            .matches(crate::render::classes::HIT_TARGET_NUMPAD)
            .count();
        assert_eq!(
            count, 12,
            "all 12 keys must carry HIT_TARGET_NUMPAD (>=56px); count={count}; got: {html}"
        );
    }

    // ── Plan 05 migration tests (Task 1: Button / Badge / Alert) ────────────

    /// D-01: render_button_inner emits full literal "fjui-btn--primary" for Primary variant.
    #[test]
    fn button_primary_emits_fjui_class() {
        let spec = spec_with_root(Element::new("Button").prop("label", "Go").prop("variant", "primary"));
        let el = spec.elements.get("root").unwrap();
        let html = render_button(el, &spec, &json!({}), 1);
        assert!(html.contains("fjui-btn--primary"), "primary variant must emit fjui-btn--primary; got: {html}");
        assert!(html.contains("fjui-btn "), "base fjui-btn class must be present; got: {html}");
    }

    /// D-01: all 5 variants emit full literal fjui-btn-- classes.
    #[test]
    fn button_all_variants_emit_fjui_classes() {
        for (variant, expected) in [
            ("primary", "fjui-btn--primary"),
            ("secondary", "fjui-btn--secondary"),
            ("outline", "fjui-btn--outline"),
            ("ghost", "fjui-btn--ghost"),
            ("destructive", "fjui-btn--destructive"),
        ] {
            let spec = spec_with_root(Element::new("Button").prop("label", "X").prop("variant", variant));
            let el = spec.elements.get("root").unwrap();
            let html = render_button(el, &spec, &json!({}), 1);
            assert!(html.contains(expected), "variant {variant} must emit {expected}; got: {html}");
        }
    }

    /// D-01: all 3 sizes emit full literal fjui-btn-- size classes.
    #[test]
    fn button_all_sizes_emit_fjui_classes() {
        for (size, expected) in [
            ("sm", "fjui-btn--sm"),
            ("md", "fjui-btn--md"),
            ("lg", "fjui-btn--lg"),
        ] {
            let spec = spec_with_root(Element::new("Button").prop("label", "X").prop("size", size));
            let el = spec.elements.get("root").unwrap();
            let html = render_button(el, &spec, &json!({}), 1);
            assert!(html.contains(expected), "size {size} must emit {expected}; got: {html}");
        }
    }

    /// D-01: no format!-assembled class names — no appearance Tailwind utilities from old btn code.
    #[test]
    fn button_emits_no_old_appearance_utilities() {
        let spec = spec_with_root(Element::new("Button").prop("label", "Go").prop("variant", "primary"));
        let el = spec.elements.get("root").unwrap();
        let html = render_button(el, &spec, &json!({}), 1);
        assert!(!html.contains("bg-primary text-primary"), "old bg-primary appearance class must be gone; got: {html}");
        assert!(!html.contains("rounded-md font-medium"), "old rounded-md font-medium must be gone; got: {html}");
        assert!(!html.contains("px-3 py-"), "old padding utilities must be gone; got: {html}");
    }

    /// Badge emits fjui-badge base class + tone modifier.
    #[test]
    fn badge_emits_fjui_class_and_no_inline_style() {
        let html = badge_inline_html(Tone::Success, "OK");
        assert!(html.contains("fjui-badge"), "badge must emit fjui-badge; got: {html}");
        assert!(html.contains("fjui-badge--success"), "success tone must emit fjui-badge--success; got: {html}");
        assert!(!html.contains("justify-self: start"), "inline justify-self must be gone; got: {html}");
        assert!(!html.contains("style="), "no inline style attr on badge; got: {html}");
    }

    /// All 4 badge tones emit correct fjui modifier literals.
    #[test]
    fn badge_all_tones_emit_fjui_classes() {
        for (tone, expected) in [
            (Tone::Neutral, "fjui-badge--neutral"),
            (Tone::Success, "fjui-badge--success"),
            (Tone::Warning, "fjui-badge--warning"),
            (Tone::Destructive, "fjui-badge--destructive"),
        ] {
            let html = badge_inline_html(tone, "label");
            assert!(html.contains(expected), "tone must emit {expected}; got: {html}");
        }
    }

    /// Badge still html-escapes the label (T-246-11 XSS guard).
    #[test]
    fn badge_escapes_label_xss() {
        let html = badge_inline_html(Tone::Neutral, "<script>bad</script>");
        assert!(html.contains("&lt;script&gt;"), "badge must escape label; got: {html}");
        assert!(!html.contains("<script>"), "raw script must not appear in badge; got: {html}");
    }

    /// Alert emits fjui-alert base + tone modifier for all 4 tones.
    #[test]
    fn alert_all_tones_emit_fjui_classes() {
        for (tone, expected) in [
            ("neutral", "fjui-alert--neutral"),
            ("success", "fjui-alert--success"),
            ("warning", "fjui-alert--warning"),
            ("destructive", "fjui-alert--destructive"),
        ] {
            let spec = spec_with_root(
                Element::new("Alert")
                    .prop("tone", tone)
                    .prop("message", "msg"),
            );
            let el = spec.elements.get("root").unwrap();
            let html = render_alert(el, &spec, &json!({}), 1);
            assert!(html.contains("fjui-alert"), "alert must emit fjui-alert base; got: {html}");
            assert!(html.contains(expected), "tone {tone} must emit {expected}; got: {html}");
        }
    }

    /// Alert still html-escapes message and title (T-246-11 XSS guard).
    #[test]
    fn alert_escapes_message_and_title_xss() {
        let spec = spec_with_root(
            Element::new("Alert")
                .prop("tone", "neutral")
                .prop("title", "<script>xss</script>")
                .prop("message", "<b>bad</b>"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_alert(el, &spec, &json!({}), 1);
        assert!(html.contains("&lt;script&gt;"), "title must be escaped; got: {html}");
        assert!(html.contains("&lt;b&gt;"), "message must be escaped; got: {html}");
    }

    /// D-07 guard: no old appearance utilities remain in the migrated Button/Badge/Alert
    /// render function bodies. Checks HTML output rather than source to avoid false
    /// positives from other components (pagination, calendar) that share some token names.
    #[test]
    fn migrated_atoms_no_raw_appearance_utilities_in_output() {
        // Button: old variant utilities must not appear in button output
        for variant in ["primary", "secondary", "outline", "ghost", "destructive"] {
            let spec = spec_with_root(Element::new("Button").prop("label", "X").prop("variant", variant));
            let el = spec.elements.get("root").unwrap();
            let html = render_button(el, &spec, &json!({}), 1);
            assert!(!html.contains("bg-primary text-primary-foreground hover:"), "old btn-primary utility in output; got: {html}");
            assert!(!html.contains("bg-secondary text-secondary-foreground"), "old btn-secondary utility in output; got: {html}");
            assert!(!html.contains("rounded-md font-medium"), "old rounded-md font-medium in btn output; got: {html}");
            assert!(!html.contains("px-3 py-1.5"), "old sm padding in btn output; got: {html}");
            assert!(!html.contains("px-4 py-2 text-sm"), "old md padding in btn output; got: {html}");
            assert!(!html.contains("px-6 py-3"), "old lg padding in btn output; got: {html}");
        }
        // Badge: old base + tone utilities must not appear
        for tone in [Tone::Neutral, Tone::Success, Tone::Warning, Tone::Destructive] {
            let html = badge_inline_html(tone, "x");
            assert!(!html.contains("rounded-full px-2.5 py-0.5 text-xs font-medium"), "old badge base in output; got: {html}");
            assert!(!html.contains("bg-success/10"), "old success tint in badge output; got: {html}");
            assert!(!html.contains("bg-warning/10"), "old warning tint in badge output; got: {html}");
            assert!(!html.contains("bg-destructive/10"), "old destructive tint in badge output; got: {html}");
        }
        // Alert: old tone utilities must not appear
        for tone in ["neutral", "success", "warning", "destructive"] {
            let spec = spec_with_root(Element::new("Alert").prop("tone", tone).prop("message", "m"));
            let el = spec.elements.get("root").unwrap();
            let html = render_alert(el, &spec, &json!({}), 1);
            assert!(!html.contains("bg-surface border-border text-text\">"), "old neutral alert utility in output; got: {html}");
            assert!(!html.contains("bg-success/10"), "old success tint in alert output; got: {html}");
            assert!(!html.contains("bg-warning/10"), "old warning tint in alert output; got: {html}");
            assert!(!html.contains("bg-destructive/10"), "old destructive tint in alert output; got: {html}");
        }
    }

    // ── Plan 06 migration tests (Task 2: interactive atoms — RED) ──────────────

    /// render_sidebar_nav_item with active=true emits fjui-sidebar__nav-item--active.
    #[test]
    fn sidebar_nav_item_active_emits_fjui_active_class() {
        let spec = spec_with_root(
            Element::new("Sidebar").prop(
                "fixed_top",
                json!([{"label": "Dashboard", "href": "/dashboard", "active": true}]),
            ),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_sidebar(el, &spec, &json!({}), 1);
        assert!(
            html.contains("fjui-sidebar__nav-item--active"),
            "active nav item must emit fjui-sidebar__nav-item--active; got: {html}"
        );
        assert!(
            html.contains("fjui-sidebar__nav-item"),
            "must emit fjui-sidebar__nav-item base; got: {html}"
        );
    }

    /// render_sidebar_nav_item with active=false emits only fjui-sidebar__nav-item (no --active).
    #[test]
    fn sidebar_nav_item_inactive_emits_only_base_class() {
        let spec = spec_with_root(
            Element::new("Sidebar").prop(
                "fixed_top",
                json!([{"label": "Settings", "href": "/settings", "active": false}]),
            ),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_sidebar(el, &spec, &json!({}), 1);
        assert!(
            html.contains("fjui-sidebar__nav-item"),
            "inactive nav item must emit fjui-sidebar__nav-item; got: {html}"
        );
        assert!(
            !html.contains("fjui-sidebar__nav-item--active"),
            "inactive nav item must NOT emit --active; got: {html}"
        );
    }

    /// render_sidebar emits fjui-sidebar on the aside and fjui-sidebar__group-label on group headers.
    #[test]
    fn sidebar_emits_fjui_sidebar_and_group_label() {
        let spec = spec_with_root(
            Element::new("Sidebar").prop(
                "groups",
                json!([{"label": "Gestione", "collapsed": false, "items": [{"label": "Prodotti", "href": "/prodotti", "active": false}]}]),
            ),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_sidebar(el, &spec, &json!({}), 1);
        assert!(
            html.contains("fjui-sidebar"),
            "sidebar aside must carry fjui-sidebar; got: {html}"
        );
        assert!(
            html.contains("fjui-sidebar__group-label"),
            "group header must carry fjui-sidebar__group-label; got: {html}"
        );
    }

    /// render_header emits fjui-header on the header element.
    #[test]
    fn header_emits_fjui_header_class() {
        let spec = spec_with_root(Element::new("Header").prop("business_name", "ACME"));
        let el = spec.elements.get("root").unwrap();
        let html = render_header(el, &spec, &json!({}), 1);
        assert!(
            html.contains("fjui-header"),
            "header must emit fjui-header; got: {html}"
        );
        assert!(
            !html.contains("bg-background border-b border-border"),
            "old appearance utilities must be gone from header; got: {html}"
        );
    }

    /// render_menu_item with destructive=false emits fjui-menu-item.
    /// render_menu_item with destructive=true emits fjui-menu-item--destructive.
    #[test]
    fn menu_item_emits_fjui_classes() {
        use crate::action::{Action, HttpMethod};
        use crate::component::DropdownMenuAction;

        let normal = DropdownMenuAction {
            label: "Edit".into(),
            action: Action {
                handler: "edit".into(),
                url: Some("/edit".into()),
                method: HttpMethod::Get,
                confirm: None,
                on_success: None,
                on_error: None,
                target: None,
            },
            destructive: false,
            visible_if: None,
        };
        let html = render_menu_item(&normal, "fjui-menu-item", "fjui-menu-item fjui-menu-item--destructive", "");
        assert!(html.contains("fjui-menu-item"), "normal item must carry fjui-menu-item; got: {html}");
        assert!(!html.contains("fjui-menu-item--destructive"), "normal must NOT carry --destructive; got: {html}");

        let destructive = DropdownMenuAction {
            label: "Delete".into(),
            action: Action {
                handler: "delete".into(),
                url: Some("/delete".into()),
                method: HttpMethod::Post,
                confirm: None,
                on_success: None,
                on_error: None,
                target: None,
            },
            destructive: true,
            visible_if: None,
        };
        let html = render_menu_item(&destructive, "fjui-menu-item", "fjui-menu-item fjui-menu-item--destructive", "");
        assert!(html.contains("fjui-menu-item--destructive"), "destructive item must carry --destructive; got: {html}");
    }

    // ── Plan 06 migration tests (Task 1: non-interactive atoms — RED) ─────────

    /// render_text with each element emits a fjui-text--* class (no appearance utilities).
    #[test]
    fn text_emits_fjui_text_class() {
        for (element, expected) in [
            ("p", "fjui-text--body"),
            ("h1", "fjui-text--display"),
            ("h2", "fjui-text--section"),
            ("h3", "fjui-text--section"),
            ("span", "fjui-text--body"),
        ] {
            let spec = spec_with_root(
                Element::new("Text")
                    .prop("content", "Hello")
                    .prop("element", element),
            );
            let el = spec.elements.get("root").unwrap();
            let html = render_text(el, &spec, &json!({}), 1);
            assert!(
                html.contains(expected),
                "element={element} must emit {expected}; got: {html}"
            );
        }
    }

    /// render_text emits no appearance utility classes (no text-base, no text-3xl, etc).
    #[test]
    fn text_emits_no_appearance_utilities() {
        for element in ["p", "h1", "h2", "h3", "span"] {
            let spec = spec_with_root(
                Element::new("Text")
                    .prop("content", "Hello")
                    .prop("element", element),
            );
            let el = spec.elements.get("root").unwrap();
            let html = render_text(el, &spec, &json!({}), 1);
            assert!(!html.contains("text-base leading-relaxed"), "element={element}: old text-base leading-relaxed in output; got: {html}");
            assert!(!html.contains("text-3xl font-bold"), "element={element}: old text-3xl in output; got: {html}");
            assert!(!html.contains("text-2xl font-semibold"), "element={element}: old text-2xl in output; got: {html}");
        }
    }

    /// render_toast emits fjui-toast base + fjui-toast--{tone} modifier.
    #[test]
    fn toast_emits_fjui_toast_classes() {
        for (tone, expected) in [
            ("neutral", "fjui-toast--neutral"),
            ("success", "fjui-toast--success"),
            ("warning", "fjui-toast--warning"),
            ("destructive", "fjui-toast--destructive"),
        ] {
            let spec = spec_with_root(
                Element::new("Toast")
                    .prop("message", "msg")
                    .prop("tone", tone),
            );
            let el = spec.elements.get("root").unwrap();
            let html = render_toast(el, &spec, &json!({}), 1);
            assert!(html.contains("fjui-toast "), "toast must emit fjui-toast base; got: {html}");
            assert!(html.contains(expected), "tone={tone} must emit {expected}; got: {html}");
        }
    }

    /// render_toast emits no old TOAST_TONE_ appearance utilities (bg-primary/70, etc).
    #[test]
    fn toast_emits_no_old_tone_utilities() {
        for tone in ["neutral", "success", "warning", "destructive"] {
            let spec = spec_with_root(
                Element::new("Toast")
                    .prop("message", "msg")
                    .prop("tone", tone),
            );
            let el = spec.elements.get("root").unwrap();
            let html = render_toast(el, &spec, &json!({}), 1);
            assert!(!html.contains("bg-primary/70"), "old neutral tone class must be gone; got: {html}");
            assert!(!html.contains("bg-success/70"), "old success tone class must be gone; got: {html}");
            assert!(!html.contains("bg-warning/70"), "old warning tone class must be gone; got: {html}");
            assert!(!html.contains("bg-destructive/70"), "old destructive tone class must be gone; got: {html}");
        }
    }

    /// render_skeleton emits fjui-skeleton (while retaining the SHIMMER animation).
    #[test]
    fn skeleton_emits_fjui_skeleton_class() {
        let spec = spec_with_root(Element::new("Skeleton"));
        let el = spec.elements.get("root").unwrap();
        let html = render_skeleton(el, &spec, &json!({}), 1);
        assert!(html.contains("fjui-skeleton"), "skeleton must emit fjui-skeleton; got: {html}");
        // SHIMMER animation is retained (behavioral, not appearance)
        assert!(html.contains("ferro-shimmer"), "shimmer animation must be retained; got: {html}");
    }

    /// render_checklist emits fjui-checklist (no old shadow-sm / rounded-lg).
    #[test]
    fn checklist_emits_fjui_class() {
        let spec = spec_with_root(
            Element::new("Checklist")
                .prop("title", "Todo")
                .prop("items", json!([{"label": "Task", "checked": false}])),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_checklist(el, &spec, &json!({}), 1);
        assert!(html.contains("fjui-checklist"), "checklist must emit fjui-checklist; got: {html}");
        assert!(!html.contains("shadow-sm"), "old shadow-sm must be gone; got: {html}");
        assert!(!html.contains("rounded-lg shadow"), "old rounded-lg shadow combo must be gone; got: {html}");
    }

    // ── Plan 05 migration tests (Task 2: StatCard / EmptyState) ─────────────

    /// StatCard emits fjui-stat-card base class + tone modifier.
    #[test]
    fn stat_card_emits_fjui_class_and_value_element() {
        let spec = spec_with_root(
            Element::new("StatCard")
                .prop("label", "Ordini")
                .prop("value", "42"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_stat_card(el, &spec, &json!({}), 1);
        assert!(html.contains("fjui-stat-card"), "must emit fjui-stat-card; got: {html}");
        assert!(html.contains("fjui-stat-card--default"), "default tone must emit fjui-stat-card--default; got: {html}");
        assert!(html.contains("fjui-stat-card__value"), "value element must carry fjui-stat-card__value; got: {html}");
    }

    /// All 4 StatCard tones emit correct fjui modifier literals.
    #[test]
    fn stat_card_all_tones_emit_fjui_classes() {
        for (tone, expected) in [
            ("neutral", "fjui-stat-card--default"),
            ("success", "fjui-stat-card--success"),
            ("warning", "fjui-stat-card--warning"),
            ("destructive", "fjui-stat-card--destructive"),
        ] {
            let spec = spec_with_root(
                Element::new("StatCard")
                    .prop("label", "L")
                    .prop("value", "0")
                    .prop("tone", tone),
            );
            let el = spec.elements.get("root").unwrap();
            let html = render_stat_card(el, &spec, &json!({}), 1);
            assert!(html.contains(expected), "tone {tone} must emit {expected}; got: {html}");
        }
    }

    /// StatCard no longer emits old Tailwind appearance utilities (shadow-sm, text-2xl, etc).
    #[test]
    fn stat_card_emits_no_old_appearance_utilities() {
        let spec = spec_with_root(
            Element::new("StatCard")
                .prop("label", "L")
                .prop("value", "5"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_stat_card(el, &spec, &json!({}), 1);
        assert!(!html.contains("shadow-sm"), "old shadow-sm must be gone; got: {html}");
        assert!(!html.contains("rounded-lg"), "old rounded-lg must be gone; got: {html}");
        assert!(!html.contains("text-2xl font-bold"), "old value typography must be gone; got: {html}");
    }

    /// EmptyState emits fjui-empty-state class and uses fjui-btn for CTA.
    #[test]
    fn empty_state_emits_fjui_class_and_btn_cta() {
        let spec = spec_with_root(
            Element::new("EmptyState")
                .prop("title", "Nessun elemento")
                .prop("description", "Aggiungi qualcosa"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_empty_state(el, &spec, &json!({}), 1);
        assert!(html.contains("fjui-empty-state"), "must emit fjui-empty-state; got: {html}");
    }

    /// EmptyState CTA uses fjui-btn classes (not old Tailwind btn utilities).
    /// Action must be passed as a JSON prop (EmptyStateProps.action), matching
    /// how the existing empty_state_cta_carries_token_focus_ring test works.
    #[test]
    fn empty_state_cta_uses_fjui_btn() {
        let spec = spec_with_root(
            Element::new("EmptyState")
                .prop("title", "Nessun elemento")
                .prop(
                    "action",
                    json!({"handler": "items.create", "url": "/items/nuovo", "method": "GET"}),
                )
                .prop("action_label", "Aggiungi"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_empty_state(el, &spec, &json!({}), 1);
        assert!(html.contains("fjui-btn"), "CTA must use fjui-btn; got: {html}");
        assert!(html.contains("fjui-btn--primary"), "CTA must use fjui-btn--primary; got: {html}");
        assert!(html.contains("fjui-btn--md"), "CTA must use fjui-btn--md; got: {html}");
        assert!(html.contains("Aggiungi"), "CTA label must appear; got: {html}");
    }

    // ── 247-02: DescriptionList responsive columns (RSK-03) ──────────────

    /// DescriptionList columns=2 emits responsive md:grid-cols-2 (RSK-03).
    #[test]
    fn description_list_two_column_responsive() {
        let spec = spec_with_root(
            Element::new("DescriptionList")
                .prop("columns", 2)
                .prop("items", json!([{"label": "A", "value": "1"}, {"label": "B", "value": "2"}])),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_description_list(el, &spec, &json!({}), 1);
        assert!(
            html.contains("md:grid-cols-2"),
            "columns=2 must emit md:grid-cols-2 for responsiveness; got: {html}"
        );
        // Must NOT emit the old non-responsive bare class (without a breakpoint prefix).
        // We check for " grid-cols-2" (space prefix) to avoid matching the correct
        // "md:grid-cols-2" and "lg:grid-cols-3" substrings.
        assert!(
            !html.contains(" grid-cols-2"),
            "must not emit bare non-responsive grid-cols-2 (without breakpoint prefix); got: {html}"
        );
    }

    /// DescriptionList columns=1 emits grid-cols-1 without md: breakpoint.
    #[test]
    fn description_list_single_column_no_breakpoint() {
        let spec = spec_with_root(
            Element::new("DescriptionList")
                .prop("columns", 1)
                .prop("items", json!([{"label": "A", "value": "1"}])),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_description_list(el, &spec, &json!({}), 1);
        assert!(
            html.contains("grid-cols-1"),
            "columns=1 must emit grid-cols-1; got: {html}"
        );
        assert!(
            !html.contains("md:grid-cols"),
            "columns=1 must not emit md: breakpoint; got: {html}"
        );
    }

    /// DescriptionItem with inline_edit_field emits enhanced <dd> with data attrs + pencil button.
    #[test]
    fn description_item_with_inline_edit_emits_data_attrs_and_pencil() {
        let spec = spec_with_root(
            Element::new("DescriptionList")
                .prop("items", json!([{
                    "label": "Nome",
                    "value": "Alice",
                    "inline_edit_field": "name",
                    "inline_edit_endpoint": "/dashboard/clienti/42/field",
                    "inline_edit_kind": "text"
                }])),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_description_list(el, &spec, &json!({}), 1);
        assert!(
            html.contains("data-inline-edit-field=\"name\""),
            "must emit data-inline-edit-field; got: {html}"
        );
        assert!(
            html.contains("data-inline-edit-endpoint=\"/dashboard/clienti/42/field\""),
            "must emit data-inline-edit-endpoint; got: {html}"
        );
        assert!(
            html.contains("data-inline-edit-kind=\"text\""),
            "must emit data-inline-edit-kind; got: {html}"
        );
        assert!(
            html.contains("fjui-inline-edit__value"),
            "must emit value span with fjui-inline-edit__value class; got: {html}"
        );
        assert!(
            html.contains("fjui-inline-edit__pencil"),
            "must emit pencil button with fjui-inline-edit__pencil class; got: {html}"
        );
        assert!(
            html.contains("aria-label=\"Modifica Nome\""),
            "pencil button must have aria-label 'Modifica {{label}}'; got: {html}"
        );
    }

    /// DescriptionItem WITHOUT inline_edit_field emits plain <dd> (existing behavior unchanged).
    #[test]
    fn description_item_without_inline_edit_emits_plain_dd() {
        let spec = spec_with_root(
            Element::new("DescriptionList")
                .prop("items", json!([{
                    "label": "Nome",
                    "value": "Alice"
                }])),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_description_list(el, &spec, &json!({}), 1);
        assert!(
            !html.contains("data-inline-edit-field"),
            "no inline_edit_field → must not emit data-inline-edit-field; got: {html}"
        );
        assert!(
            !html.contains("fjui-inline-edit__pencil"),
            "no inline_edit_field → must not emit pencil button; got: {html}"
        );
        assert!(
            html.contains("fjui-description-list__detail"),
            "plain dd must still have fjui-description-list__detail class; got: {html}"
        );
        assert!(
            html.contains("Alice"),
            "plain dd must contain the value; got: {html}"
        );
    }
}
