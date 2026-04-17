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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
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
}

/// Alert visual variants.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlertVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// Badge visual variants (aligned to shadcn/ui).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
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

/// Props for Card component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CardProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<FormMaxWidth>,
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
    pub items: Vec<DescriptionItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<u8>,
}

/// A single tab within a Tabs component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Tab {
    pub value: String,
    pub label: String,
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

/// Toast visual variants.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
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
}

/// Props for ButtonGroup component -- horizontal button row with consistent gap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ButtonGroupProps {
    /// Gap between buttons. Defaults to small spacing.
    #[serde(default)]
    pub gap: GapSize,
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
}

/// Props for a single column in a KanbanBoard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct KanbanColumnProps {
    pub id: String,
    pub title: String,
    pub count: u32,
}

/// Props for KanbanBoard component — horizontal scrollable columns on desktop, tab-based on mobile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct KanbanBoardProps {
    pub columns: Vec<KanbanColumnProps>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mobile_default_column: Option<String>,
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
