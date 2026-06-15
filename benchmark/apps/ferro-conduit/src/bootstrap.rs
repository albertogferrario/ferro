//! Application Bootstrap — Conduit benchmark app

use ferro::{register_global_middleware, Cors, DB};

/// Register global middleware and services.
pub async fn register() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ferro=info".parse().unwrap()),
        )
        .init();

    // Conduit conformance runners may issue CORS preflight (RESEARCH Pitfall 6).
    register_global_middleware(Cors::permissive());

    DB::init().await.expect("Failed to connect to database");
}
