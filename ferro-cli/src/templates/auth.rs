// ============================================================================
// Auth scaffolding templates
// ============================================================================

/// Migration template that adds auth fields to an existing users table.
///
/// Uses ALTER TABLE to add name, email (unique), password, and remember_token.
pub fn auth_migration_template() -> String {
    r#"use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add auth fields to existing users table
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::Name).string().not_null().default(""))
                    .add_column(ColumnDef::new(Users::Email).string().not_null().default(""))
                    .add_column(ColumnDef::new(Users::Password).string().not_null().default(""))
                    .add_column(ColumnDef::new(Users::RememberToken).string().null())
                    .to_owned(),
            )
            .await?;

        // Add unique index on email
        manager
            .create_index(
                Index::create()
                    .name("idx_users_email_unique")
                    .table(Users::Table)
                    .col(Users::Email)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_email_unique")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::Name)
                    .drop_column(Users::Email)
                    .drop_column(Users::Password)
                    .drop_column(Users::RememberToken)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Name,
    Email,
    Password,
    RememberToken,
}
"#
    .to_string()
}

/// Auth controller template with register, login, and logout handlers.
pub fn auth_controller_template() -> String {
    r#"//! Authentication controller
//!
//! Handles user registration, login, and logout.
//!
//! Tip: Use AuthUser<user::Model> to auto-extract the authenticated user:
//!
//!   use ferro::AuthUser;
//!
//!   #[handler]
//!   pub async fn profile(user: AuthUser<user::Model>) -> Response {
//!       Ok(HttpResponse::json(serde_json::json!({"user": user.name})))
//!   }

use ferro::database::ModelMut;
use ferro::http::{HttpResponse, Request, Response};
use ferro::{handler, hash, json_response, rules, verify};
use ferro::{Auth, Validator, required, string, email, min};
use ferro::ActiveValue;
use serde::Deserialize;

use crate::models::user;

#[derive(Deserialize)]
struct RegisterInput {
    name: String,
    email: String,
    password: String,
    password_confirmation: String,
}

#[derive(Deserialize)]
struct LoginInput {
    email: String,
    password: String,
}

/// Register a new user
#[handler]
pub async fn register(req: Request) -> Response {
    let input: RegisterInput = req.input().await.map_err(|_| {
        HttpResponse::json(serde_json::json!({
            "message": "Invalid request body."
        }))
        .status(422)
    })?;

    // Validate input
    let data = serde_json::json!({
        "name": input.name,
        "email": input.email,
        "password": input.password,
        "password_confirmation": input.password_confirmation,
    });

    let mut validator = Validator::new(&data)
        .rules("name", rules![required(), string()])
        .rules("email", rules![required(), email()])
        .rules("password", rules![required(), string(), min(8)]);

    // Check password confirmation
    if input.password != input.password_confirmation {
        validator = validator.with_error("password_confirmation", "Passwords do not match.");
    }

    // Check email uniqueness
    if let Some(_existing) = user::Model::find_by_email(&input.email).await.map_err(|e| {
        HttpResponse::json(serde_json::json!({
            "message": format!("Database error: {}", e)
        }))
        .status(500)
    })? {
        validator = validator.with_error("email", "This email is already registered.");
    }

    if let Err(errors) = validator.validate() {
        return Err(HttpResponse::json(serde_json::json!({
            "message": "Validation failed.",
            "errors": errors,
        }))
        .status(422));
    }

    // Hash password
    let password_hash = hash(&input.password).map_err(|e| {
        HttpResponse::json(serde_json::json!({
            "message": format!("Failed to hash password: {}", e)
        }))
        .status(500)
    })?;

    // Create user
    let user = user::ActiveModel {
        name: ActiveValue::Set(input.name.clone()),
        email: ActiveValue::Set(input.email.clone()),
        password: ActiveValue::Set(password_hash),
        remember_token: ActiveValue::Set(None),
        ..Default::default()
    };

    let db = ferro::DB::connection()
        .map_err(|e| ferro::error_response!(500, e.to_string()))?;
    let user = user::Entity::insert(user)
        .exec_with_returning(db.inner())
        .await
        .map_err(|e| {
            HttpResponse::json(serde_json::json!({
                "message": format!("Failed to create user: {}", e)
            }))
            .status(500)
        })?;

    // Log in the new user
    Auth::login(user.id as i64);

    Ok(HttpResponse::json(serde_json::json!({
        "user": {
            "id": user.id,
            "name": user.name,
            "email": user.email,
        }
    }))
    .status(201))
}

/// Log in an existing user
#[handler]
pub async fn login(req: Request) -> Response {
    let input: LoginInput = req.input().await.map_err(|_| {
        HttpResponse::json(serde_json::json!({
            "message": "Invalid request body."
        }))
        .status(422)
    })?;

    // Validate input
    let data = serde_json::json!({
        "email": input.email,
        "password": input.password,
    });

    if let Err(errors) = Validator::new(&data)
        .rules("email", rules![required(), email()])
        .rules("password", rules![required()])
        .validate()
    {
        return Err(HttpResponse::json(serde_json::json!({
            "message": "Validation failed.",
            "errors": errors,
        }))
        .status(422));
    }

    // Attempt authentication
    let email = input.email.clone();
    let password = input.password.clone();

    let result = Auth::attempt(|| async {
        let user = user::Model::find_by_email(&email).await?;
        match user {
            Some(user) => {
                if verify(&password, &user.password)? {
                    Ok(Some(user.id as i64))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    })
    .await;

    match result {
        Ok(Some(_id)) => {
            // Re-fetch user for response
            let user = user::Model::find_by_email(&input.email)
                .await
                .map_err(|e| {
                    HttpResponse::json(serde_json::json!({
                        "message": format!("Database error: {}", e)
                    }))
                    .status(500)
                })?;

            match user {
                Some(user) => json_response!({
                    "user": {
                        "id": user.id,
                        "name": user.name,
                        "email": user.email,
                    }
                }),
                None => Err(HttpResponse::json(serde_json::json!({
                    "email": ["These credentials do not match our records."]
                }))
                .status(422)),
            }
        }
        Ok(None) => Err(HttpResponse::json(serde_json::json!({
            "email": ["These credentials do not match our records."]
        }))
        .status(422)),
        Err(e) => Err(HttpResponse::json(serde_json::json!({
            "message": format!("Authentication error: {}", e)
        }))
        .status(500)),
    }
}

/// Log out the current user
#[handler]
pub async fn logout(_req: Request) -> Response {
    Auth::logout();
    json_response!({
        "message": "Logged out successfully."
    })
}
"#
    .to_string()
}
