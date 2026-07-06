# Request & Response

## The Request Object

Every handler receives a `Request` object:

```rust
#[handler]
pub async fn show(req: Request) -> Response {
    // Use request data
}
```

### Accessing Request Data

```rust
// Path: /users/123
let id = req.param::<i64>("id")?;

// Query: /users?page=2&sort=name
let page = req.query::<i64>("page").unwrap_or(1);
let sort = req.query::<String>("sort");

// Headers
let auth = req.header("Authorization");
let content_type = req.content_type();

// Method and path
let method = req.method();
let path = req.path();
let full_url = req.url();
```

### Request Body

```rust
// JSON body
let data: CreateUserData = req.json().await?;

// Form data
let form: HashMap<String, String> = req.form().await?;

// Raw body
let bytes = req.body_bytes().await?;
let text = req.body_string().await?;
```

### File Uploads

```rust
let file = req.file("avatar").await?;

// File properties
let filename = file.filename();
let content_type = file.content_type();
let size = file.size();

// Save file
file.store("avatars").await?;
// or
file.store_as("avatars", "custom-name.jpg").await?;
```

### Authentication

```rust
// Check if authenticated
if req.is_authenticated() {
    let user = req.user()?;
    println!("Hello, {}", user.name);
}

// Get optional user
if let Some(user) = req.user_optional() {
    // ...
}
```

### Session Data

```rust
// Read session
let user_id = req.session().get::<i64>("user_id");

// Write session (needs mutable access)
req.session_mut().set("flash", "Welcome back!");

// Flash messages
let flash = req.session().flash("message");
```

### Cookies

```rust
// Read cookie
let token = req.cookie("remember_token");

// Cookies are typically set on responses
```

### Database Access

```rust
let db = req.db();
let users = User::find().all(db).await?;
```

## Responses

Handlers return `Response` which is `Result<HttpResponse, HttpResponse>`:

```rust
pub type Response = Result<HttpResponse, HttpResponse>;
```

### JSON Responses

```rust
#[handler]
pub async fn index(req: Request) -> Response {
    Ok(json!({
        "users": users,
        "total": 100
    }))
}

// Or using HttpResponse directly
Ok(HttpResponse::json(data))
```

### HTML Responses

```rust
Ok(HttpResponse::html("<h1>Hello</h1>"))
```

### Text Responses

```rust
Ok(HttpResponse::text("Hello, World!"))
```

### Inertia Responses

```rust
// Basic Inertia response
Inertia::render(&req, "Users/Index", props)

// With saved context (for form handlers)
let ctx = SavedInertiaContext::from(&req);
let form = req.input().await?;  // Consumes request
Inertia::render_ctx(&ctx, "Users/Form", props)
```

See [Controllers - Form Handling](controllers.md#form-handling-with-savedinertiacontext) for complete examples.

### Redirects

```rust
// Simple redirect
Ok(Redirect::to("/dashboard"))

// Redirect back
Ok(Redirect::back(&req))

// Redirect with flash message
Ok(Redirect::to("/users").with("success", "User created!"))

// Named route redirect
Ok(Redirect::route("users.show", [("id", "1")]))
```

### Status Codes

```rust
// Success codes
Ok(HttpResponse::ok(body))
Ok(HttpResponse::created(body))
Ok(HttpResponse::no_content())

// Error codes
Err(HttpResponse::bad_request("Invalid input"))
Err(HttpResponse::unauthorized("Please login"))
Err(HttpResponse::forbidden("Access denied"))
Err(HttpResponse::not_found("User not found"))
Err(HttpResponse::server_error("Something went wrong"))
```

### Custom Status

```rust
Ok(HttpResponse::json(data).status(202))
```

### Response Headers

```rust
Ok(HttpResponse::json(data)
    .header("X-Custom", "value")
    .header("Cache-Control", "no-cache"))
```

### Cookies

```rust
Ok(HttpResponse::json(data)
    .cookie(Cookie::new("token", "abc123"))
    .cookie(Cookie::new("remember", "true")
        .http_only(true)
        .secure(true)
        .max_age(Duration::days(30))))
```

### Binary Responses

Serve raw binary data (images, PDFs, generated files):

```rust
// Serve binary with explicit content type
Ok(HttpResponse::bytes(png_bytes)
    .header("Content-Type", "image/png"))

// File download with Content-Disposition
Ok(HttpResponse::download(pdf_bytes, "report.pdf"))
```

`download()` auto-detects Content-Type from the filename extension and sets the `Content-Disposition: attachment` header.

For static files served from `public/`, use the built-in static file serving (no handler needed).

## Error Handling

Return errors as `HttpResponse`:

```rust
#[handler]
pub async fn show(req: Request, id: i64) -> Response {
    let user = User::find_by_id(id)
        .one(&req.db())
        .await?
        .ok_or_else(|| HttpResponse::not_found("User not found"))?;

    Ok(json!(user))
}
```

### Domain Errors

Use `#[domain_error]` for typed errors:

```rust
#[domain_error]
pub enum UserError {
    #[error("User not found")]
    #[status(404)]
    NotFound,

    #[error("Email already taken")]
    #[status(409)]
    EmailTaken,
}

#[handler]
pub async fn store(req: Request) -> Response {
    let result = create_user().await?; // ? converts UserError to HttpResponse
    Ok(json!(result))
}
```

## Form Requests

For automatic validation:

```rust
#[derive(FormRequest)]
pub struct CreateUserRequest {
    #[validate(required, email)]
    pub email: String,

    #[validate(required, min(8))]
    pub password: String,

    #[validate(same("password"))]
    pub password_confirmation: String,
}

#[handler]
pub async fn store(req: Request, form: CreateUserRequest) -> Response {
    // form is validated, safe to use
    Ok(json!({"email": form.email}))
}
```

If validation fails, Ferro automatically returns a 422 response with errors.
