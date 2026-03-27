# Validation

Ferro provides a powerful validation system with a fluent API, built-in rules, custom messages, and automatic request validation through Form Requests.

## Basic Usage

### Creating a Validator

```rust
use ferro::Validator;

let data = serde_json::json!({
    "name": "John Doe",
    "email": "john@example.com",
    "age": 25
});

let errors = Validator::new()
    .rule("name", rules![required(), string(), min(2)])
    .rule("email", rules![required(), email()])
    .rule("age", rules![required(), integer(), between(18, 120)])
    .validate(&data);

if errors.is_empty() {
    println!("Validation passed!");
} else {
    println!("Errors: {:?}", errors.all());
}
```

### Quick Validation

```rust
use ferro::validate;

let data = serde_json::json!({
    "email": "invalid-email"
});

let errors = validate(&data, vec![
    ("email", rules![required(), email()]),
]);

if errors.fails() {
    for (field, messages) in errors.all() {
        println!("{}: {:?}", field, messages);
    }
}
```

## Built-in Rules

### Required Rules

```rust
// rules like required(), string(), etc. are available directly (no import needed)

// Field must be present and not empty
required()

// Field required only if another field has a specific value
required_if("role", "admin")
```

### Type Rules

```rust
// Must be a string
string()

// Must be an integer
integer()

// Must be numeric (integer or float)
numeric()

// Must be a boolean
boolean()

// Must be an array
array()
```

### Size Rules

```rust
// Minimum length (strings) or value (numbers)
min(5)

// Maximum length (strings) or value (numbers)
max(100)

// Between minimum and maximum (inclusive)
between(1, 10)
```

### Format Rules

```rust
// Valid email address
email()

// Valid URL
url()

// Matches a regex pattern
regex(r"^[A-Z]{2}\d{4}$")

// Only alphabetic characters
alpha()

// Only alphanumeric characters
alpha_num()

// Alphanumeric, dashes, and underscores
alpha_dash()

// Valid date (YYYY-MM-DD format)
date()
```

### Comparison Rules

```rust
// Must match {field}_confirmation
confirmed()

// Must be one of the specified values
in_array(vec!["active", "inactive", "pending"])

// Must NOT be one of the specified values
not_in(vec!["admin", "root"])

// Must be different from another field
different("old_password")

// Must be the same as another field
same("password")
```

### Special Rules

```rust
// Field can be null/missing (stops validation if null)
nullable()

// Must be "yes", "on", "1", or true
accepted()
```

## Validation Examples

### User Registration

```rust
use ferro::Validator;

let data = serde_json::json!({
    "username": "johndoe",
    "email": "john@example.com",
    "password": "secret123",
    "password_confirmation": "secret123",
    "age": 25,
    "terms": "yes"
});

let errors = Validator::new()
    .rule("username", rules![required(), string(), min(3), max(20), alpha_dash()])
    .rule("email", rules![required(), email()])
    .rule("password", rules![required(), string(), min(8), confirmed()])
    .rule("age", rules![required(), integer(), between(13, 120)])
    .rule("terms", rules![accepted()])
    .validate(&data);
```

### Nested Data Validation

Use dot notation to validate nested JSON structures:

```rust
let data = serde_json::json!({
    "user": {
        "profile": {
            "name": "John",
            "bio": "Developer"
        }
    },
    "settings": {
        "notifications": true
    }
});

let errors = Validator::new()
    .rule("user.profile.name", rules![required(), string(), min(2)])
    .rule("user.profile.bio", rules![nullable(), string(), max(500)])
    .rule("settings.notifications", rules![required(), boolean()])
    .validate(&data);
```

### Conditional Validation

```rust
let data = serde_json::json!({
    "type": "business",
    "company_name": "Acme Corp",
    "tax_id": "123456789"
});

let errors = Validator::new()
    .rule("type", rules![required(), in_array(vec!["personal", "business"])])
    .rule("company_name", rules![required_if("type", "business"), string()])
    .rule("tax_id", rules![required_if("type", "business"), string()])
    .validate(&data);
```

## Custom Messages

Override default error messages for specific fields and rules:

```rust
let errors = Validator::new()
    .rule("email", rules![required(), email()])
    .rule("password", rules![required(), min(8)])
    .message("email.required", "Please provide your email address")
    .message("email.email", "The email format is invalid")
    .message("password.required", "Password is required")
    .message("password.min", "Password must be at least 8 characters")
    .validate(&data);
```

## Custom Attributes

Replace field names in error messages with friendlier names:

```rust
let errors = Validator::new()
    .rule("dob", rules![required(), date()])
    .rule("cc_number", rules![required(), string()])
    .attribute("dob", "date of birth")
    .attribute("cc_number", "credit card number")
    .validate(&data);

// Error: "The date of birth field is required"
// Instead of: "The dob field is required"
```

## Validation Errors

The `ValidationError` type collects and manages validation errors:

```rust
use ferro::ValidationError;

let errors: ValidationError = validator.validate(&data);

// Check if validation failed
if errors.fails() {
    // Get first error for a field
    if let Some(message) = errors.first("email") {
        println!("Email error: {}", message);
    }

    // Get all errors for a field
    if let Some(messages) = errors.get("password") {
        for msg in messages {
            println!("Password: {}", msg);
        }
    }

    // Check if specific field has errors
    if errors.has("username") {
        println!("Username has validation errors");
    }

    // Get all errors as HashMap
    let all_errors = errors.all();

    // Get total error count
    println!("Total errors: {}", errors.count());

    // Convert to JSON for API responses
    let json = errors.to_json();
}
```

### JSON Error Response

```rust
use ferro::{Response, json_response};

if errors.fails() {
    return json_response!(422, {
        "message": "Validation failed",
        "errors": errors
    });
}
```

## Form Requests

Form Requests provide automatic validation and authorization for HTTP requests.

### Defining a Form Request

```rust
use ferro::FormRequest;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 2, max = 50))]
    pub name: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,

    #[validate(range(min = 13, max = 120))]
    pub age: Option<i32>,
}

impl FormRequest for CreateUserRequest {}
```

### Using Form Requests in Handlers

```rust
use ferro::{handler, Response, json_response};
use crate::requests::CreateUserRequest;

#[handler]
pub async fn store(request: CreateUserRequest) -> Response {
    // Request is automatically validated
    // If validation fails, 422 response is returned

    let user = User::create(
        &request.name,
        &request.email,
        &request.password,
    ).await?;

    json_response!(201, { "user": user })
}
```

### Authorization

Override the `authorize` method to add authorization logic:

```rust
use ferro::{FormRequest, Request};

impl FormRequest for UpdatePostRequest {
    fn authorize(req: &Request) -> bool {
        // Check if user can update the post
        if let Some(user) = req.user() {
            if let Some(post_id) = req.param("post") {
                return user.can_edit_post(post_id);
            }
        }
        false
    }
}
```

### Validation Attributes

The `validator` crate provides these validation attributes:

```rust
#[derive(Deserialize, Validate)]
pub struct ExampleRequest {
    // Length validation
    #[validate(length(min = 1, max = 100))]
    pub title: String,

    // Email validation
    #[validate(email)]
    pub email: String,

    // URL validation
    #[validate(url)]
    pub website: Option<String>,

    // Range validation
    #[validate(range(min = 0, max = 100))]
    pub score: i32,

    // Regex validation
    #[validate(regex(path = "RE_PHONE"))]
    pub phone: String,

    // Custom validation
    #[validate(custom(function = "validate_username"))]
    pub username: String,

    // Nested validation
    #[validate(nested)]
    pub address: Address,

    // Required (use Option for optional fields)
    pub required_field: String,
    pub optional_field: Option<String>,
}

// Define regex patterns
lazy_static! {
    static ref RE_PHONE: Regex = Regex::new(r"^\+?[1-9]\d{1,14}$").expect("valid regex pattern");
}

// Custom validation function
fn validate_username(username: &str) -> Result<(), validator::ValidationError> {
    if username.contains("admin") {
        return Err(validator::ValidationError::new("username_reserved"));
    }
    Ok(())
}
```

## Custom Rules

Create custom validation rules by implementing the `Rule` trait:

```rust
use ferro::Rule;
use serde_json::Value;

pub struct Uppercase;

impl Rule for Uppercase {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        match value {
            Value::String(s) => {
                if s.chars().all(|c| c.is_uppercase() || !c.is_alphabetic()) {
                    Ok(())
                } else {
                    Err(format!("The {} field must be uppercase.", field))
                }
            }
            _ => Err(format!("The {} field must be a string.", field)),
        }
    }

    fn name(&self) -> &'static str {
        "uppercase"
    }
}

// Usage
let errors = Validator::new()
    .rule("code", rules![required(), Uppercase])
    .validate(&data);
```

### Custom Rule with Parameters

```rust
pub struct StartsWithRule {
    prefix: String,
}

impl StartsWithRule {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self { prefix: prefix.into() }
    }
}

impl Rule for StartsWithRule {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        match value {
            Value::String(s) => {
                if s.starts_with(&self.prefix) {
                    Ok(())
                } else {
                    Err(format!("The {} field must start with {}.", field, self.prefix))
                }
            }
            _ => Err(format!("The {} field must be a string.", field)),
        }
    }

    fn name(&self) -> &'static str {
        "starts_with"
    }
}

// Helper function
pub fn starts_with(prefix: impl Into<String>) -> StartsWithRule {
    StartsWithRule::new(prefix)
}

// Usage
let errors = Validator::new()
    .rule("product_code", rules![required(), starts_with("PRD-")])
    .validate(&data);
```

## API Validation Pattern

```rust
use ferro::{handler, Request, Response, json_response, Validator};

#[handler]
pub async fn store(req: Request) -> Response {
    let data: serde_json::Value = req.json().await?;

    let errors = Validator::new()
        .rule("title", rules![required(), string(), min(1), max(200)])
        .rule("content", rules![required(), string()])
        .rule("status", rules![required(), in_array(vec!["draft", "published"])])
        .rule("tags", rules![nullable(), array()])
        .message("title.required", "Please provide a title")
        .message("content.required", "Content cannot be empty")
        .validate(&data);

    if errors.fails() {
        return json_response!(422, {
            "message": "The given data was invalid.",
            "errors": errors
        });
    }

    // Proceed with valid data
    let post = Post::create(&data).await?;
    json_response!(201, { "post": post })
}
```

## Rules Reference

| Rule | Description | Example |
|------|-------------|---------|
| `required()` | Field must be present and not empty | `required()` |
| `required_if(field, value)` | Required if another field equals value | `required_if("type", "business")` |
| `string()` | Must be a string | `string()` |
| `integer()` | Must be an integer | `integer()` |
| `numeric()` | Must be numeric | `numeric()` |
| `boolean()` | Must be a boolean | `boolean()` |
| `array()` | Must be an array | `array()` |
| `min(n)` | Minimum length/value | `min(8)` |
| `max(n)` | Maximum length/value | `max(255)` |
| `between(min, max)` | Value between min and max | `between(1, 100)` |
| `email()` | Valid email format | `email()` |
| `url()` | Valid URL format | `url()` |
| `regex(pattern)` | Matches regex pattern | `regex(r"^\d{5}$")` |
| `alpha()` | Only alphabetic characters | `alpha()` |
| `alpha_num()` | Only alphanumeric | `alpha_num()` |
| `alpha_dash()` | Alphanumeric, dashes, underscores | `alpha_dash()` |
| `date()` | Valid date (YYYY-MM-DD) | `date()` |
| `confirmed()` | Must match `{field}_confirmation` | `confirmed()` |
| `in_array(values)` | Must be one of values | `in_array(vec!["a", "b"])` |
| `not_in(values)` | Must not be one of values | `not_in(vec!["x", "y"])` |
| `different(field)` | Must differ from field | `different("old_email")` |
| `same(field)` | Must match field | `same("password")` |
| `nullable()` | Can be null (stops if null) | `nullable()` |
| `accepted()` | Must be "yes", "on", "1", true | `accepted()` |

## Best Practices

1. **Use Form Requests for complex validation** - Keeps controllers clean
2. **Provide custom messages** - User-friendly error messages improve UX
3. **Use custom attributes** - Replace technical field names with readable ones
4. **Validate early** - Fail fast with clear error messages
5. **Use nullable() for optional fields** - Prevents errors on missing optional data
6. **Create custom rules** - Reuse validation logic across the application
7. **Return 422 status** - Standard HTTP status for validation errors
8. **Structure errors as JSON** - Easy to consume by frontend applications

## MCP Tools

Use `code_templates` with the `validation` category to generate validator boilerplate without memorizing rule names.

### `code_templates`

Returns ready-to-use code snippets for common validation patterns. Pass `category: "validation"` to get templates for the fluent `Validator::new()` API, Form Requests with the `validator` crate, and custom rule implementations. Useful when setting up validation for a new handler quickly.
