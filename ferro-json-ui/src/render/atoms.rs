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
    ActionCardProps, ActionCardVariant, AlertProps, AlertVariant, AvatarProps, BadgeProps,
    BadgeVariant, BreadcrumbProps, ButtonProps, ButtonType, ButtonVariant, CalendarCellProps,
    ChecklistProps, DescriptionListProps, DropdownMenuAction, DropdownMenuProps, EmptyStateProps,
    HeaderProps, IconPosition, ImageProps, NotificationDropdownProps, Orientation, PaginationProps,
    ProductTileProps, ProgressProps, RawHtmlProps, SeparatorProps, SidebarNavItem, SidebarProps,
    Size, SkeletonProps, StatCardProps, TextElement, TextProps, ToastProps, ToastVariant,
};
use crate::spec::{Element, Spec};

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
    match props.element {
        TextElement::P => format!("<p class=\"text-base leading-relaxed text-text\">{content}</p>"),
        TextElement::H1 => format!("<h1 class=\"text-3xl font-bold leading-tight tracking-tight text-text\">{content}</h1>"),
        TextElement::H2 => format!("<h2 class=\"text-2xl font-semibold leading-tight tracking-tight text-text\">{content}</h2>"),
        TextElement::H3 => format!("<h3 class=\"text-xl font-semibold leading-snug text-text\">{content}</h3>"),
        TextElement::Span => format!("<span class=\"text-base text-text\">{content}</span>"),
        TextElement::Div => format!("<div class=\"text-base leading-relaxed text-text\">{content}</div>"),
        TextElement::Section => format!("<section class=\"text-base leading-relaxed text-text\">{content}</section>"),
    }
}

// ── 2. Button ────────────────────────────────────────────────────────────

/// Emits the inner `<button>` markup for a `Button` element. Reads
/// `ButtonProps` (variant, size, icon, label, disabled) and produces the
/// styled button without any wrapping anchor — the anchor wrap is applied by
/// [`render_button`] when the element carries a GET action.
fn render_button_inner(props: &ButtonProps) -> String {
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

    format!(
        "<button{type_attr}{form_attr} class=\"{base} {variant_classes} {size_classes}{disabled_classes}\"{disabled_attr}>{content}</button>"
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
    let base = "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium";
    let variant_classes = match props.variant {
        BadgeVariant::Default => "bg-primary/10 text-primary",
        BadgeVariant::Secondary => "bg-secondary/10 text-secondary-foreground",
        BadgeVariant::Destructive => "bg-destructive/10 text-destructive",
        BadgeVariant::Outline => "border border-border text-text",
    };
    // Inline `justify-self: start` keeps the pill at its natural width when
    // it lands inside a Grid cell — default `place-self: stretch` otherwise
    // stretches it to the column width. Inline because consumer Tailwind
    // builds don't always emit the `justify-self-start` utility.
    format!(
        "<span class=\"{} {}\" style=\"justify-self: start;\">{}</span>",
        base,
        variant_classes,
        html_escape(&props.label)
    )
}

// ── 4. Alert ─────────────────────────────────────────────────────────────

pub(crate) fn render_alert(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: AlertProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Alert", e),
    };
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

// ── 5. Separator ─────────────────────────────────────────────────────────

pub(crate) fn render_separator(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: SeparatorProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Separator", e),
    };
    let orientation = props.orientation.as_ref().cloned().unwrap_or_default();
    match orientation {
        Orientation::Horizontal => "<hr class=\"my-4 border-border\">".to_string(),
        Orientation::Vertical => "<div class=\"mx-4 h-full w-px bg-border\"></div>".to_string(),
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

    let mut html = String::from("<div class=\"w-full\" role=\"progressbar\">");
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

// ── 7. Avatar ────────────────────────────────────────────────────────────

pub(crate) fn render_avatar(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: AvatarProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Avatar", e),
    };
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
        let fallback_text = props.fallback.as_deref().unwrap_or(&props.alt);
        let initials: String = fallback_text.chars().take(2).collect();
        format!(
            "<span class=\"inline-flex items-center justify-center rounded-full bg-card text-text-muted {}\">{}</span>",
            size_classes,
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

    let placeholder = match &props.placeholder_label {
        Some(label) => format!(
            "<div class=\"absolute inset-0 flex items-center justify-center \
             rounded-md bg-surface text-xs text-text-muted\">{}</div>",
            html_escape(label)
        ),
        None => String::from("<div class=\"absolute inset-0 rounded-md bg-surface\"></div>"),
    };

    format!(
        "<div class=\"relative w-full\"{container_style}>\
            {placeholder}\
            <img src=\"{src}\" alt=\"{alt}\" \
                 class=\"relative w-full h-full rounded-md object-cover object-top\" \
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
    let rounded = if props.rounded == Some(true) {
        "rounded-full"
    } else {
        "rounded-md"
    };
    // Width and height are passed through `html_escape` before interpolation
    // into the `style` attribute to preserve the XSS discipline that every
    // interpolated string is escaped.
    format!(
        "{SHIMMER_CSS}<div class=\"ferro-shimmer {rounded}\" style=\"width: {}; height: {}\"></div>",
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

    let mut html = String::from("<nav class=\"flex items-center space-x-1\">");

    // Previous button.
    if current > 1 {
        let prev = current - 1;
        html.push_str(&format!(
            "<a href=\"{base_url_escaped}page={prev}\" class=\"px-3 py-1 rounded-md bg-background text-text hover:bg-surface transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\">&laquo;</a>"
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
                "<a href=\"{base_url_escaped}page={page}\" class=\"px-3 py-1 rounded-md bg-background text-text hover:bg-surface transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\">{page}</a>"
            ));
        }
        prev_page = page;
    }

    // Next button.
    if current < total_pages {
        let next = current + 1;
        html.push_str(&format!(
            "<a href=\"{base_url_escaped}page={next}\" class=\"px-3 py-1 rounded-md bg-background text-text hover:bg-surface transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\">&raquo;</a>"
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
    let mut html = format!("<dl class=\"grid grid-cols-{columns} gap-4\">");
    for item in &items {
        html.push_str(&format!(
            "<div><dt class=\"text-sm font-medium text-text-muted\">{}</dt><dd class=\"mt-1 text-sm text-text\">{}</dd></div>",
            html_escape(&item.label),
            html_escape(&item.value)
        ));
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
    // Centered placeholder card. Same outer shell as the DataTable empty
    // state so a tab without rows and a tab with no resource-shaped form
    // (e.g. owner notes) read as the same visual surface. Title and action
    // are optional; description is the primary message.
    let mut html = String::from(
        "<div class=\"rounded-lg border border-border bg-card min-h-40 py-8 px-6 flex items-center justify-center\">\
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
        html.push_str(&format!(
            "<a href=\"{}\" class=\"mt-4 inline-flex items-center justify-center rounded-md \
             border border-border bg-card text-text px-4 py-2 text-sm font-medium \
             hover:bg-surface transition-colors\">{}</a>",
            html_escape(url),
            html_escape(label)
        ));
    }
    html.push_str("</div></div>");
    html
}

// ── 14. StatCard ─────────────────────────────────────────────────────────

pub(crate) fn render_stat_card(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: StatCardProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("StatCard", e),
    };
    let mut html =
        String::from("<div class=\"bg-card rounded-lg shadow-sm p-4 border border-border\">");
    if let Some(ref icon) = props.icon {
        // Icon string is emitted raw — trusted SVG supplied by server-side
        // authoring; not user input.
        html.push_str(&format!(
            "<span class=\"inline-block mb-2 w-6 h-6\">{icon}</span>"
        ));
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

// ── 15. Checklist ────────────────────────────────────────────────────────

pub(crate) fn render_checklist(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: ChecklistProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Checklist", e),
    };
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

// ── 16. Toast ────────────────────────────────────────────────────────────

pub(crate) fn render_toast(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: ToastProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Toast", e),
    };
    // Solid, opaque variant backgrounds with contrast text. Matches the JS
    // showToast() palette so server-rendered and JS-created toasts look
    // identical.
    // Translucent variant tint over a backdrop blur — content underneath
    // shows through but the toast stays clearly readable. 70% alpha gives
    // visible translucency even on plain backgrounds.
    let variant_classes = match props.variant {
        ToastVariant::Info => "bg-primary/70 text-primary-foreground",
        ToastVariant::Success => "bg-success/70 text-primary-foreground",
        ToastVariant::Warning => "bg-warning/70 text-primary-foreground",
        ToastVariant::Error => "bg-destructive/70 text-primary-foreground",
    };
    let variant_str = match props.variant {
        ToastVariant::Info => "info",
        ToastVariant::Success => "success",
        ToastVariant::Warning => "warning",
        ToastVariant::Error => "error",
    };
    let timeout = props.timeout.unwrap_or(5);
    // No close button — toast auto-dismisses via the runtime's
    // setupServerToasts() handler reading data-toast-timeout.
    format!(
        "<div class=\"fixed top-4 right-4 z-50 rounded-lg px-4 py-3 shadow-lg max-w-sm transition-opacity duration-300 backdrop-blur-md {variant_classes}\" \
         data-toast-variant=\"{variant_str}\" data-toast-timeout=\"{timeout}\">\
         <p class=\"text-sm\">{}</p>\
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
    let unread_count = props.notifications.iter().filter(|n| !n.read).count();
    let mut html = String::from("<div class=\"relative\" data-notification-dropdown>");
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

// ── 18. Sidebar ──────────────────────────────────────────────────────────

pub(crate) fn render_sidebar(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    let props: SidebarProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("Sidebar", e),
    };
    let mut html =
        String::from("<aside class=\"flex flex-col h-full bg-background border-r border-border\">");
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
    let (tag, classes) = if disabled {
        (
            "span",
            "flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium text-text-muted opacity-50 cursor-not-allowed select-none",
        )
    } else if item.active {
        (
            "a",
            "flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium bg-card text-primary transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2",
        )
    } else {
        (
            "a",
            "flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium text-text-muted hover:text-text hover:bg-surface transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2",
        )
    };
    let mut html = if disabled {
        format!("<{tag} aria-disabled=\"true\" class=\"{classes}\">",)
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
    let mut html = String::from(
        "<header class=\"relative flex items-center justify-between px-6 py-4 bg-background border-b border-border\">",
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
            "<a href=\"{}\" class=\"text-sm text-text-muted hover:text-text\">Logout</a>",
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

pub(crate) fn render_dropdown_menu(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    let props: DropdownMenuProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("DropdownMenu", e),
    };
    let mut html = String::new();

    let trigger_icon = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><circle cx=\"12\" cy=\"5\" r=\"1\"/><circle cx=\"12\" cy=\"12\" r=\"1\"/><circle cx=\"12\" cy=\"19\" r=\"1\"/></svg>";
    html.push_str(&format!(
        "<button type=\"button\" popovertarget=\"{}\" aria-label=\"{}\" \
         class=\"inline-flex items-center justify-center rounded-md p-1.5 \
         text-text-muted hover:text-text hover:bg-surface transition-colors duration-150 \
         focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary \
         focus-visible:ring-offset-2\">{}</button>",
        html_escape(&props.menu_id),
        html_escape(&props.trigger_label),
        trigger_icon,
    ));

    html.push_str(&format!(
        "<div popover id=\"{}\" data-popover-menu \
         class=\"w-48 rounded-md border border-border bg-card shadow-md text-left p-0\">",
        html_escape(&props.menu_id),
    ));

    for item in &props.items {
        html.push_str(&render_menu_item(
            item,
            "block px-4 py-2 text-sm text-text hover:bg-surface transition-colors duration-150",
            "block px-4 py-2 text-sm text-destructive hover:bg-destructive/10 transition-colors duration-150",
            "",
        ));
    }

    html.push_str("</div>"); // close popover panel
    html
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
    let opacity = if props.is_current_month {
        ""
    } else {
        " opacity-40"
    };
    let hover = if props.is_current_month {
        " hover:bg-surface/60 transition-colors cursor-pointer"
    } else {
        " cursor-pointer"
    };

    let mut html = format!(
        "<div class=\"flex flex-col min-h-[5rem] p-2 border border-border -mt-px -ml-px{opacity}{hover}\">",
    );

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

    let total = if !props.dot_colors.is_empty() {
        props.dot_colors.len() as u32
    } else {
        props.event_count
    };
    if total > 0 && total <= 3 {
        html.push_str("<div class=\"flex gap-1 mt-auto pt-1\">");
        if props.dot_colors.is_empty() {
            for _ in 0..total {
                html.push_str("<span class=\"w-1.5 h-1.5 rounded-full bg-primary\"></span>");
            }
        } else {
            for color in props.dot_colors.iter().take(3) {
                html.push_str(&format!(
                    "<span class=\"w-1.5 h-1.5 rounded-full {}\"></span>",
                    html_escape(color)
                ));
            }
        }
        html.push_str("</div>");
    } else if total > 3 {
        html.push_str(&format!(
            "<span class=\"mt-auto pt-1 text-xs font-medium text-primary\">{total} prenot.</span>"
        ));
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
            return format!("{diagnostic}<a href=\"{href}\" class=\"block\">{html}</a>");
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
    let border_class = match props.variant {
        ActionCardVariant::Default => "border-l-primary",
        ActionCardVariant::Setup => "border-l-warning",
        ActionCardVariant::Danger => "border-l-destructive",
    };

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
                "<div class=\"rounded-lg border-l-4 {border_class} border border-border bg-card shadow-sm p-4 flex items-center gap-4 cursor-pointer hover:bg-surface transition-colors duration-150\">"
            ),
            "</div>".to_string(),
        )
    };

    let mut html = open_tag;

    if let Some(ref icon) = props.icon {
        html.push_str(&format!(
            "<div class=\"w-10 h-10 flex-shrink-0 rounded-md bg-surface flex items-center justify-center text-text-muted\">{icon}</div>",
        ));
    }

    html.push_str(&format!(
        "<div class=\"flex-1 min-w-0\"><p class=\"text-sm font-semibold text-text\">{}</p><p class=\"text-sm text-text-muted mt-0.5\">{}</p></div>",
        html_escape(&props.title),
        html_escape(&props.description)
    ));

    html.push_str("<span class=\"text-text-muted flex-shrink-0 text-lg\">&rsaquo;</span>");

    html.push_str(&close_tag);
    html
}

// ── 23. ProductTile ──────────────────────────────────────────────────────

pub(crate) fn render_product_tile(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    let props: ProductTileProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("ProductTile", e),
    };
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
        // Active page marker: a <span> (not <a>) containing the page number.
        assert!(
            html.contains("bg-primary text-primary-foreground\">5<"),
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
                .prop("variant", "success"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_toast(el, &spec, &json!({}), 1);
        assert!(html.contains("Saved"));
        assert!(html.contains("data-toast-variant=\"success\""));
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

    // ── 20. DropdownMenu ───────────────────────────────────────────────

    #[test]
    fn dropdown_menu_emits_actions() {
        let spec = spec_with_root(
            Element::new("DropdownMenu")
                .prop("menu_id", "m1")
                .prop("trigger_label", "Actions")
                .prop(
                    "items",
                    json!([
                        {
                            "label": "Edit",
                            "action": {"handler": "edit", "method": "POST", "url": "/edit"}
                        }
                    ]),
                ),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_dropdown_menu(el, &spec, &json!({}), 1);
        assert!(html.contains("Edit"), "got: {html}");
        assert!(html.contains("popovertarget=\"m1\""), "got: {html}");
        assert!(
            html.contains("popover id=\"m1\" data-popover-menu"),
            "got: {html}"
        );
    }

    #[test]
    fn dropdown_menu_get_action_renders_anchor() {
        let spec = spec_with_root(
            Element::new("DropdownMenu")
                .prop("menu_id", "m1")
                .prop("trigger_label", "Actions")
                .prop(
                    "items",
                    json!([
                        {
                            "label": "View",
                            "action": {"handler": "view", "method": "GET", "url": "/view"}
                        }
                    ]),
                ),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_dropdown_menu(el, &spec, &json!({}), 1);
        assert!(html.contains("<a href=\"/view\""), "got: {html}");
    }

    // ── 21. CalendarCell ───────────────────────────────────────────────

    #[test]
    fn calendar_cell_emits_day() {
        let spec = spec_with_root(Element::new("CalendarCell").prop("day", 15));
        let el = spec.elements.get("root").unwrap();
        let html = render_calendar_cell(el, &spec, &json!({}), 1);
        assert!(html.contains(">15<"), "got: {html}");
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

    // ── 23. ProductTile ────────────────────────────────────────────────

    #[test]
    fn product_tile_emits_name_and_price() {
        let spec = spec_with_root(
            Element::new("ProductTile")
                .prop("product_id", "p1")
                .prop("name", "Widget")
                .prop("price", "9.99")
                .prop("field", "qty_p1"),
        );
        let el = spec.elements.get("root").unwrap();
        let html = render_product_tile(el, &spec, &json!({}), 1);
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
        assert_eq!(crate::render::BUILTIN_TYPES.len(), 44);
    }
}
