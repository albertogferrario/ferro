//! API key provider implementation

use crate::models::api_key::{self, Entity as ApiKey};
use ferro::{async_trait, serde_json, verify_api_key_hash, ApiKeyInfo, ApiKeyProvider};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

/// Database-backed API key provider.
pub struct ApiKeyProviderImpl;

#[async_trait]
impl ApiKeyProvider for ApiKeyProviderImpl {
    async fn verify_key(&self, raw_key: &str) -> Result<ApiKeyInfo, ()> {
        let prefix = &raw_key[..16.min(raw_key.len())];

        let db = ferro::DB::connection().map_err(|_| ())?;
        let record = ApiKey::find()
            .filter(api_key::Column::Prefix.eq(prefix))
            .one(db.inner())
            .await
            .map_err(|_| ())?
            .ok_or(())?;

        // Check revocation
        if record.revoked_at.is_some() {
            return Err(());
        }

        // Check expiry
        if let Some(expires_at) = record.expires_at {
            if expires_at < chrono::Utc::now() {
                return Err(());
            }
        }

        // Constant-time hash verification
        if !verify_api_key_hash(raw_key, &record.hashed_key) {
            return Err(());
        }

        let scopes: Vec<String> = record
            .scopes
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        Ok(ApiKeyInfo {
            id: record.id,
            name: record.name,
            scopes,
        })
    }
}
