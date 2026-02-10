# Authentication

Ferro provides session-based authentication with an `Auth` facade, `Authenticatable` trait, `UserProvider` interface, password hashing (bcrypt), and route protection middleware.

## Quick Start

Scaffold a complete auth system with one command:

```bash
ferro make:auth
```

This generates:
- User migration with `email` and `password` fields
- `Authenticatable` implementation on your User model
- `DatabaseUserProvider` for user retrieval
- `AuthController` with register, login, and logout handlers
- Routes with auth/guest middleware

## Configuration

### Session

Authentication state is stored in server-side sessions. Configure via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `SESSION_LIFETIME` | `120` | Session lifetime in minutes |
| `SESSION_COOKIE` | `ferro_session` | Cookie name |
| `SESSION_SECURE` | `true` | HTTPS-only cookies |
| `SESSION_PATH` | `/` | Cookie path |
| `SESSION_SAME_SITE` | `Lax` | SameSite attribute (`Strict`, `Lax`, `None`) |

Load configuration from environment:

```rust
use ferro::session::SessionConfig;

let config = SessionConfig::from_env();
```

### Password Hashing

Ferro uses bcrypt with a default cost factor of 12 (same as Laravel). The cost factor is not configurable via environment variables -- change it by calling `hashing::hash_with_cost()` directly.

## The Authenticatable Trait

Implement `Authenticatable` on your User model to enable `Auth::user()` and `Auth::user_as::<T>()`.

```rust
use ferro::auth::Authenticatable;
use std::any::Any;

impl Authenticatable for User {
    fn auth_identifier(&self) -> i64 {
        self.id as i64
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
```

### Methods

| Method | Default | Description |
|--------|---------|-------------|
| `auth_identifier(&self) -> i64` | Required | Returns the user's unique ID (primary key) |
| `auth_identifier_name(&self) -> &'static str` | `"id"` | Column name for the identifier |
| `as_any(&self) -> &dyn Any` | Required | Enables downcasting via `Auth::user_as::<T>()` |

## User Provider

The `UserProvider` trait retrieves users from your data store. Register it in `bootstrap.rs` to enable `Auth::user()`.

```rust
use ferro::auth::{UserProvider, Authenticatable};
use ferro::FrameworkError;
use async_trait::async_trait;
use std::sync::Arc;

pub struct DatabaseUserProvider;

#[async_trait]
impl UserProvider for DatabaseUserProvider {
    async fn retrieve_by_id(
        &self,
        id: i64,
    ) -> Result<Option<Arc<dyn Authenticatable>>, FrameworkError> {
        let user = User::query()
            .filter(Column::Id.eq(id as i32))
            .first()
            .await?;
        Ok(user.map(|u| Arc::new(u) as Arc<dyn Authenticatable>))
    }
}
```

### Methods

| Method | Required | Description |
|--------|----------|-------------|
| `retrieve_by_id(id)` | Yes | Load user by primary key |
| `retrieve_by_credentials(credentials)` | No | Load user by arbitrary credentials (e.g., email lookup) |
| `validate_credentials(user, credentials)` | No | Check credentials against a user (e.g., password verification) |

### Registration

Register the provider in `bootstrap.rs`:

```rust
bind!(dyn UserProvider, DatabaseUserProvider);
```

## Auth Facade

The `Auth` struct provides static methods for all authentication operations.

### Checking Authentication State

```rust
use ferro::Auth;

// Check if a user is authenticated
if Auth::check() {
    // ...
}

// Check if the user is a guest
if Auth::guest() {
    // ...
}

// Get the authenticated user's ID
if let Some(user_id) = Auth::id() {
    println!("User ID: {}", user_id);
}

// Get the ID as a different type (e.g., i32 for SeaORM)
let user_id: i32 = Auth::id_as().expect("must be authenticated");
```

### Logging In

```rust
// Log in by user ID
// Regenerates session ID (prevents session fixation) and CSRF token
Auth::login(user_id);

// Log in with "remember me"
Auth::login_remember(user_id, &remember_token);
```

### Logging Out

```rust
// Clear authentication state, regenerate CSRF token
Auth::logout();

// Destroy entire session (logout everywhere)
Auth::logout_and_invalidate();
```

### Credential Validation

```rust
// Attempt authentication: validates credentials, logs in on success
let result = Auth::attempt(|| async {
    let user = find_user_by_email(&email).await?;
    match user {
        Some(u) if ferro::verify(&password, &u.password_hash)? => Ok(Some(u.id as i64)),
        _ => Ok(None),
    }
}).await?;

if result.is_some() {
    // Authenticated and logged in
}

// Validate credentials without logging in (e.g., password confirmation)
let valid = Auth::validate(|| async {
    let user_id = Auth::id().unwrap();
    let user = find_user_by_id(user_id).await?;
    Ok(ferro::verify(&password, &user.password_hash)?)
}).await?;
```

### Retrieving the Current User

```rust
// Get as trait object
if let Some(user) = Auth::user().await? {
    println!("User #{}", user.auth_identifier());
}

// Get as concrete type (requires Authenticatable + Clone)
if let Some(user) = Auth::user_as::<User>().await? {
    println!("Welcome, {}!", user.name);
}
```

### Method Reference

| Method | Returns | Description |
|--------|---------|-------------|
| `Auth::check()` | `bool` | `true` if authenticated |
| `Auth::guest()` | `bool` | `true` if not authenticated |
| `Auth::id()` | `Option<i64>` | Authenticated user's ID |
| `Auth::id_as::<T>()` | `Option<T>` | ID converted to type `T` |
| `Auth::login(id)` | `()` | Log in, regenerate session + CSRF |
| `Auth::login_remember(id, token)` | `()` | Log in with remember-me |
| `Auth::logout()` | `()` | Clear auth state, regenerate CSRF |
| `Auth::logout_and_invalidate()` | `()` | Destroy entire session |
| `Auth::attempt(validator)` | `Result<Option<i64>>` | Validate + auto-login |
| `Auth::validate(validator)` | `Result<bool>` | Validate without login |
| `Auth::user()` | `Result<Option<Arc<dyn Authenticatable>>>` | Current user (trait object) |
| `Auth::user_as::<T>()` | `Result<Option<T>>` | Current user (concrete type) |

## Handler Extractors

For ergonomic access to the current user in handler signatures, Ferro provides typed extractors that work as handler parameters.

### AuthUser\<T\>

Injects the authenticated user directly into the handler. Returns `401 Unauthenticated` if no user is logged in.

```rust
use ferro::{handler, AuthUser, Response, HttpResponse};
use crate::models::users;

#[handler]
pub async fn profile(user: AuthUser<users::Model>) -> Response {
    Ok(HttpResponse::json(serde_json::json!({
        "id": user.id,
        "name": user.name,
        "email": user.email
    })))
}
```

### OptionalUser\<T\>

Same as `AuthUser<T>`, but returns `None` for guests instead of a 401 error.

```rust
use ferro::{handler, OptionalUser, Response, HttpResponse};
use crate::models::users;

#[handler]
pub async fn home(user: OptionalUser<users::Model>) -> Response {
    let greeting = match user.as_ref() {
        Some(u) => format!("Welcome back, {}!", u.name),
        None => "Welcome, guest!".to_string(),
    };
    Ok(HttpResponse::json(serde_json::json!({"greeting": greeting})))
}
```

### Deref Behavior

Both `AuthUser<T>` and `OptionalUser<T>` implement `Deref`, so you access fields directly on the wrapper:

- `AuthUser<T>` derefs to `T` -- use `user.name`, not `user.0.name`
- `OptionalUser<T>` derefs to `Option<T>` -- use `user.as_ref()`, `user.is_some()`, etc.

### Limitations

`AuthUser` and `OptionalUser` count as a `FromRequest` parameter. Only one `FromRequest` parameter is allowed per handler signature, so they cannot be combined with `FormRequest` types in the same handler. If you need both the authenticated user and the request body, use `Request` and call `Auth::user_as::<T>()` manually:

```rust
#[handler]
pub async fn update_profile(req: Request) -> Response {
    let user: users::Model = Auth::user_as::<users::Model>().await?
        .ok_or_else(|| HttpResponse::json(serde_json::json!({"error": "Unauthenticated"})).status(401))?;
    let input: UpdateInput = req.json().await?;
    // ... use both user and input
}
```

## Password Hashing

Ferro provides bcrypt hashing functions re-exported at the crate root.

```rust
use ferro::{hash, verify, needs_rehash};

// Hash a password (bcrypt, cost 12)
let hashed = ferro::hash("my_password")?;

// Verify a password against a stored hash (constant-time comparison)
let valid = ferro::verify("my_password", &hashed)?;

// Check if a hash was created with a lower cost factor
if ferro::needs_rehash(&stored_hash) {
    let new_hash = ferro::hash(&password)?;
    // Update stored hash
}
```

### Custom Cost Factor

```rust
use ferro::hashing;

// Hash with a specific cost (higher = slower + more secure)
let hashed = hashing::hash_with_cost("my_password", 14)?;
```

## Middleware

### AuthMiddleware

Protects routes that require authentication.

```rust
use ferro::{AuthMiddleware, group, get};

// API routes: returns 401 JSON for unauthenticated requests
group!("/api")
    .middleware(AuthMiddleware::new())
    .routes([...]);

// Web routes: redirects to login page
group!("/dashboard")
    .middleware(AuthMiddleware::redirect_to("/login"))
    .routes([...]);
```

**Inertia-aware:** When an Inertia request hits `AuthMiddleware::redirect_to()`, it returns a `409` response with `X-Inertia-Location` header instead of a `302` redirect. This tells the Inertia client to perform a full page visit to the login URL.

### GuestMiddleware

Protects routes that should only be accessible to unauthenticated users (login, register pages).

```rust
use ferro::{GuestMiddleware, group, get};

group!("/")
    .middleware(GuestMiddleware::redirect_to("/dashboard"))
    .routes([
        get!("/login", auth::show_login),
        get!("/register", auth::show_register),
    ]);
```

Authenticated users visiting these routes are redirected to the specified path. Also Inertia-aware.

## Login and Registration Example

### Register Handler

```rust
use ferro::{handler, Request, Response, Auth, json_response};
use ferro::validation::{Validator, rules};

#[handler]
pub async fn register(req: Request) -> Response {
    let data: serde_json::Value = req.json().await?;

    // Validate input
    let errors = Validator::new()
        .rule("name", rules![required(), string(), min(2)])
        .rule("email", rules![required(), email()])
        .rule("password", rules![required(), string(), min(8), confirmed()])
        .validate(&data);

    if errors.fails() {
        return json_response!(422, {
            "message": "Validation failed.",
            "errors": errors
        });
    }

    // Hash password
    let password_hash = ferro::hash(data["password"].as_str().unwrap())?;

    // Insert user
    let user = User::insert(NewUser {
        name: data["name"].as_str().unwrap().to_string(),
        email: data["email"].as_str().unwrap().to_string(),
        password: password_hash,
    }).await?;

    // Log in
    Auth::login(user.id as i64);

    json_response!(201, { "message": "Registered." })
}
```

### Login Handler

```rust
#[handler]
pub async fn login(req: Request) -> Response {
    let data: serde_json::Value = req.json().await?;

    let errors = Validator::new()
        .rule("email", rules![required(), email()])
        .rule("password", rules![required(), string()])
        .validate(&data);

    if errors.fails() {
        return json_response!(422, {
            "message": "Validation failed.",
            "errors": errors
        });
    }

    let email = data["email"].as_str().unwrap();
    let password = data["password"].as_str().unwrap();

    // Attempt authentication
    let result = Auth::attempt(|| async {
        let user = User::find_by_email(email).await?;
        match user {
            Some(u) if ferro::verify(password, &u.password)? => Ok(Some(u.id as i64)),
            _ => Ok(None),
        }
    }).await?;

    match result {
        Some(_) => json_response!(200, { "message": "Authenticated." }),
        None => json_response!(401, { "message": "Invalid credentials." }),
    }
}
```

### Logout Handler

```rust
#[handler]
pub async fn logout(_req: Request) -> Response {
    Auth::logout();
    json_response!(200, { "message": "Logged out." })
}
```

### Route Registration

```rust
use ferro::{group, get, post, AuthMiddleware, GuestMiddleware};

// Guest-only routes (login/register pages)
group!("/")
    .middleware(GuestMiddleware::redirect_to("/dashboard"))
    .routes([
        post!("/register", auth::register),
        post!("/login", auth::login),
    ]);

// Authenticated routes
group!("/")
    .middleware(AuthMiddleware::redirect_to("/login"))
    .routes([
        post!("/logout", auth::logout),
        get!("/dashboard", dashboard::index),
    ]);
```

## Security

Ferro's auth system applies these protections:

| Protection | Mechanism |
|-----------|-----------|
| Session fixation | Session ID regenerated on `Auth::login()` |
| CSRF | Token regenerated on login and logout |
| Timing attacks | Bcrypt uses constant-time comparison |
| Cookie theft | `HttpOnly` flag set by default (not accessible via JavaScript) |
| Cross-site requests | `SameSite=Lax` cookie attribute by default |
| HTTPS enforcement | `Secure` cookie flag on by default |
| Session hijacking | Database-backed sessions with configurable lifetime |
| Password storage | Bcrypt with cost factor 12 (adaptive hashing) |
