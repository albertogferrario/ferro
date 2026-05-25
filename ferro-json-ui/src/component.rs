//! Component catalog for JSON-UI.
//!
//! Defines the available UI components with typed props. Each component
//! uses serde's tagged enum representation so JSON includes `"type": "Card"`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::action::Action;

/// Shared size enum for components (Button, Badge, Avatar, Input).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Size {
    Xs,
    Sm,
    #[default]
    Default,
    Lg,
}

/// Icon placement relative to button label.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IconPosition {
    #[default]
    Left,
    Right,
}

/// Sort direction for table columns.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

/// Separator orientation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

/// Button visual variants (aligned to shadcn/ui).
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, strum::AsRefStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ButtonVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    Link,
}

/// Input field types.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InputType {
    #[default]
    Text,
    Email,
    Password,
    Number,
    Textarea,
    Hidden,
    Date,
    Time,
    Url,
    Tel,
    Search,
    File,
}

/// Alert visual variants.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, strum::AsRefStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AlertVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// Badge visual variants (aligned to shadcn/ui).
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, strum::AsRefStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
}

/// Text element types for semantic HTML rendering.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextElement {
    #[default]
    P,
    H1,
    H2,
    H3,
    Span,
    Div,
    Section,
}

/// Column display format for tables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ColumnFormat {
    Date,
    DateTime,
    Currency,
    Boolean,
}

/// Table column definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Column {
    pub key: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<ColumnFormat>,
}

/// Select option (value + label pair).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

/// Visual variant for Card chrome.
///
/// - `Bordered` (default): `border + bg-card + shadow-sm` with `p-4`.
///   Dashboard cards in dense layouts.
/// - `Elevated`: `bg-card + shadow-md` (no border) with `p-8`.
///   Auth pages, error pages, standalone marketing cards.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CardVariant {
    #[default]
    Bordered,
    Elevated,
}

/// Props for Card component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CardProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional muted secondary line rendered immediately below the title and
    /// above the description. Pattern: name → role, customer → staff,
    /// title → category. Visually `text-sm text-text-muted`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// Optional small badge text rendered alongside the title. Visually a
    /// Badge-styled pill inside the Card chrome — for status indicators,
    /// counters, countdown labels, etc. Independent of the title hierarchy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<FormMaxWidth>,
    /// IDs of footer elements (resolved against `Spec.elements`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footer: Vec<String>,
    #[serde(default)]
    pub variant: CardVariant,
}

/// Props for Table component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TableProps {
    pub columns: Vec<Column>,
    pub data_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_actions: Option<Vec<Action>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sortable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_column: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_direction: Option<SortDirection>,
}

/// Maximum width constraint for form containers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum FormMaxWidth {
    #[default]
    Default,
    Narrow,
    Wide,
}

/// Props for Form component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FormProps {
    pub action: Action,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<crate::action::HttpMethod>,
    /// Form guard type. When set, the runtime JS disables the submit button
    /// until the guard condition is met. Value: `"number-gt-0"` — at least
    /// one number input must have value > 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    /// Optional max-width constraint for the form container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<FormMaxWidth>,
    /// Optional HTML `id` attribute for the rendered `<form>`. Pair with a
    /// Button's `form` prop to submit this form from a button placed outside
    /// it (e.g. in a PageHeader actions slot).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// HTML form `enctype` attribute. Set to `"multipart/form-data"` for forms
    /// carrying a file input. Without this, the browser default encoding
    /// (`application/x-www-form-urlencoded`) is used and file inputs are sent
    /// as plain text rather than a multipart body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enctype: Option<String>,
}

/// HTML button type attribute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum ButtonType {
    #[default]
    Button,
    Submit,
}

/// Props for Button component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ButtonProps {
    pub label: String,
    #[serde(default)]
    pub variant: ButtonVariant,
    #[serde(default)]
    pub size: Size,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_position: Option<IconPosition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub button_type: Option<ButtonType>,
    /// HTML5 `form` attribute. Lets a submit button rendered outside its
    /// target `<form>` (e.g. in a PageHeader actions slot) still submit
    /// that form, by matching the form's `id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub form: Option<String>,
}

/// Props for Input component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct InputProps {
    /// Form field name for data binding.
    pub field: String,
    pub label: String,
    #[serde(default)]
    pub input_type: InputType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    /// Data path for pre-filling from handler data (e.g., "/data/user/name").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
    /// HTML step attribute for number inputs (e.g., "any", "0.01").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    /// HTML datalist id for autocomplete suggestions.
    /// When set, renders `list="..."` on the input and a companion `<datalist>`
    /// whose options come from a view data key matching this id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list: Option<String>,
    /// HTML `accept` attribute for `input_type = "file"`. Comma-separated MIME
    /// types or extensions (e.g. `"image/jpeg,image/png,image/webp"`). Browser-
    /// side filter hint only — server-side MIME validation is the consumer's
    /// responsibility (the spec layer does not enforce file content type).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accept: Option<String>,
}

/// Props for RichTextEditor leaf element — rendered by the Quill 2.0.3 plugin.
///
/// The plugin emits a container div (`<div data-ferro-quill ...>`) and a hidden
/// input that receives the editor's HTML on every text-change event. The form
/// handler receives standard `field=<html>` POST data on submit.
///
/// # Security
/// The editor produces user-controlled HTML. Sanitization on submit is the
/// consumer's responsibility — handle this in the form handler before
/// persisting (e.g. via `ammonia`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RichTextEditorProps {
    pub field: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Props for Select component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SelectProps {
    /// Form field name for data binding.
    pub field: String,
    pub label: String,
    pub options: Vec<SelectOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    /// Data path for pre-filling from handler data (e.g., "/data/user/name").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
}

/// Props for Alert component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AlertProps {
    pub message: String,
    #[serde(default)]
    pub variant: AlertVariant,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Props for Badge component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BadgeProps {
    pub label: String,
    #[serde(default)]
    pub variant: BadgeVariant,
}

/// Props for Modal component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ModalProps {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_label: Option<String>,
    /// IDs of footer elements (resolved against `Spec.elements`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footer: Vec<String>,
}

/// Props for Text component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TextProps {
    pub content: String,
    #[serde(default)]
    pub element: TextElement,
}

/// Props for Checkbox component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CheckboxProps {
    /// Form field name for data binding.
    pub field: String,
    /// HTML value attribute. When set, the checkbox submits this value instead of "1".
    /// Required for multi-value checkbox groups (same name, different values).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    /// Data path for pre-filling from handler data (e.g., "/data/user/name").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Props for CheckboxList component — multi-select checkbox group.
///
/// Each checked option submits as `field=value`. Options may be supplied
/// statically via `options` or resolved at render time from `options_path`.
/// Pre-selected values are read from `selected_path` (a `Vec<String>`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CheckboxListProps {
    /// Shared form field name; each checkbox submits as `field=value`.
    pub field: String,
    /// Static options list. When empty and `options_path` is set, options are
    /// resolved from the data at render time.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<SelectOption>,
    /// Data path to an array of `{value, label}` objects for data-driven options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options_path: Option<String>,
    /// Data path to a `Vec<String>` of pre-selected values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Props for Switch component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SwitchProps {
    /// Form field name for data binding.
    pub field: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    /// Data path for pre-filling from handler data (e.g., "/data/user/name").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Auto-submit action. When set, the switch renders inside a minimal
    /// form and submits on change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    /// When true, applies `scale-75 origin-left` CSS to the switch container
    /// for compact inline display (e.g. per-row settings toggles).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compact: Option<bool>,
}

/// Props for Separator component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SeparatorProps {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<Orientation>,
}

/// A single item in a description list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DescriptionItem {
    pub label: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<ColumnFormat>,
}

/// Props for DescriptionList component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DescriptionListProps {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<DescriptionItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<u8>,
    /// Optional data-path override of `items`. When set, the renderer
    /// resolves the array at this path and decodes each entry as a
    /// `DescriptionItem`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
}

/// A single tab within a Tabs component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Tab {
    pub value: String,
    pub label: String,
    /// IDs of elements rendered inside this tab's panel.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
}

/// Props for Tabs component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TabsProps {
    pub default_tab: String,
    pub tabs: Vec<Tab>,
}

/// A single item in a breadcrumb trail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BreadcrumbItem {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Props for Breadcrumb component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BreadcrumbProps {
    pub items: Vec<BreadcrumbItem>,
}

/// Props for Pagination component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PaginationProps {
    pub current_page: u32,
    pub per_page: u32,
    pub total: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

/// Props for Progress component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProgressProps {
    /// Percentage value (0-100).
    pub value: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Props for Image component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImageProps {
    #[serde(default)]
    pub src: String,
    pub alt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    /// Optional label shown in a skeleton placeholder that sits behind the
    /// image. When the image fails to load (or is still being generated),
    /// the `<img>` is hidden via `onerror` and the placeholder remains
    /// visible, keeping the container at its aspect-ratio size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder_label: Option<String>,
    /// Server-rendered inline SVG string. When set, the SVG is emitted verbatim
    /// inside a `<div aria-label="{alt}">` wrapper; no `<img>` tag is produced.
    ///
    /// # Safety
    /// Content is NOT sanitized. The SVG string is emitted into the response
    /// verbatim. Pass only server-constructed SVG (e.g. bar charts, QR codes).
    /// Do NOT pass untrusted input. `alt` is required and is HTML-escaped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_svg: Option<String>,
    /// Optional data-path override of `src`. When set, the renderer resolves
    /// the value at this path against handler data and uses it as the
    /// `<img src>`. Falls back to `src` when missing or non-string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
}

impl ImageProps {
    /// Convenience constructor for inline-SVG images. `src` is set to the
    /// empty string; the renderer takes the SVG path when `inline_svg` is `Some`.
    ///
    /// # Safety
    /// `svg` is emitted verbatim. See [`ImageProps::inline_svg`] for the trust model.
    pub fn inline_svg(svg: impl Into<String>, alt: impl Into<String>) -> Self {
        Self {
            src: String::new(),
            alt: alt.into(),
            aspect_ratio: None,
            placeholder_label: None,
            inline_svg: Some(svg.into()),
            data_path: None,
        }
    }
}

/// Props for Avatar component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AvatarProps {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    pub alt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<Size>,
}

/// Props for Skeleton loading placeholder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SkeletonProps {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rounded: Option<bool>,
}

/// Props for the `RawHtml` component — server-injected HTML island.
///
/// # Safety
/// `html` is emitted into the response VERBATIM with NO sanitization. The
/// component exists to bridge server-rendered HTML fragments (e.g. a status
/// pill, a link badge) into a v2 spec where a first-class component would
/// be over-engineering.
///
/// Sanitization is the CONSUMER's responsibility — pass only server-
/// constructed HTML, or run untrusted input through a sanitiser (e.g.
/// `ammonia`) in the handler before embedding. This mirrors
/// `RichTextEditorProps` discipline (see component.rs).
///
/// For richer widgets (interactive forms, charts, OAuth flows), use the
/// first-class plugin system (`JsonUiPlugin`) instead — see
/// `docs/src/json-ui/plugins.md`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RawHtmlProps {
    /// Server-constructed HTML emitted verbatim. NOT sanitized.
    #[serde(default)]
    pub html: String,
}

/// Toast visual variants.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, strum::AsRefStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ToastVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// A single item in a checklist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ChecklistItem {
    pub label: String,
    #[serde(default)]
    pub checked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

/// A single item in a notification dropdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NotificationItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub read: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_url: Option<String>,
}

/// A single navigation item in the sidebar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SidebarNavItem {
    pub label: String,
    pub href: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default)]
    pub active: bool,
    /// When true, the item renders as a muted, non-clickable `<span>`
    /// instead of an `<a>` — useful for "coming soon" placeholders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// A collapsible group in the sidebar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SidebarGroup {
    pub label: String,
    #[serde(default)]
    pub collapsed: bool,
    pub items: Vec<SidebarNavItem>,
}

/// Props for StatCard component (live-updatable metric card).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StatCardProps {
    pub label: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// SSE target key for live updates; maps to `data-sse-target` on the value element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sse_target: Option<String>,
}

/// Props for Checklist component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ChecklistProps {
    pub title: String,
    pub items: Vec<ChecklistItem>,
    #[serde(default = "default_true")]
    pub dismissible: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dismiss_label: Option<String>,
    /// Server-side state persistence key for this checklist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_key: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Props for Toast component (declarative notification intent).
///
/// The JS runtime reads data attributes from the rendered element to
/// display the toast. Timeouts and dismissal are handled client-side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToastProps {
    pub message: String,
    #[serde(default)]
    pub variant: ToastVariant,
    /// Seconds before auto-dismiss. Default 5.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    #[serde(default = "default_true")]
    pub dismissible: bool,
}

/// Props for NotificationDropdown component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NotificationDropdownProps {
    pub notifications: Vec<NotificationItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty_text: Option<String>,
}

/// Props for Sidebar component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SidebarProps {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixed_top: Vec<SidebarNavItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<SidebarGroup>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixed_bottom: Vec<SidebarNavItem>,
}

/// Props for Header component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HeaderProps {
    pub business_name: String,
    /// Unread notification count for badge display.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notification_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_avatar: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logout_url: Option<String>,
}

/// Gap size for Grid layout.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GapSize {
    None,
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
}

/// Props for Grid component — multi-column layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GridProps {
    /// Number of columns (1-12) at base (mobile) viewport.
    #[serde(default = "default_grid_columns")]
    pub columns: u8,
    /// Number of columns at md breakpoint (768px+). When set, creates a responsive grid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub md_columns: Option<u8>,
    /// Number of columns at lg breakpoint (1024px+). Optional; falls back to md.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lg_columns: Option<u8>,
    /// Gap between grid items.
    #[serde(default)]
    pub gap: GapSize,
    /// Enables horizontal scroll mode. Children get `min-w-[280px]` and the grid
    /// uses `grid-flow-col` auto-cols layout for Trello-like horizontal scrolling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scrollable: Option<bool>,
}

fn default_grid_columns() -> u8 {
    2
}

/// Props for Collapsible section — expandable `<details>`/`<summary>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CollapsibleProps {
    pub title: String,
    #[serde(default)]
    pub expanded: bool,
}

/// Props for EmptyState component — standardized empty view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EmptyStateProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_label: Option<String>,
}

/// Layout variant for form sections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum FormSectionLayout {
    #[default]
    Stacked,
    TwoColumn,
}

/// Props for FormSection component — visual grouping within forms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FormSectionProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional layout variant. Defaults to stacked (single column).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<FormSectionLayout>,
}

/// Props for PageHeader component -- page title with optional breadcrumb and action buttons.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PageHeaderProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breadcrumb: Vec<BreadcrumbItem>,
    /// IDs of action button elements rendered to the right of the title.
    #[serde(
        default,
        deserialize_with = "deserialize_actions_lax",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub actions: Vec<String>,
}

/// Props for ButtonGroup component -- horizontal button row with consistent gap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ButtonGroupProps {
    /// Gap between buttons. Defaults to small spacing.
    #[serde(default)]
    pub gap: GapSize,
}

/// Props for DetailPage component -- opinionated resource-detail skeleton.
///
/// Renders a PageHeader (title + breadcrumb + actions), an info Card
/// wrapping the `info` slot IDs (typically a Badge plus a DescriptionList),
/// and `Element.children` as stacked sections below the card (tabs,
/// related-resource lists, action panels). Centralizes the visual contract
/// every dashboard detail page follows so per-page rebuilds cannot drift
/// from the canonical shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DetailPageProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breadcrumb: Vec<BreadcrumbItem>,
    /// IDs of action button elements rendered to the right of the title.
    #[serde(
        default,
        deserialize_with = "deserialize_actions_lax",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub actions: Vec<String>,
    /// IDs of elements rendered inside the info Card
    /// (typically a Badge and a DescriptionList). Omit to skip the card.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub info: Vec<String>,
}

/// A single action item in a dropdown menu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DropdownMenuAction {
    pub label: String,
    pub action: Action,
    #[serde(default)]
    pub destructive: bool,
}

/// Props for DropdownMenu component — trigger button with absolutely-positioned action panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DropdownMenuProps {
    pub menu_id: String,
    pub trigger_label: String,
    pub items: Vec<DropdownMenuAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_variant: Option<ButtonVariant>,
}

/// Props for the DataTable component — Stripe-style alternating rows with DropdownMenu per row,
/// mobile card fallback, and empty state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DataTableProps {
    pub columns: Vec<Column>,
    pub data_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_actions: Option<Vec<DropdownMenuAction>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_key: Option<String>,
    /// URL pattern for row click navigation. Use `{row_key}` as placeholder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_href: Option<String>,
}

/// Props for MediaCardGrid — a responsive card grid backed by a data array.
/// Mirrors DataTable's row_key/row_actions/data_path contract but renders
/// cards with an optional screenshot image instead of table rows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MediaCardGridProps {
    pub data_path: String,
    /// Key in each row object whose value becomes the card title.
    pub title_key: String,
    /// Key for the subtitle/URL line below the title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_key: Option<String>,
    /// Key for the screenshot image URL. No image rendered when absent or empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_key: Option<String>,
    /// Key for the URL the image links to (opens in new tab).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_href_key: Option<String>,
    /// CSS aspect-ratio value for the image (default "4/5").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_aspect_ratio: Option<String>,
    /// Key for the footer badge label text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge_key: Option<String>,
    /// Key for the badge variant string: "outline" | "destructive" | "default".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge_variant_key: Option<String>,
    /// Key used for {row_key} substitution in row_action URLs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_actions: Option<Vec<DropdownMenuAction>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty_message: Option<String>,
    /// Number of columns in the grid (default 3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<u8>,
}

/// Props for a single column in a KanbanBoard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct KanbanColumnProps {
    pub id: String,
    pub title: String,
    pub count: u32,
    /// IDs of elements rendered inside this column.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
}

/// Props for KanbanBoard component — horizontal scrollable columns on desktop, tab-based on mobile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct KanbanBoardProps {
    /// Static columns. Used when `data_path` is None.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<KanbanColumnProps>,
    /// Optional data-path override of `columns`. When set, the renderer
    /// resolves the array at this path against handler data and decodes
    /// each entry as a `KanbanColumnProps`. Takes precedence over
    /// `columns` when both are set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mobile_default_column: Option<String>,
    /// Placeholder text shown inside columns whose `children` list is empty.
    /// When `None`, columns with no children render no placeholder (back-compat).
    /// Provide a short, neutral message — e.g. "Nessun ordine", "Nothing here".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty_label: Option<String>,
}

/// Props for a calendar day cell.
///
/// Renders a single day in a month grid with today highlight,
/// out-of-month muting, and event count indicator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CalendarCellProps {
    pub day: u8,
    #[serde(default)]
    pub is_today: bool,
    #[serde(default)]
    pub is_current_month: bool,
    #[serde(default)]
    pub event_count: u32,
    /// Optional per-event Tailwind color classes (e.g. "bg-blue-500").
    /// When non-empty, colored dots are rendered instead of plain primary dots.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dot_colors: Vec<String>,
}

/// Visual variant for action cards.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionCardVariant {
    #[default]
    Default,
    Setup,
    Danger,
}

/// Props for a horizontal action card with variant-colored left border.
///
/// Renders icon + title + description + chevron in a clickable row.
/// When `href` is set, the card wraps in an `<a>` element with `aria-label`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ActionCardProps {
    pub title: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default)]
    pub variant: ActionCardVariant,
    /// Optional navigation URL. When set, the card renders as an `<a>` element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

/// Props for a touch-friendly product tile with quantity controls.
///
/// Renders product name, price, and +/- buttons that drive a hidden input
/// via JS. Used for POS-style product selection during order creation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ProductTileProps {
    pub product_id: String,
    pub name: String,
    pub price: String,
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_quantity: Option<u32>,
}

/// Lax deserializer for PageHeader.actions. Per D-19/F6:
/// Accepts: missing field (via #[serde(default)]), null, [], empty string "",
/// and array of strings. Rejects: non-empty strings, arrays of non-strings.
/// This loosens the wire-format contract for actions only — other Vec<String>
/// ID-slot fields (e.g. CardProps.footer) remain strict.
fn deserialize_actions_lax<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Vec<String>, D::Error> {
    use serde::de::Error;
    let v = serde_json::Value::deserialize(d)?;
    match v {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::String(s) if s.is_empty() => Ok(Vec::new()),
        serde_json::Value::Array(arr) => arr
            .into_iter()
            .map(|item| {
                item.as_str()
                    .map(String::from)
                    .ok_or_else(|| D::Error::custom("PageHeader.actions: expected string in array"))
            })
            .collect(),
        other => Err(D::Error::custom(format!(
            "PageHeader.actions: expected null, empty string, or array of strings; got {other:?}"
        ))),
    }
}

#[cfg(test)]
mod schema_smoke_tests {
    //! Runtime `schema_for!` smoke tests per D-32.
    //!
    //! Each test asserts that the generated JSON Schema for the given Props
    //! struct is a non-empty JSON object with a populated `properties` field.
    //! This proves the `JsonSchema` derive executes without panic on every
    //! surviving Props struct — a compile-time `#[derive(JsonSchema)]` alone
    //! does not prove the generated code runs.
    //!
    //! One `#[test]` per type for clear failure localization.

    use super::*;

    fn assert_schema_nonempty_object<T: schemars::JsonSchema>(type_label: &str) {
        let schema = schemars::schema_for!(T);
        let value = serde_json::to_value(&schema).expect("schema serializes to JSON");
        assert!(
            value.is_object(),
            "{type_label}: schema must be a JSON object"
        );
        let props = value
            .get("properties")
            .and_then(|p| p.as_object())
            .map(|o| !o.is_empty())
            .unwrap_or(false);
        assert!(
            props,
            "{type_label}: schema must have a non-empty `properties` field"
        );
    }

    #[test]
    fn schema_for_card_props_generates() {
        assert_schema_nonempty_object::<CardProps>("CardProps");
    }

    #[test]
    fn schema_for_table_props_generates() {
        assert_schema_nonempty_object::<TableProps>("TableProps");
    }

    #[test]
    fn schema_for_form_props_generates() {
        assert_schema_nonempty_object::<FormProps>("FormProps");
    }

    #[test]
    fn schema_for_button_props_generates() {
        assert_schema_nonempty_object::<ButtonProps>("ButtonProps");
    }

    #[test]
    fn schema_for_input_props_generates() {
        assert_schema_nonempty_object::<InputProps>("InputProps");
    }

    #[test]
    fn schema_for_select_props_generates() {
        assert_schema_nonempty_object::<SelectProps>("SelectProps");
    }

    #[test]
    fn schema_for_alert_props_generates() {
        assert_schema_nonempty_object::<AlertProps>("AlertProps");
    }

    #[test]
    fn schema_for_badge_props_generates() {
        assert_schema_nonempty_object::<BadgeProps>("BadgeProps");
    }

    #[test]
    fn schema_for_modal_props_generates() {
        assert_schema_nonempty_object::<ModalProps>("ModalProps");
    }

    #[test]
    fn schema_for_text_props_generates() {
        assert_schema_nonempty_object::<TextProps>("TextProps");
    }

    #[test]
    fn schema_for_checkbox_props_generates() {
        assert_schema_nonempty_object::<CheckboxProps>("CheckboxProps");
    }

    #[test]
    fn schema_for_switch_props_generates() {
        assert_schema_nonempty_object::<SwitchProps>("SwitchProps");
    }

    #[test]
    fn schema_for_separator_props_generates() {
        assert_schema_nonempty_object::<SeparatorProps>("SeparatorProps");
    }

    #[test]
    fn schema_for_description_list_props_generates() {
        assert_schema_nonempty_object::<DescriptionListProps>("DescriptionListProps");
    }

    #[test]
    fn schema_for_tab_generates() {
        assert_schema_nonempty_object::<Tab>("Tab");
    }

    #[test]
    fn schema_for_tabs_props_generates() {
        assert_schema_nonempty_object::<TabsProps>("TabsProps");
    }

    #[test]
    fn schema_for_breadcrumb_props_generates() {
        assert_schema_nonempty_object::<BreadcrumbProps>("BreadcrumbProps");
    }

    #[test]
    fn schema_for_pagination_props_generates() {
        assert_schema_nonempty_object::<PaginationProps>("PaginationProps");
    }

    #[test]
    fn schema_for_progress_props_generates() {
        assert_schema_nonempty_object::<ProgressProps>("ProgressProps");
    }

    #[test]
    fn schema_for_image_props_generates() {
        assert_schema_nonempty_object::<ImageProps>("ImageProps");
    }

    #[test]
    fn image_inline_svg_factory_roundtrips_via_serde() {
        let p = ImageProps::inline_svg("<svg/>", "alt");
        let json = serde_json::to_value(&p).expect("serialization must not fail");
        let parsed: ImageProps =
            serde_json::from_value(json).expect("deserialization must not fail");
        assert_eq!(parsed.inline_svg, Some("<svg/>".to_string()));
        assert_eq!(parsed.alt, "alt");
        assert_eq!(parsed.src, "");
    }

    #[test]
    fn schema_for_avatar_props_generates() {
        assert_schema_nonempty_object::<AvatarProps>("AvatarProps");
    }

    #[test]
    fn schema_for_skeleton_props_generates() {
        assert_schema_nonempty_object::<SkeletonProps>("SkeletonProps");
    }

    #[test]
    fn schema_for_stat_card_props_generates() {
        assert_schema_nonempty_object::<StatCardProps>("StatCardProps");
    }

    #[test]
    fn schema_for_checklist_props_generates() {
        assert_schema_nonempty_object::<ChecklistProps>("ChecklistProps");
    }

    #[test]
    fn schema_for_toast_props_generates() {
        assert_schema_nonempty_object::<ToastProps>("ToastProps");
    }

    #[test]
    fn schema_for_notification_dropdown_props_generates() {
        assert_schema_nonempty_object::<NotificationDropdownProps>("NotificationDropdownProps");
    }

    #[test]
    fn schema_for_sidebar_props_generates() {
        assert_schema_nonempty_object::<SidebarProps>("SidebarProps");
    }

    #[test]
    fn schema_for_header_props_generates() {
        assert_schema_nonempty_object::<HeaderProps>("HeaderProps");
    }

    #[test]
    fn schema_for_grid_props_generates() {
        assert_schema_nonempty_object::<GridProps>("GridProps");
    }

    #[test]
    fn schema_for_collapsible_props_generates() {
        assert_schema_nonempty_object::<CollapsibleProps>("CollapsibleProps");
    }

    #[test]
    fn schema_for_empty_state_props_generates() {
        assert_schema_nonempty_object::<EmptyStateProps>("EmptyStateProps");
    }

    #[test]
    fn schema_for_form_section_props_generates() {
        assert_schema_nonempty_object::<FormSectionProps>("FormSectionProps");
    }

    #[test]
    fn schema_for_page_header_props_generates() {
        assert_schema_nonempty_object::<PageHeaderProps>("PageHeaderProps");
    }

    #[test]
    fn schema_for_button_group_props_generates() {
        assert_schema_nonempty_object::<ButtonGroupProps>("ButtonGroupProps");
    }

    #[test]
    fn schema_for_dropdown_menu_action_generates() {
        assert_schema_nonempty_object::<DropdownMenuAction>("DropdownMenuAction");
    }

    #[test]
    fn schema_for_dropdown_menu_props_generates() {
        assert_schema_nonempty_object::<DropdownMenuProps>("DropdownMenuProps");
    }

    #[test]
    fn schema_for_data_table_props_generates() {
        assert_schema_nonempty_object::<DataTableProps>("DataTableProps");
    }

    #[test]
    fn schema_for_kanban_column_props_generates() {
        assert_schema_nonempty_object::<KanbanColumnProps>("KanbanColumnProps");
    }

    #[test]
    fn schema_for_kanban_board_props_generates() {
        assert_schema_nonempty_object::<KanbanBoardProps>("KanbanBoardProps");
    }

    #[test]
    fn schema_for_calendar_cell_props_generates() {
        assert_schema_nonempty_object::<CalendarCellProps>("CalendarCellProps");
    }

    #[test]
    fn schema_for_action_card_props_generates() {
        assert_schema_nonempty_object::<ActionCardProps>("ActionCardProps");
    }

    #[test]
    fn schema_for_product_tile_props_generates() {
        assert_schema_nonempty_object::<ProductTileProps>("ProductTileProps");
    }

    #[test]
    fn card_props_round_trips_footer() {
        let original = CardProps {
            title: "Hero".to_string(),
            description: None,
            subtitle: None,
            badge: None,
            max_width: None,
            footer: vec!["btn1".to_string(), "btn2".to_string()],
            variant: CardVariant::Bordered,
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: CardProps = serde_json::from_str(&json).unwrap();
        assert_eq!(original.footer, parsed.footer);
    }

    #[test]
    fn tab_round_trips_children() {
        let original = Tab {
            value: "overview".to_string(),
            label: "Overview".to_string(),
            children: vec!["panel1".to_string()],
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: Tab = serde_json::from_str(&json).unwrap();
        assert_eq!(original.children, parsed.children);
    }

    #[test]
    fn card_props_omits_empty_footer_in_json() {
        let card = CardProps {
            title: "Card".to_string(),
            description: None,
            subtitle: None,
            badge: None,
            max_width: None,
            footer: Vec::new(),
            variant: CardVariant::Bordered,
        };
        let json = serde_json::to_string(&card).unwrap();
        assert!(
            !json.contains("\"footer\""),
            "empty footer must be skipped, got: {json}"
        );
    }

    #[test]
    fn card_props_round_trips_badge() {
        let original = CardProps {
            title: "Hero".to_string(),
            description: None,
            subtitle: None,
            badge: Some("Scade tra 9m".to_string()),
            max_width: None,
            footer: Vec::new(),
            variant: CardVariant::Bordered,
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: CardProps = serde_json::from_str(&json).unwrap();
        assert_eq!(original.badge, parsed.badge);
    }

    #[test]
    fn card_props_omits_empty_badge_in_json() {
        let card = CardProps {
            title: "Card".to_string(),
            description: None,
            subtitle: None,
            badge: None,
            max_width: None,
            footer: Vec::new(),
            variant: CardVariant::Bordered,
        };
        let json = serde_json::to_string(&card).unwrap();
        assert!(
            !json.contains("\"badge\""),
            "empty badge must be skipped, got: {json}"
        );
    }

    #[test]
    fn card_props_round_trips_subtitle() {
        let original = CardProps {
            title: "Hero".to_string(),
            description: None,
            subtitle: Some("Marco Rossi".to_string()),
            badge: None,
            max_width: None,
            footer: Vec::new(),
            variant: CardVariant::Bordered,
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: CardProps = serde_json::from_str(&json).unwrap();
        assert_eq!(original.subtitle, parsed.subtitle);
    }

    #[test]
    fn card_props_omits_empty_subtitle_in_json() {
        let card = CardProps {
            title: "Card".to_string(),
            description: None,
            subtitle: None,
            badge: None,
            max_width: None,
            footer: Vec::new(),
            variant: CardVariant::Bordered,
        };
        let json = serde_json::to_string(&card).unwrap();
        assert!(
            !json.contains("\"subtitle\""),
            "empty subtitle must be skipped, got: {json}"
        );
    }

    #[test]
    fn card_props_schema_includes_badge() {
        let schema = schemars::schema_for!(CardProps);
        let value = serde_json::to_value(&schema).expect("schema serializes to JSON");
        let props = value
            .get("properties")
            .and_then(|p| p.as_object())
            .expect("schema has a properties object");
        assert!(
            props.contains_key("badge"),
            "CardProps schema must expose a `badge` property; got keys: {:?}",
            props.keys().collect::<Vec<_>>()
        );
        // `badge: Option<String>` — schemars 1.x emits either {"type": ["string","null"]}
        // or a {"type":"string"} entry inside a oneOf/anyOf branch. We only assert
        // presence + that the rendered schema mentions a string somewhere under
        // the badge entry, which is robust to either encoding.
        let badge_schema = props.get("badge").expect("badge entry");
        let badge_json = badge_schema.to_string();
        assert!(
            badge_json.contains("\"string\""),
            "badge schema entry must mention string type; got: {badge_json}"
        );
    }

    #[test]
    fn card_props_schema_includes_subtitle() {
        let schema = schemars::schema_for!(CardProps);
        let value = serde_json::to_value(&schema).expect("schema serializes to JSON");
        let props = value
            .get("properties")
            .and_then(|p| p.as_object())
            .expect("schema has a properties object");
        assert!(
            props.contains_key("subtitle"),
            "CardProps schema must expose a `subtitle` property; got keys: {:?}",
            props.keys().collect::<Vec<_>>()
        );
        // Same robustness note as `card_props_schema_includes_badge` —
        // `subtitle: Option<String>` may surface as type-union or oneOf depending
        // on the schemars version. Assert string is mentioned in the rendered
        // entry rather than locking down the exact null encoding.
        let subtitle_schema = props.get("subtitle").expect("subtitle entry");
        let subtitle_json = subtitle_schema.to_string();
        assert!(
            subtitle_json.contains("\"string\""),
            "subtitle schema entry must mention string type; got: {subtitle_json}"
        );
    }

    #[test]
    fn schema_for_checkbox_list_props_generates() {
        assert_schema_nonempty_object::<CheckboxListProps>("CheckboxListProps");
    }

    #[test]
    fn checkbox_list_props_serde_roundtrip() {
        let json = serde_json::json!({
            "field": "services",
            "options": [{"value": "a", "label": "Alpha"}, {"value": "b", "label": "Beta"}],
            "selected_path": "/preselected"
        });
        let parsed: CheckboxListProps = serde_json::from_value(json.clone()).expect("decode");
        assert_eq!(parsed.field, "services");
        assert_eq!(parsed.options.len(), 2);
        assert_eq!(parsed.selected_path.as_deref(), Some("/preselected"));
        let reserialized = serde_json::to_value(&parsed).expect("encode");
        // None/empty fields are omitted by serde.
        assert!(reserialized.get("label").is_none());
        assert!(reserialized.get("disabled").is_none());
    }

    #[test]
    fn schema_for_rich_text_editor_props_generates() {
        assert_schema_nonempty_object::<RichTextEditorProps>("RichTextEditorProps");
    }

    #[test]
    fn rich_text_editor_props_serde_roundtrip() {
        let json = serde_json::json!({
            "field": "body",
            "label": "Body"
        });
        let parsed: RichTextEditorProps = serde_json::from_value(json).expect("decode");
        assert_eq!(parsed.field, "body");
        assert_eq!(parsed.label, "Body");
        assert!(parsed.placeholder.is_none());
        assert!(parsed.default_value.is_none());
        assert!(parsed.data_path.is_none());
        assert!(parsed.error.is_none());
        let reserialized = serde_json::to_value(&parsed).expect("encode");
        // Optional None fields are omitted.
        assert!(reserialized.get("placeholder").is_none());
        assert!(reserialized.get("error").is_none());
    }
}

#[cfg(test)]
mod strum_tests {
    use super::*;

    /// Assert AsRef<str> matches serde JSON wire format for every variant of
    /// AlertVariant, BadgeVariant, ButtonVariant, and ToastVariant.
    /// Threat T-162-08-01: strum and serde must agree on every snake_case string.
    #[test]
    fn variant_enums_strum_matches_serde_wire_format() {
        fn check<T: AsRef<str> + serde::Serialize>(variants: &[T], label: &str) {
            for v in variants {
                let json = serde_json::to_string(v).expect("serialize");
                let json_stripped = json.trim_matches('"');
                assert_eq!(
                    v.as_ref(),
                    json_stripped,
                    "strum AsRefStr drifted from serde for {label} variant"
                );
            }
        }
        check(
            &[
                AlertVariant::Info,
                AlertVariant::Success,
                AlertVariant::Warning,
                AlertVariant::Error,
            ],
            "AlertVariant",
        );
        check(
            &[
                BadgeVariant::Default,
                BadgeVariant::Secondary,
                BadgeVariant::Destructive,
                BadgeVariant::Outline,
            ],
            "BadgeVariant",
        );
        check(
            &[
                ButtonVariant::Default,
                ButtonVariant::Secondary,
                ButtonVariant::Destructive,
                ButtonVariant::Outline,
                ButtonVariant::Ghost,
                ButtonVariant::Link,
            ],
            "ButtonVariant",
        );
        check(
            &[
                ToastVariant::Info,
                ToastVariant::Success,
                ToastVariant::Warning,
                ToastVariant::Error,
            ],
            "ToastVariant",
        );
    }

    #[test]
    fn alert_variant_as_ref_str_matches_wire_format() {
        assert_eq!(AlertVariant::Success.as_ref(), "success");
        assert_eq!(AlertVariant::Warning.as_ref(), "warning");
        assert_eq!(AlertVariant::Info.as_ref(), "info");
        assert_eq!(AlertVariant::Error.as_ref(), "error");
    }
}

#[cfg(test)]
mod card_variant_tests {
    use super::*;

    #[test]
    fn card_variant_default_is_bordered() {
        assert_eq!(CardVariant::default(), CardVariant::Bordered);
    }

    #[test]
    fn card_variant_serializes_snake_case() {
        assert_eq!(
            serde_json::to_value(CardVariant::Bordered).unwrap(),
            serde_json::json!("bordered")
        );
        assert_eq!(
            serde_json::to_value(CardVariant::Elevated).unwrap(),
            serde_json::json!("elevated")
        );
    }

    #[test]
    fn card_variant_deserializes_snake_case() {
        assert_eq!(
            serde_json::from_value::<CardVariant>(serde_json::json!("bordered")).unwrap(),
            CardVariant::Bordered
        );
        assert_eq!(
            serde_json::from_value::<CardVariant>(serde_json::json!("elevated")).unwrap(),
            CardVariant::Elevated
        );
    }

    #[test]
    fn card_props_without_variant_defaults_to_bordered() {
        let v = serde_json::json!({"title": "x"});
        let p: CardProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.variant, CardVariant::Bordered);
    }

    #[test]
    fn card_props_with_elevated_variant() {
        let v = serde_json::json!({"title": "x", "variant": "elevated"});
        let p: CardProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.variant, CardVariant::Elevated);
    }

    #[test]
    fn card_props_roundtrip_preserves_variant() {
        let p = CardProps {
            title: "x".into(),
            description: None,
            subtitle: None,
            badge: None,
            max_width: None,
            footer: vec![],
            variant: CardVariant::Elevated,
        };
        let j = serde_json::to_value(&p).unwrap();
        let back: CardProps = serde_json::from_value(j).unwrap();
        assert_eq!(back.variant, CardVariant::Elevated);
    }
}

#[cfg(test)]
mod kanban_board_props_tests {
    use super::*;

    #[test]
    fn kanban_board_props_serde_static_columns() {
        let v = serde_json::json!({
            "columns": [{"title": "To Do", "items": [], "id": "todo", "count": 0}]
        });
        let p: KanbanBoardProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.columns.len(), 1);
        assert!(p.data_path.is_none());
    }

    #[test]
    fn kanban_board_props_serde_data_path() {
        let v = serde_json::json!({"data_path": "/columns"});
        let p: KanbanBoardProps = serde_json::from_value(v).unwrap();
        assert!(p.columns.is_empty());
        assert_eq!(p.data_path.as_deref(), Some("/columns"));
    }

    #[test]
    fn kanban_board_props_serde_neither() {
        let v = serde_json::json!({});
        let p: KanbanBoardProps = serde_json::from_value(v).unwrap();
        assert!(p.columns.is_empty());
        assert!(p.data_path.is_none());
    }

    #[test]
    fn kanban_board_props_empty_columns_skipped_on_serialize() {
        let p = KanbanBoardProps {
            columns: vec![],
            data_path: Some("/x".into()),
            mobile_default_column: None,
            empty_label: None,
        };
        let j = serde_json::to_value(&p).unwrap();
        assert!(
            j.get("columns").is_none(),
            "empty columns must be skipped, got: {j}"
        );
        assert_eq!(j.get("data_path").and_then(|v| v.as_str()), Some("/x"));
    }
}

#[cfg(test)]
mod page_header_actions_tests {
    use super::*;

    #[test]
    fn page_header_actions_missing_field() {
        let v = serde_json::json!({"title": "X"});
        let p: PageHeaderProps = serde_json::from_value(v).unwrap();
        assert!(p.actions.is_empty());
    }

    #[test]
    fn page_header_actions_null() {
        let v = serde_json::json!({"title": "X", "actions": null});
        let p: PageHeaderProps = serde_json::from_value(v).unwrap();
        assert!(p.actions.is_empty());
    }

    #[test]
    fn page_header_actions_empty_string() {
        let v = serde_json::json!({"title": "X", "actions": ""});
        let p: PageHeaderProps = serde_json::from_value(v).unwrap();
        assert!(p.actions.is_empty());
    }

    #[test]
    fn page_header_actions_empty_array() {
        let v = serde_json::json!({"title": "X", "actions": []});
        let p: PageHeaderProps = serde_json::from_value(v).unwrap();
        assert!(p.actions.is_empty());
    }

    #[test]
    fn page_header_actions_non_empty_array() {
        let v = serde_json::json!({"title": "X", "actions": ["a", "b"]});
        let p: PageHeaderProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.actions, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn page_header_actions_non_empty_string_rejected() {
        let v = serde_json::json!({"title": "X", "actions": "not-empty"});
        let result: Result<PageHeaderProps, _> = serde_json::from_value(v);
        assert!(result.is_err(), "non-empty string must be rejected");
    }

    #[test]
    fn page_header_actions_non_string_array_rejected() {
        let v = serde_json::json!({"title": "X", "actions": [1, 2, 3]});
        let result: Result<PageHeaderProps, _> = serde_json::from_value(v);
        assert!(result.is_err(), "array of non-strings must be rejected");
    }
}
