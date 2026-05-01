//! JSON-UI component catalog tool — structured reference of all built-in
//! components plus plugin components (Map, etc.).
//
// CATALOG IS HAND-MAINTAINED FROM ferro_json_ui::component::Component.
// When a new Component variant is added to ferro-json-ui, this catalog MUST be updated.
// TODO(ferro): derive this at compile time via a schemars-based introspection pass.

use serde::Serialize;

/// Complete catalog of JSON-UI components with builder and action API.
#[derive(Debug, Serialize)]
pub struct JsonUiCatalog {
    pub components: Vec<CatalogComponent>,
    pub plugin_components: Vec<CatalogComponent>,
    pub builder_api: String,
    pub action_api: String,
}

/// A single component in the catalog.
#[derive(Debug, Serialize)]
pub struct CatalogComponent {
    pub name: String,
    pub description: String,
    pub props: Vec<PropInfo>,
    pub variants: Option<Vec<String>>,
}

/// A prop on a component.
#[derive(Debug, Serialize)]
pub struct PropInfo {
    pub name: String,
    pub type_name: String,
    pub required: bool,
    pub description: String,
}

/// Execute the JSON-UI catalog tool.
///
/// When `component` is Some, returns only the matching component (case-insensitive).
/// When None, returns the full catalog.
pub fn execute(component: Option<&str>) -> JsonUiCatalog {
    let all = build_catalog();
    let all_plugins = build_plugin_catalog();

    let (components, plugin_components) = match component {
        Some(name) => {
            let lower = name.to_lowercase();
            let filtered: Vec<_> = all
                .into_iter()
                .filter(|c| c.name.to_lowercase() == lower)
                .collect();
            let filtered_plugins: Vec<_> = all_plugins
                .into_iter()
                .filter(|c| c.name.to_lowercase() == lower)
                .collect();
            (filtered, filtered_plugins)
        }
        None => (all, all_plugins),
    };

    JsonUiCatalog {
        components,
        plugin_components,
        builder_api: BUILDER_API.to_string(),
        action_api: ACTION_API.to_string(),
    }
}

const BUILDER_API: &str = "\
JsonUiView::new() -> JsonUiView
  .title(impl Into<String>) -> Self
  .layout(impl Into<String>) -> Self
  .data(serde_json::Value) -> Self
  .errors(HashMap<String, Vec<String>>) -> Self
  .component(ComponentNode) -> Self
  .components(Vec<ComponentNode>) -> Self

ComponentNode { key: String, component: Component, action: Option<Action>, visibility: Option<Visibility> }
  - key: Unique identifier for this node in the view tree
  - component: One of the Component enum variants
  - action: Optional Action binding (click/submit handler)
  - visibility: Optional Visibility rule (show/hide based on data path)";

const ACTION_API: &str = "\
Action::new(handler) -> Action (POST)
Action::get(handler) -> Action (GET)
Action::delete(handler) -> Action (DELETE)
  .method(HttpMethod) -> Self
  .confirm(title) -> Self (default dialog)
  .confirm_danger(title) -> Self (danger dialog)
  .on_success(ActionOutcome) -> Self
  .on_error(ActionOutcome) -> Self

Handler format: \"controller.method\" (e.g., \"users.store\")

ActionOutcome variants:
  Redirect { url: String }
  ShowErrors
  Refresh
  Notify { message: String, variant: NotifyVariant }

ConfirmDialog { title: String, message: Option<String>, variant: DialogVariant (default|danger) }";

fn build_catalog() -> Vec<CatalogComponent> {
    vec![
        CatalogComponent {
            name: "Text".to_string(),
            description: "Renders text content with semantic HTML element selection.".to_string(),
            props: vec![
                prop("content", "String", true, "Text content to display"),
                prop(
                    "element",
                    "TextElement",
                    false,
                    "HTML element: h1, h2, h3, span, div, section, p (default: p)",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Button".to_string(),
            description: "Interactive button with visual variants and optional icon.".to_string(),
            props: vec![
                prop("label", "String", true, "Button label text"),
                prop(
                    "variant",
                    "ButtonVariant",
                    false,
                    "Visual style (default: default)",
                ),
                prop("size", "Size", false, "Button size (default: default)"),
                prop(
                    "disabled",
                    "Option<bool>",
                    false,
                    "Whether button is disabled",
                ),
                prop("icon", "Option<String>", false, "Icon name"),
                prop(
                    "icon_position",
                    "Option<IconPosition>",
                    false,
                    "Icon placement: left or right (default: left)",
                ),
            ],
            variants: Some(vec![
                "default".to_string(),
                "secondary".to_string(),
                "destructive".to_string(),
                "outline".to_string(),
                "ghost".to_string(),
                "link".to_string(),
            ]),
        },
        CatalogComponent {
            name: "Card".to_string(),
            description: "Container with title, optional description, children, and footer."
                .to_string(),
            props: vec![
                prop("title", "String", true, "Card title"),
                prop(
                    "description",
                    "Option<String>",
                    false,
                    "Card description below title",
                ),
                prop(
                    "children",
                    "Vec<ComponentNode>",
                    false,
                    "Nested components inside the card body",
                ),
                prop(
                    "footer",
                    "Vec<ComponentNode>",
                    false,
                    "Components in the card footer",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Table".to_string(),
            description: "Data table with columns, row actions, sorting, and empty state."
                .to_string(),
            props: vec![
                prop(
                    "columns",
                    "Vec<Column>",
                    true,
                    "Column definitions: { key, label, format? }",
                ),
                prop(
                    "data_path",
                    "String",
                    true,
                    "Data path to the array of rows (e.g., \"/data/users\")",
                ),
                prop(
                    "row_actions",
                    "Option<Vec<Action>>",
                    false,
                    "Actions available for each row",
                ),
                prop(
                    "empty_message",
                    "Option<String>",
                    false,
                    "Message when table has no data",
                ),
                prop(
                    "sortable",
                    "Option<bool>",
                    false,
                    "Whether columns are sortable",
                ),
                prop(
                    "sort_column",
                    "Option<String>",
                    false,
                    "Currently sorted column key",
                ),
                prop(
                    "sort_direction",
                    "Option<SortDirection>",
                    false,
                    "Sort direction: asc or desc",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Form".to_string(),
            description: "Form container with action binding and field components.".to_string(),
            props: vec![
                prop("action", "Action", true, "Action to execute on form submit"),
                prop(
                    "fields",
                    "Vec<ComponentNode>",
                    true,
                    "Form field components (Input, Select, Checkbox, etc.)",
                ),
                prop(
                    "method",
                    "Option<HttpMethod>",
                    false,
                    "HTTP method override (GET, POST, PUT, PATCH, DELETE)",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "DetailForm".to_string(),
            description: "Split-mode detail page with inline edit: View renders a <dl> + 'Modifica' link; Edit wraps the same <dl> in a <form> with 'Salva'/'Annulla' actions. Mode is URL-driven (?mode=edit); server-side only (no JS). Authoring rule (Option A): when DetailField.input is an Input/Select/Textarea/Checkbox/Switch, the caller MUST set its label to \"\" — the <dt> provides the visible label. DetailForm does not mutate caller-supplied props; the caller should also set aria-label on each input derived from the field's label so screen readers retain context.".to_string(),
            props: vec![
                prop(
                    "mode",
                    "EditMode",
                    true,
                    "View (default) or Edit — typically set via EditMode::from_query(req.query(\"mode\").as_deref())",
                ),
                prop(
                    "action",
                    "Action",
                    true,
                    "Form submit target used in Edit mode; resolver populates action.url from action.handler",
                ),
                prop(
                    "fields",
                    "Vec<DetailField>",
                    true,
                    "Rows: { label, value, input: ComponentNode }",
                ),
                prop(
                    "edit_url",
                    "String",
                    true,
                    "Href for the 'Modifica' link (View mode). Emitted verbatim after html_escape; NOT resolved by the route registry",
                ),
                prop(
                    "cancel_url",
                    "String",
                    true,
                    "Href for the 'Annulla' link (Edit mode). Emitted verbatim after html_escape; NOT resolved by the route registry",
                ),
                prop(
                    "edit_label",
                    "Option<String>",
                    false,
                    "Override for the default 'Modifica' label",
                ),
                prop(
                    "save_label",
                    "Option<String>",
                    false,
                    "Override for the default 'Salva' label",
                ),
                prop(
                    "cancel_label",
                    "Option<String>",
                    false,
                    "Override for the default 'Annulla' label",
                ),
                prop(
                    "method",
                    "Option<HttpMethod>",
                    false,
                    "HTTP method override for the form (else uses action.method); PUT/PATCH/DELETE auto-emit <input type=\"hidden\" name=\"_method\"> spoofing",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "KeyValueEditor".to_string(),
            description: "Dynamic key/value editor backed by a hidden JSON field. Renders a list of key/value row inputs plus an 'Add row' button; the runtime JS keeps the hidden field in sync on every add/delete/input. When allow_custom_keys is true, the key input is a text field with a <datalist> from suggested_keys; when false, the key input is a <select> restricted to suggested_keys. Last-write-wins on duplicate keys.".to_string(),
            props: vec![
                prop(
                    "field",
                    "String",
                    true,
                    "Name of the hidden input that receives the serialized JSON object",
                ),
                prop(
                    "label",
                    "Option<String>",
                    false,
                    "Optional visible label above the editor block",
                ),
                prop(
                    "suggested_keys",
                    "Vec<String>",
                    false,
                    "Keys offered as suggestions via <datalist> (custom mode) or <select> options (restricted mode)",
                ),
                prop(
                    "allow_custom_keys",
                    "bool",
                    false,
                    "If true (default), keys can be any text with suggestions; if false, keys are restricted to suggested_keys",
                ),
                prop(
                    "data_path",
                    "Option<String>",
                    false,
                    "JSON pointer path whose resolved object seeds the initial rows",
                ),
                prop(
                    "error",
                    "Option<String>",
                    false,
                    "Validation error message rendered below the editor",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "RichTextEditor".to_string(),
            description: "Rich-text editor backed by Quill 2.0.3 (Snow theme, jsDelivr CDN, SHA-384 SRI-pinned). Emits two hidden inputs on submit: `{name}_delta` (Delta JSON, canonical) and `{name}_html` (sanitized HTML). The `formats` whitelist constrains both the Quill toolbar (init time) and the HTML allowlist (submit time) — image/video/HTML-paste paths are not reachable through the prop surface. Quill JS+CSS load once per page, deduplicated across multiple editor instances. Multiple editors in the same form work without ID collisions; each writes its own pair of hidden inputs.".to_string(),
            props: vec![
                prop(
                    "name",
                    "String",
                    true,
                    "Base form field name. The IIFE emits {name}_delta and {name}_html on submit.",
                ),
                prop(
                    "value",
                    "Option<String>",
                    false,
                    "Initial editor content. Auto-detected: parses as JSON Delta when the string has an `ops` array; otherwise loaded as HTML filtered by the formats allowlist.",
                ),
                prop(
                    "formats",
                    "Vec<String>",
                    false,
                    "Toolbar/allowlist whitelist. Drives both Quill's toolbar config and the HTML post-process. Default: [\"bold\",\"italic\",\"underline\",\"list\",\"header\",\"link\"]",
                ),
                prop(
                    "placeholder",
                    "Option<String>",
                    false,
                    "Placeholder shown when the editor is empty.",
                ),
                prop(
                    "theme",
                    "String",
                    false,
                    "Quill theme. Only \"snow\" supported in v1; defaults to \"snow\".",
                ),
                prop(
                    "label",
                    "Option<String>",
                    false,
                    "Optional label rendered above the editor host.",
                ),
                prop(
                    "error",
                    "Option<String>",
                    false,
                    "Validation error rendered below the editor with destructive token styling.",
                ),
                prop(
                    "data_path",
                    "Option<String>",
                    false,
                    "JSON pointer for pre-fill at render time. Overridden by explicit `value` if both set.",
                ),
                prop(
                    "required",
                    "Option<bool>",
                    false,
                    "When true, the IIFE prevents submission when editor content is empty (after trim).",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Input".to_string(),
            description: "Text input field with type variants, validation error, and data binding."
                .to_string(),
            props: vec![
                prop("field", "String", true, "Form field name for data binding"),
                prop("label", "String", true, "Input label text"),
                prop(
                    "input_type",
                    "InputType",
                    false,
                    "Input type (default: text)",
                ),
                prop("placeholder", "Option<String>", false, "Placeholder text"),
                prop(
                    "required",
                    "Option<bool>",
                    false,
                    "Whether field is required",
                ),
                prop(
                    "disabled",
                    "Option<bool>",
                    false,
                    "Whether field is disabled",
                ),
                prop("error", "Option<String>", false, "Validation error message"),
                prop(
                    "description",
                    "Option<String>",
                    false,
                    "Help text below the input",
                ),
                prop("default_value", "Option<String>", false, "Pre-filled value"),
                prop(
                    "data_path",
                    "Option<String>",
                    false,
                    "Data path for pre-filling from handler data",
                ),
            ],
            variants: Some(vec![
                "text".to_string(),
                "email".to_string(),
                "password".to_string(),
                "number".to_string(),
                "textarea".to_string(),
                "hidden".to_string(),
                "date".to_string(),
                "time".to_string(),
                "url".to_string(),
                "tel".to_string(),
                "search".to_string(),
            ]),
        },
        CatalogComponent {
            name: "Select".to_string(),
            description: "Dropdown select field with options, validation error, and data binding."
                .to_string(),
            props: vec![
                prop("field", "String", true, "Form field name for data binding"),
                prop("label", "String", true, "Select label text"),
                prop(
                    "options",
                    "Vec<SelectOption>",
                    true,
                    "Options: { value, label }",
                ),
                prop("placeholder", "Option<String>", false, "Placeholder text"),
                prop(
                    "required",
                    "Option<bool>",
                    false,
                    "Whether field is required",
                ),
                prop(
                    "disabled",
                    "Option<bool>",
                    false,
                    "Whether field is disabled",
                ),
                prop("error", "Option<String>", false, "Validation error message"),
                prop(
                    "description",
                    "Option<String>",
                    false,
                    "Help text below the select",
                ),
                prop(
                    "default_value",
                    "Option<String>",
                    false,
                    "Pre-selected value",
                ),
                prop(
                    "data_path",
                    "Option<String>",
                    false,
                    "Data path for pre-filling from handler data",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Alert".to_string(),
            description: "Alert message with variant-based styling.".to_string(),
            props: vec![
                prop("message", "String", true, "Alert message content"),
                prop(
                    "variant",
                    "AlertVariant",
                    false,
                    "Visual style (default: info)",
                ),
                prop("title", "Option<String>", false, "Alert title"),
            ],
            variants: Some(vec![
                "info".to_string(),
                "success".to_string(),
                "warning".to_string(),
                "error".to_string(),
            ]),
        },
        CatalogComponent {
            name: "Badge".to_string(),
            description: "Small label with variant-based styling.".to_string(),
            props: vec![
                prop("label", "String", true, "Badge text"),
                prop(
                    "variant",
                    "BadgeVariant",
                    false,
                    "Visual style (default: default)",
                ),
            ],
            variants: Some(vec![
                "default".to_string(),
                "secondary".to_string(),
                "destructive".to_string(),
                "outline".to_string(),
            ]),
        },
        CatalogComponent {
            name: "Modal".to_string(),
            description: "Dialog overlay with title, content, footer, and trigger button."
                .to_string(),
            props: vec![
                prop("title", "String", true, "Modal title"),
                prop("description", "Option<String>", false, "Modal description"),
                prop(
                    "children",
                    "Vec<ComponentNode>",
                    false,
                    "Content components inside the modal body",
                ),
                prop(
                    "footer",
                    "Vec<ComponentNode>",
                    false,
                    "Components in the modal footer",
                ),
                prop(
                    "trigger_label",
                    "Option<String>",
                    false,
                    "Label for the trigger button",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Checkbox".to_string(),
            description: "Boolean checkbox field with label, description, and data binding."
                .to_string(),
            props: vec![
                prop("field", "String", true, "Form field name for data binding"),
                prop("label", "String", true, "Checkbox label text"),
                prop(
                    "description",
                    "Option<String>",
                    false,
                    "Help text below the checkbox",
                ),
                prop("checked", "Option<bool>", false, "Default checked state"),
                prop(
                    "data_path",
                    "Option<String>",
                    false,
                    "Data path for pre-filling from handler data",
                ),
                prop(
                    "required",
                    "Option<bool>",
                    false,
                    "Whether field is required",
                ),
                prop(
                    "disabled",
                    "Option<bool>",
                    false,
                    "Whether field is disabled",
                ),
                prop("error", "Option<String>", false, "Validation error message"),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Switch".to_string(),
            description: "Toggle switch (visual alternative to Checkbox, same props).".to_string(),
            props: vec![
                prop("field", "String", true, "Form field name for data binding"),
                prop("label", "String", true, "Switch label text"),
                prop(
                    "description",
                    "Option<String>",
                    false,
                    "Help text below the switch",
                ),
                prop("checked", "Option<bool>", false, "Default checked state"),
                prop(
                    "data_path",
                    "Option<String>",
                    false,
                    "Data path for pre-filling from handler data",
                ),
                prop(
                    "required",
                    "Option<bool>",
                    false,
                    "Whether field is required",
                ),
                prop(
                    "disabled",
                    "Option<bool>",
                    false,
                    "Whether field is disabled",
                ),
                prop("error", "Option<String>", false, "Validation error message"),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Separator".to_string(),
            description: "Visual divider between content sections.".to_string(),
            props: vec![prop(
                "orientation",
                "Option<Orientation>",
                false,
                "Direction: horizontal (default) or vertical",
            )],
            variants: None,
        },
        CatalogComponent {
            name: "DescriptionList".to_string(),
            description: "Key-value pairs displayed as a description list.".to_string(),
            props: vec![
                prop(
                    "items",
                    "Vec<DescriptionItem>",
                    true,
                    "Items: { label, value, format? }",
                ),
                prop(
                    "columns",
                    "Option<u8>",
                    false,
                    "Number of columns for layout",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Tabs".to_string(),
            description: "Tabbed content with multiple panels.".to_string(),
            props: vec![
                prop(
                    "default_tab",
                    "String",
                    true,
                    "Value of the initially active tab",
                ),
                prop(
                    "tabs",
                    "Vec<Tab>",
                    true,
                    "Tab definitions: { value, label, children }",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Breadcrumb".to_string(),
            description: "Navigation breadcrumb trail.".to_string(),
            props: vec![prop(
                "items",
                "Vec<BreadcrumbItem>",
                true,
                "Breadcrumb items: { label, url? }",
            )],
            variants: None,
        },
        CatalogComponent {
            name: "Pagination".to_string(),
            description: "Page navigation for paginated data.".to_string(),
            props: vec![
                prop("current_page", "u32", true, "Current page number"),
                prop("per_page", "u32", true, "Items per page"),
                prop("total", "u32", true, "Total number of items"),
                prop(
                    "base_url",
                    "Option<String>",
                    false,
                    "Base URL for page links",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Progress".to_string(),
            description: "Progress bar with percentage value.".to_string(),
            props: vec![
                prop("value", "u8", true, "Percentage value (0-100)"),
                prop("max", "Option<u8>", false, "Maximum value"),
                prop("label", "Option<String>", false, "Label text above the bar"),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Avatar".to_string(),
            description: "User avatar with image, fallback text, and size variants.".to_string(),
            props: vec![
                prop("src", "Option<String>", false, "Image URL"),
                prop(
                    "alt",
                    "String",
                    true,
                    "Alt text (required for accessibility)",
                ),
                prop(
                    "fallback",
                    "Option<String>",
                    false,
                    "Fallback initials when no image",
                ),
                prop(
                    "size",
                    "Option<Size>",
                    false,
                    "Avatar size: xs, sm, default, lg",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Skeleton".to_string(),
            description: "Loading placeholder with configurable dimensions.".to_string(),
            props: vec![
                prop(
                    "width",
                    "Option<String>",
                    false,
                    "CSS width (e.g., \"100%\", \"200px\")",
                ),
                prop(
                    "height",
                    "Option<String>",
                    false,
                    "CSS height (e.g., \"40px\")",
                ),
                prop(
                    "rounded",
                    "Option<bool>",
                    false,
                    "Whether to use rounded corners",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "StatCard".to_string(),
            description: "Live-updatable metric card with label, value, icon, and optional SSE target."
                .to_string(),
            props: vec![
                prop("label", "String", true, "Metric label"),
                prop("value", "String", true, "Metric value"),
                prop("icon", "Option<String>", false, "Icon name"),
                prop("subtitle", "Option<String>", false, "Secondary text under the value"),
                prop(
                    "sse_target",
                    "Option<String>",
                    false,
                    "SSE target key for live updates (data-sse-target on the value element)",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Checklist".to_string(),
            description: "Onboarding-style checklist with optional dismissal and server-side state."
                .to_string(),
            props: vec![
                prop("title", "String", true, "Checklist title"),
                prop(
                    "items",
                    "Vec<ChecklistItem>",
                    true,
                    "Items: { label, checked, href? }",
                ),
                prop(
                    "dismissible",
                    "bool",
                    false,
                    "Whether the checklist can be dismissed (default: true)",
                ),
                prop("dismiss_label", "Option<String>", false, "Dismiss button label"),
                prop(
                    "data_key",
                    "Option<String>",
                    false,
                    "Server-side state persistence key",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Toast".to_string(),
            description: "Declarative notification intent consumed by the JS runtime via data attributes."
                .to_string(),
            props: vec![
                prop("message", "String", true, "Toast message text"),
                prop("variant", "ToastVariant", false, "Visual style (default: info)"),
                prop(
                    "timeout",
                    "Option<u32>",
                    false,
                    "Seconds before auto-dismiss (default: 5)",
                ),
                prop(
                    "dismissible",
                    "bool",
                    false,
                    "Whether the toast can be manually dismissed (default: true)",
                ),
            ],
            variants: Some(vec![
                "info".to_string(),
                "success".to_string(),
                "warning".to_string(),
                "error".to_string(),
            ]),
        },
        CatalogComponent {
            name: "NotificationDropdown".to_string(),
            description: "Dropdown listing notification items with icons, timestamps, and read state."
                .to_string(),
            props: vec![
                prop(
                    "notifications",
                    "Vec<NotificationItem>",
                    true,
                    "Items: { icon?, text, timestamp?, read, action_url? }",
                ),
                prop(
                    "empty_text",
                    "Option<String>",
                    false,
                    "Text shown when there are no notifications",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Sidebar".to_string(),
            description: "Dashboard sidebar with fixed top/bottom items and collapsible nav groups."
                .to_string(),
            props: vec![
                prop(
                    "fixed_top",
                    "Vec<SidebarNavItem>",
                    false,
                    "Pinned items rendered above groups",
                ),
                prop(
                    "groups",
                    "Vec<SidebarGroup>",
                    false,
                    "Collapsible groups: { label, collapsed, items }",
                ),
                prop(
                    "fixed_bottom",
                    "Vec<SidebarNavItem>",
                    false,
                    "Pinned items rendered below groups",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Header".to_string(),
            description: "Dashboard top bar with business name, notification badge, and user menu."
                .to_string(),
            props: vec![
                prop("business_name", "String", true, "Business/application name"),
                prop(
                    "notification_count",
                    "Option<u32>",
                    false,
                    "Unread notification count for the badge",
                ),
                prop("user_name", "Option<String>", false, "Current user name"),
                prop("user_avatar", "Option<String>", false, "Current user avatar URL"),
                prop("logout_url", "Option<String>", false, "URL for the logout link"),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Grid".to_string(),
            description: "Responsive multi-column grid layout with configurable breakpoint columns, gap, and optional horizontal scroll mode."
                .to_string(),
            props: vec![
                prop("columns", "u8", false, "Base (mobile) columns 1-12 (default: 2)"),
                prop(
                    "md_columns",
                    "Option<u8>",
                    false,
                    "Columns at md breakpoint (768px+)",
                ),
                prop(
                    "lg_columns",
                    "Option<u8>",
                    false,
                    "Columns at lg breakpoint (1024px+)",
                ),
                prop("gap", "GapSize", false, "Gap between items: none, sm, md, lg, xl"),
                prop(
                    "scrollable",
                    "Option<bool>",
                    false,
                    "Enable Trello-style horizontal scroll layout",
                ),
                prop("children", "Vec<ComponentNode>", false, "Grid children"),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Collapsible".to_string(),
            description: "Expandable <details>/<summary> section.".to_string(),
            props: vec![
                prop("title", "String", true, "Summary title"),
                prop("expanded", "bool", false, "Whether the section starts expanded"),
                prop("children", "Vec<ComponentNode>", false, "Hidden/expanded content"),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "EmptyState".to_string(),
            description: "Standardized empty view with title, description, and optional call-to-action."
                .to_string(),
            props: vec![
                prop("title", "String", true, "Empty state title"),
                prop("description", "Option<String>", false, "Supporting text"),
                prop("action", "Option<Action>", false, "Optional CTA action"),
                prop("action_label", "Option<String>", false, "Label for the CTA button"),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "FormSection".to_string(),
            description: "Visual grouping within a form with title, description, and layout variant."
                .to_string(),
            props: vec![
                prop("title", "String", true, "Section title"),
                prop("description", "Option<String>", false, "Section description"),
                prop("children", "Vec<ComponentNode>", false, "Fields inside the section"),
                prop(
                    "layout",
                    "Option<FormSectionLayout>",
                    false,
                    "Layout: stacked (default) or two_column",
                ),
            ],
            variants: Some(vec!["stacked".to_string(), "two_column".to_string()]),
        },
        CatalogComponent {
            name: "PageHeader".to_string(),
            description: "Page title with optional breadcrumb trail and action buttons.".to_string(),
            props: vec![
                prop("title", "String", true, "Page title"),
                prop(
                    "breadcrumb",
                    "Vec<BreadcrumbItem>",
                    false,
                    "Breadcrumb items: { label, url? }",
                ),
                prop(
                    "actions",
                    "Vec<ComponentNode>",
                    false,
                    "Action components rendered on the right",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "ButtonGroup".to_string(),
            description: "Horizontal button row with a consistent gap between buttons.".to_string(),
            props: vec![prop(
                "buttons",
                "Vec<ComponentNode>",
                false,
                "Button components rendered inline",
            )],
            variants: None,
        },
        CatalogComponent {
            name: "DropdownMenu".to_string(),
            description: "Trigger button with an absolutely-positioned action panel (kebab menu)."
                .to_string(),
            props: vec![
                prop("menu_id", "String", true, "Unique id for the menu element"),
                prop("trigger_label", "String", true, "Label for the trigger button"),
                prop(
                    "items",
                    "Vec<DropdownMenuAction>",
                    true,
                    "Menu items: { label, action, destructive }",
                ),
                prop(
                    "trigger_variant",
                    "Option<ButtonVariant>",
                    false,
                    "Visual style of the trigger button",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "DataTable".to_string(),
            description: "Stripe-style alternating-row table with per-row DropdownMenu, mobile card fallback, and empty state."
                .to_string(),
            props: vec![
                prop(
                    "columns",
                    "Vec<Column>",
                    true,
                    "Column definitions: { key, label, format? }",
                ),
                prop(
                    "data_path",
                    "String",
                    true,
                    "Data path to the array of rows (e.g., \"/data/products\")",
                ),
                prop(
                    "row_actions",
                    "Option<Vec<DropdownMenuAction>>",
                    false,
                    "Per-row dropdown menu actions",
                ),
                prop("empty_message", "Option<String>", false, "Message when empty"),
                prop(
                    "row_key",
                    "Option<String>",
                    false,
                    "Row key field for stable identification",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "KanbanBoard".to_string(),
            description: "Horizontally scrollable kanban columns on desktop, tab-based switching on mobile."
                .to_string(),
            props: vec![
                prop(
                    "columns",
                    "Vec<KanbanColumnProps>",
                    true,
                    "Columns: { id, title, count, children }",
                ),
                prop(
                    "mobile_default_column",
                    "Option<String>",
                    false,
                    "Column id shown by default on mobile",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "CalendarCell".to_string(),
            description: "Single day cell in a month grid with today highlight, out-of-month muting, and event indicators."
                .to_string(),
            props: vec![
                prop("day", "u8", true, "Day of month (1-31)"),
                prop("is_today", "bool", false, "Whether this day is today"),
                prop(
                    "is_current_month",
                    "bool",
                    false,
                    "Whether this day belongs to the current month",
                ),
                prop("event_count", "u32", false, "Number of events on this day"),
                prop(
                    "dot_colors",
                    "Vec<String>",
                    false,
                    "Per-event Tailwind color classes (e.g., \"bg-blue-500\")",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "ActionCard".to_string(),
            description: "Horizontal clickable row with icon, title, description, and chevron; variant-colored left border."
                .to_string(),
            props: vec![
                prop("title", "String", true, "Card title"),
                prop("description", "String", true, "Card description"),
                prop("icon", "Option<String>", false, "Icon name"),
                prop(
                    "variant",
                    "ActionCardVariant",
                    false,
                    "Visual style (default, setup, danger)",
                ),
                prop(
                    "href",
                    "Option<String>",
                    false,
                    "Navigation URL; when set the card renders as an <a>",
                ),
            ],
            variants: Some(vec![
                "default".to_string(),
                "setup".to_string(),
                "danger".to_string(),
            ]),
        },
        CatalogComponent {
            name: "ProductTile".to_string(),
            description: "Touch-friendly POS product tile with name, price, and +/- quantity controls bound to a hidden form field."
                .to_string(),
            props: vec![
                prop("product_id", "String", true, "Stable product identifier"),
                prop("name", "String", true, "Product name"),
                prop("price", "String", true, "Formatted product price"),
                prop("field", "String", true, "Form field name for the quantity input"),
                prop(
                    "default_quantity",
                    "Option<u32>",
                    false,
                    "Initial quantity value",
                ),
            ],
            variants: None,
        },
        CatalogComponent {
            name: "Image".to_string(),
            description: "Bounded visual asset rendered into a box. Accepts either an \
                          external URL (src) or a server-constructed inline SVG string \
                          (svg) — exactly one of the two must be set. The URL variant \
                          HTML-escapes the src attribute; the SVG variant emits the svg string verbatim \
                          (intended for server-constructed SVG — charts, sparklines, icons — not user input). \
                          alt is required on both variants (compile-enforced accessibility). \
                          placeholder_label applies to the URL variant only."
                .to_string(),
            props: vec![
                prop(
                    "src",
                    "String",
                    false,
                    "Image source URL (URL variant — use when svg is absent). \
                     HTML-escaped as an attribute value.",
                ),
                prop(
                    "svg",
                    "String",
                    false,
                    "Inline SVG string emitted verbatim (SVG variant — use when src is absent). \
                     Server-constructed SVG only; not suitable for user input.",
                ),
                prop(
                    "alt",
                    "String",
                    true,
                    "Alt text for accessibility — required on both source variants.",
                ),
                prop(
                    "aspect_ratio",
                    "Option<String>",
                    false,
                    "CSS aspect ratio (e.g., \"16/9\").",
                ),
                prop(
                    "placeholder_label",
                    "Option<String>",
                    false,
                    "Label shown in the skeleton placeholder behind the image (URL variant only).",
                ),
            ],
            variants: None,
        },
    ]
}

/// Plugin components registered in the framework.
///
/// Plugin components use the same `{"type": "Map", ...}` JSON syntax as
/// built-in components. They are rendered by the plugin system which also
/// handles loading their required JS/CSS assets.
fn build_plugin_catalog() -> Vec<CatalogComponent> {
    vec![CatalogComponent {
        name: "Map".to_string(),
        description:
            "Interactive map powered by Leaflet. Renders markers, custom tiles, and popups. \
             Assets loaded via CDN automatically. Plugin component — uses the same JSON syntax \
             as built-in components."
                .to_string(),
        props: vec![
            prop(
                "center",
                "[f64; 2]",
                true,
                "Map center as [latitude, longitude]",
            ),
            prop("zoom", "u8", false, "Zoom level 0-18 (default: 13)"),
            prop(
                "height",
                "String",
                false,
                "CSS height of the map container (default: \"400px\")",
            ),
            prop(
                "markers",
                "Vec<MapMarker {lat, lng, popup?}>",
                false,
                "Markers to display on the map",
            ),
            prop(
                "tile_url",
                "Option<String>",
                false,
                "Custom tile layer URL template (default: OpenStreetMap)",
            ),
            prop(
                "attribution",
                "Option<String>",
                false,
                "Tile layer attribution text",
            ),
            prop(
                "max_zoom",
                "Option<u8>",
                false,
                "Maximum zoom level for the tile layer",
            ),
        ],
        variants: None,
    }]
}

fn prop(name: &str, type_name: &str, required: bool, description: &str) -> PropInfo {
    PropInfo {
        name: name.to_string(),
        type_name: type_name.to_string(),
        required,
        description: description.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_components_present() {
        let catalog = execute(None);
        assert_eq!(
            catalog.components.len(),
            42,
            "Catalog should contain all 42 built-in components (including DetailForm + KeyValueEditor + RichTextEditor backfill), got {}",
            catalog.components.len()
        );

        let names: Vec<&str> = catalog.components.iter().map(|c| c.name.as_str()).collect();
        let expected = [
            "Text",
            "Button",
            "Card",
            "Table",
            "Form",
            "Input",
            "Select",
            "Alert",
            "Badge",
            "Modal",
            "Checkbox",
            "Switch",
            "Separator",
            "DescriptionList",
            "Tabs",
            "Breadcrumb",
            "Pagination",
            "Progress",
            "Avatar",
            "Skeleton",
            "StatCard",
            "Checklist",
            "Toast",
            "NotificationDropdown",
            "Sidebar",
            "Header",
            "Grid",
            "Collapsible",
            "EmptyState",
            "FormSection",
            "PageHeader",
            "ButtonGroup",
            "DropdownMenu",
            "DataTable",
            "KanbanBoard",
            "CalendarCell",
            "ActionCard",
            "ProductTile",
            "Image",
            "DetailForm",
            "KeyValueEditor",
            "RichTextEditor",
        ];
        for name in &expected {
            assert!(names.contains(name), "Missing component: {name}");
        }
    }

    #[test]
    fn test_plugin_components_present() {
        let catalog = execute(None);
        assert_eq!(
            catalog.plugin_components.len(),
            1,
            "Catalog should contain 1 plugin component (Map), got {}",
            catalog.plugin_components.len()
        );
        assert_eq!(catalog.plugin_components[0].name, "Map");
    }

    #[test]
    fn test_filter_by_component() {
        let catalog = execute(Some("Button"));
        assert_eq!(catalog.components.len(), 1);
        assert_eq!(catalog.components[0].name, "Button");
        assert!(catalog.plugin_components.is_empty());
    }

    #[test]
    fn test_filter_by_plugin_component() {
        let catalog = execute(Some("Map"));
        assert!(catalog.components.is_empty());
        assert_eq!(catalog.plugin_components.len(), 1);
        assert_eq!(catalog.plugin_components[0].name, "Map");
    }

    #[test]
    fn test_filter_case_insensitive() {
        let catalog = execute(Some("button"));
        assert_eq!(catalog.components.len(), 1);
        assert_eq!(catalog.components[0].name, "Button");

        let catalog = execute(Some("CARD"));
        assert_eq!(catalog.components.len(), 1);
        assert_eq!(catalog.components[0].name, "Card");
    }

    #[test]
    fn test_unknown_component_returns_empty() {
        let catalog = execute(Some("NonExistent"));
        assert!(
            catalog.components.is_empty(),
            "Unknown component should return empty list"
        );
    }

    #[test]
    fn test_serialization() {
        let catalog = execute(None);
        let json = serde_json::to_string(&catalog);
        assert!(json.is_ok(), "Catalog should serialize to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("components"));
        assert!(json_str.contains("plugin_components"));
        assert!(json_str.contains("builder_api"));
        assert!(json_str.contains("action_api"));
        assert!(json_str.contains("Button"));
        assert!(json_str.contains("Map"));
        assert!(json_str.contains("props"));
    }

    #[test]
    fn test_builder_api_present() {
        let catalog = execute(None);
        assert!(
            catalog.builder_api.contains("JsonUiView::new()"),
            "Builder API should document JsonUiView::new()"
        );
        assert!(
            catalog.builder_api.contains("ComponentNode"),
            "Builder API should document ComponentNode"
        );
    }

    #[test]
    fn test_action_api_present() {
        let catalog = execute(None);
        assert!(
            catalog.action_api.contains("Action::new"),
            "Action API should document Action::new"
        );
        assert!(
            catalog.action_api.contains("Action::get"),
            "Action API should document Action::get"
        );
        assert!(
            catalog.action_api.contains("Action::delete"),
            "Action API should document Action::delete"
        );
        assert!(
            catalog.action_api.contains("ActionOutcome"),
            "Action API should document ActionOutcome"
        );
    }

    #[test]
    fn test_components_have_props() {
        let catalog = execute(None);
        for component in &catalog.components {
            assert!(
                !component.description.is_empty(),
                "{} should have a description",
                component.name
            );
            // All components have at least one prop except Skeleton (all optional) and Separator
            // But even those have props defined
            let no_required = ["Separator", "Skeleton", "Sidebar", "Grid", "ButtonGroup"];
            if !no_required.contains(&component.name.as_str()) {
                assert!(
                    component.props.iter().any(|p| p.required),
                    "{} should have at least one required prop",
                    component.name
                );
            }
        }
    }

    #[test]
    fn test_button_has_variants() {
        let catalog = execute(Some("Button"));
        let button = &catalog.components[0];
        let variants = button
            .variants
            .as_ref()
            .expect("Button should have variants");
        assert_eq!(variants.len(), 6);
        assert!(variants.contains(&"default".to_string()));
        assert!(variants.contains(&"destructive".to_string()));
    }

    #[test]
    fn test_filter_returns_all_fields() {
        let catalog = execute(Some("Table"));
        assert_eq!(catalog.components.len(), 1);
        // Builder and action API still present even when filtering
        assert!(!catalog.builder_api.is_empty());
        assert!(!catalog.action_api.is_empty());
    }
}
