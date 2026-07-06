# Coding Conventions

**Analysis Date:** 2026-01-15

## Naming Patterns

**Files:**
- snake_case.rs for all Rust files (`query_builder.rs`, `form_request.rs`)
- mod.rs for module roots
- lib.rs for library crate roots
- PascalCase.tsx for React components (`Home.tsx`, `App.tsx`)
- kebab-case for crate names (`ferro-events`, `ferro-queue`)

**Functions:**
- snake_case for all functions (`dispatch()`, `validate()`, `with_params()`)
- Constructor pattern: `new()`, `default()`
- Builder pattern: `with_*()` methods (`with_status()`, `with_message()`)
- Handler functions: verb-based (`index`, `show`, `store`, `update`, `destroy`)

**Variables:**
- snake_case for variables and parameters
- No underscore prefix for private (Rust visibility handles this)
- Constants: UPPER_SNAKE_CASE (`DEFAULT_COST`, `HASH_DEFAULT_COST`)

**Types:**
- PascalCase for structs: `EventDispatcher`, `ValidationError`, `Request`
- PascalCase for traits: `Model`, `FromRequest`, `Authenticatable`, `HttpError`
- PascalCase for enums: `Commands`, `Channel`, `SameSite`
- Suffixes indicate purpose: `Middleware`, `Provider`, `Driver`, `Handler`, `Error`

## Code Style

**Formatting:**
- 4-space indentation (Rust default)
- No custom rustfmt.toml (uses Rust defaults)
- Double quotes for strings
- No trailing commas required

**Linting:**
- Clippy for linting (`cargo clippy`)
- Configured in bacon.toml for watch mode
- No custom clippy.toml

## Import Organization

**Order:**
1. Standard library (`std::*`)
2. External crates (`serde`, `tokio`, `sea_orm`)
3. Internal crates (`crate::*`)
4. Local modules (`super::*`)

**Grouping:**
- Blank line between groups
- `use` statements at top of file
- Re-exports in lib.rs for public API

**Path Aliases:**
- `crate::` for current crate
- `super::` for parent module
- Full crate names for dependencies

## Error Handling

**Patterns:**
- Result types for fallible operations
- `?` operator for error propagation
- Custom errors implement `std::error::Error`
- `HttpError` trait for HTTP status code mapping

**Error Types:**
- `FrameworkError` enum for framework errors (`framework/src/error.rs`)
- `ValidationError` for validation failures
- `AuthorizationError` for auth failures
- `AppError` for application-level errors with status codes

**Conversion:**
- Implement `From<E> for HttpResponse` for automatic conversion
- Use `thiserror` for derive-based error types

## Logging

**Framework:**
- tracing crate for structured logging
- Spans for request lifecycle tracking
- Console output by default

**Patterns:**
- `tracing::info!()`, `tracing::debug!()`, `tracing::error!()`
- Structured fields: `tracing::info!(user_id = %id, "User logged in")`
- No `println!()` in production code

## Comments

**When to Comment:**
- Module-level: `//!` doc comments explaining purpose
- Public API: `///` doc comments with examples
- Complex logic: Inline `//` comments
- Avoid obvious comments

**JSDoc/Rustdoc:**
- Required for all public functions/structs
- Use `# Example` sections with code blocks
- Use `# Errors` section for Result-returning functions

**TODO Comments:**
- Format: `// TODO: description`
- Located in: `app/src/middleware/share_inertia.rs` (incomplete features)

## Function Design

**Size:**
- Keep under 50 lines
- Extract helpers for complex logic
- One level of abstraction per function

**Parameters:**
- Use `impl Into<T>` for flexible input types
- Use `&self` for methods
- Async functions use `async fn`

**Return Values:**
- Explicit return types
- Result types for fallible operations
- Response type alias: `Response = Result<HttpResponse, HttpResponse>`

## Module Design

**Exports:**
- Re-export from `lib.rs` for public API
- `pub use` for common types
- Keep internal helpers private

**Barrel Files:**
- `mod.rs` for module organization
- `pub mod` for public submodules
- `pub use` for convenient imports

**Visibility:**
- `pub` for external API
- `pub(crate)` for crate-internal
- Default private within module

## Macros

**Naming:**
- snake_case for macro names: `routes!()`, `get!()`, `json!()`
- Attribute macros: `#[handler]`, `#[service]`, `#[derive(InertiaProps)]`

**Usage:**
- Prefer functions over macros when possible
- Use macros for DSL-like syntax (routing, validation rules)
- Document macro behavior with examples

## Async Patterns

**Conventions:**
- `async fn` for async functions
- `.await` for awaiting futures
- `tokio::spawn` for background tasks
- Use `async_trait` for async trait methods

**Channels:**
- Tokio channels for concurrency
- `Arc<Mutex<T>>` for shared state (prefer `DashMap` when applicable)

---

*Convention analysis: 2026-01-15*
*Update when patterns change*
