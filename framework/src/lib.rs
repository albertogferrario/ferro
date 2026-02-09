pub mod app;
pub mod auth;
pub mod authorization;
pub mod cache;
pub mod config;
pub mod container;
pub mod csrf;
pub mod database;
pub mod debug;
pub mod error;
pub mod hashing;
pub mod http;
pub mod inertia;
pub mod json_ui;
pub mod metrics;
pub mod middleware;
pub mod routing;
pub mod schedule;
pub mod seeder;
pub mod server;
pub mod session;
pub mod testing;
pub mod validation;

pub use app::Application;
pub use auth::{Auth, AuthMiddleware, Authenticatable, GuestMiddleware, UserProvider};
pub use authorization::{AuthResponse, Authorizable, AuthorizationError, Authorize, Gate, Policy};
pub use cache::{Cache, CacheConfig, CacheStore, InMemoryCache, RedisCache};
pub use config::{env, env_optional, env_required, AppConfig, Config, Environment, ServerConfig};
pub use container::{App, Container};
pub use csrf::{csrf_field, csrf_meta_tag, csrf_token, CsrfMiddleware};
pub use database::{
    AutoRouteBinding, Database, DatabaseConfig, DatabaseType, DbConnection, Model, ModelMut,
    RouteBinding, DB,
};

// Re-export commonly used SeaORM traits for convenience
// This saves users from having to add `use sea_orm::*` imports
pub use error::{AppError, FrameworkError, HttpError, ValidationErrors};
pub use ferro_json_ui::{
    resolve_actions, resolve_actions_strict, resolve_path, resolve_path_string, Action,
    ActionOutcome, AlertProps, AlertVariant, AvatarProps, BadgeProps, BadgeVariant, BreadcrumbItem,
    BreadcrumbProps, ButtonProps, ButtonVariant, CardProps, CheckboxProps, Column, ColumnFormat,
    Component, ComponentNode, ConfirmDialog, DescriptionItem, DescriptionListProps, DialogVariant,
    FormProps, HttpMethod, IconPosition, InputProps, InputType, JsonUiConfig, JsonUiView,
    ModalProps, NotifyVariant, Orientation, PaginationProps, ProgressProps, SelectOption,
    SelectProps, SeparatorProps, Size, SkeletonProps, SortDirection, SwitchProps, Tab, TableProps,
    TabsProps, TextElement, TextProps, Visibility as JsonUiVisibility, VisibilityCondition,
    VisibilityOperator, SCHEMA_VERSION,
};
pub use hashing::{hash, needs_rehash, verify, DEFAULT_COST as HASH_DEFAULT_COST};
pub use http::{
    json, text, Cookie, CookieOptions, FormRequest, FromParam, FromRequest, HttpResponse,
    InertiaRedirect, Redirect, Request, Response, ResponseExt, SameSite,
};
pub use inertia::{Inertia, InertiaConfig, InertiaResponse, InertiaShared, SavedInertiaContext};
pub use json_ui::JsonUi;
pub use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};
pub use session::{
    session, session_mut, SessionConfig, SessionData, SessionMiddleware, SessionStore,
};
// Deprecated - kept for backward compatibility
#[allow(deprecated)]
pub use inertia::InertiaContext;
pub use metrics::{get_metrics, MetricsSnapshot, RouteMetrics, RouteMetricsView};
pub use middleware::{
    register_global_middleware, MetricsMiddleware, Middleware, MiddlewareFuture,
    MiddlewareRegistry, Next, RateLimitConfig, RateLimiter, RateLimiters, Throttle,
};
pub use routing::{
    // Internal functions used by macros (hidden from docs)
    __box_handler,
    __delete_impl,
    __fallback_impl,
    __get_impl,
    __post_impl,
    __put_impl,
    get_registered_routes,
    route,
    validate_route_path,
    FallbackDefBuilder,
    GroupBuilder,
    GroupDef,
    GroupItem,
    GroupRoute,
    GroupRouter,
    IntoGroupItem,
    ResourceAction,
    ResourceDef,
    ResourceRoute,
    RouteBuilder,
    RouteDefBuilder,
    RouteInfo,
    Router,
};
pub use schedule::{CronExpression, DayOfWeek, Schedule, Task, TaskBuilder, TaskEntry, TaskResult};
pub use seeder::{DatabaseSeeder, Seeder, SeederRegistry};
pub use server::Server;

// Re-export ferro-events for event-driven architecture
pub use ferro_events::{
    dispatch as dispatch_event, dispatch_sync, Error as EventError, Event, EventDispatcher,
    Listener, ShouldQueue,
};

// Re-export ferro-queue for background job processing
pub use ferro_queue::{
    dispatch as queue_dispatch, dispatch_later, dispatch_to, Error as QueueError, Job, JobPayload,
    PendingDispatch, Queue, QueueConfig, QueueConnection, Queueable, Worker, WorkerConfig,
};

// Re-export ferro-notifications for multi-channel notifications
pub use ferro_notifications::{
    Channel as NotificationChannel, ChannelResult, DatabaseMessage, DatabaseNotificationStore,
    Error as NotificationError, MailConfig, MailMessage, Notifiable, Notification,
    NotificationConfig, NotificationDispatcher, SlackAttachment, SlackField, SlackMessage,
    StoredNotification,
};

// Re-export ferro-broadcast for real-time WebSocket channels
pub use ferro_broadcast::{
    AuthData, Broadcast, BroadcastBuilder, BroadcastMessage, Broadcaster, ChannelAuthorizer,
    ChannelInfo, ChannelType, Client as BroadcastClient, ClientMessage, Error as BroadcastError,
    PresenceMember, ServerMessage,
};

// Re-export ferro-storage for file storage abstraction
pub use ferro_storage::{
    Disk, DiskConfig, DiskDriver, Error as StorageError, FileMetadata, LocalDriver,
    MemoryDriver as StorageMemoryDriver, PutOptions, Storage, StorageDriver, Visibility,
};

// Re-export ferro-cache for caching with tags
pub use ferro_cache::{
    Cache as TaggableCache, CacheConfig as TaggableCacheConfig, CacheStore as TaggableCacheStore,
    Error as TaggableCacheError, MemoryStore as TaggableCacheMemoryStore, TaggedCache,
};

// Re-export async_trait for middleware implementations
pub use async_trait::async_trait;

// Re-export inventory for #[service(ConcreteType)] macro
#[doc(hidden)]
pub use inventory;

// Re-export for macro usage
#[doc(hidden)]
pub use serde_json;

// Re-export serde for InertiaProps derive macro
pub use serde;

// Re-export validator crate for derive-based validation
pub use validator;
pub use validator::Validate;

// Re-export our Laravel-style validation module
pub use validation::{
    // Rules
    accepted,
    alpha,
    alpha_dash,
    alpha_num,
    array,
    between,
    boolean,
    confirmed,
    date,
    different,
    email,
    in_array,
    integer,
    max,
    min,
    not_in,
    nullable,
    numeric,
    regex,
    required,
    required_if,
    same,
    string,
    url,
    validate,
    Rule,
    Validatable,
    ValidationError,
    Validator,
};

// Re-export the proc-macros for compile-time component validation and type safety
pub use ferro_macros::domain_error;
pub use ferro_macros::ferro_test;
pub use ferro_macros::handler;
pub use ferro_macros::inertia_response;
pub use ferro_macros::injectable;
pub use ferro_macros::redirect;
pub use ferro_macros::request;
pub use ferro_macros::service;
pub use ferro_macros::FerroModel;
pub use ferro_macros::FormRequest as FormRequestDerive;
pub use ferro_macros::InertiaProps;
pub use ferro_macros::ValidateRules;

// Re-export Jest-like testing macros
pub use ferro_macros::describe;
pub use ferro_macros::test;

// Re-export testing utilities
pub use testing::{
    Factory, FactoryBuilder, Fake, Sequence, TestClient, TestContainer, TestContainerGuard,
    TestDatabase, TestRequestBuilder, TestResponse,
};

#[macro_export]
macro_rules! json_response {
    ($($json:tt)+) => {
        Ok($crate::HttpResponse::json($crate::serde_json::json!($($json)+)))
    };
}

#[macro_export]
macro_rules! text_response {
    ($text:expr) => {
        Ok($crate::HttpResponse::text($text))
    };
}

/// Register global middleware that runs on every request
///
/// Global middleware is registered in `bootstrap.rs` and runs in registration order,
/// before any route-specific middleware.
///
/// # Example
///
/// ```rust,ignore
/// // In bootstrap.rs
/// use ferro_rs::global_middleware;
/// use ferro_rs::middleware;
///
/// pub fn register() {
///     global_middleware!(middleware::LoggingMiddleware);
///     global_middleware!(middleware::CorsMiddleware);
/// }
/// ```
#[macro_export]
macro_rules! global_middleware {
    ($middleware:expr) => {
        $crate::register_global_middleware($middleware)
    };
}

/// Create an expectation for fluent assertions
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::expect;
///
/// expect!(actual).to_equal(expected);
/// expect!(result).to_be_ok();
/// expect!(vec).to_have_length(3);
/// ```
///
/// On failure, shows clear output:
/// ```text
/// Test: "returns all todos"
///   at src/actions/todo_action.rs:25
///
///   expect!(actual).to_equal(expected)
///
///   Expected: 0
///   Received: 3
/// ```
#[macro_export]
macro_rules! expect {
    ($value:expr) => {
        $crate::testing::Expect::new($value, concat!(file!(), ":", line!()))
    };
}
