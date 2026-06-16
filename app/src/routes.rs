use ferro::{get, group, post, resource, routes};
use ferro::{AuthMiddleware as SessionAuthMiddleware, GuestMiddleware};
use ferro::{JwtClaimResolver, TenantFailureMode, TenantMiddleware};
use ferro_mcp_oauth::handlers::{
    authorization_server_handler, authorize_get, authorize_post, device_authorization,
    device_verification_get, device_verification_post, protected_resource_handler, register_client,
    token_exchange,
};
use ferro_mcp_server::McpServerConfig;

use crate::api::docs::docs_routes;
use crate::api::routes::api_routes;
use crate::controllers;
use crate::middleware::bearer_auth::BearerAuthMiddleware;
use crate::middleware::AuthMiddleware;
use crate::tenant_resolver::SessionUserTenantResolver;

routes! {
    get!("/", controllers::home::index).name("home"),
    get!("/redirect-example", controllers::user::redirect_example),
    get!("/config", controllers::config_example::show).name("config.show"),
    get!("/pagamenti", controllers::pagamenti::index).name("pagamenti.index"),

    // User routes - all 7 RESTful endpoints from a single line
    resource!("/users", controllers::user),

    // Protected routes - requires Authorization header
    group!("/protected", {
        get!("/", controllers::home::index).name("protected.home"),
    }).middleware(AuthMiddleware),

    // Todo routes group
    group!("/todos", {
        get!("/", controllers::todo::list).name("todos.index"),
        post!("/random", controllers::todo::create_random).name("todos.create_random"),
    }),

    // Auth routes - guest only (redirects authenticated users)
    group!("/auth", {
        get!("/login", controllers::auth_controller::login_page).name("auth.login.page"),
        get!("/verify", controllers::auth_controller::verify_magic_link).name("auth.verify"),
        post!("/register", controllers::auth_controller::register).name("auth.register"),
        post!("/login", controllers::auth_controller::login).name("auth.login"),
    }).middleware(GuestMiddleware::redirect_to("/")),

    // Auth routes - authenticated only
    group!("/auth", {
        get!("/profile", controllers::auth_controller::profile).name("auth.profile"),
        post!("/logout", controllers::auth_controller::logout).name("auth.logout"),
    }).middleware(SessionAuthMiddleware::new()),

    // MCP Streamable HTTP endpoint.
    // Middleware order: BearerAuthMiddleware (inserts serde_json::Value principal)
    //                  → TenantMiddleware(JwtClaimResolver) (reads claims, sets current_tenant()).
    // Bearer MUST run before Tenant — JwtClaimResolver reads the inserted serde_json::Value.
    // Failure mode: Forbidden — unknown tenant_id in a validated token → 403, not 404 (Pitfall 6).
    group!("/", {
        post!("/mcp", controllers::mcp::handle).name("mcp.endpoint"),
        get!("/mcp", controllers::mcp::method_not_allowed).name("mcp.endpoint.get"),
        post!("/mcp/chat", controllers::mcp_chat::handle_chat).name("mcp.chat"),
    }).middleware(BearerAuthMiddleware {
        mcp_config: McpServerConfig::from_env(),
    }).middleware(
        TenantMiddleware::new()
            .resolver(JwtClaimResolver::new("tenant_id", crate::tenant_lookup::get()))
            .on_failure(TenantFailureMode::Forbidden),
    ),

    // Authorization + consent endpoint — TenantMiddleware with session-user resolver so
    // minted tokens carry a real tenant_id (D-07). Failure mode: Allow — a not-yet-logged-in
    // visitor must still reach the login redirect; token gets tenant_id=None on the
    // single-tenant path, which is safe.
    group!("/", {
        get!("/authorize", authorize_get),
        post!("/authorize", authorize_post),
    }).middleware(
        TenantMiddleware::new()
            .resolver(SessionUserTenantResolver::new())
            .on_failure(TenantFailureMode::Allow),
    ),

    // OAuth discovery (public, no middleware)
    get!("/.well-known/oauth-protected-resource", protected_resource_handler),
    get!("/.well-known/oauth-authorization-server", authorization_server_handler),

    // Dynamic Client Registration (public)
    post!("/register", register_client),

    // Token exchange (public, no session needed)
    post!("/token", token_exchange),

    // Device Authorization Grant (RFC 8628) — public (no session, like /register and /token)
    post!("/device_authorization", device_authorization),

    // Device verification page — session + tenant (like the /authorize group).
    // TenantFailureMode::Allow so an unauthenticated visitor reaches the handler
    // for the login-redirect path.
    group!("/", {
        get!("/device", device_verification_get),
        post!("/device", device_verification_post),
    }).middleware(
        TenantMiddleware::new()
            .resolver(SessionUserTenantResolver::new())
            .on_failure(TenantFailureMode::Allow),
    ),

    // Visual/form transition-write surface (Phase 232, EXEC-05).
    // Receives the projection-emitted POST /{service}/{action} action-button URL
    // and drives the SAME framework::write kernel as /mcp, with audit channel "web".
    //
    // Tenant-scoped: SessionUserTenantResolver populates current_tenant() from the
    // browser session so the handler authenticates the tenant from auth context
    // (never the form body, T-232-07). Registered AFTER the explicit named routes;
    // the router prefers literal segments, so this {service}/{action} pattern is the
    // catch-all transition surface and does not shadow /auth/*, /mcp, /token, etc.
    // Failure mode: Forbidden — a request without a resolvable tenant is denied, not
    // silently allowed onto the write path.
    group!("/", {
        post!("/{service}/{action}", controllers::visual_action::handle).name("projection.visual.action"),
    }).middleware(
        TenantMiddleware::new()
            .resolver(SessionUserTenantResolver::new())
            .on_failure(TenantFailureMode::Forbidden),
    ),

    // API CRUD routes - protected by API key middleware
    api_routes(),

    // API documentation and OpenAPI spec
    docs_routes(),

    // Broadcasting auth (uncomment when broadcasting is configured in bootstrap):
    // post!("/broadcasting/auth", ferro::broadcasting_auth),
}
