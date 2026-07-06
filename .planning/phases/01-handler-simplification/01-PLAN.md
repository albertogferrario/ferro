# Phase 1, Plan 1: Sample App Handler Modernization

## Overview

Update the sample application to use `#[handler]` macro consistently across all controllers. The macro already exists but the sample app uses manual patterns, creating a poor reference for agents.

## Goal

Demonstrate reduced ceremony by migrating sample app handlers to use:
- `#[handler]` macro for automatic parameter extraction
- Typed path parameters instead of `req.param("id")?`
- Direct model binding for database lookups
- Clean response patterns without explicit `.status(200)`

## Scope

**Files to modify:**
- `app/src/controllers/user.rs` - User CRUD operations
- `app/src/controllers/todo.rs` - Todo CRUD with actions
- `app/src/controllers/home.rs` - Home page handler
- `app/src/controllers/auth.rs` - Authentication handlers

**Not in scope:**
- Handler macro internals (works as-is)
- New macro features (save for later plan if needed)

## Current vs Target

### Current Pattern (user.rs:show)
```rust
pub async fn show(req: Request) -> Response {
    let id = req.param("id")?;
    json_response!({ "id": id, "name": format!("User {}", id) })
}
```

### Target Pattern
```rust
#[handler]
pub async fn show(id: i32) -> Response {
    json_response!({ "id": id, "name": format!("User {id}") })
}
```

### Current Pattern (todo.rs:list)
```rust
pub async fn list(_req: Request) -> Response {
    let action = App::resolve::<ListTodosAction>()?;
    match action.execute().await {
        Ok(todos) => json_response!({ "success": true, "todos": todos }).status(200),
        Err(e) => json_response!({ "success": false, "error": e.to_string() }).status(500),
    }
}
```

### Target Pattern
```rust
#[handler]
pub async fn list(action: ListTodosAction) -> Response {
    let todos = action.execute().await.map_err(|e|
        json_response!({ "success": false, "error": e.to_string() }).status(500)
    )?;
    json_response!({ "success": true, "todos": todos })
}
```

## Tasks

### Wave 1: Core Handler Migration

- [ ] **1.1** Update `home.rs` - simplest case, validate macro works
  - Remove `_req: Request` parameter
  - Add `#[handler]` attribute
  - Verify compilation and behavior

- [ ] **1.2** Update `user.rs` - typed path parameters
  - Add `#[handler]` to all handlers (index, show, store, update, destroy)
  - Replace `req.param("id")?` with typed `id: i32` parameter
  - Remove unused Request parameters

- [ ] **1.3** Update `auth.rs` - form request patterns
  - Add `#[handler]` to login/logout/register handlers
  - Ensure FormRequest types work with macro
  - Preserve session handling logic

### Wave 2: Action Pattern Cleanup

- [ ] **2.1** Update `todo.rs` handlers with action injection
  - Add `#[handler]` to all todo handlers
  - Inject actions as typed parameters
  - Simplify error handling patterns
  - Remove explicit `.status(200)` calls

### Wave 3: Validation

- [ ] **3.1** Run full test suite
  - `cargo test --all-features`
  - `cargo clippy`
  - `cargo fmt --check`

- [ ] **3.2** Manual verification
  - Start sample app
  - Test each endpoint works correctly
  - Verify error responses are unchanged

## Success Criteria

1. All sample app handlers use `#[handler]` macro
2. No manual `req.param()` calls in controllers
3. No unused `_req` parameters
4. All tests pass
5. Sample app functions identically to before

## Risks

- **Handler macro might not support all extraction patterns**: Mitigate by testing incrementally
- **Action injection might require macro changes**: Fall back to `App::resolve` if needed, document for future plan

## Notes

This plan validates the existing `#[handler]` macro capabilities. If limitations are found, they'll be documented for a follow-up plan to enhance the macro itself.
