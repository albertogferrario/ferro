//! Application Bootstrap — minimal benchmark app

use ferro::DB;

/// Register global middleware and services
pub async fn register() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ferro=info".parse().unwrap()),
        )
        .init();

    DB::init().await.expect("Failed to connect to database");
}
