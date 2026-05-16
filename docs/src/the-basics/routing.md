# Routing

Ferro provides an expressive routing API similar to Laravel.

## Basic Routes

Define routes in `src/routes.rs`:

```rust
use ferro::Router;

pub fn routes() -> Router {
    Router::new()
        .get("/", home)
        .get("/users", users::index)
        .post("/users", users::store)
        .put("/users/:id", users::update)
        .delete("/users/:id", users::destroy)
}
```

## Route Parameters

Parameters are extracted from the URL path:

```rust
#[handler]
pub async fn show(req: Request, id: i64) -> Response {
    // id is extracted from /users/:id
    let user = User::find_by_id(id).one(&db).await?;
    Ok(json!(user))
}
```

### Optional Parameters

Use `Option<T>` for optional parameters:

```rust
#[handler]
pub async fn index(req: Request, page: Option<i64>) -> Response {
    let page = page.unwrap_or(1);
    // ...
}
```

## Route Groups

Group routes with shared middleware or prefixes:

```rust
pub fn routes() -> Router {
    Router::new()
        .group("/api", |group| {
            group
                .get("/users", api::users::index)
                .post("/users", api::users::store)
                .middleware(ApiAuthMiddleware)
        })
        .group("/admin", |group| {
            group
                .get("/dashboard", admin::dashboard)
                .middleware(AdminMiddleware)
        })
}
```

### Root routes inside a group

A `"/"` route inside a non-root group matches both the bare prefix and the
prefix with a trailing slash. Other paths concatenate normally.

```rust
Router::new()
    .group("/s/{slug}", |r| {
        r.get("/", show_item)        // matches /s/foo AND /s/foo/
         .get("/edit", edit_item)    // matches /s/foo/edit
    })
```

A trailing slash on the group prefix is stripped before concatenation, so
`group("/api/", ...)` with a child `/x` route produces `/api/x`, not
`/api//x` (double-slash). Route introspection via `get_registered_routes()` and
`ferro-mcp list_routes` reports one entry per logical handler — the
canonical path without trailing slash.

## Named Routes

```rust
Router::new()
    .get("/users/:id", users::show).name("users.show")
```

Generate URLs:

```rust
let url = route("users.show", [("id", "1")]);
// => "/users/1"
```

## Resource Routes

Generate CRUD routes for a resource:

```rust
Router::new()
    .resource("/users", users_controller)
```

This creates:

| Method | URI | Handler |
|--------|-----|---------|
| GET | /users | index |
| GET | /users/create | create |
| POST | /users | store |
| GET | /users/:id | show |
| GET | /users/:id/edit | edit |
| PUT | /users/:id | update |
| DELETE | /users/:id | destroy |

## Route Middleware

Apply middleware to specific routes:

```rust
Router::new()
    .get("/dashboard", dashboard)
    .middleware(AuthMiddleware)
```

Or to groups:

```rust
Router::new()
    .group("/admin", |group| {
        group
            .get("/users", admin::users)
            .middleware(AdminMiddleware)
    })
```

## Fallback Routes

Handle 404s:

```rust
Router::new()
    .get("/", home)
    .fallback(not_found)
```

## Route Constraints

Validate route parameters:

```rust
Router::new()
    .get("/users/:id", users::show)
    .where_("id", r"\d+")  // Must be numeric
```

## API Routes

For JSON APIs, typically group under `/api`:

```rust
Router::new()
    .group("/api/v1", |api| {
        api
            .get("/users", api::users::index)
            .post("/users", api::users::store)
            .middleware(ApiAuthMiddleware)
    })
```

## View Routes

For simple views without controller logic:

```rust
Router::new()
    .view("/about", "About", AboutProps::default())
```

## Redirect Routes

```rust
Router::new()
    .redirect("/old-path", "/new-path")
    .redirect_permanent("/legacy", "/modern")
```

## Route Caching

In production, routes are compiled at build time for optimal performance.
