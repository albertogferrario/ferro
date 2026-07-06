//! Procedural macros for the Ferro framework
//!
//! This crate provides compile-time validated macros for:
//! - Inertia.js responses with component validation
//! - Named route redirects with route validation
//! - Service auto-registration
//! - Handler attribute for controller methods
//! - FormRequest for validated request data
//! - Jest-like testing with describe! and test! macros

use proc_macro::TokenStream;

mod action;
mod describe;
mod domain_error;
mod ferro_test;
mod handler;
mod inertia;
mod injectable;
mod model;
mod redirect;
mod request;
mod resource;
mod resource_get;
mod resource_post;
mod service;
mod test_macro;
mod utils;
mod validate;

/// Derive macro for generating `Serialize` implementation for Inertia props
///
/// # Example
///
/// ```rust,ignore
/// #[derive(InertiaProps)]
/// struct HomeProps {
///     title: String,
///     user: User,
/// }
/// ```
#[proc_macro_derive(InertiaProps, attributes(inertia))]
pub fn derive_inertia_props(input: TokenStream) -> TokenStream {
    inertia::derive_inertia_props_impl(input)
}

/// Create an Inertia response with compile-time component validation
///
/// # Examples
///
/// ## With typed struct (recommended for type safety):
/// ```rust,ignore
/// #[derive(InertiaProps)]
/// struct HomeProps {
///     title: String,
///     user: User,
/// }
///
/// inertia_response!("Home", HomeProps { title: "Welcome".into(), user })
/// ```
///
/// ## With JSON-like syntax (for quick prototyping):
/// ```rust,ignore
/// inertia_response!("Dashboard", { "user": { "name": "John" } })
/// ```
///
/// This macro validates that the component file exists at compile time.
/// If `frontend/src/pages/Dashboard.tsx` doesn't exist, you'll get a compile error.
#[proc_macro]
pub fn inertia_response(input: TokenStream) -> TokenStream {
    inertia::inertia_response_impl(input)
}

/// Create a redirect to a path or named route
///
/// # Examples
///
/// ```rust,ignore
/// // Path redirect (starts with /)
/// redirect!("/dashboard").into()
///
/// // Named route redirect
/// redirect!("users.index").into()
///
/// // Redirect with route parameters
/// redirect!("users.show").with("id", "42").into()
///
/// // Redirect with query parameters
/// redirect!("users.index").query("page", "1").into()
/// ```
///
/// For named routes, this macro validates that the route exists at compile time.
/// Path redirects (starting with `/`) bypass validation and redirect directly.
#[proc_macro]
pub fn redirect(input: TokenStream) -> TokenStream {
    redirect::redirect_impl(input)
}

/// Mark a trait as a service for the App container
///
/// This attribute macro automatically adds `Send + Sync + 'static` bounds
/// to your trait, making it suitable for use with the dependency injection
/// container.
///
/// # Example
///
/// ```rust,ignore
/// use ferro::service;
///
/// #[service]
/// pub trait HttpClient {
///     async fn get(&self, url: &str) -> Result<String, Error>;
/// }
///
/// // This expands to:
/// pub trait HttpClient: Send + Sync + 'static {
///     async fn get(&self, url: &str) -> Result<String, Error>;
/// }
/// ```
///
/// Then you can use it with the App container:
///
/// ```rust,ignore
/// // Register
/// App::bind::<dyn HttpClient>(Arc::new(RealHttpClient::new()));
///
/// // Resolve
/// let client: Arc<dyn HttpClient> = App::make::<dyn HttpClient>().unwrap();
/// ```
#[proc_macro_attribute]
pub fn service(attr: TokenStream, input: TokenStream) -> TokenStream {
    service::service_impl(attr, input)
}

/// Attribute macro to auto-register a concrete type as a singleton
///
/// This macro automatically:
/// 1. Derives `Default` and `Clone` for the struct
/// 2. Registers it as a singleton in the App container at startup
///
/// # Example
///
/// ```rust,ignore
/// use ferro::injectable;
///
/// #[injectable]
/// pub struct AppState {
///     pub counter: u32,
/// }
///
/// // Automatically registered at startup
/// // Resolve via:
/// let state: AppState = App::get().unwrap();
/// ```
#[proc_macro_attribute]
pub fn injectable(_attr: TokenStream, input: TokenStream) -> TokenStream {
    injectable::injectable_impl(input)
}

/// Define a domain error with automatic HTTP response conversion
///
/// This macro automatically:
/// 1. Derives `Debug` and `Clone` for the type
/// 2. Implements `Display`, `Error`, and `HttpError` traits
/// 3. Implements `From<T> for FrameworkError` for seamless `?` usage
///
/// # Attributes
///
/// - `status`: HTTP status code (default: 500)
/// - `message`: Error message for Display (default: struct name converted to sentence)
///
/// # Example
///
/// ```rust,ignore
/// use ferro::domain_error;
///
/// #[domain_error(status = 404, message = "User not found")]
/// pub struct UserNotFoundError {
///     pub user_id: i32,
/// }
///
/// // Usage in controller - just use ? operator
/// pub async fn get_user(id: i32) -> Result<User, FrameworkError> {
///     users.find(id).ok_or(UserNotFoundError { user_id: id })?
/// }
/// ```
#[proc_macro_attribute]
pub fn domain_error(attr: TokenStream, input: TokenStream) -> TokenStream {
    domain_error::domain_error_impl(attr, input)
}

/// Attribute macro for controller handler methods
///
/// Transforms handler functions to automatically extract typed parameters
/// from HTTP requests using the `FromRequest` trait.
///
/// # Examples
///
/// ## With Request parameter:
/// ```rust,ignore
/// use ferro::{handler, Request, Response, json_response};
///
/// #[handler]
/// pub async fn index(req: Request) -> Response {
///     json_response!({ "message": "Hello" })
/// }
/// ```
///
/// ## With FormRequest parameter:
/// ```rust,ignore
/// use ferro::{handler, Response, json_response, request};
///
/// #[request]
/// pub struct CreateUserRequest {
///     #[validate(email)]
///     pub email: String,
/// }
///
/// #[handler]
/// pub async fn store(form: CreateUserRequest) -> Response {
///     // `form` is already validated - returns 422 if invalid
///     json_response!({ "email": form.email })
/// }
/// ```
///
/// ## Without parameters:
/// ```rust,ignore
/// #[handler]
/// pub async fn health_check() -> Response {
///     json_response!({ "status": "ok" })
/// }
/// ```
#[proc_macro_attribute]
pub fn handler(attr: TokenStream, input: TokenStream) -> TokenStream {
    handler::handler_impl(attr, input)
}

/// Attribute macro for POST-style action handlers that mutate state and redirect.
///
/// Transforms an async function returning `ActionResult` into a
/// `Response`-returning handler. On `Ok(())` emits a 303 redirect to
/// `redirect_to`; on `Err(ActionError)` emits a 303 redirect to `redirect_to`
/// (or `err.redirect_override` if set and same-origin) with a flash payload
/// and back-compat `?error=...&msg=...` query parameters.
///
/// # Required attributes
///
/// - `redirect_to = "<path>"` — the default 303 target on success.
///
/// # Optional attributes
///
/// - `method = "<METHOD>"` — HTTP method hint (default `"POST"`).
///
/// # Example
///
/// ```rust,ignore
/// use ferro::{action, ActionError, ActionResult, Request};
///
/// #[action(redirect_to = "/dashboard/pagine")]
/// pub async fn publish_by_id(req: Request) -> ActionResult {
///     let id: i64 = req.param("id")?.parse()?;
///     publish_page(id).await?;
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn action(attr: TokenStream, input: TokenStream) -> TokenStream {
    action::action_impl(attr, input)
}

/// Derive macro for FormRequest trait
///
/// Generates the `FormRequest` trait implementation for a struct.
/// The struct must also derive `serde::Deserialize` and `validator::Validate`.
///
/// For the cleanest DX, use the `#[request]` attribute macro instead,
/// which handles all derives automatically.
///
/// # Example
///
/// ```rust,ignore
/// use ferro::{FormRequest, Deserialize, Validate};
///
/// #[derive(Deserialize, Validate, FormRequest)]
/// pub struct CreateUserRequest {
///     #[validate(email)]
///     pub email: String,
///
///     #[validate(length(min = 8))]
///     pub password: String,
/// }
/// ```
#[proc_macro_derive(FormRequest)]
pub fn derive_form_request(input: TokenStream) -> TokenStream {
    request::derive_request_impl(input)
}

/// Attribute macro for clean request data definition
///
/// This is the recommended way to define validated request types.
/// It automatically adds the necessary derives and generates the trait impl.
///
/// Works with both:
/// - `application/json` - JSON request bodies
/// - `application/x-www-form-urlencoded` - HTML form submissions
///
/// # Example
///
/// ```rust,ignore
/// use ferro::request;
///
/// #[request]
/// pub struct CreateUserRequest {
///     #[validate(email)]
///     pub email: String,
///
///     #[validate(length(min = 8))]
///     pub password: String,
/// }
///
/// // This can now be used directly in handlers:
/// #[handler]
/// pub async fn store(form: CreateUserRequest) -> Response {
///     // Automatically validated - returns 422 with errors if invalid
///     json_response!({ "email": form.email })
/// }
/// ```
#[proc_macro_attribute]
pub fn request(attr: TokenStream, input: TokenStream) -> TokenStream {
    request::request_attr_impl(attr, input)
}

/// Attribute macro for database-enabled tests
///
/// This macro simplifies writing tests that need database access by automatically
/// setting up an in-memory SQLite database with migrations applied.
///
/// By default, it uses `crate::migrations::Migrator` as the migrator type,
/// following Ferro's convention for migration location.
///
/// # Examples
///
/// ## Basic usage (recommended):
/// ```rust,ignore
/// use ferro::ferro_test;
/// use ferro::testing::TestDatabase;
///
/// #[ferro_test]
/// async fn test_user_creation(db: TestDatabase) {
///     // db is an in-memory SQLite database with all migrations applied
///     // Any code using DB::connection() will use this test database
///     let action = CreateUserAction::new();
///     let user = action.execute("test@example.com").await.unwrap();
///     assert!(user.id > 0);
/// }
/// ```
///
/// ## Without TestDatabase parameter:
/// ```rust,ignore
/// #[ferro_test]
/// async fn test_action_without_direct_db_access() {
///     // Database is set up but not directly accessed
///     // Actions using DB::connection() still work
///     let action = MyAction::new();
///     action.execute().await.unwrap();
/// }
/// ```
///
/// ## With custom migrator:
/// ```rust,ignore
/// #[ferro_test(migrator = my_crate::CustomMigrator)]
/// async fn test_with_custom_migrator(db: TestDatabase) {
///     // Uses custom migrator instead of default
/// }
/// ```
#[proc_macro_attribute]
pub fn ferro_test(attr: TokenStream, input: TokenStream) -> TokenStream {
    ferro_test::ferro_test_impl(attr, input)
}

/// Group related tests with a descriptive name
///
/// Creates a module containing related tests, similar to Jest's describe blocks.
/// Supports nesting for hierarchical test organization.
///
/// # Example
///
/// ```rust,ignore
/// use ferro::{describe, test, expect};
/// use ferro::testing::TestDatabase;
///
/// describe!("ListTodosAction", {
///     test!("returns empty list when no todos exist", async fn(db: TestDatabase) {
///         let action = ListTodosAction::new();
///         let todos = action.execute().await.unwrap();
///         expect!(todos).to_be_empty();
///     });
///
///     // Nested describe for grouping related tests
///     describe!("with pagination", {
///         test!("returns first page", async fn(db: TestDatabase) {
///             // ...
///         });
///     });
/// });
/// ```
#[proc_macro]
pub fn describe(input: TokenStream) -> TokenStream {
    describe::describe_impl(input)
}

/// Define an individual test case with a descriptive name
///
/// Creates a test function with optional TestDatabase parameter.
/// The test name is displayed in failure output for easy identification.
///
/// # Examples
///
/// ## Async test with database
/// ```rust,ignore
/// test!("creates a user", async fn(db: TestDatabase) {
///     let user = CreateUserAction::new().execute("test@example.com").await.unwrap();
///     expect!(user.email).to_equal("test@example.com".to_string());
/// });
/// ```
///
/// ## Async test without database
/// ```rust,ignore
/// test!("calculates sum", async fn() {
///     let result = calculate_sum(1, 2).await;
///     expect!(result).to_equal(3);
/// });
/// ```
///
/// ## Sync test
/// ```rust,ignore
/// test!("adds numbers", fn() {
///     expect!(1 + 1).to_equal(2);
/// });
/// ```
///
/// On failure, the test name is shown:
/// ```text
/// Test: "creates a user"
///   at src/actions/user_action.rs:25
///
///   expect!(actual).to_equal(expected)
///
///   Expected: "test@example.com"
///   Received: "wrong@email.com"
/// ```
#[proc_macro]
pub fn test(input: TokenStream) -> TokenStream {
    test_macro::test_impl(input)
}

/// Derive macro for reducing SeaORM model boilerplate
///
/// Generates create builder, update builder, and convenience methods for Ferro models.
/// Apply to a SeaORM Model struct to get:
/// - `Model::query()` - Start a new QueryBuilder
/// - `Model::create()` - Get a builder for inserting new records
/// - `model.update()` - Get an UpdateBuilder for selective field updates
/// - `model.delete()` - Delete the record
///
/// # Example
///
/// ```rust,ignore
/// use ferro::FerroModel;
/// use sea_orm::entity::prelude::*;
///
/// #[derive(Clone, Debug, DeriveEntityModel, FerroModel)]
/// #[sea_orm(table_name = "users")]
/// pub struct Model {
///     #[sea_orm(primary_key)]
///     pub id: i32,
///     pub name: String,
///     pub email: String,
///     pub bio: Option<String>,
/// }
///
/// // Create a new record
/// let user = User::create()
///     .set_name("John")
///     .set_email("john@example.com")
///     .insert()
///     .await?;
///
/// // Update specific fields only (unchanged fields are not sent to DB)
/// let updated = user
///     .update()
///     .set_name("John Doe")
///     .set_bio("Developer")
///     .save()
///     .await?;
///
/// // Clear an optional field to NULL
/// let updated = updated
///     .update()
///     .clear_bio()
///     .save()
///     .await?;
///
/// // Query records
/// let users = User::query()
///     .filter(Column::Name.contains("John"))
///     .all()
///     .await?;
/// ```
#[proc_macro_derive(FerroModel)]
pub fn derive_ferro_model(input: TokenStream) -> TokenStream {
    model::ferro_model_impl(input)
}

/// Derive macro for declarative struct validation using Ferro's rules
///
/// Generates `Validatable` trait implementation from field attributes.
/// Validation rules are co-located with the struct definition.
///
/// This uses Ferro's Laravel-style validation rules (required(), email(), etc.)
/// rather than the external `validator` crate.
///
/// # Example
///
/// ```rust,ignore
/// use ferro::ValidateRules;
///
/// #[derive(ValidateRules)]
/// struct CreateUserRequest {
///     #[rule(required, email)]
///     email: String,
///
///     #[rule(required, min(8))]
///     password: String,
///
///     #[rule(required, integer, min(18))]
///     age: Option<i32>,
/// }
///
/// // Usage
/// let request = CreateUserRequest { ... };
/// request.validate()?;
/// ```
#[proc_macro_derive(ValidateRules, attributes(rule))]
pub fn derive_validate_rules(input: TokenStream) -> TokenStream {
    validate::validate_impl(input)
}

/// Derive macro for generating `Resource` trait implementation from struct annotations
///
/// Supports struct-level and field-level `#[resource(...)]` attributes:
///
/// - `#[resource(model = "path::to::Model")]` (struct-level) — generates `From<Model>` impl
/// - `#[resource(rename = "new_name")]` (field-level) — use a different key in JSON output
/// - `#[resource(skip)]` (field-level) — exclude field from JSON output
///
/// # Example
///
/// ```rust,ignore
/// use ferro::ApiResource;
///
/// #[derive(ApiResource)]
/// #[resource(model = "entities::users::Model")]
/// pub struct UserResource {
///     pub id: i32,
///     pub name: String,
///     #[resource(rename = "member_since")]
///     pub created_at: String,
///     #[resource(skip)]
///     pub password_hash: String,
/// }
/// ```
#[proc_macro_derive(ApiResource, attributes(resource))]
pub fn derive_api_resource(input: TokenStream) -> TokenStream {
    resource::api_resource_impl(input)
}

/// Attribute macro for GET handlers displaying a single tenant-scoped resource.
///
/// Folds id-extraction + tenant resolution + tenant-scoped lookup + 404-on-miss
/// into a single attribute. Tenant and resource remain real typed function
/// parameters; the user body moves to a named inner fn `__<name>_inner`.
///
/// # Required arguments
///
/// - First positional arg: the resource type implementing `TenantScoped`, e.g. `Customer`.
///
/// # Optional arguments
///
/// - `on_miss = "/url"` — redirect target on lookup miss; omitted → 404.
///   Supports `{id}` placeholder (substituted with the extracted resource id).
/// - `tenant = "expr"` — escape-hatch Rust expression for tenant resolution (default: `current_tenant()`).
/// - `find = "path::fn"` — override the lookup function (default: `TenantScoped::find_for_tenant`).
///
/// # Example
///
/// ```ignore
/// use ferro::{resource_get, Response, Request, TenantContext};
///
/// #[resource_get(Customer, on_miss = "/dashboard/clienti")]
/// pub async fn edit(req: &mut Request, tenant: &TenantContext, customer: &Customer) -> Response {
///     // customer is guaranteed to exist and belong to tenant
///     Ok(ferro::HttpResponse::new())
/// }
/// ```
///
/// # Expands to (abridged)
///
/// The attribute is equivalent to the following expansion (shown via `cargo expand`):
///
/// ```ignore
/// // Generated outer fn — accepts a raw Request, performs prelude, delegates.
/// pub async fn edit(__ferro_req: ::ferro::Request) -> ::ferro::Response {
///     let mut __ferro_req = __ferro_req;
///     let __resource_id: <Customer as ::ferro::TenantScoped>::Id =
///         __ferro_req.param_as("id").map_err(|_| ::ferro::HttpResponse::new().status(400))?;
///     let __tenant: ::ferro::TenantContext = ::ferro::current_tenant()
///         .ok_or_else(|| ::ferro::HttpResponse::new().status(400))?;
///     let __resource_opt = <Customer as ::ferro::TenantScoped>::find_for_tenant(
///         __resource_id, __tenant.id,
///     ).await.map_err(|_| ::ferro::HttpResponse::new().status(500))?;
///     let __resource = match __resource_opt {
///         Some(r) => r,
///         None => return Err(::ferro::HttpResponse::new().status(302).header("Location", "/dashboard/clienti")),
///     };
///     __edit_inner(&mut __ferro_req, &__tenant, &__resource).await
/// }
///
/// // Named inner fn — tenant and resource are real typed parameters; IDE jump-to-def works.
/// async fn __edit_inner(
///     req: &mut ::ferro::Request,
///     tenant: &::ferro::TenantContext,
///     customer: &Customer,
/// ) -> ::ferro::Response {
///     // user body here
///     Ok(ferro::HttpResponse::new())
/// }
/// ```
///
/// # Security
///
/// The generated lookup always calls `TenantScoped::find_for_tenant(id, tenant.id)` —
/// cross-tenant reads are structurally impossible through this macro. T-212-01.
#[proc_macro_attribute]
pub fn resource_get(attr: TokenStream, input: TokenStream) -> TokenStream {
    resource_get::resource_get_impl(attr, input)
}

/// Attribute macro for POST handlers mutating a single tenant-scoped resource.
///
/// Folds the same prelude as `#[resource_get]` plus the validation-failure
/// redirect envelope (via `handle_action_result`). Requires `redirect_to`.
///
/// # Required arguments
///
/// - First positional arg: the resource type implementing `TenantScoped`.
/// - `redirect_to = "/url"` — default 303 redirect on success (and error fallback).
///
/// # Optional arguments
///
/// - `form_url = "/url/{id}/edit"` — the edit form URL, synthesized from extracted
///   path params; injected as `__form_url: &str` in the inner fn body.
/// - `on_miss = "/url"` — 303 redirect on lookup miss; omitted → 404 `HttpResponse`.
/// - `tenant = "expr"` — escape-hatch expression for tenant resolution.
/// - `find = "path::fn"` — override the lookup function.
///
/// # Example
///
/// ```ignore
/// use ferro::{resource_post, ActionResult, Request, TenantContext};
///
/// #[resource_post(Customer,
///     redirect_to = "/dashboard/clienti",
///     form_url = "/dashboard/clienti/{id}/modifica")]
/// pub async fn save(req: &mut Request, tenant: &TenantContext, customer: &Customer) -> ActionResult {
///     let data = serde_json::json!({ "name": "test" });
///     ferro::Validator::new(&data)
///         .rules("name", ferro::rules![ferro::required()])
///         .validate_or_redirect(__form_url)?;
///     Ok(())
/// }
/// ```
///
/// # Expands to (abridged)
///
/// The attribute is equivalent to the following expansion (shown via `cargo expand`):
///
/// ```ignore
/// // Generated outer fn — same prelude as resource_get, plus form_url synthesis and
/// // validation-redirect envelope via handle_action_result.
/// pub async fn save(__ferro_req: ::ferro::Request) -> ::ferro::Response {
///     let mut __ferro_req = __ferro_req;
///     let __resource_id: <Customer as ::ferro::TenantScoped>::Id =
///         __ferro_req.param_as("id").map_err(|_| ::ferro::HttpResponse::new().status(400))?;
///     let __tenant: ::ferro::TenantContext = ::ferro::current_tenant()
///         .ok_or_else(|| ::ferro::HttpResponse::new().status(400))?;
///     let __resource_opt = <Customer as ::ferro::TenantScoped>::find_for_tenant(
///         __resource_id, __tenant.id,
///     ).await.map_err(|_| ::ferro::HttpResponse::new().status(500))?;
///     let __resource = match __resource_opt {
///         Some(r) => r,
///         None => return Err(::ferro::HttpResponse::new().status(404)),
///     };
///     let __form_url_owned = format!("/dashboard/clienti/{}/modifica", __resource_id);
///     let __form_url: &str = &__form_url_owned;
///     // Inner fn borrow ends before handle_action_result borrows __ferro_req again (Pitfall 3).
///     let __action_result: ::ferro::ActionResult =
///         __save_inner(&mut __ferro_req, &__tenant, &__resource, __form_url).await;
///     ::ferro::http::action::handle_action_result(
///         __action_result, "/dashboard/clienti", "module::save", &mut __ferro_req,
///     )
/// }
///
/// // Named inner fn — tenant, resource, and __form_url are real typed parameters.
/// async fn __save_inner(
///     req: &mut ::ferro::Request,
///     tenant: &::ferro::TenantContext,
///     customer: &Customer,
///     __form_url: &str,
/// ) -> ::ferro::ActionResult {
///     // user body here
///     Ok(())
/// }
/// ```
///
/// # Security
///
/// Same tenant-scoping guarantee as `#[resource_get]`: lookup always passes
/// `tenant.id`. T-212-01.
#[proc_macro_attribute]
pub fn resource_post(attr: TokenStream, input: TokenStream) -> TokenStream {
    resource_post::resource_post_impl(attr, input)
}
