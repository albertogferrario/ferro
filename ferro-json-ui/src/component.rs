//! Component catalog for JSON-UI.
//!
//! Defines the available UI components with typed props. Each component
//! uses serde's tagged enum representation so JSON includes `"type": "Card"`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::action::Action;

/// Visual weight of interactive elements (buttons, action items). `primary` is the default.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    strum::AsRefStr,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Variant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
}

/// Semantic status color of stateful display components. `neutral` is the default and
/// reproduces today's non-status look.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    strum::AsRefStr,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Tone {
    #[default]
    Neutral,
    Success,
    Warning,
    Destructive,
}

/// Component size scale. `md` is the default.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    strum::AsRefStr,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Size {
    Sm,
    #[default]
    Md,
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
///
/// `Badge` cells expect the row value to be an object `{tone, label}` matching
/// [`BadgeProps`]. Other variants are display hints layered over plain cell text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ColumnFormat {
    Date,
    DateTime,
    Currency,
    Boolean,
    Badge,
    /// Cell value is an image URL string; rendered as an `<img>` thumbnail.
    Image,
    /// Cell value is a built-in icon name (e.g. `folder`, `file`); rendered as
    /// an inline outline SVG that inherits `currentColor`. Unknown names render
    /// an empty cell. Use for type/status glyphs that should match the line-icon
    /// system rather than emoji.
    Icon,
}

/// Horizontal text alignment for a table column (header + cells).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ColumnAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Table column definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Column {
    pub key: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<ColumnFormat>,
    /// Horizontal alignment of the header and cells. Defaults to left.
    /// Use `right` for numeric/currency columns so magnitudes line up.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<ColumnAlign>,
    /// Display label when the boolean cell value is true. Defaults to "Sì".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_true: Option<String>,
    /// Display label when the boolean cell value is false. Defaults to "No".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_false: Option<String>,
    /// When set, opts the link column into peek-cards by emitting
    /// data-peek-entity and data-peek-id attributes on the rendered <a>.
    /// Value is the entity kind, e.g. "clienti", "prodotti".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peek_entity: Option<String>,
}

/// Select option (value + label pair).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

/// Structural chrome of a Card — NOT a weight or status axis (renamed from `CardVariant`).
///
/// - `Bordered` (default): `border + bg-card + shadow-sm` with `p-4`.
///   Dashboard cards in dense layouts.
/// - `Elevated`: `bg-card + shadow-md` (no border) with `p-8`.
///   Auth pages, error pages, standalone marketing cards.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CardAppearance {
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
    pub appearance: CardAppearance,
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
    /// When `true`, the rendered `<form>` joins the fill-viewport height chain:
    /// it emits `flex flex-col h-full min-h-0 [&>*]:flex-1 [&>*]:min-h-0`
    /// instead of the default `flex flex-wrap` layout, stretching its single
    /// child to the full height of the parent so an inner fill Grid resolves
    /// `h-full` against a real (viewport-constrained) height rather than
    /// content height. Set by the Register layout template
    /// (`emit_register_root`) so the SelectionPanel footer pins while the
    /// panes scroll independently (256 D-15). Absent/`false` keeps the default
    /// content-sized form layout — byte-identical to prior renders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<bool>,
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
    pub variant: Variant,
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
    /// When `true`, emits `data-disable-on-submit` on the rendered button; the runtime guard
    /// disables this button after the first form submission to prevent double-posting (D-16).
    /// Pairs with a per-render `idempotency_key` hidden input for server-side deduplication
    /// (see `dispatch_write` step 2 in docs/src/features/write-kernel.md).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_on_submit: Option<bool>,
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
    /// Progressive enhancement: wrap the native date/time input in the
    /// `[data-date-picker]` markup so the client-side calendar picker activates.
    ///
    /// Valid only for `input_type = "date"`, `"time"`, or `"datetime-local"`.
    /// The native input remains the form value carrier (no-JS fallback intact).
    /// When `false` or absent, a plain `<input>` is emitted as usual.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_picker: Option<bool>,
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
    /// When true, renders a progressive-enhancement combobox overlay over the
    /// native <select>. The native select remains the form value carrier (D-06).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub searchable: Option<bool>,
}

/// Props for Alert component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AlertProps {
    pub message: String,
    #[serde(default)]
    pub tone: Tone,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Props for Badge component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BadgeProps {
    pub label: String,
    #[serde(default)]
    pub tone: Tone,
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
    /// Field name for inline editing (e.g., "name", "email"). None = read-only.
    /// Security boundary: this name is HTML-escaped and echoed in data-attrs;
    /// the server allowlist in the inline-edit endpoint is the real gate (D-10).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_edit_field: Option<String>,
    /// POST endpoint URL for inline edit (e.g., "/dashboard/clienti/42/field").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_edit_endpoint: Option<String>,
    /// Input kind: "text" | "textarea" | "number". Defaults to "text".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_edit_kind: Option<String>,
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

/// Props for the `StreamText` component — SSE token stream renderer.
///
/// Connects to `sse_url` via the browser `EventSource` API and appends arriving
/// tokens as plain text nodes. The SSE endpoint MUST emit `event: done` on
/// completion to prevent `EventSource` auto-reconnect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StreamTextProps {
    /// URL of the server-sent-events endpoint that streams tokens.
    /// Must emit `event: done` on completion.
    #[serde(default)]
    pub sse_url: String,
    /// Text shown inside the content area before the first token arrives.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// Status text shown while the stream is open.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loading_text: Option<String>,
}

/// Props for the `LiveFragment` builtin — binds a child template to a
/// `ferro-projection` per-key snapshot for server-push in-place re-render.
///
/// First paint: the handler resolves `projection` + `key` via
/// `ProjectionRuntime::read`, serializes the state (or uses `{}` when absent,
/// per D-04), and passes the `Value` as the data scope for `template`.
///
/// On delta: the registered fragment hook re-renders `template` against the
/// new snapshot and broadcasts `{ html }` on the same
/// `projection.{name}.{key}` channel; the client runtime swaps `innerHTML`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LiveFragmentProps {
    /// ferro-projection NAME — the `Projection::NAME` const of the target projection.
    #[serde(default)]
    pub projection: String,
    /// Per-key channel selector (the `key` segment of `projection.{name}.{key}`).
    #[serde(default)]
    pub key: String,
    /// Child template spec rendered against the snapshot as its data scope.
    /// A `serde_json::Value` encoding a valid ferro-json-ui `Spec`.
    pub template: serde_json::Value,
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
    /// Semantic status color for the value/icon accent. `neutral` (default)
    /// reproduces the plain non-status look.
    #[serde(default)]
    pub tone: Tone,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// SSE target key for live updates; maps to `data-sse-target` on the value element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sse_target: Option<String>,
    /// Resolves the initial displayed value from handler data at render time.
    /// Format: `/segment/segment` (same JSON-pointer as `data::resolve_path`).
    /// Falls back to `value` when missing or non-string. Mirrors
    /// `ImageProps.data_path` / `DescriptionListProps.data_path`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_path: Option<String>,
    /// Pre-rendered inline SVG sparkline string from the consumer handler.
    /// Emitted as a sibling <div> of the value element — NOT inside the
    /// data-sse-target element (Pitfall 5: SSE updates replace the value element's
    /// textContent and must not erase the sparkline).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sparkline_svg: Option<String>,
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
    pub tone: Tone,
    /// Seconds before auto-dismiss. Default 5. `0` with `dismissible: true`
    /// keeps the toast visible until manually closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    /// Render a manual close button. When `false`, `timeout` is clamped to a
    /// minimum of 1 second so the toast always auto-dismisses.
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
    /// POST endpoint for the avatar-menu theme toggle. The endpoint is
    /// app-specific, so consumers must provide it; `None` omits the Tema
    /// menu item entirely.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme_url: Option<String>,
    /// Destination of the avatar-menu "Profilo" item. The settings route is
    /// app-specific, so consumers must provide it; `None` omits the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_url: Option<String>,
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
    /// Per-child column spans, aligned positionally with `children` (missing
    /// entries default to 1). A child with span N occupies N tracks — e.g.
    /// `columns: 1, md_columns: 3, spans: [2, 1]` renders a 2/3 + 1/3 row.
    /// Supported spans: 2–4 on the base grid, 2–3 at the `md` breakpoint.
    /// Ignored in `scrollable` mode.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spans: Vec<u8>,
    /// Per-row height weights for fill-mode grids. Positional alignment with
    /// `children` (missing entries default to equal weight). A row with weight N
    /// receives N fractional units of available height — e.g. `row_weights: [2, 1]`
    /// gives the first row 2/3 and the second 1/3. Meaningful only when
    /// `fill: true`; ignored in `scrollable` mode. The render path (fractional
    /// `grid-template-rows` via inline style) lands in Phase 256.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub row_weights: Vec<u8>,
    /// Fill mode for viewport workspaces (pages with `Spec.fill_viewport`):
    /// the grid stretches to its parent's height with equal-height rows and
    /// every child cell scrolls internally. The document never scrolls —
    /// each pane does. Combine with `spans` for asymmetric panes (e.g. a
    /// POS register: 1/3 cart + 2/3 product grid). Ignored in `scrollable`
    /// mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<bool>,
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

/// A single action in an `ActionGroup`'s ordered item list.
///
/// Inline items (non-destructive, within `max_inline`) render as buttons.
/// The `destructive` flag forces the item into the overflow kebab and renders
/// it last regardless of its position in `items`.
///
/// `visible_if` is a fail-closed row gate (same semantics as
/// `DropdownMenuAction.visible_if`): when set, the item is hidden unless
/// `row[field]` is truthy. An absent or falsy field hides the item — a typo
/// in the field name cannot leak an action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ActionItem {
    pub label: String,
    pub action: Action,
    /// When true, this item is forced into the overflow kebab and rendered last,
    /// regardless of position in `items`. Does not count toward `max_inline`.
    #[serde(default)]
    pub destructive: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<Variant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Fail-closed row gate (same semantics as `DropdownMenuAction.visible_if`).
    /// When set, the item is only shown when `row[visible_if]` is truthy.
    /// Absent/falsy field hides the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_if: Option<String>,
}

/// Props for `ActionGroup` — ordered action list rendering inline buttons (up to
/// `max_inline`) plus a trailing overflow kebab for the remainder. Destructive
/// items are always in the kebab, rendered last, regardless of input order.
///
/// Input order determines button priority: the first item in `items` is the
/// primary action and renders first inline. Use `variant` on an item to control
/// button styling.
///
/// The overflow kebab is hidden entirely when nothing overflows (≤ `max_inline`
/// non-destructive items and zero destructive items).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ActionGroupProps {
    pub items: Vec<ActionItem>,
    /// ID pairing the overflow popover to its trigger button. Required; callers
    /// must supply a unique value per page to prevent DOM id collisions.
    pub menu_id: String,
    /// Maximum non-destructive items rendered inline (default 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_inline: Option<u8>,
    /// Aria-label for the overflow trigger button (default "Azioni").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow_label: Option<String>,
    /// Key used for `{row_key}` substitution in action URLs (DataTable / Kanban context).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_key: Option<String>,
}

/// Props for SegmentedControl — a tightly-packed cluster of toggle/nav links
/// rendered as a single bordered group with no gap between segments.
///
/// Items come either as a literal `items` array or from runtime data via
/// `data_path` (controller-built). At least one of the two must be supplied;
/// `items` wins when both are present.
///
/// Visual model: rounded outer container with a single border, internal
/// dividers between segments, one segment marked `active=true` and styled
/// distinctly. The label can be the literal segment text (e.g. "Oggi") or a
/// glyph (e.g. "←", "→"). Each segment carries an optional `aria_label`
/// override for accessibility on glyph-only segments.
///
/// Use cases captured by this primitive: date scroll clusters (prev/today/next),
/// view toggles (Day/Month, List/Grid), pagination steppers, mode switchers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub struct SegmentedControlProps {
    /// Literal items list. Skipped when empty; `data_path` is the fallback.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<SegmentedItem>,
    /// JSON Pointer into runtime data resolving to an array of `SegmentedItem`s.
    /// Used when items shape depends on per-request data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
    /// Visual size — defaults to `default`.
    #[serde(default)]
    pub size: Size,
    /// Accessible label for the group (`<div role="tablist" aria-label="...">`).
    /// Omit when the surrounding context already announces purpose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aria_label: Option<String>,
}

/// One segment of a `SegmentedControl`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub struct SegmentedItem {
    /// Visible label or glyph.
    pub label: String,
    /// Destination URL — segments render as `<a href>` so they work without JS.
    pub href: String,
    /// Active segment (one per group, typically). Highlighted, `aria-selected=true`.
    #[serde(default)]
    pub active: bool,
    /// Optional accessible label override — useful when `label` is a glyph
    /// like "←" or "→" that screen readers cannot pronounce.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aria_label: Option<String>,
}

/// Props for SidebarLayout — a two-column layout with a sticky vertical nav
/// on the left and a main content slot on the right. Replaces the common
/// pattern of opener/closer `RawHtml` blocks faking asymmetric grids.
///
/// The element's `children` IDs render inside the main slot. Each child is
/// expected to carry its own `visible` rule keyed against `active` (typically
/// `{ path: "/active_tab", operator: "eq", value: "<slug>" }`) so only the
/// matching section is in the DOM at a time.
///
/// On mobile (below `md`), the sidebar collapses into a horizontally
/// scrollable strip above the main content, and the asymmetric grid layout
/// flattens to a single column.
///
/// Use cases: settings pages with many sections, account dashboards,
/// onboarding wizards with persistent navigation, admin consoles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub struct SidebarLayoutProps {
    /// Literal sidebar items. Skipped when empty; `data_path` is the fallback.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<SidebarLayoutItem>,
    /// JSON Pointer into runtime data resolving to an array of `SidebarLayoutItem`s.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
    /// Slug of the currently-active item. Matched against `SidebarLayoutItem.slug`.
    /// Typically bound via `{ "$data": "/active_tab" }`.
    pub active: String,
    /// Accessible label for the nav (`<nav aria-label="...">`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aria_label: Option<String>,
}

/// One sidebar nav item in a `SidebarLayout`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub struct SidebarLayoutItem {
    /// Item identifier — matched against `SidebarLayoutProps.active` to determine
    /// which item is highlighted.
    pub slug: String,
    /// Visible label.
    pub label: String,
    /// Destination URL. Typically `"?tab={slug}"` for query-driven routing,
    /// but can be any absolute or relative URL.
    pub url: String,
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
    /// When set, this item is only emitted in a DataTable row when the row's
    /// `visible_if` field is truthy (true / non-zero number / non-empty string /
    /// non-empty array or object). An absent or falsy field hides the item —
    /// fail-closed so a typo in the view spec cannot leak an action onto every
    /// row. Outside DataTable contexts (e.g. standalone `DropdownMenu` element)
    /// the field is ignored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_if: Option<String>,
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
    /// When true, renders a leading checkbox column for bulk row selection.
    /// Selection behavior (floating bar, action dispatch) is Phase 249 (LIST-03).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bulk_select: Option<bool>,
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
    /// CSS object-position for the cropped image: "top" | "center" | "bottom"
    /// (or any valid object-position value). Default "center".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_position: Option<String>,
    /// Key for the footer badge label text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge_key: Option<String>,
    /// Key for the badge tone string: "neutral" | "success" | "warning" | "destructive".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge_tone_key: Option<String>,
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

/// Props for a single column (lane) in a KanbanBoard.
///
/// A column is structure: its `id` is the lane key matched against each
/// item's `group_by` value, and `title` is the lane header. `count` and
/// `children` are only honored by static specs that set neither
/// `KanbanBoardProps.items_path` nor `group_by`; in the data-bound path the
/// renderer computes the count and renders cards from `items_path`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct KanbanColumnProps {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub count: u32,
    /// IDs of elements rendered inside this column (static specs only).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
}

/// Props for KanbanBoard — horizontal scrollable columns on desktop, tab-based
/// on mobile.
///
/// A kanban is fixed lanes plus items sorted into them by a status field.
/// `columns` is structure only (lane `id` + `title`) and is always rendered —
/// an empty lane still shows its header and a zero count. Card content is
/// data-bound: `items_path` resolves a flat array of entity objects, each
/// bucketed into the column whose `id` equals the item's `group_by` value,
/// then rendered as a card via the `card_*` / `row_*` bindings. This is the
/// same prescribed-card + field-key convention used by `DataTable` and
/// `MediaCardGrid`. For fully-custom card structure, template the cards with
/// the `$each` directive inside a `KanbanColumn` instead.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct KanbanBoardProps {
    /// Lane structure — `id` + `title`. Always rendered.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<KanbanColumnProps>,
    /// JSON-Pointer to a flat array of entity objects. Each item is bucketed
    /// into the column whose `id` equals the item's `group_by` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items_path: Option<String>,
    /// Field on each item that selects its lane: `column.id == item[group_by]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_by: Option<String>,
    /// Item field whose value becomes the card title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_title_key: Option<String>,
    /// Item field whose value becomes the card subtitle/description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_description_key: Option<String>,
    /// Per-card dropdown actions. `{row_key}` / `{id}` interpolate from the
    /// item, matching `DataTable` / `MediaCardGrid`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_actions: Option<Vec<DropdownMenuAction>>,
    /// Item field used for `{row_key}` substitution in action URLs
    /// (defaults to `id`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mobile_default_column: Option<String>,
    /// Placeholder text shown inside empty lanes. When `None`, empty lanes
    /// render no placeholder (back-compat). Provide a short, neutral message —
    /// e.g. "Nessun ordine", "Nothing here".
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
    /// When true the day is marked closed (unavailable): a neutral diagonal hatch
    /// (repeating stripes) is drawn across the cell. Independent of `event_count`
    /// — a closed day may still carry existing bookings, so the dots still render.
    #[serde(default)]
    pub closed: bool,
}

/// Props for a horizontal action card with tone-colored left border.
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
    pub tone: Tone,
    /// Optional navigation URL. When set, the card renders as an `<a>` element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

/// Props for a touch-friendly product tile with quantity controls.
///
/// Renders item name, price, and +/- buttons that drive a hidden input
/// via JS. Used for touch-first selection screens (e.g. POS-style order creation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TileProps {
    pub item_id: String,
    pub name: String,
    pub price: String,
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_quantity: Option<u32>,
    /// Category memberships for client-side filtering. Rendered by
    /// `render_tile` as a space-separated `data-filter-tokens` attribute,
    /// emitted only when non-empty; the `setupFilters` runtime reads it.
    /// Plural because an item may belong to several categories (a one-element
    /// vec covers the singular case).
    ///
    /// Token-list constraint: because the attribute is space-separated, spaces
    /// inside a category name are normalized to hyphens at render time
    /// (`"Bevande calde"` becomes the token `Bevande-calde`). Filter runtimes
    /// must apply the same normalization to category labels before matching.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    /// Optional item image URL. Declared here for the Phase 256 tile visual;
    /// not rendered in Phase 254 (D-03).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Optional accent tone for the tile border (Phase 256 visual, D-03).
    /// Maps through an exhaustive match to a full-literal border class in
    /// `render_tile`; `None` or `Neutral` → the default `border-border`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Tone>,
    /// Optional stock badge text (e.g. "Low", "Out"). Phase 256 visual (D-03).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stock_badge: Option<String>,
    /// Machine-readable unit price in integer cents. Rendered as
    /// `data-unit-price="{cents}"` on the tile root wrapper. The client-computed
    /// running total (SelectionPanel, Phase 256) reads this attribute because
    /// `price` is a display string that cannot be parsed. Both fields are
    /// expected to agree — the Phase 257 projector emits both from one source.
    /// The runtime treats a missing attribute as 0 cents. Integer cents only —
    /// never float (see PITFALLS.md).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_cents: Option<u64>,
}

/// Props for the TileGrid builtin — a touch-first, responsive tile grid
/// whose Tile children iterate via the `$each` contract (Phase 257 target).
/// Renderer + registration land in Phase 256; this is the contract only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TileGridProps {
    /// JSON pointer to the product array the grid iterates over via `$each`.
    pub data_path: String,
    /// The HTML `id` of the `Form` element that owns this grid's hidden
    /// inputs. Both the grid and its paired SelectionPanel must be descendants
    /// of that form (D-11): the selection runtime scopes its queries and its
    /// input-event listener to `document.getElementById(form_id)`, so tiles
    /// placed outside the form neither submit with it nor appear in the panel.
    /// Emitted as `data-selection-form="{form_id}"` on the grid root — the
    /// same attribute the SelectionPanel root carries — so the pairing is
    /// introspectable in markup.
    pub form_id: String,
    /// JSON pointer to a category string array for the integrated category strip.
    /// Absent → no category strip is rendered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub categories_path: Option<String>,
    /// Override for the base-viewport grid column count (Phase 256 render default is 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<u8>,
    /// Enables the client-side text-search input (Phase 255 `setupFilters`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<bool>,
    /// Placeholder text for the search input. Render default is "Search"
    /// (neutral English — this crate is project-agnostic). Pass
    /// `search_placeholder: "Cerca"` or any locale string from the consumer.
    /// Ignored when `search` is absent/false (no input is rendered).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_placeholder: Option<String>,
    /// Label for the integrated category strip's "show all" tab. Render
    /// default is "All" (neutral English — this crate is project-agnostic).
    /// Pass `all_label: "Tutte"` or any locale string from the consumer.
    /// Ignored when `categories_path` is absent (no strip is rendered).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all_label: Option<String>,
}

/// Props for the SelectionPanel builtin — a server-rendered selection summary that
/// pins and scrolls internally under `fill_viewport`. Renderer lands in Phase 256.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SelectionPanelProps {
    /// Scope isolator matching the paired TileGrid `form_id`.
    pub form_id: String,
    /// Heading text shown in the EmptyState when the panel has no line items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty_message: Option<String>,
    /// Optional supplementary body text shown below `empty_message` in the
    /// EmptyState. Omit when no actionable guidance exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty_body: Option<String>,
    /// Optional currency symbol (e.g. "€") emitted as `data-selection-currency`
    /// on the running-total element. Neutral default is no symbol — the runtime
    /// formats the integer-cents total with two decimals and a "," separator and
    /// prepends this symbol only when present. No locale tables; display only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Label for the running-total row. Render default is "Total" (neutral
    /// English — this crate is project-agnostic). Pass `total_label:
    /// "Totale"` or any locale string from the consumer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_label: Option<String>,
}

/// Props for the FilterTabs builtin (standalone builtin, operator-locked).
/// Filters visible tiles client-side via `data-filter-tokens` matching
/// (Phase 255 runtime). Renderer lands in Phase 256.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FilterTabsProps {
    /// Category labels rendered as filter tabs. May be `$data`-bound at render
    /// time. Matching against `data-filter-tokens` must normalize spaces to
    /// hyphens, mirroring `TileProps::categories` rendering.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
    /// Label for the "show all" tab. Phase 256 render default is "All" (neutral
    /// English — this crate is project-agnostic). Pass `all_label: "Tutte"` or
    /// any locale string from the consumer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all_label: Option<String>,
}

/// Props for the QuantityStepper POS builtin — a reusable +/- stepper driving a
/// hidden input on the Tile contract. Renderer lands in Phase 256.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct QuantityStepperProps {
    /// Name of the hidden input this stepper increments/decrements.
    pub field: String,
    /// Lower bound (Phase 256 render default is 0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<u32>,
    /// Upper bound; unbounded when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<u32>,
    /// Increment size (Phase 256 render default is 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<u32>,
}

/// Input mode for the Numpad — governs which characters the keypad accepts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum NumpadMode {
    /// Integer entry only.
    #[default]
    Quantity,
    /// Two-decimal-place monetary entry.
    Price,
}

/// Props for the Numpad POS builtin — a tap-surface numeric keypad that writes to a
/// target field and NEVER renders a native input (so the software keyboard is never
/// triggered). Renderer lands in Phase 256.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NumpadProps {
    /// Name of the input this numpad writes into.
    pub target_field: String,
    /// Entry mode (quantity | price). Defaults to quantity.
    #[serde(default)]
    pub mode: NumpadMode,
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
    fn schema_for_action_item_generates() {
        assert_schema_nonempty_object::<ActionItem>("ActionItem");
    }

    #[test]
    fn schema_for_action_group_props_generates() {
        assert_schema_nonempty_object::<ActionGroupProps>("ActionGroupProps");
    }

    #[test]
    fn schema_for_dropdown_menu_action_generates() {
        assert_schema_nonempty_object::<DropdownMenuAction>("DropdownMenuAction");
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
    fn schema_for_tile_props_generates() {
        assert_schema_nonempty_object::<TileProps>("TileProps");
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
            appearance: CardAppearance::Bordered,
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
            appearance: CardAppearance::Bordered,
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
            appearance: CardAppearance::Bordered,
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
            appearance: CardAppearance::Bordered,
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
            appearance: CardAppearance::Bordered,
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
            appearance: CardAppearance::Bordered,
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

    #[test]
    fn schema_for_tile_grid_props_generates() {
        assert_schema_nonempty_object::<TileGridProps>("TileGridProps");
    }

    #[test]
    fn schema_for_selection_panel_props_generates() {
        assert_schema_nonempty_object::<SelectionPanelProps>("SelectionPanelProps");
    }

    #[test]
    fn schema_for_filter_tabs_props_generates() {
        assert_schema_nonempty_object::<FilterTabsProps>("FilterTabsProps");
    }

    #[test]
    fn schema_for_quantity_stepper_props_generates() {
        assert_schema_nonempty_object::<QuantityStepperProps>("QuantityStepperProps");
    }

    #[test]
    fn schema_for_numpad_props_generates() {
        assert_schema_nonempty_object::<NumpadProps>("NumpadProps");
    }
}

#[cfg(test)]
mod strum_tests {
    use super::*;

    use strum::VariantArray;

    /// Assert AsRef<str> matches serde JSON wire format for EVERY variant of
    /// the canonical `Variant`, `Tone`, and `Size` enums.
    /// Threat T-162-08-01: strum and serde must agree on every snake_case string.
    /// The variant lists come from `strum::VariantArray`, so omitting a
    /// variant (the pre-251 `BadgeVariant::Warning` gap) is structurally
    /// impossible.
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
        check(Variant::VARIANTS, "Variant");
        check(Tone::VARIANTS, "Tone");
        check(Size::VARIANTS, "Size");
    }

    /// Pin the canonical value-set sizes so an added/removed variant is a
    /// conscious decision (the D-19 schema guard in Plan 03 walks the catalog;
    /// this is the enum-side anchor).
    #[test]
    fn canonical_enums_have_expected_variant_counts() {
        assert_eq!(Variant::VARIANTS.len(), 5);
        assert_eq!(Tone::VARIANTS.len(), 4);
        assert_eq!(Size::VARIANTS.len(), 3);
    }

    #[test]
    fn tone_as_ref_str_matches_wire_format() {
        assert_eq!(Tone::Neutral.as_ref(), "neutral");
        assert_eq!(Tone::Success.as_ref(), "success");
        assert_eq!(Tone::Warning.as_ref(), "warning");
        assert_eq!(Tone::Destructive.as_ref(), "destructive");
    }
}

#[cfg(test)]
mod canonical_enum_tests {
    use super::*;

    // ── Defaults ────────────────────────────────────────────────────────

    #[test]
    fn variant_default_is_primary() {
        assert_eq!(Variant::default(), Variant::Primary);
    }

    #[test]
    fn tone_default_is_neutral() {
        assert_eq!(Tone::default(), Tone::Neutral);
    }

    #[test]
    fn size_default_is_md() {
        assert_eq!(Size::default(), Size::Md);
    }

    // ── snake_case wire format (serialize + deserialize + roundtrip) ───

    #[test]
    fn variant_serde_snake_case_roundtrip() {
        use strum::VariantArray;
        for v in Variant::VARIANTS {
            let json = serde_json::to_value(v).unwrap();
            assert_eq!(json, serde_json::json!(v.as_ref()));
            let back: Variant = serde_json::from_value(json).unwrap();
            assert_eq!(back, *v);
        }
        assert_eq!(
            serde_json::from_str::<Variant>("\"primary\"").unwrap(),
            Variant::Primary
        );
    }

    #[test]
    fn tone_serde_snake_case_roundtrip() {
        use strum::VariantArray;
        for t in Tone::VARIANTS {
            let json = serde_json::to_value(t).unwrap();
            assert_eq!(json, serde_json::json!(t.as_ref()));
            let back: Tone = serde_json::from_value(json).unwrap();
            assert_eq!(back, *t);
        }
    }

    #[test]
    fn size_serde_snake_case_roundtrip() {
        use strum::VariantArray;
        for s in Size::VARIANTS {
            let json = serde_json::to_value(s).unwrap();
            assert_eq!(json, serde_json::json!(s.as_ref()));
            let back: Size = serde_json::from_value(json).unwrap();
            assert_eq!(back, *s);
        }
        assert!(serde_json::from_str::<Size>("\"md\"").is_ok());
        assert!(serde_json::from_str::<Size>("\"sm\"").is_ok());
        assert!(serde_json::from_str::<Size>("\"lg\"").is_ok());
    }

    // ── Retired values are rejected at parse (D-12: no serde aliases) ──

    #[test]
    fn retired_size_values_are_rejected() {
        assert!(
            serde_json::from_str::<Size>("\"xs\"").is_err(),
            "size 'xs' was retired (migrate to 'sm')"
        );
        assert!(
            serde_json::from_str::<Size>("\"default\"").is_err(),
            "size 'default' was retired (migrate to 'md')"
        );
    }

    #[test]
    fn retired_variant_values_are_rejected() {
        assert!(
            serde_json::from_str::<Variant>("\"default\"").is_err(),
            "variant 'default' was retired (migrate to 'primary')"
        );
        assert!(
            serde_json::from_str::<Variant>("\"link\"").is_err(),
            "variant 'link' was removed (migrate to 'ghost')"
        );
    }

    #[test]
    fn retired_tone_values_are_rejected() {
        assert!(
            serde_json::from_str::<Tone>("\"info\"").is_err(),
            "tone 'info' was retired (migrate to 'neutral')"
        );
        assert!(
            serde_json::from_str::<Tone>("\"error\"").is_err(),
            "tone 'error' was retired (migrate to 'destructive')"
        );
    }

    #[test]
    fn button_spec_with_link_variant_fails_to_decode() {
        let v = serde_json::json!({"variant": "link", "label": "x"});
        assert!(
            serde_json::from_value::<ButtonProps>(v).is_err(),
            "Button variant 'link' must fail decode (migrate to 'ghost')"
        );
    }

    // ── Props defaults ──────────────────────────────────────────────────

    #[test]
    fn button_props_defaults_to_primary_md() {
        let v = serde_json::json!({"label": "x"});
        let p: ButtonProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.variant, Variant::Primary);
        assert_eq!(p.size, Size::Md);
    }

    #[test]
    fn alert_props_without_tone_defaults_to_neutral() {
        let v = serde_json::json!({"message": "x"});
        let p: AlertProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.tone, Tone::Neutral);
    }

    #[test]
    fn alert_props_with_tone_neutral_decodes() {
        let v = serde_json::json!({"message": "x", "tone": "neutral"});
        let p: AlertProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.tone, Tone::Neutral);
    }

    #[test]
    fn badge_props_without_tone_defaults_to_neutral() {
        let v = serde_json::json!({"label": "x"});
        let p: BadgeProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.tone, Tone::Neutral);
    }

    #[test]
    fn toast_props_without_tone_defaults_to_neutral() {
        let v = serde_json::json!({"message": "x"});
        let p: ToastProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.tone, Tone::Neutral);
    }

    #[test]
    fn action_card_props_with_success_tone_decodes() {
        let v = serde_json::json!({"title": "x", "description": "y", "tone": "success"});
        let p: ActionCardProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.tone, Tone::Success);
    }

    #[test]
    fn stat_card_props_without_tone_defaults_to_neutral() {
        let v = serde_json::json!({"label": "x", "value": "1"});
        let p: StatCardProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.tone, Tone::Neutral);
    }

    #[test]
    fn stat_card_props_roundtrip_preserves_tone() {
        let v = serde_json::json!({"label": "x", "value": "1", "tone": "warning"});
        let p: StatCardProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.tone, Tone::Warning);
        let j = serde_json::to_value(&p).unwrap();
        let back: StatCardProps = serde_json::from_value(j).unwrap();
        assert_eq!(back.tone, Tone::Warning);
    }
}

#[cfg(test)]
mod card_appearance_tests {
    use super::*;

    #[test]
    fn card_appearance_default_is_bordered() {
        assert_eq!(CardAppearance::default(), CardAppearance::Bordered);
    }

    #[test]
    fn card_appearance_serializes_snake_case() {
        assert_eq!(
            serde_json::to_value(CardAppearance::Bordered).unwrap(),
            serde_json::json!("bordered")
        );
        assert_eq!(
            serde_json::to_value(CardAppearance::Elevated).unwrap(),
            serde_json::json!("elevated")
        );
    }

    #[test]
    fn card_appearance_deserializes_snake_case() {
        assert_eq!(
            serde_json::from_value::<CardAppearance>(serde_json::json!("bordered")).unwrap(),
            CardAppearance::Bordered
        );
        assert_eq!(
            serde_json::from_value::<CardAppearance>(serde_json::json!("elevated")).unwrap(),
            CardAppearance::Elevated
        );
    }

    #[test]
    fn card_props_without_appearance_defaults_to_bordered() {
        let v = serde_json::json!({"title": "x"});
        let p: CardProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.appearance, CardAppearance::Bordered);
    }

    #[test]
    fn card_props_with_elevated_appearance() {
        let v = serde_json::json!({"title": "x", "appearance": "elevated"});
        let p: CardProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.appearance, CardAppearance::Elevated);
    }

    #[test]
    fn card_props_roundtrip_preserves_appearance() {
        let p = CardProps {
            title: "x".into(),
            description: None,
            subtitle: None,
            badge: None,
            max_width: None,
            footer: vec![],
            appearance: CardAppearance::Elevated,
        };
        let j = serde_json::to_value(&p).unwrap();
        let back: CardProps = serde_json::from_value(j).unwrap();
        assert_eq!(back.appearance, CardAppearance::Elevated);
    }
}

#[cfg(test)]
mod kanban_board_props_tests {
    use super::*;

    #[test]
    fn kanban_board_props_serde_static_columns() {
        let v = serde_json::json!({
            "columns": [{"title": "To Do", "id": "todo", "count": 0}]
        });
        let p: KanbanBoardProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.columns.len(), 1);
        assert!(p.items_path.is_none());
        assert!(p.group_by.is_none());
    }

    #[test]
    fn kanban_board_props_serde_data_bound() {
        let v = serde_json::json!({
            "columns": [{"title": "Open", "id": "open"}],
            "items_path": "/data/order",
            "group_by": "status",
            "card_title_key": "name"
        });
        let p: KanbanBoardProps = serde_json::from_value(v).unwrap();
        assert_eq!(p.columns.len(), 1);
        assert_eq!(p.items_path.as_deref(), Some("/data/order"));
        assert_eq!(p.group_by.as_deref(), Some("status"));
        assert_eq!(p.card_title_key.as_deref(), Some("name"));
    }

    #[test]
    fn kanban_board_props_serde_neither() {
        let v = serde_json::json!({});
        let p: KanbanBoardProps = serde_json::from_value(v).unwrap();
        assert!(p.columns.is_empty());
        assert!(p.items_path.is_none());
        assert!(p.group_by.is_none());
    }

    #[test]
    fn kanban_board_props_empty_columns_skipped_on_serialize() {
        let p = KanbanBoardProps {
            columns: vec![],
            items_path: Some("/data/order".into()),
            group_by: Some("status".into()),
            card_title_key: None,
            card_description_key: None,
            row_actions: None,
            row_key: None,
            mobile_default_column: None,
            empty_label: None,
        };
        let j = serde_json::to_value(&p).unwrap();
        assert!(
            j.get("columns").is_none(),
            "empty columns must be skipped, got: {j}"
        );
        assert_eq!(
            j.get("items_path").and_then(|v| v.as_str()),
            Some("/data/order")
        );
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

#[cfg(test)]
mod tile_contract_tests {
    //! Backward-compatibility and field-contract tests for TileProps and GridProps.
    //! RED-phase guard: these tests are written before the new fields exist and must
    //! fail to compile until the GREEN-phase fields are added.

    use super::*;

    /// Legacy Tile JSON (no new fields) must deserialize cleanly and
    /// re-serialize without emitting the new keys — SC-1 / D-04 backward-compat.
    #[test]
    fn tile_legacy_json_round_trips_unchanged() {
        let json = r#"{"item_id":"p1","name":"Widget","price":"€10,00","field":"qty_p1"}"#;
        let tile: TileProps = serde_json::from_str(json).expect("legacy json must deserialize");
        assert!(
            tile.categories.is_empty(),
            "categories must default to empty vec"
        );
        assert!(tile.image_url.is_none(), "image_url must default to None");
        assert!(tile.color.is_none(), "color must default to None");
        assert!(
            tile.stock_badge.is_none(),
            "stock_badge must default to None"
        );
        let serialized = serde_json::to_string(&tile).expect("must serialize");
        assert!(
            !serialized.contains("categories"),
            "re-serialized must not contain 'categories'; got: {serialized}"
        );
        assert!(
            !serialized.contains("image_url"),
            "re-serialized must not contain 'image_url'; got: {serialized}"
        );
        assert!(
            !serialized.contains("color"),
            "re-serialized must not contain 'color'; got: {serialized}"
        );
        assert!(
            !serialized.contains("stock_badge"),
            "re-serialized must not contain 'stock_badge'; got: {serialized}"
        );
    }

    /// TileProps with categories set must re-serialize with the categories key present.
    #[test]
    fn tile_with_categories_serializes() {
        let tile = TileProps {
            item_id: "p2".to_string(),
            name: "Espresso".to_string(),
            price: "\u{20ac}2,00".to_string(),
            field: "qty_p2".to_string(),
            default_quantity: None,
            categories: vec!["drinks".to_string(), "food".to_string()],
            image_url: None,
            color: None,
            stock_badge: None,
            price_cents: None,
        };
        let serialized = serde_json::to_string(&tile).expect("must serialize");
        assert!(
            serialized.contains(r#""categories":["drinks","food"]"#),
            "serialized must contain categories array; got: {serialized}"
        );
    }

    /// GridProps with empty row_weights omits the key; with weights set, round-trips.
    #[test]
    fn grid_props_row_weights_round_trips() {
        // Empty row_weights must be skipped in serialization.
        let default_grid: GridProps = serde_json::from_value(serde_json::json!({}))
            .expect("must deserialize default GridProps");
        let json = serde_json::to_string(&default_grid).expect("must serialize");
        assert!(
            !json.contains("row_weights"),
            "empty row_weights must be skipped in serialization; got: {json}"
        );

        // Non-empty row_weights must appear in serialization and round-trip.
        let with_weights: GridProps =
            serde_json::from_value(serde_json::json!({"row_weights": [2, 1]}))
                .expect("must deserialize GridProps with row_weights");
        let json2 = serde_json::to_string(&with_weights).expect("must serialize with weights");
        assert!(
            json2.contains(r#""row_weights":[2,1]"#),
            "row_weights must appear in serialization; got: {json2}"
        );
        let parsed: GridProps =
            serde_json::from_str(&json2).expect("must deserialize from serialized");
        assert_eq!(
            parsed.row_weights,
            vec![2u8, 1u8],
            "row_weights must round-trip unchanged"
        );
    }

    /// TileProps.price_cents round-trips; absent price_cents is skipped.
    #[test]
    fn tile_props_price_cents_round_trips() {
        // Absent price_cents must not appear in serialization.
        let no_price: TileProps = serde_json::from_str(
            r#"{"item_id":"p1","name":"Coffee","price":"€2,00","field":"qty_p1"}"#,
        )
        .expect("must deserialize");
        assert!(
            no_price.price_cents.is_none(),
            "price_cents must default to None"
        );
        let json = serde_json::to_string(&no_price).expect("must serialize");
        assert!(
            !json.contains("price_cents"),
            "absent price_cents must be skipped; got: {json}"
        );

        // price_cents: Some(250) must round-trip.
        let with_price: TileProps =
            serde_json::from_str(r#"{"item_id":"p1","name":"Coffee","price":"€2,50","field":"qty_p1","price_cents":250}"#)
                .expect("must deserialize with price_cents");
        assert_eq!(
            with_price.price_cents,
            Some(250u64),
            "price_cents must round-trip"
        );
        let json2 = serde_json::to_string(&with_price).expect("must serialize");
        assert!(
            json2.contains(r#""price_cents":250"#),
            "price_cents must appear in serialization; got: {json2}"
        );
        let parsed: TileProps =
            serde_json::from_str(&json2).expect("must deserialize from serialized");
        assert_eq!(
            parsed.price_cents,
            Some(250u64),
            "price_cents must round-trip unchanged"
        );
    }

    /// TileProps.color as Option<Tone> round-trips; arbitrary string fails.
    #[test]
    fn tile_props_color_tone_round_trips_and_rejects_unknown() {
        // color: Some(Tone::Success) must round-trip.
        let with_color: TileProps = serde_json::from_str(
            r#"{"item_id":"p1","name":"Tea","price":"€1,00","field":"qty_p1","color":"success"}"#,
        )
        .expect("must deserialize color:success");
        assert_eq!(
            with_color.color,
            Some(Tone::Success),
            "color:success must deserialize to Tone::Success"
        );
        let json = serde_json::to_string(&with_color).expect("must serialize");
        assert!(
            json.contains(r#""color":"success""#),
            "Tone::Success must serialize as \"success\"; got: {json}"
        );
        let parsed: TileProps =
            serde_json::from_str(&json).expect("must deserialize from serialized");
        assert_eq!(
            parsed.color,
            Some(Tone::Success),
            "color must round-trip unchanged"
        );

        // Arbitrary string "blue" must fail to deserialize (enum-enforced).
        let result: Result<TileProps, _> = serde_json::from_str(
            r#"{"item_id":"p1","name":"Tea","price":"€1,00","field":"qty_p1","color":"blue"}"#,
        );
        assert!(
            result.is_err(),
            "color:\"blue\" must fail to deserialize — Tone enum enforced"
        );
    }

    /// SelectionPanelProps.currency round-trips; absent currency is skipped.
    #[test]
    fn selection_panel_props_currency_round_trips() {
        // Absent currency must not appear in serialization.
        let no_currency: SelectionPanelProps =
            serde_json::from_str(r#"{"form_id":"order-form"}"#).expect("must deserialize");
        assert!(
            no_currency.currency.is_none(),
            "currency must default to None"
        );
        let json = serde_json::to_string(&no_currency).expect("must serialize");
        assert!(
            !json.contains("currency"),
            "absent currency must be skipped; got: {json}"
        );

        // currency: Some("€") must round-trip.
        let with_currency: SelectionPanelProps =
            serde_json::from_str(r#"{"form_id":"order-form","currency":"€"}"#)
                .expect("must deserialize with currency");
        assert_eq!(
            with_currency.currency,
            Some("€".to_string()),
            "currency must round-trip"
        );
        let json2 = serde_json::to_string(&with_currency).expect("must serialize");
        assert!(
            json2.contains(r#""currency":"€""#),
            "currency must appear in serialization; got: {json2}"
        );
    }
}
