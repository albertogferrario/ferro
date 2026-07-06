//! Password reset tokens model

use ferro::database::{Model as DatabaseModel, ModelMut, QueryBuilder, DB};
use chrono::{Duration, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::Set;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "password_reset_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub email: String,
    pub token: String,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl DatabaseModel for Entity {}
impl ModelMut for Entity {}

/// Type alias for convenient access
pub type PasswordResetToken = Model;

/// Token expiry duration in hours
const TOKEN_EXPIRY_HOURS: i64 = 1;

impl Model {
    /// Start a query builder
    pub fn query() -> QueryBuilder<Entity> {
        QueryBuilder::new()
    }

    /// Create a password reset token for an email
    pub async fn create_for_email(email: &str) -> Result<String, ferro::FrameworkError> {
        // Delete any existing token for this email
        Self::delete_for_email(email).await?;

        // Generate a new token
        let token = ferro::session::generate_session_id();
        let hashed_token = ferro::hashing::hash(&token)?;

        let model = ActiveModel {
            email: Set(email.to_string()),
            token: Set(hashed_token),
            created_at: Set(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        };

        Entity::insert_one(model).await?;

        Ok(token)
    }

    /// Find a valid token by email
    pub async fn find_valid_by_email(email: &str) -> Result<Option<Self>, ferro::FrameworkError> {
        let token = Self::query()
            .filter(Column::Email.eq(email))
            .first()
            .await?;

        // Check if token exists and is not expired
        if let Some(ref t) = token {
            if t.is_expired() {
                return Ok(None);
            }
        }

        Ok(token)
    }

    /// Verify a token matches for an email
    pub async fn verify(email: &str, token: &str) -> Result<bool, ferro::FrameworkError> {
        let record = Self::find_valid_by_email(email).await?;

        match record {
            Some(r) => ferro::hashing::verify(token, &r.token),
            None => Ok(false),
        }
    }

    /// Delete token for an email
    pub async fn delete_for_email(email: &str) -> Result<(), ferro::FrameworkError> {
        Entity::delete_many()
            .filter(Column::Email.eq(email))
            .exec(DB::get()?.inner())
            .await?;
        Ok(())
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        let created_at = chrono::NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S");

        match created_at {
            Ok(dt) => {
                let expiry = dt + Duration::hours(TOKEN_EXPIRY_HOURS);
                Utc::now().naive_utc() > expiry
            }
            Err(_) => true, // If we can't parse, consider it expired
        }
    }
}
