//! `OAuthCode` (cached) and `OAuthClient` (DB entity) for the OAuth server.
//!
//! `OAuthCode` is stored in `ferro::Cache` with a ~60 second TTL (D-03).
//! `OAuthClient` mirrors the `oauth_clients` table created by `migration.rs`.

use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

/// Short-lived authorization code stored in the cache during PKCE flows (D-03).
///
/// Serialized into `ferro::Cache` under key `"mcp:code:{code}"` with ~60 s TTL.
/// Deserialized by `POST /token` to verify the PKCE challenge and mint the JWT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCode {
    /// The registered client that initiated this flow.
    pub client_id: String,
    /// The redirect URI the code will be delivered to (exact-match verified at /token).
    pub redirect_uri: String,
    /// BASE64URL(SHA256(code_verifier)) — stored verbatim for S256 PKCE verification.
    pub code_challenge: String,
    /// Authenticated user id at authorize time.
    pub user_id: i64,
    /// Tenant id at authorize time; `None` for single-tenant apps (D-06).
    pub tenant_id: Option<i64>,
    /// Unix timestamp (seconds) when the code was issued.
    pub created_at: i64,
}

// ── SeaORM entity for the `oauth_clients` table (D-04) ───────────────────────

/// SeaORM entity model for `oauth_clients`.
///
/// Mirrors the columns defined in `migration.rs`:
/// `id`, `client_id`, `client_name`, `redirect_uris` (JSON-array text), `created_at`.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "oauth_clients")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// High-entropy random id (UUIDv4-length) — not sequential (T-199-DCR).
    pub client_id: String,
    /// Optional human-readable name registered by the client.
    pub client_name: Option<String>,
    /// JSON-encoded array of registered redirect URIs (verbatim, for exact-match — T-199-04a).
    pub redirect_uris: String,
    /// Row creation timestamp.
    pub created_at: DateTimeUtc,
}

/// No relations required for the OAuth clients table.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Convenience alias — mirrors the `ApiKey`/`User` model alias convention in the app.
pub type OAuthClient = Model;

/// Insert a new OAuth client into `oauth_clients` and return the persisted model.
///
/// `redirect_uris_json` must be a JSON-array string (e.g. `["http://localhost/cb"]`).
/// Returns `Err(DbErr)` on insert failure (e.g. duplicate `client_id`, DB down).
pub async fn insert_client(
    db: &DatabaseConnection,
    client_id: String,
    client_name: Option<String>,
    redirect_uris_json: String,
) -> Result<Model, DbErr> {
    let now = chrono::Utc::now();
    let active = ActiveModel {
        client_id: Set(client_id),
        client_name: Set(client_name),
        redirect_uris: Set(redirect_uris_json),
        created_at: Set(now),
        ..Default::default()
    };
    Entity::insert(active).exec_with_returning(db).await
}

/// Look up a registered client by its `client_id`.
///
/// Returns `None` when no matching row exists (unknown or revoked client).
/// Used by `/authorize` and `/token` to validate the `client_id` parameter (Plan 04).
pub async fn find_by_client_id(
    db: &DatabaseConnection,
    client_id: &str,
) -> Result<Option<Model>, DbErr> {
    Entity::find()
        .filter(Column::ClientId.eq(client_id))
        .one(db)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, DatabaseConnection};
    use sea_orm_migration::MigratorTrait;

    async fn fresh_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory sqlite");

        struct TestMigrator;

        #[async_trait::async_trait]
        impl sea_orm_migration::MigratorTrait for TestMigrator {
            fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
                vec![Box::new(crate::migration::Migration)]
            }
        }

        TestMigrator::up(&conn, None)
            .await
            .expect("apply oauth_clients migration");

        conn
    }

    #[tokio::test]
    async fn insert_and_find_by_client_id_roundtrip() {
        let db = fresh_db().await;
        let client_id = "test-client-abc123".to_string();
        let redirect_uris = r#"["http://localhost:3000/callback"]"#.to_string();

        let inserted = insert_client(
            &db,
            client_id.clone(),
            Some("Test Client".to_string()),
            redirect_uris.clone(),
        )
        .await
        .expect("insert should succeed");

        assert_eq!(inserted.client_id, client_id);
        assert_eq!(inserted.redirect_uris, redirect_uris);
        assert_eq!(inserted.client_name.as_deref(), Some("Test Client"));

        let found = find_by_client_id(&db, &client_id)
            .await
            .expect("find should not error");
        assert!(found.is_some(), "inserted client should be findable");
        assert_eq!(found.unwrap().client_id, client_id);
    }

    #[tokio::test]
    async fn find_by_client_id_returns_none_for_unknown() {
        let db = fresh_db().await;
        let result = find_by_client_id(&db, "does-not-exist")
            .await
            .expect("find should not error");
        assert!(result.is_none(), "unknown client_id must return None");
    }
}
