# Architecture

**Analysis Date:** 2026-01-15

## Pattern Overview

**Overall:** Modular Monorepo (Cargo Workspace) with Laravel-inspired Architecture

**Key Characteristics:**
- Laravel/Rails-inspired developer experience
- Full-stack framework with integrated CLI, ORM, validation, authentication
- End-to-end type safety: Rust backend → Auto-generated TypeScript → React frontend
- Batteries-included: Database, caching, sessions, notifications, broadcasting, background jobs
- Compile-time route validation via procedural macros

## Layers

**HTTP/Request-Response Layer:**
- Purpose: Handle incoming HTTP requests and produce responses
- Contains: Request/Response types, cookies, Inertia.js adapter
- Location: `framework/src/http/`, `ferro-inertia/src/`
- Depends on: Hyper HTTP primitives
- Used by: Routing layer, handlers

**Routing & Middleware Layer:**
- Purpose: Match requests to handlers, apply middleware chain
- Contains: Router, route definitions, middleware registry
- Location: `framework/src/routing/`, `framework/src/middleware/`
- Depends on: HTTP layer, matchit router
- Used by: Application server

**Business Logic Layer:**
- Purpose: Handle application-specific logic
- Contains: Controllers (handlers), actions, validation, authorization
- Location: `app/src/controllers/`, `app/src/actions/`, `framework/src/validation/`, `framework/src/authorization/`
- Depends on: Data layer, service container
- Used by: Route handlers

**Data & Persistence Layer:**
- Purpose: Database access and model definitions
- Contains: SeaORM wrapper, models, migrations, query builder
- Location: `framework/src/database/`, `app/src/models/`, `app/src/migrations/`
- Depends on: SeaORM, database drivers
- Used by: Business logic layer

**Cross-Cutting Concerns:**
- Purpose: Shared functionality across layers
- Contains: Auth, sessions, cache, queue, events, notifications, broadcasting, storage
- Location: `framework/src/auth/`, `framework/src/session/`, `ferro-cache/`, `ferro-queue/`, `ferro-events/`, `ferro-notifications/`, `ferro-broadcast/`, `ferro-storage/`
- Depends on: Configuration, external services
- Used by: All layers

**Infrastructure Layer:**
- Purpose: Framework foundation
- Contains: DI container, configuration, error types, hashing, metrics
- Location: `framework/src/container/`, `framework/src/config/`, `framework/src/error.rs`
- Depends on: External crates
- Used by: All layers

## Data Flow

**HTTP Request Lifecycle:**

1. TCP connection arrives at `Server::run()` (`framework/src/server.rs`)
2. Request created with params extraction (`framework/src/http/request.rs`)
3. Global middleware chain executes in order (`framework/src/middleware/chain.rs`)
   - MetricsMiddleware, SessionMiddleware, CsrfMiddleware, custom middleware
4. Route matching via matchit (`framework/src/routing/router.rs`)
5. Per-route middleware executes (`framework/src/middleware/registry.rs`)
6. Handler parameter extraction via FromRequest (`framework/src/http/extract.rs`)
7. Controller/handler executes (`app/src/controllers/*`)
8. Response created (JSON, Inertia, redirect)
9. Response returned through middleware chain (reverse order)
10. Hyper sends response to client

**State Management:**
- Stateless request handling
- Session data in database or memory driver
- Application state via DI container (singleton services)
- No in-memory request state between requests

## Key Abstractions

**Service Container:**
- Purpose: Dependency injection and service resolution
- Location: `framework/src/container/mod.rs`
- Pattern: Type-erased HashMap with Arc wrappers
- Usage: `#[service(ConcreteType)]` macro, `App::make::<T>()`

**Handler Functions:**
- Purpose: HTTP request handlers with auto parameter extraction
- Location: `ferro-macros/src/handler.rs`
- Pattern: Attribute macro with FromRequest trait
- Usage: `#[handler] async fn show(req: Request, user: User) -> Response`

**Model Trait:**
- Purpose: Database entity abstraction with query builder
- Location: `framework/src/database/model.rs`
- Pattern: Trait extending SeaORM EntityTrait
- Usage: `User::query().where_(...).all().await`

**Middleware:**
- Purpose: Request/response pipeline stages
- Location: `framework/src/middleware/mod.rs`
- Pattern: Async trait with next() continuation
- Usage: `global_middleware!(MyMiddleware)`, `.middleware(Auth)`

**Policy:**
- Purpose: Per-model authorization rules
- Location: `framework/src/authorization/policy.rs`
- Pattern: Trait with ability methods
- Usage: `impl Policy<Post> for PostPolicy { fn update(...) }`

**Event Dispatcher:**
- Purpose: Decoupled event handling
- Location: `ferro-events/src/dispatcher.rs`
- Pattern: Observer pattern with async listeners
- Usage: `dispatch_event(UserCreated { ... })`

**Job Queue:**
- Purpose: Background task execution
- Location: `ferro-queue/src/`
- Pattern: Redis-backed job queue with workers
- Usage: `queue_dispatch(SendEmail::new(...))`

## Entry Points

**CLI Tool:**
- Location: `ferro-cli/src/main.rs`
- Triggers: `ferro new`, `ferro make:*`, `ferro migrate`, `ferro serve`
- Responsibilities: Project scaffolding, code generation, migrations

**Application Server:**
- Location: `app/src/main.rs`
- Triggers: `cargo run`, compiled binary
- Responsibilities: Parse CLI args, initialize database, register services, start HTTP server

**Framework Library:**
- Location: `framework/src/lib.rs`
- Triggers: `use ferro::*` in application code
- Responsibilities: Export public API (Request, Response, Router, DB, etc.)

**Proc Macros:**
- Location: `ferro-macros/src/lib.rs`
- Triggers: `#[handler]`, `routes!()`, `#[service]`, `#[derive(InertiaProps)]`
- Responsibilities: Code generation, compile-time validation

## Error Handling

**Strategy:** Result types with error conversion to HTTP responses

**Patterns:**
- Handlers return `Response = Result<HttpResponse, HttpResponse>`
- `?` operator for early returns with error responses
- Custom errors implement `HttpError` trait for status codes
- `FrameworkError` enum for framework-level errors (`framework/src/error.rs`)
- Validation errors convert to 422 with field messages

## Cross-Cutting Concerns

**Logging:**
- Tracing crate throughout codebase
- Structured logging with spans
- Console output by default

**Validation:**
- Builder pattern: `Validator::new(&data).rules("field", rules![...])`
- Validation errors as HTTP 422 with field-level messages
- Location: `framework/src/validation/`

**Authentication:**
- Session-based auth with pluggable providers
- `Auth::user()` for current user
- `Auth::attempt()` for login
- Location: `framework/src/auth/`

**Authorization:**
- Policy-based with `Gate::allows("ability", model)`
- Middleware: `AuthorizeMiddleware`
- Location: `framework/src/authorization/`

---

*Architecture analysis: 2026-01-15*
*Update when major patterns change*
