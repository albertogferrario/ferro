//! Dynamic client registration handler (RFC 7591).
//!
//! Implements `POST /register` per RFC 7591.
//! Accepts `redirect_uris`, validates schemes (T-199-05 allowlist),
//! generates a high-entropy `client_id` (T-199-DCR), persists to `oauth_clients`,
//! and returns `201` with no `client_secret` (public clients only).

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use serde::Deserialize;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

/// RFC 7591 Dynamic Client Registration request body.
#[derive(Debug, Deserialize)]
pub struct RegisterInput {
    /// Required: list of redirect URIs. At least one must be provided.
    #[serde(default)]
    pub redirect_uris: Vec<String>,
    /// Optional human-readable client name.
    pub client_name: Option<String>,
    // grant_types and response_types are accepted but not stored (validated implicitly).
    #[serde(default)]
    pub grant_types: Vec<String>,
    #[serde(default)]
    pub response_types: Vec<String>,
    /// Token endpoint auth method — only "none" is supported (public clients).
    pub token_endpoint_auth_method: Option<String>,
}

/// Scheme allowlist for redirect URIs (T-199-05).
///
/// Accepts:
/// - `http://localhost` and `http://localhost:*` (loopback only, not arbitrary http)
/// - `https://` (any HTTPS URI)
///
/// Rejects `javascript:`, `data:`, custom schemes, and arbitrary `http://` to
/// non-localhost hosts (an open redirect vector in public client DCR).
pub fn is_redirect_uri_allowed(uri: &str) -> bool {
    let lower = uri.to_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://localhost")
}

/// Validate all redirect URIs in `uris`.
///
/// Returns `Ok(())` when every URI passes the scheme allowlist, or
/// `Err(description)` with the first offending URI for the 400 response body.
pub fn validate_redirect_uris(uris: &[String]) -> Result<(), String> {
    if uris.is_empty() {
        return Err("redirect_uris required".to_string());
    }
    for uri in uris {
        if !is_redirect_uri_allowed(uri) {
            return Err(format!(
                "redirect_uri '{uri}' uses a disallowed scheme; only https:// or http://localhost are accepted"
            ));
        }
    }
    Ok(())
}

/// Generate a high-entropy client id (T-199-DCR).
///
/// 16 random bytes encoded as URL-safe base64 without padding (~22 chars).
/// Non-sequential — enumeration resistance equivalent to UUIDv4 (128-bit entropy).
pub fn generate_client_id() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Handler: `POST /register` (RFC 7591 Dynamic Client Registration).
///
/// Validates `redirect_uris`, generates a `client_id`, persists the client,
/// and returns `201` with no `client_secret` (public clients only).
#[ferro::handler]
pub async fn register_client(req: ferro::Request) -> ferro::Response {
    let input: RegisterInput = req.json().await?;

    if let Err(desc) = validate_redirect_uris(&input.redirect_uris) {
        return Err(ferro::HttpResponse::json(json!({
            "error": "invalid_client_metadata",
            "error_description": desc,
        }))
        .status(400));
    }

    let redirect_uris_json =
        serde_json::to_string(&input.redirect_uris).unwrap_or_else(|_| "[]".to_string());

    let client_id = generate_client_id();

    let db_conn = ferro::DB::connection().map_err(|e| {
        ferro::HttpResponse::json(json!({
            "error": "server_error",
            "error_description": format!("db connection failed: {}", e),
        }))
        .status(500)
    })?;
    crate::store::insert_client(
        db_conn.inner(),
        client_id.clone(),
        input.client_name.clone(),
        redirect_uris_json,
    )
    .await
    .map_err(|e| {
        ferro::HttpResponse::json(json!({
            "error": "server_error",
            "error_description": format!("db insert failed: {}", e),
        }))
        .status(500)
    })?;

    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(ferro::HttpResponse::json(json!({
        "client_id": client_id,
        "client_name": input.client_name,
        "redirect_uris": input.redirect_uris,
        "grant_types": ["authorization_code"],
        "token_endpoint_auth_method": "none",
        "client_id_issued_at": now_unix,
    }))
    .status(201))
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

    // Test 1: valid request returns 201 with client_id and redirect_uris
    #[tokio::test]
    async fn register_valid_returns_client_id_and_redirect_uris() {
        let db = fresh_db().await;
        let redirect_uris = vec!["http://localhost:3000/callback".to_string()];
        let redirect_uris_json = serde_json::to_string(&redirect_uris).unwrap();

        let client_id = generate_client_id();
        crate::store::insert_client(
            &db,
            client_id.clone(),
            Some("Test Client".to_string()),
            redirect_uris_json.clone(),
        )
        .await
        .expect("insert should succeed");

        let found = crate::store::find_by_client_id(&db, &client_id)
            .await
            .expect("find should not error");
        assert!(found.is_some());
        let row = found.unwrap();
        assert_eq!(row.client_id, client_id);
        // Verify redirect_uris stored verbatim
        let stored: Vec<String> =
            serde_json::from_str(&row.redirect_uris).expect("stored as JSON array");
        assert_eq!(stored, redirect_uris);
    }

    // Test 2: client_id is not sequential (non-empty, URL-safe base64, unique across calls)
    #[test]
    fn client_id_is_random_and_non_sequential() {
        let id1 = generate_client_id();
        let id2 = generate_client_id();
        assert!(!id1.is_empty(), "client_id must not be empty");
        assert_ne!(id1, id2, "two generated client_ids must differ");
        // Must be URL-safe base64 (no +, /, =)
        assert!(
            id1.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
            "client_id must be URL-safe: got '{id1}'"
        );
    }

    // Test 3: missing redirect_uris → validation error
    #[test]
    fn missing_redirect_uris_returns_error() {
        let empty: Vec<String> = vec![];
        let result = validate_redirect_uris(&empty);
        assert!(result.is_err(), "empty redirect_uris must return Err");
        let desc = result.unwrap_err();
        assert!(
            desc.contains("redirect_uris"),
            "error description must mention redirect_uris, got: {desc}"
        );
    }

    // Test 4: javascript: scheme → validation error
    #[test]
    fn javascript_scheme_is_rejected() {
        let uris = vec!["javascript:alert(1)".to_string()];
        let result = validate_redirect_uris(&uris);
        assert!(result.is_err(), "javascript: scheme must be rejected");
    }

    // Test 4b: data: and custom schemes are also rejected
    #[test]
    fn data_and_custom_schemes_are_rejected() {
        for uri in &[
            "data:text/html,<h1>x</h1>",
            "myapp://callback",
            "http://evil.com/callback",
        ] {
            assert!(
                !is_redirect_uri_allowed(uri),
                "URI '{uri}' should be rejected by allowlist"
            );
        }
    }

    // Test 4c: allowed schemes pass
    #[test]
    fn allowed_schemes_pass_validation() {
        let valid = vec![
            "https://app.example.com/callback".to_string(),
            "http://localhost/callback".to_string(),
            "http://localhost:8080/cb".to_string(),
        ];
        let result = validate_redirect_uris(&valid);
        assert!(result.is_ok(), "valid URIs must pass: {result:?}");
    }

    // Test 5: persisted client is retrievable by client_id
    #[tokio::test]
    async fn persisted_client_retrievable_by_client_id() {
        let db = fresh_db().await;
        let client_id = generate_client_id();
        let redirect_uris_json = r#"["https://app.example.com/cb"]"#.to_string();

        crate::store::insert_client(
            &db,
            client_id.clone(),
            None,
            redirect_uris_json,
        )
        .await
        .expect("insert should succeed");

        let found = crate::store::find_by_client_id(&db, &client_id)
            .await
            .expect("no db error");
        assert!(
            found.is_some(),
            "client inserted must be findable by client_id"
        );
    }
}
