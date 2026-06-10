//! Application Bootstrap
//!
//! This is where you register global middleware and services that need runtime configuration.
//! Services that don't need runtime config can use `#[service(ConcreteType)]` instead.
//!
//! # Example
//!
//! ```rust,ignore
//! // For services with no runtime config, use the macro:
//! #[service(RedisCache)]
//! pub trait CacheStore { ... }
//!
//! // For services needing runtime config, register here:
//! pub async fn register() {
//!     // Initialize database (errors provide actionable guidance)
//!     DB::init().await.unwrap_or_else(|e| {
//!         eprintln!("Error: Failed to connect to database");
//!         eprintln!("  Cause: {}", e);
//!         std::process::exit(1);
//!     });
//!
//!     // Global middleware
//!     global_middleware!(middleware::LoggingMiddleware);
//!
//!     // Services
//!     bind!(dyn Database, PostgresDB::new());
//! }
//! ```

#[allow(unused_imports)]
use ferro::{
    bind, global_middleware, singleton, ApiKeyProvider, App, AuthResponse, Gate, LangMiddleware,
    Limit, RateLimiter, SessionConfig, SessionMiddleware, UserProvider, DB,
};

use crate::middleware;
use crate::providers::{ApiKeyProviderImpl, DatabaseUserProvider};

/// Register global middleware and services
///
/// Called from main.rs before `Server::from_config()`.
/// Middleware and services registered here can use environment variables, config files, etc.
pub async fn register() {
    // Initialize database connection
    DB::init().await.unwrap_or_else(|e| {
        eprintln!("Error: Failed to connect to database");
        eprintln!("  Cause: {e}");
        eprintln!();
        eprintln!("How to fix:");
        eprintln!("  1. Check DATABASE_URL is set in .env");
        eprintln!("  2. Ensure the database server is running");
        eprintln!("  3. Verify connection credentials are correct");
        eprintln!();
        eprintln!("Example .env:");
        eprintln!("  DATABASE_URL=sqlite://./database.db");
        std::process::exit(1);
    });

    // Global middleware (runs on every request in registration order).
    // SessionMiddleware MUST be registered first so the session context (and the
    // session cookie it issues + persists) wraps every downstream middleware —
    // including the OAuth /authorize group, whose login reuse and CSRF consent
    // depend on a session that survives across requests.
    global_middleware!(SessionMiddleware::new(SessionConfig::from_env()));
    global_middleware!(middleware::LoggingMiddleware);
    global_middleware!(middleware::ShareInertiaData);
    global_middleware!(LangMiddleware);

    // Register the user provider for Auth::user()
    bind!(dyn UserProvider, DatabaseUserProvider);

    // Register the API key provider for ApiKeyMiddleware
    bind!(dyn ApiKeyProvider, ApiKeyProviderImpl);

    // Define named rate limiters for API routes
    RateLimiter::define("api", |_req| Limit::per_minute(60));

    // -------------------------------------------------------------------------
    // Phase 200: Gate ability + DbTenantLookup + two-tenant seed
    // -------------------------------------------------------------------------

    // Gate ability "view-orders" — any authenticated user may view their tenant's orders.
    // Tenant scoping (which orders a user sees) is enforced by dispatch (D-02); this gate
    // only determines whether the tool is accessible at all. SC-2 / AMCP-11.
    Gate::define("view-orders", |user, _resource| {
        user.as_any()
            .downcast_ref::<crate::models::users::User>()
            .map(|_u| AuthResponse::allow())
            .unwrap_or_else(AuthResponse::deny_silent)
    });

    // Build and register the global DbTenantLookup (shared with route middleware).
    let tenant_lookup = crate::tenant_lookup::build();
    crate::tenant_lookup::init(tenant_lookup);

    // Idempotent seed: insert two tenants, one user per tenant, two orders per tenant.
    // Only runs when the tenants table is empty (guard prevents duplicate inserts).
    seed_dogfood_data().await;

    // Example: Register a trait binding with runtime config
    // bind!(dyn Database, PostgresDB::new());

    // Example: Register a concrete singleton
    // singleton!(CacheService::new());
}

/// Insert dogfood fixture data when the tenants table is empty.
///
/// Seed credentials:
///   Tenant "acme"   → user alice@acme.test   / password "password123"
///   Tenant "globex" → user bob@globex.test   / password "password123"
///
/// Each tenant has 2 orders seeded. Run `app db:fresh` to re-seed from scratch.
async fn seed_dogfood_data() {
    use ferro::DB;

    let db = match DB::connection() {
        Ok(c) => c,
        Err(_) => return, // DB not yet connected — skip silently
    };

    // Guard: only seed when the tenants table is empty.
    use crate::models::entities::tenants::Entity as TenantEntity;
    use ferro::{EntityTrait, PaginatorTrait};
    let count = match TenantEntity::find().count(db.inner()).await {
        Ok(n) => n,
        Err(_) => return,
    };
    if count > 0 {
        return;
    }

    let now = "2026-06-10T00:00:00+00:00";

    // --- Tenants ---
    use crate::models::entities::tenants::ActiveModel as TenantActive;
    use sea_orm::{ActiveModelTrait, ActiveValue::Set};

    let acme = TenantActive {
        id: Set(1),
        slug: Set("acme".into()),
        name: Set("Acme".into()),
        created_at: Set(now.into()),
    };
    let globex = TenantActive {
        id: Set(2),
        slug: Set("globex".into()),
        name: Set("Globex".into()),
        created_at: Set(now.into()),
    };

    if let Err(e) = acme.insert(db.inner()).await {
        eprintln!("[seed] Failed to insert acme tenant: {e}");
        return;
    }
    if let Err(e) = globex.insert(db.inner()).await {
        eprintln!("[seed] Failed to insert globex tenant: {e}");
        return;
    }

    // --- Users (one per tenant) ---
    // Password "password123" — hashed with ferro's default bcrypt cost.
    let hashed = ferro::hash("password123").unwrap_or_else(|_| "password123".into());

    use crate::models::entities::users::ActiveModel as UserActive;
    let alice = UserActive {
        id: Set(901),
        name: Set("Alice Acme".into()),
        email: Set("alice@acme.test".into()),
        password: Set(hashed.clone()),
        remember_token: Set(None),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        tenant_id: Set(Some(1)),
    };
    let bob = UserActive {
        id: Set(902),
        name: Set("Bob Globex".into()),
        email: Set("bob@globex.test".into()),
        password: Set(hashed),
        remember_token: Set(None),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        tenant_id: Set(Some(2)),
    };

    if let Err(e) = alice.insert(db.inner()).await {
        eprintln!("[seed] Failed to insert alice user: {e}");
        return;
    }
    if let Err(e) = bob.insert(db.inner()).await {
        eprintln!("[seed] Failed to insert bob user: {e}");
        return;
    }

    // --- Orders (2 per tenant) ---
    use crate::models::entities::orders::ActiveModel as OrderActive;
    let orders: Vec<OrderActive> = vec![
        // Acme orders (tenant_id = 1)
        OrderActive {
            id: Set(1),
            customer_name: Set("Alice Acme".into()),
            total: Set(120.00),
            status: Set("submitted".into()),
            created_at: Set(now.into()),
            tenant_id: Set(1),
        },
        OrderActive {
            id: Set(2),
            customer_name: Set("Alice Acme".into()),
            total: Set(85.50),
            status: Set("delivered".into()),
            created_at: Set(now.into()),
            tenant_id: Set(1),
        },
        // Globex orders (tenant_id = 2)
        OrderActive {
            id: Set(3),
            customer_name: Set("Bob Globex".into()),
            total: Set(999.99),
            status: Set("draft".into()),
            created_at: Set(now.into()),
            tenant_id: Set(2),
        },
        OrderActive {
            id: Set(4),
            customer_name: Set("Bob Globex".into()),
            total: Set(250.00),
            status: Set("approved".into()),
            created_at: Set(now.into()),
            tenant_id: Set(2),
        },
    ];

    for order in orders {
        if let Err(e) = order.insert(db.inner()).await {
            eprintln!("[seed] Failed to insert order: {e}");
            return;
        }
    }

    println!("[seed] Dogfood fixture seeded: 2 tenants, 2 users, 4 orders");
}
