# Getting Started with JSON-UI

Build server-rendered pages with Ferro's component system. No frontend toolchain required.

## Prerequisites

- An existing Ferro application
- No additional dependencies -- JSON-UI is built into the framework

## Your First View

Create a handler that returns a JSON-UI page. The view is a tree of components rendered to HTML with Tailwind classes.

### 1. Build the view

```rust
use ferro::{handler, JsonUi, JsonUiView, ComponentNode, Component, CardProps,
    TextProps, TextElement, StatCardProps, Response};

#[handler]
pub async fn dashboard() -> Response {
    let view = JsonUiView::new()
        .title("Dashboard")
        .layout("dashboard")
        .component(ComponentNode::card("welcome", CardProps {
            title: "Welcome".to_string(),
            description: Some("Your application dashboard".to_string()),
            children: vec![
                ComponentNode {
                    key: "intro".to_string(),
                    component: Component::Text(TextProps {
                        content: "This page is rendered entirely from Rust.".to_string(),
                        element: TextElement::P,
                    }),
                    action: None,
                    visibility: None,
                },
            ],
            footer: vec![],
        }))
        .component(ComponentNode::stat_card("orders", StatCardProps {
            label: "Orders Today".to_string(),
            value: "42".to_string(),
            icon: Some("shopping-bag".to_string()),
            subtitle: Some("Last updated just now".to_string()),
            sse_target: Some("orders_today".to_string()),
        }));

    JsonUi::render(&view, &serde_json::json!({}))
}
```

### 2. Register the route

```rust
use ferro::get;

get!("/dashboard", controllers::dashboard::dashboard);
```

That's it. Visit `/dashboard` to see a styled card with your content, wrapped in the dashboard layout with sidebar and navigation.

## Adding a Form

Forms use the `Form` component with `Input` fields. The form's `action` references a named route that the framework resolves to a URL at render time.

### Create form

```rust
use ferro::{handler, JsonUi, JsonUiView, ComponentNode, Component, FormProps,
    InputProps, InputType, SelectProps, SelectOption, Action, Response};

#[handler]
pub async fn create() -> Response {
    let view = JsonUiView::new()
        .title("Create User")
        .layout("dashboard")
        .component(ComponentNode {
            key: "form".to_string(),
            component: Component::Form(FormProps {
                action: Action::new("users.store"),
                fields: vec![
                    ComponentNode {
                        key: "name".to_string(),
                        component: Component::Input(InputProps {
                            field: "name".to_string(),
                            label: "Name".to_string(),
                            input_type: InputType::Text,
                            placeholder: Some("Enter full name".to_string()),
                            required: Some(true),
                            disabled: None,
                            error: None,
                            description: None,
                            default_value: None,
                            data_path: None,
                            step: None,
                        }),
                        action: None,
                        visibility: None,
                    },
                    ComponentNode {
                        key: "email".to_string(),
                        component: Component::Input(InputProps {
                            field: "email".to_string(),
                            label: "Email".to_string(),
                            input_type: InputType::Email,
                            placeholder: Some("user@example.com".to_string()),
                            required: Some(true),
                            disabled: None,
                            error: None,
                            description: None,
                            default_value: None,
                            data_path: None,
                            step: None,
                        }),
                        action: None,
                        visibility: None,
                    },
                    ComponentNode {
                        key: "role".to_string(),
                        component: Component::Select(SelectProps {
                            field: "role".to_string(),
                            label: "Role".to_string(),
                            options: vec![
                                SelectOption { value: "user".to_string(), label: "User".to_string() },
                                SelectOption { value: "admin".to_string(), label: "Admin".to_string() },
                            ],
                            placeholder: Some("Select a role".to_string()),
                            required: Some(true),
                            disabled: None,
                            error: None,
                            description: None,
                            default_value: None,
                            data_path: None,
                        }),
                        action: None,
                        visibility: None,
                    },
                ],
                method: None,
            }),
            action: None,
            visibility: None,
        });

    JsonUi::render(&view, &serde_json::json!({}))
}
```

### Pre-fill an edit form

For edit forms, pass the existing record as data and use `data_path` on each field to bind values:

```rust
#[handler]
pub async fn edit(user: User) -> Response {
    let view = JsonUiView::new()
        .title("Edit User")
        .layout("dashboard")
        .component(ComponentNode {
            key: "form".to_string(),
            component: Component::Form(FormProps {
                action: Action::new("users.update"),
                fields: vec![
                    ComponentNode {
                        key: "name".to_string(),
                        component: Component::Input(InputProps {
                            field: "name".to_string(),
                            label: "Name".to_string(),
                            input_type: InputType::Text,
                            placeholder: None,
                            required: Some(true),
                            disabled: None,
                            error: None,
                            description: None,
                            default_value: None,
                            data_path: Some("/data/user/name".to_string()),
                            step: None,
                        }),
                        action: None,
                        visibility: None,
                    },
                    ComponentNode {
                        key: "email".to_string(),
                        component: Component::Input(InputProps {
                            field: "email".to_string(),
                            label: "Email".to_string(),
                            input_type: InputType::Email,
                            placeholder: None,
                            required: Some(true),
                            disabled: None,
                            error: None,
                            description: None,
                            default_value: None,
                            data_path: Some("/data/user/email".to_string()),
                            step: None,
                        }),
                        action: None,
                        visibility: None,
                    },
                ],
                method: None,
            }),
            action: None,
            visibility: None,
        });

    let data = serde_json::json!({
        "user": {
            "name": user.name,
            "email": user.email,
        }
    });

    JsonUi::render(&view, &data)
}
```

The `data_path` value `"/data/user/name"` tells the renderer to look up `data.user.name` and pre-fill the input.

### Validation errors

When validation fails, use `JsonUi::render_validation_error()` to automatically populate error messages on the corresponding form fields:

```rust
use ferro::{handler, Request, Response, Validator, HttpResponse};

#[handler]
pub async fn store(req: Request) -> Response {
    let form: serde_json::Value = req.json().await?;

    let result = Validator::new(&form)
        .rules("name", rules![required(), string()])
        .rules("email", rules![required(), email()])
        .validate();

    if let Err(errors) = result {
        let view = create_form_view(); // reuse the view from create()
        return JsonUi::render_validation_error(&view, &serde_json::json!({}), &errors);
    }

    // Create user and redirect...
    Ok(HttpResponse::redirect("/users"))
}
```

The framework matches error field names (`"name"`, `"email"`) to input `field` values and sets the `error` prop on each matching component.

## Adding a Table

Tables bind to a data path and render rows automatically from the handler data.

```rust
use ferro::{handler, JsonUi, JsonUiView, ComponentNode, Component, TableProps,
    Column, ColumnFormat, Action, PaginationProps, Response};

#[handler]
pub async fn index() -> Response {
    let view = JsonUiView::new()
        .title("Users")
        .layout("dashboard")
        .component(ComponentNode::table("users-table", TableProps {
            columns: vec![
                Column { key: "name".to_string(), label: "Name".to_string(), format: None },
                Column { key: "email".to_string(), label: "Email".to_string(), format: None },
                Column {
                    key: "created_at".to_string(),
                    label: "Created".to_string(),
                    format: Some(ColumnFormat::Date),
                },
            ],
            data_path: "/data/users".to_string(),
            row_actions: Some(vec![
                Action::get("users.edit"),
                Action::delete("users.destroy")
                    .confirm_danger("Delete this user?"),
            ]),
            empty_message: Some("No users found".to_string()),
            sortable: None,
            sort_column: None,
            sort_direction: None,
        }))
        .component(ComponentNode {
            key: "pagination".to_string(),
            component: Component::Pagination(PaginationProps {
                current_page: 1,
                per_page: 25,
                total: 100,
                base_url: Some("/users".to_string()),
            }),
            action: None,
            visibility: None,
        });

    let data = serde_json::json!({
        "users": [
            {"name": "Alice", "email": "alice@example.com", "created_at": "2026-01-15"},
            {"name": "Bob", "email": "bob@example.com", "created_at": "2026-01-20"},
        ]
    });

    JsonUi::render(&view, &data)
}
```

Key points:
- `data_path` tells the table where to find its row data in the handler response
- `row_actions` adds action buttons to each row (Edit link, Delete with confirmation)
- `ColumnFormat::Date` formats the `created_at` column as a date
- `Pagination` renders page navigation below the table

## Using the CLI

Generate view files with the CLI:

```bash
ferro make:json-view UserIndex
```

With an Anthropic API key configured, the command reads your models and routes to generate a complete view file with appropriate components. Without an API key, it produces a static template as a starting point.

## Next Steps

- **[Components](components.md)** -- Reference for all 26 built-in component types
- **[Actions](actions.md)** -- Navigation, form submission, confirmations, and outcomes
- **[Data Binding & Visibility](data-binding.md)** -- Data paths and conditional rendering
- **[Layouts](layouts.md)** -- Page structure, DashboardLayout, and custom layouts
- **[Plugins](plugins.md)** -- Extend the component catalog with custom interactive components
