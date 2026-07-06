---
name: ferro:controller
description: Generate a controller with handlers
allowed-tools:
  - Bash
  - Read
  - Write
  - Glob
  - AskUserQuestion
---

<objective>
Generate a new Ferro controller with HTTP handlers.

Creates:
1. Controller file in `src/controllers/{name}.rs`
2. Updates `src/controllers/mod.rs`
3. Optionally generates resource routes
</objective>

<arguments>
Required:
- `name` - Controller name (e.g., `PostController`, `posts`)

Optional:
- `--resource` - Generate full CRUD handlers (index, show, store, update, destroy)
- `--api` - Generate API-style handlers (JSON responses)
- `--inertia` - Generate Inertia.js handlers (for SSR)

Examples:
- `/ferro:controller PostController`
- `/ferro:controller posts --resource`
- `/ferro:controller api/users --resource --api`
</arguments>

<process>

<step name="parse_args">

Parse controller name and options:
- Convert to snake_case for file name
- Determine if resource handlers needed
- Check for api/inertia flags

</step>

<step name="check_existing">

```bash
if [ -f "src/controllers/{snake_name}.rs" ]; then
    echo "EXISTS"
fi
```

</step>

<step name="generate_controller">

**Standard controller:**

```rust
//! {ControllerName} handlers

use ferro_rs::prelude::*;

/// List all items
#[handler]
pub async fn index(req: Request) -> Response {
    // TODO: Implement
    Ok(json!({"message": "index"}))
}

/// Show single item
#[handler]
pub async fn show(req: Request, id: Path<i32>) -> Response {
    // TODO: Implement
    Ok(json!({"id": id.0}))
}
```

**Resource controller (--resource):**

```rust
//! {ControllerName} - Resource handlers
//!
//! CRUD operations for {ResourceName}

use ferro_rs::prelude::*;
use crate::models::{model_name}::{ModelName, Entity, Column};

/// GET /{resources} - List all {resources}
#[handler]
pub async fn index(req: Request) -> Response {
    let items = {ModelName}::query().all().await?;
    Ok(json!(items))
}

/// GET /{resources}/{id} - Show single {resource}
#[handler]
pub async fn show(req: Request, id: Path<i32>) -> Response {
    let item = {ModelName}::find(id.0).await?
        .ok_or_else(|| HttpResponse::not_found())?;
    Ok(json!(item))
}

/// POST /{resources} - Create new {resource}
#[handler]
pub async fn store(req: Request) -> Response {
    let input: Create{ModelName}Request = req.validate().await?;

    let item = {ModelName}::create(input.into()).await?;

    Ok(HttpResponse::created().json(item))
}

/// PUT /{resources}/{id} - Update {resource}
#[handler]
pub async fn update(req: Request, id: Path<i32>) -> Response {
    let input: Update{ModelName}Request = req.validate().await?;

    let item = {ModelName}::find(id.0).await?
        .ok_or_else(|| HttpResponse::not_found())?;

    let updated = item.update(input.into()).await?;

    Ok(json!(updated))
}

/// DELETE /{resources}/{id} - Delete {resource}
#[handler]
pub async fn destroy(req: Request, id: Path<i32>) -> Response {
    let item = {ModelName}::find(id.0).await?
        .ok_or_else(|| HttpResponse::not_found())?;

    item.delete().await?;

    Ok(HttpResponse::no_content())
}

// ============================================================================
// REQUEST VALIDATION
// ============================================================================

#[form_request]
pub struct Create{ModelName}Request {
    // Add fields with validation rules
}

#[form_request]
pub struct Update{ModelName}Request {
    // Add fields with validation rules
}
```

**Inertia controller (--inertia):**

```rust
//! {ControllerName} - Inertia handlers

use ferro_rs::prelude::*;
use ferro_inertia::Inertia;
use crate::models::{model_name}::{ModelName};

/// GET /{resources} - List page
#[handler]
pub async fn index(req: Request) -> Response {
    let items = {ModelName}::query().all().await?;

    Inertia::render(&req, "{ModelName}/Index", json!({
        "items": items
    }))
}

/// GET /{resources}/{id} - Show page
#[handler]
pub async fn show(req: Request, id: Path<i32>) -> Response {
    let item = {ModelName}::find(id.0).await?
        .ok_or_else(|| HttpResponse::not_found())?;

    Inertia::render(&req, "{ModelName}/Show", json!({
        "item": item
    }))
}

/// GET /{resources}/create - Create form
#[handler]
pub async fn create(req: Request) -> Response {
    Inertia::render(&req, "{ModelName}/Create", json!({}))
}

/// GET /{resources}/{id}/edit - Edit form
#[handler]
pub async fn edit(req: Request, id: Path<i32>) -> Response {
    let item = {ModelName}::find(id.0).await?
        .ok_or_else(|| HttpResponse::not_found())?;

    Inertia::render(&req, "{ModelName}/Edit", json!({
        "item": item
    }))
}

/// POST /{resources} - Store
#[handler]
pub async fn store(req: Request) -> Response {
    let ctx = SavedInertiaContext::from(&req);
    let input: Create{ModelName}Request = req.validate().await?;

    let item = {ModelName}::create(input.into()).await?;

    Inertia::redirect_ctx(&ctx, &format!("/{resources}/{}", item.id))
}

/// PUT /{resources}/{id} - Update
#[handler]
pub async fn update(req: Request, id: Path<i32>) -> Response {
    let ctx = SavedInertiaContext::from(&req);
    let input: Update{ModelName}Request = req.validate().await?;

    let item = {ModelName}::find(id.0).await?
        .ok_or_else(|| HttpResponse::not_found())?;

    item.update(input.into()).await?;

    Inertia::redirect_ctx(&ctx, &format!("/{resources}/{}", id.0))
}

/// DELETE /{resources}/{id} - Destroy
#[handler]
pub async fn destroy(req: Request, id: Path<i32>) -> Response {
    let ctx = SavedInertiaContext::from(&req);

    let item = {ModelName}::find(id.0).await?
        .ok_or_else(|| HttpResponse::not_found())?;

    item.delete().await?;

    Inertia::redirect_ctx(&ctx, "/{resources}")
}
```

</step>

<step name="update_mod">

Update `src/controllers/mod.rs`:

```rust
pub mod {snake_name};
```

</step>

<step name="suggest_routes">

Output route suggestions:

```
Created controller: {ControllerName}

File: src/controllers/{snake_name}.rs

Add routes to src/routes.rs:

// Resource routes
Route::resource("/{resources}", controllers::{snake_name});

// Or individual routes
Route::get("/{resources}", controllers::{snake_name}::index);
Route::get("/{resources}/{id}", controllers::{snake_name}::show);
Route::post("/{resources}", controllers::{snake_name}::store);
Route::put("/{resources}/{id}", controllers::{snake_name}::update);
Route::delete("/{resources}/{id}", controllers::{snake_name}::destroy);
```

</step>

</process>
