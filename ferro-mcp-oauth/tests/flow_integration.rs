//! Full PKCE flow integration test harness.
//!
//! Plan 04 fills: DCR→authorize→consent→token→validate

use ferro_mcp_oauth::{config::OAuthConfig, CreateOauthClientsTable};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

/// Boot an in-memory SQLite database and apply the oauth_clients migration.
async fn fresh_db() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:")
        .await
        .expect("connect to in-memory sqlite");

    struct TestMigrator;

    #[async_trait::async_trait]
    impl sea_orm_migration::MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(CreateOauthClientsTable)]
        }
    }

    TestMigrator::up(&conn, None)
        .await
        .expect("apply oauth_clients migration");

    conn
}

/// Set a deterministic test MCP_TOKEN_SECRET (>= 32 bytes) and return OAuthConfig.
fn test_oauth_config() -> OAuthConfig {
    std::env::set_var(
        "MCP_TOKEN_SECRET",
        "test_secret_that_is_at_least_32_bytes_long_for_hs256",
    );
    std::env::set_var("APP_URL", "http://localhost:8080");
    std::env::set_var("APP_NAME", "TestApp");
    OAuthConfig::from_env().expect("OAuthConfig::from_env should succeed with test secret")
}

#[tokio::test]
async fn full_pkce_flow() {
    // Harness: DB connection + migration applied + config Ok.
    // Plan 04 fills: DCR→authorize→consent→token→validate
    let _db = fresh_db().await;
    let _config = test_oauth_config();

    // Verify the harness itself: migration applied, config loaded.
    // The real flow assertions are added by Plan 04's e2e task.
    assert!(
        !_config.app_url.is_empty(),
        "app_url should be set from env"
    );
    assert!(
        !_config.token_secret.is_empty(),
        "token_secret should be populated"
    );
}
