// Make command templates (used by `make:*` commands)

/// Template for generating new middleware with make:middleware command
pub fn middleware_template(name: &str, struct_name: &str) -> String {
    format!(
        r#"//! {name} middleware

use ferro::{{async_trait, Middleware, Next, Request, Response}};

/// {name} middleware
pub struct {struct_name};

#[async_trait]
impl Middleware for {struct_name} {{
    async fn handle(&self, request: Request, next: Next) -> Response {{
        // TODO: Implement middleware logic
        next(request).await
    }}
}}
"#
    )
}

/// Template for generating new resource with make:resource command
pub fn resource_template(name: &str, model: Option<&str>) -> String {
    let model_attribute = match model {
        Some(path) => format!("#[resource(model = \"{path}\")]\n"),
        None => String::new(),
    };

    format!(
        r#"use ferro::{{ApiResource, Resource, ResourceMap, Request}};

#[derive(ApiResource)]
{model_attribute}pub struct {name} {{
    pub id: i64,
    // Add fields from your model here
    // #[resource(rename = "display_name")]
    // pub name: String,
    // #[resource(skip)]
    // pub password_hash: String,
}}
"#
    )
}

/// Template for generating new controller with make:controller command
pub fn controller_template(name: &str) -> String {
    format!(
        r#"//! {name} controller

use ferro::{{handler, json_response, Request, Response}};

#[handler]
pub async fn invoke(_req: Request) -> Response {{
    json_response!({{
        "controller": "{name}"
    }})
}}
"#
    )
}

/// Template for generating new action with make:action command
pub fn action_template(name: &str, struct_name: &str) -> String {
    format!(
        r#"//! {name} action

use ferro::injectable;

#[injectable]
pub struct {struct_name} {{
    // Dependencies injected via container
}}

impl {struct_name} {{
    pub fn execute(&self) {{
        // TODO: Implement action logic
    }}
}}
"#
    )
}

/// Template for generating new Inertia page with make:inertia command
pub fn inertia_page_template(component_name: &str) -> String {
    format!(
        r#"export default function {component_name}() {{
  return (
    <div className="font-sans p-8 max-w-xl mx-auto">
      <h1 className="text-3xl font-bold">{component_name}</h1>
      <p className="mt-2">
        Edit <code className="bg-muted px-1 rounded">frontend/src/pages/{component_name}.tsx</code> to get started.
      </p>
    </div>
  )
}}
"#
    )
}

/// Template for generating a JSON-UI view file (--no-ai fallback).
pub fn json_view_template(name: &str, title: &str, layout: &str) -> String {
    format!(
        r#"//! {title} JSON-UI view

use ferro::{{
    ComponentNode, Component, CardProps, JsonUiView, TextElement, TextProps,
}};

/// Build the {title} view.
pub fn view() -> JsonUiView {{
    JsonUiView::new()
        .title("{title}")
        .layout("{layout}")
        .component(ComponentNode {{
            key: "heading".to_string(),
            component: Component::Text(TextProps {{
                content: "{title}".to_string(),
                element: TextElement::H1,
            }}),
            action: None,
            visibility: None,
        }})
        .component(ComponentNode {{
            key: "card".to_string(),
            component: Component::Card(CardProps {{
                title: "{title}".to_string(),
                description: Some("Edit src/views/{name}.rs to customize this view.".to_string()),
                children: vec![],
                footer: vec![],
            }}),
            action: None,
            visibility: None,
        }})
}}
"#,
    )
}

/// Template for generating new error with make:error command
pub fn error_template(struct_name: &str) -> String {
    // Convert PascalCase to human readable message
    let mut message = String::new();
    for (i, c) in struct_name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            message.push(' ');
            message.push(c.to_lowercase().next().unwrap());
        } else {
            message.push(c);
        }
    }

    format!(
        r#"//! {struct_name} error

use ferro::domain_error;

#[domain_error(status = 500, message = "{message}")]
pub struct {struct_name};
"#
    )
}

/// Template for generating new scheduled task with make:task command
pub fn task_template(file_name: &str, struct_name: &str) -> String {
    format!(
        r#"//! {struct_name} scheduled task
//!
//! Created with `ferro make:task {file_name}`

use async_trait::async_trait;
use ferro::{{Task, TaskResult}};

/// {struct_name} - A scheduled task
///
/// Implement your task logic in the `handle()` method.
/// Register this task in `src/schedule.rs` with the fluent API.
///
/// # Example Registration
///
/// ```rust,ignore
/// // In src/schedule.rs
/// use crate::tasks::{file_name};
///
/// schedule.add(
///     schedule.task({struct_name}::new())
///         .daily()
///         .at("03:00")
///         .name("{file_name}")
///         .description("TODO: Add task description")
/// );
/// ```
pub struct {struct_name};

impl {struct_name} {{
    /// Create a new instance of this task
    pub fn new() -> Self {{
        Self
    }}
}}

impl Default for {struct_name} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[async_trait]
impl Task for {struct_name} {{
    async fn handle(&self) -> TaskResult {{
        // TODO: Implement your task logic here
        println!("Running {struct_name}...");
        Ok(())
    }}
}}
"#
    )
}

// Event templates

/// Template for generating new events with make:event command
pub fn event_template(file_name: &str, struct_name: &str) -> String {
    format!(
        r#"//! {struct_name} event
//!
//! Created with `ferro make:event {file_name}`

use ferro_events::Event;
use serde::{{Deserialize, Serialize}};

/// {struct_name} - A domain event
///
/// Events represent something that has happened in your application.
/// Listeners can react to these events asynchronously.
///
/// # Dispatching
///
/// ```rust,ignore
/// use crate::events::{file_name}::{struct_name};
///
/// // Ergonomic dispatch (awaits all listeners)
/// {struct_name} {{ /* fields */ }}.dispatch().await?;
///
/// // Fire and forget (spawns background task)
/// {struct_name} {{ /* fields */ }}.dispatch_sync();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {struct_name} {{
    // TODO: Add event data fields
    // pub user_id: i64,
    // pub created_at: chrono::DateTime<chrono::Utc>,
}}

impl Event for {struct_name} {{
    fn name(&self) -> &'static str {{
        "{struct_name}"
    }}
}}
"#
    )
}

/// Template for events/mod.rs
pub fn events_mod() -> &'static str {
    r#"//! Application events
//!
//! This module contains domain events that can be dispatched
//! and handled by listeners.

"#
}

// Listener templates

/// Template for generating new listeners with make:listener command
pub fn listener_template(file_name: &str, struct_name: &str, event_type: &str) -> String {
    format!(
        r#"//! {struct_name} listener
//!
//! Created with `ferro make:listener {file_name}`

use ferro_events::{{async_trait, Error, Listener}};
// TODO: Import the event type
// use crate::events::your_event::YourEvent;

/// {struct_name} - An event listener
///
/// Listeners react to events and perform side effects.
/// They can be synchronous or queued for background processing.
///
/// # Example Registration
///
/// ```rust,ignore
/// // In your app initialization
/// use ferro_events::EventDispatcher;
/// use crate::listeners::{file_name}::{struct_name};
///
/// let mut dispatcher = EventDispatcher::new();
/// dispatcher.listen::<{event_type}, _>({struct_name});
/// ```
pub struct {struct_name};

#[async_trait]
impl Listener<{event_type}> for {struct_name} {{
    async fn handle(&self, event: &{event_type}) -> Result<(), Error> {{
        // TODO: Implement listener logic
        tracing::info!("{struct_name} handling event: {{:?}}", event);
        Ok(())
    }}
}}
"#
    )
}

/// Template for listeners/mod.rs
pub fn listeners_mod() -> &'static str {
    r#"//! Application event listeners
//!
//! This module contains listeners that react to domain events.

"#
}

// Job templates

/// Template for generating new jobs with make:job command
pub fn job_template(file_name: &str, struct_name: &str) -> String {
    format!(
        r#"//! {struct_name} background job
//!
//! Created with `ferro make:job {file_name}`

use ferro_queue::{{async_trait, Error, Job, Queueable}};
use serde::{{Deserialize, Serialize}};

/// {struct_name} - A background job
///
/// Jobs are queued for background processing by workers.
/// They support retries, delays, and queue prioritization.
///
/// # Example
///
/// ```rust,ignore
/// use crate::jobs::{file_name}::{struct_name};
///
/// // Dispatch immediately
/// {struct_name} {{ /* fields */ }}.dispatch().await?;
///
/// // Dispatch with delay
/// {struct_name} {{ /* fields */ }}
///     .delay(std::time::Duration::from_secs(60))
///     .dispatch()
///     .await?;
///
/// // Dispatch to specific queue
/// {struct_name} {{ /* fields */ }}
///     .on_queue("high-priority")
///     .dispatch()
///     .await?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {struct_name} {{
    // TODO: Add job data fields
    // pub user_id: i64,
    // pub payload: String,
}}

#[async_trait]
impl Job for {struct_name} {{
    async fn handle(&self) -> Result<(), Error> {{
        // TODO: Implement job logic
        tracing::info!("Processing {struct_name}: {{:?}}", self);
        Ok(())
    }}

    fn max_retries(&self) -> u32 {{
        3
    }}

    fn retry_delay(&self, attempt: u32) -> std::time::Duration {{
        // Exponential backoff
        std::time::Duration::from_secs(2u64.pow(attempt))
    }}
}}
"#
    )
}

/// Template for jobs/mod.rs
pub fn jobs_mod() -> &'static str {
    r#"//! Application background jobs
//!
//! This module contains jobs that are processed asynchronously
//! by queue workers.

"#
}

// Notification templates

/// Template for generating new notifications with make:notification command
pub fn notification_template(file_name: &str, struct_name: &str) -> String {
    format!(
        r#"//! {struct_name} notification
//!
//! Created with `ferro make:notification {file_name}`

use ferro_notifications::{{Channel, DatabaseMessage, MailMessage, Notification}};

/// {struct_name} - A multi-channel notification
///
/// Notifications can be sent through multiple channels:
/// - Mail: Email via SMTP
/// - Database: In-app notifications
/// - Slack: Webhook messages
///
/// # Example
///
/// ```rust,ignore
/// use crate::notifications::{file_name}::{struct_name};
///
/// // Send notification to a user
/// user.notify({struct_name} {{ /* fields */ }}).await?;
/// ```
pub struct {struct_name} {{
    // TODO: Add notification data fields
    // pub order_id: i64,
    // pub tracking_number: String,
}}

impl Notification for {struct_name} {{
    fn via(&self) -> Vec<Channel> {{
        // TODO: Choose notification channels
        vec![Channel::Mail, Channel::Database]
    }}

    fn to_mail(&self) -> Option<MailMessage> {{
        Some(MailMessage::new()
            .subject("{struct_name}")
            .body("TODO: Add notification message"))
    }}

    fn to_database(&self) -> Option<DatabaseMessage> {{
        Some(DatabaseMessage::new("{file_name}")
            // TODO: Add notification data
            // .data("order_id", self.order_id)
        )
    }}
}}
"#
    )
}

/// Template for notifications/mod.rs
pub fn notifications_mod() -> &'static str {
    r#"//! Application notifications
//!
//! This module contains notifications that can be sent
//! through multiple channels (mail, database, slack, etc.).

"#
}

// Seeder templates

/// Template for generating new seeder with make:seeder command
pub fn seeder_template(file_name: &str, struct_name: &str) -> String {
    format!(
        r#"//! {struct_name} database seeder
//!
//! Created with `ferro make:seeder {file_name}`

use ferro::{{async_trait, FrameworkError, Seeder}};
use sea_orm::DatabaseConnection;

/// {struct_name} - A database seeder
///
/// Seeders populate the database with test or initial data.
/// Implement the `run` method to insert records.
///
/// # Example Registration
///
/// ```rust,ignore
/// // In src/seeders/mod.rs
/// use ferro::SeederRegistry;
/// use super::{file_name}::{struct_name};
///
/// pub fn register() -> SeederRegistry {{
///     SeederRegistry::new()
///         .add::<{struct_name}>()
/// }}
/// ```
#[derive(Default)]
pub struct {struct_name};

#[async_trait]
impl Seeder for {struct_name} {{
    async fn run(&self, db: &DatabaseConnection) -> Result<(), FrameworkError> {{
        // TODO: Implement seeder logic using `db`
        // Example:
        // use sea_orm::{{ActiveModelTrait, ActiveValue::Set}};
        // users::ActiveModel {{ name: Set("Admin".into()), ..Default::default() }}
        //     .insert(db).await?;

        Ok(())
    }}
}}
"#
    )
}

/// Template for seeders/mod.rs
pub fn seeders_mod() -> &'static str {
    r#"//! Database seeders
//!
//! This module contains seeders that populate the database with test
//! or initial data.
//!
//! # Usage
//!
//! Register seeders in the `register()` function and run with:
//! ```bash
//! ./target/debug/app db:seed           # Run all seeders
//! ./target/debug/app db:seed --class UsersSeeder  # Run specific seeder
//! ```

use ferro::SeederRegistry;

/// Register all seeders
///
/// Add your seeders here in the order you want them to run.
/// Seeders are executed in registration order.
pub fn register() -> SeederRegistry {
    SeederRegistry::new()
        // .add::<UsersSeeder>()
        // .add::<ProductsSeeder>()
}
"#
}

/// Template for generating new factory with make:factory command
pub fn factory_template(file_name: &str, struct_name: &str, model_name: &str) -> String {
    format!(
        r#"//! {struct_name} factory
//!
//! Created with `ferro make:factory {file_name}`

use ferro::testing::{{Factory, FactoryTraits, Fake}};
// use ferro::testing::DatabaseFactory;
// use crate::models::{model_name};

/// Factory for creating {model_name} instances in tests
#[derive(Clone)]
pub struct {struct_name} {{
    // Add fields matching your model
    pub id: i64,
    pub name: String,
    pub email: String,
    pub created_at: String,
}}

impl Factory for {struct_name} {{
    fn definition() -> Self {{
        Self {{
            id: 0, // Will be set by database
            name: Fake::name(),
            email: Fake::email(),
            created_at: Fake::datetime(),
        }}
    }}

    fn traits() -> FactoryTraits<Self> {{
        FactoryTraits::new()
            // .define("admin", |m: &mut Self| m.role = "admin".to_string())
            // .define("verified", |m: &mut Self| m.verified = true)
    }}
}}

// Uncomment to enable database persistence with create():
//
// #[ferro::async_trait]
// impl DatabaseFactory for {struct_name} {{
//     type Entity = {model_name}::Entity;
//     type ActiveModel = {model_name}::ActiveModel;
// }}

// Usage in tests:
//
// // Make without persisting:
// let model = {struct_name}::factory().make();
//
// // Apply named trait:
// let admin = {struct_name}::factory().trait_("admin").make();
//
// // With inline state:
// let model = {struct_name}::factory()
//     .state(|m| m.name = "Custom".into())
//     .make();
//
// // Create with database persistence:
// let model = {struct_name}::factory().create().await?;
//
// // Create multiple:
// let models = {struct_name}::factory().count(5).create_many().await?;
"#
    )
}

/// Template for factories/mod.rs
pub fn factories_mod() -> &'static str {
    r#"//! Test factories
//!
//! This module contains factories for generating fake model data in tests.
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::factories::UserFactory;
//! use ferro::testing::Factory;
//!
//! // Make without persisting
//! let user = UserFactory::factory().make();
//!
//! // Create with database persistence
//! let user = UserFactory::factory().create().await?;
//!
//! // Create multiple
//! let users = UserFactory::factory().count(5).create_many().await?;
//! ```

"#
}

/// Template for generating new policy with make:policy command
pub fn policy_template(file_name: &str, struct_name: &str, model_name: &str) -> String {
    format!(
        r#"//! {struct_name} authorization policy
//!
//! Created with `ferro make:policy {file_name}`

use ferro::authorization::{{AuthResponse, Policy}};
// TODO: Import your model and user types
// use crate::models::{model_name}::{{self, Model as {model_name}}};
// use crate::models::users::Model as User;

/// {struct_name} - Authorization policy for {model_name}
///
/// This policy defines who can perform actions on {model_name} records.
///
/// # Example Usage
///
/// ```rust,ignore
/// use crate::policies::{file_name}::{struct_name};
///
/// let policy = {struct_name};
///
/// // Check if user can update the model
/// if policy.update(&user, &model).allowed() {{
///     // Proceed with update
/// }}
///
/// // Use the check method for string-based ability lookup
/// let response = policy.check(&user, "update", Some(&model));
/// ```
pub struct {struct_name};

impl Policy<{model_name}> for {struct_name} {{
    type User = User;

    /// Run before any other authorization checks.
    ///
    /// Return `Some(true)` to allow, `Some(false)` to deny,
    /// or `None` to continue to the specific ability check.
    fn before(&self, user: &Self::User, _ability: &str) -> Option<bool> {{
        // Example: Admin bypass
        // if user.is_admin {{
        //     return Some(true);
        // }}
        None
    }}

    /// Determine whether the user can view any models.
    fn view_any(&self, _user: &Self::User) -> AuthResponse {{
        // TODO: Implement authorization logic
        AuthResponse::allow()
    }}

    /// Determine whether the user can view the model.
    fn view(&self, _user: &Self::User, _model: &{model_name}) -> AuthResponse {{
        // TODO: Implement authorization logic
        AuthResponse::allow()
    }}

    /// Determine whether the user can create models.
    fn create(&self, _user: &Self::User) -> AuthResponse {{
        // TODO: Implement authorization logic
        AuthResponse::allow()
    }}

    /// Determine whether the user can update the model.
    fn update(&self, user: &Self::User, model: &{model_name}) -> AuthResponse {{
        // TODO: Implement authorization logic
        // Example: Only owner can update
        // if user.auth_identifier() == model.user_id as i64 {{
        //     AuthResponse::allow()
        // }} else {{
        //     AuthResponse::deny("You do not own this resource.")
        // }}
        AuthResponse::deny_silent()
    }}

    /// Determine whether the user can delete the model.
    fn delete(&self, user: &Self::User, model: &{model_name}) -> AuthResponse {{
        // Same as update by default
        self.update(user, model)
    }}

    /// Determine whether the user can restore the model.
    fn restore(&self, user: &Self::User, model: &{model_name}) -> AuthResponse {{
        self.update(user, model)
    }}

    /// Determine whether the user can permanently delete the model.
    fn force_delete(&self, user: &Self::User, model: &{model_name}) -> AuthResponse {{
        // Usually more restrictive than delete
        self.delete(user, model)
    }}
}}

// TODO: Uncomment and define placeholder types until you import the real ones
// struct {model_name};
// struct User;
// impl ferro::auth::Authenticatable for User {{
//     fn auth_identifier(&self) -> i64 {{ 0 }}
//     fn as_any(&self) -> &dyn std::any::Any {{ self }}
// }}
"#
    )
}

// Lang templates

/// Template for lang/{locale}/validation.json (English validation messages)
pub fn lang_validation_json() -> &'static str {
    include_str!("files/lang/validation.json.tpl")
}

/// Template for lang/{locale}/app.json (starter application translations)
pub fn lang_app_json() -> &'static str {
    include_str!("files/lang/app.json.tpl")
}

/// Template for policies/mod.rs
pub fn policies_mod() -> &'static str {
    r#"//! Authorization policies
//!
//! This module contains policies that define who can perform actions
//! on specific models or resources.
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::policies::PostPolicy;
//! use ferro::authorization::Policy;
//!
//! let policy = PostPolicy;
//!
//! // Check authorization
//! if policy.update(&user, &post).allowed() {
//!     // Proceed with update
//! }
//!
//! // Or use the generic check method
//! let response = policy.check(&user, "update", Some(&post));
//! ```

"#
}
