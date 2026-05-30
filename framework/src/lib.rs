//! Ferro — a full-stack web framework for Rust.
//!
//! Provides routing, database access, validation, authentication, queues,
//! events, notifications, broadcasting, storage, caching, and Inertia.js
//! integration in a single cohesive package.
#![warn(missing_docs)]

/// API key management and OpenAPI specification generation.
pub mod api;
pub mod app;
pub mod auth;
pub mod authorization;
pub mod broadcast;
pub mod cache;
pub mod config;
pub mod container;
pub mod csrf;
pub mod database;
pub mod debug;
pub mod error;
pub mod hashing;
/// HTTP request, response, cookie, and resource types.
pub mod http;
#[cfg(feature = "inertia")]
pub mod inertia;
#[cfg(feature = "json-ui")]
pub mod json_ui;
pub mod lang;
pub mod metrics;
pub mod middleware;
/// Route definition and registration.
pub mod routing;
pub mod schedule;
pub mod seeder;
/// HTTP server builder and runner.
pub mod server;
pub mod session;
pub(crate) mod static_files;
pub mod tenant;
pub mod testing;
#[cfg(feature = "theme")]
pub mod theme;
pub mod validation;
mod websocket;

pub use api::api_key::{
    generate_api_key, hash_api_key, verify_api_key_hash, ApiKeyInfo, ApiKeyMiddleware,
    ApiKeyProvider, GeneratedApiKey,
};
pub use api::openapi::{
    build_openapi_spec, openapi_docs_response, openapi_json_response, OpenApiConfig,
};

pub use app::Application;
pub use auth::{
    Auth, AuthMiddleware, AuthUser, Authenticatable, GuestMiddleware, OptionalUser, UserProvider,
};
pub use authorization::{AuthResponse, Authorizable, AuthorizationError, Authorize, Gate, Policy};
pub use cache::{Cache, CacheConfig, CacheStore, InMemoryCache, RedisCache};
pub use config::{
    env, env_optional, env_required, AppConfig, Config, Environment, LangConfig, LangConfigBuilder,
    ServerConfig,
};
pub use container::{App, Container};
pub use csrf::{csrf_field, csrf_meta_tag, csrf_token, CsrfMiddleware};
pub use database::{
    AutoRouteBinding, Database, DatabaseConfig, DatabaseType, DbConnection, Model, ModelMut,
    RouteBinding, DB,
};
// Re-export utoipa and utoipa-redoc for advanced OpenAPI customization
pub use utoipa;
pub use utoipa_redoc;

// Re-export commonly used SeaORM traits for convenience
// This saves users from having to add `use sea_orm::*` imports
pub use error::{AppError, FrameworkError, HttpError, ValidationErrors};
#[cfg(feature = "json-ui")]
pub use ferro_json_ui::{
    resolve_actions, resolve_actions_strict, resolve_errors, resolve_errors_all, Action,
    ActionCardProps, ActionCardVariant, ActionOutcome, AlertProps, AlertVariant, AvatarProps,
    BadgeProps, BadgeVariant, BreadcrumbItem, BreadcrumbProps, ButtonProps, ButtonType,
    ButtonVariant, CardProps, CheckboxProps, ChecklistItem, ChecklistProps, Column, ColumnFormat,
    ConfirmDialog, DashboardLayout, DashboardLayoutConfig, DescriptionItem, DescriptionListProps,
    DialogVariant, Element, ElementBuilder, FormProps, HeaderProps, HttpMethod, IconPosition,
    ImageProps, InputProps, InputType, JsonUiConfig, Layout, LayoutContext, LayoutRegistry,
    ModalProps, NavItem, NotificationDropdownProps, NotificationItem, NotifyVariant, Orientation,
    PaginationProps, ProgressProps, SelectOption, SelectProps, SeparatorProps, SidebarGroup,
    SidebarNavItem, SidebarProps, SidebarSection, Size, SkeletonProps, SortDirection, Spec,
    SpecBuilder, SpecError, StatCardProps, SwitchProps, Tab, TableProps, TabsProps, TextElement,
    TextProps, ToastProps, ToastVariant, Visibility as JsonUiVisibility, VisibilityCondition,
    VisibilityOperator, MAX_NESTING_DEPTH, SCHEMA_VERSION,
};
#[cfg(feature = "stripe")]
pub use ferro_stripe::{
    account, checkout, refund, verify_webhook, CheckoutBuilder, CheckoutIntent,
    Error as StripeError, LineItem, MemoryProcessedLog, Mode, ProcessStripeWebhook,
    ProcessedEventLog, Stripe, StripeChargeDisputeCreated, StripeChargeRefunded,
    StripeCheckoutCompleted, StripeCheckoutExpired, StripeConfig, StripeConnectAccountUpdated,
    StripeConnectPaymentSucceeded, StripeEvent, StripeInvoicePaid, StripePaymentIntentFailed,
    StripeSubscriptionDeleted, StripeSubscriptionUpdated, SyncDispatcher,
};
#[cfg(feature = "theme")]
pub use ferro_theme::{IntentModeTemplates, IntentSlotTemplate, Theme, ThemeError, ThemeTemplates};
pub use hashing::{hash, needs_rehash, verify, DEFAULT_COST as HASH_DEFAULT_COST};
pub use http::action::{
    ActionError, ActionKind, ActionResult, ActionResultExt, FlashVariant, IntoActionError,
};
pub use http::{
    bytes, json, request_host, text, validate_mime, validate_size, Cookie, CookieOptions,
    FormRequest, FromParam, FromRequest, HttpResponse, InertiaRedirect, MultipartForm,
    PaginationLinks, PaginationMeta, Redirect, Request, Resource, ResourceCollection, ResourceMap,
    Response, ResponseExt, SameSite, UploadedFile,
};
#[cfg(feature = "inertia")]
pub use inertia::{Inertia, InertiaConfig, InertiaResponse, InertiaShared, SavedInertiaContext};
#[cfg(feature = "json-ui")]
pub use json_ui::JsonUi;
pub use lang::{lang_choice, lang_init, locale, set_locale, t, trans, LangMiddleware};
pub use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};
pub use session::{
    invalidate_all_for_user, session, session_mut, DatabaseSessionDriver, SessionConfig,
    SessionData, SessionMiddleware, SessionStore,
};
#[cfg(feature = "stripe")]
pub use tenant::RequiresPlan;
pub use tenant::{
    current_tenant, DbTenantLookup, FrameworkTenantScopeProvider, HeaderResolver, JwtClaimResolver,
    PathResolver, SubdomainResolver, TenantContext, TenantFailureMode, TenantLookup,
    TenantMiddleware, TenantResolver, TenantScope,
};
#[cfg(feature = "theme")]
pub use theme::{
    current_theme, DefaultResolver, HeaderThemeResolver, TenantThemeResolver, ThemeMiddleware,
    ThemeResolver,
};
// Deprecated - kept for backward compatibility
#[cfg(feature = "inertia")]
#[allow(deprecated)]
pub use inertia::InertiaContext;
pub use metrics::{get_metrics, MetricsSnapshot, RouteMetrics, RouteMetricsView};
pub use middleware::{
    get_pre_route_middleware, register_global_middleware, register_pre_route_middleware,
    rewrite_request_path, Cors, Limit, LimiterResponse, MetricsMiddleware, Middleware,
    MiddlewareFuture, MiddlewareRegistry, Next, PreRouteMiddleware, PreRouteResult, RateLimiter,
    SecurityHeaders, Throttle,
};
pub use routing::{
    // Internal functions used by macros (hidden from docs)
    __box_handler,
    __delete_impl,
    __fallback_impl,
    __get_impl,
    __patch_impl,
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
    dispatch as queue_dispatch, dispatch_later, dispatch_to, register_tenant_capture_hook,
    Error as QueueError, Job, JobPayload, PendingDispatch, Queue, QueueConfig, QueueConnection,
    Queueable, TenantScopeProvider, Worker, WorkerConfig,
};

// Re-export ferro-notifications for multi-channel notifications
pub use ferro_notifications::{
    Channel as NotificationChannel, ChannelResult, DatabaseMessage, DatabaseNotificationStore,
    Error as NotificationError, InAppConfig, InAppMessage, InAppSeverity, MailAttachment,
    MailConfig, MailDriver, MailMessage, Notifiable, Notification, NotificationConfig,
    NotificationDispatcher, PushMessage, ResendConfig, SlackAttachment, SlackField, SlackMessage,
    SmsMessage, SmtpConfig, StoredNotification, WhatsAppMessage,
};

// Re-export ferro-broadcast for real-time WebSocket channels
pub use ferro_broadcast::{
    AuthData, Broadcast, BroadcastBuilder, BroadcastConfig, BroadcastMessage, Broadcaster,
    ChannelAuthorizer, ChannelInfo, ChannelType, Client as BroadcastClient, ClientMessage,
    Error as BroadcastError, PresenceMember, ServerMessage,
};

// Re-export broadcasting auth handler
pub use broadcast::broadcasting_auth;

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

// Re-export ferro-lang for localization
pub use ferro_lang::{LangError, Translator};

// Re-export ferro-ai for AI classification and confirmation primitives
#[cfg(feature = "ai")]
pub use ferro_ai::{
    AnthropicProvider, ClassificationProvider, ClassificationResult, Classifier, ClassifierConfig,
    ConfirmationExpired, ConfirmationStore, Error as AiError, InMemoryConfirmationStore,
    PendingActionInfo,
};

// Re-export ferro-whatsapp for WhatsApp Business Cloud API integration
#[cfg(feature = "whatsapp")]
pub use ferro_whatsapp::{
    verify_whatsapp_webhook, DeduplicationStore, DeliveryStatus, Error as WhatsAppError,
    InMemoryDeduplicationStore, Message as WhatsAppRawMessage, ProcessWhatsAppWebhook,
    SendResult as WhatsAppSendResult, SenderIdentity, WhatsApp, WhatsAppConfig,
    WhatsAppStatusUpdate, WhatsAppTextReceived,
};

// Re-export ferro-projections for service projection definitions
#[cfg(feature = "projections")]
pub use ferro_projections::{
    derive_intents, infer_meaning, ActionDef, Cardinality, DataType, Error as ProjectionsError,
    FieldDef, FieldMeaning, GuardDef, InputDef, Intent, IntentHint, IntentScore, NavigationHint,
    RelationshipDef, Renderer, ServiceDef, StateDef, StateMachine, Transition,
    Warning as ProjectionsWarning,
};
// Re-export visual renderer types from ferro-json-ui
#[cfg(feature = "projections")]
pub use ferro_json_ui::{JsonUiRenderer, RenderMode, VisualContext};

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
    // Bridge
    register_validation_translator,
    required,
    required_if,
    same,
    string,
    url,
    validate,
    Rule,
    TranslatorFn,
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
pub use ferro_macros::ApiResource;
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

/// Return a JSON response from a handler using `serde_json::json!` syntax.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::json_response;
///
/// pub async fn index() -> Response {
///     json_response!({ "status": "ok" })
/// }
/// ```
#[macro_export]
macro_rules! json_response {
    ($($json:tt)+) => {
        Ok($crate::HttpResponse::json($crate::serde_json::json!($($json)+)))
    };
}

/// Return a plain-text response from a handler.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::text_response;
///
/// pub async fn ping() -> Response {
///     text_response!("pong")
/// }
/// ```
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

/// Register a pre-route middleware that runs before path extraction and route matching.
///
/// Pre-route middleware operates on the raw hyper request and can rewrite the path
/// (via `rewrite_request_path`) before the router selects a handler. Use this for
/// host-based routing, path aliasing, or any rewrite that must influence which
/// route is matched. Runs in registration order, before standard global middleware.
///
/// # Example
///
/// ```rust,ignore
/// // In bootstrap.rs
/// pre_route_middleware!(middleware::host::HostMiddleware::new());
/// ```
#[macro_export]
macro_rules! pre_route_middleware {
    ($middleware:expr) => {
        $crate::register_pre_route_middleware($middleware)
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
