//! auth:link command — generate a magic-link login URL for a given email address.
//!
//! Inserts a fresh token directly into the database (bypassing email delivery),
//! then prints the full /auth/verify URL. Useful for admin impersonation and
//! local development.

use chrono::{Duration, Utc};
use console::style;
use rand::RngCore;
use sea_orm::{ConnectionTrait, Database, DbBackend, Statement, Value};
use sha2::{Digest, Sha256};
use std::env;
use std::process;

pub fn run(email: String) {
    dotenvy::dotenv().ok();

    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!(
                "{} DATABASE_URL not set in .env",
                style("Error:").red().bold()
            );
            process::exit(1);
        }
    };

    let app_url = env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8080".into());

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        match generate_link(&database_url, &app_url, &email).await {
            Ok(url) => {
                println!(
                    "{} {}",
                    style("Magic link for").dim(),
                    style(&email).cyan()
                );
                println!("{}", url);
                println!("{}", style("Expires in 15 minutes.").dim());
            }
            Err(e) => {
                eprintln!("{} {}", style("Error:").red().bold(), e);
                process::exit(1);
            }
        }
    });
}

async fn generate_link(
    database_url: &str,
    app_url: &str,
    email: &str,
) -> Result<String, String> {
    let backend = if database_url.starts_with("sqlite") {
        DbBackend::Sqlite
    } else {
        DbBackend::Postgres
    };

    let db = Database::connect(database_url)
        .await
        .map_err(|e| format!("Cannot connect to database: {e}"))?;

    let row = db
        .query_one(Statement::from_sql_and_values(
            backend,
            "SELECT id FROM users WHERE email = ?",
            [Value::String(Some(Box::new(email.to_string())))],
        ))
        .await
        .map_err(|e| format!("Query failed: {e}"))?;

    let user_id: i64 = match row {
        Some(r) => r
            .try_get_by_index::<i64>(0)
            .map_err(|e| format!("Failed to read user id: {e}"))?,
        None => return Err(format!("No user found with email: {email}")),
    };

    // 32-byte random token encoded as hex (matches the application auth controller)
    let mut raw = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut raw);
    let token_raw: String = raw.iter().map(|b| format!("{b:02x}")).collect();

    // SHA256 of the hex string stored as binary blob
    let token_hash: Vec<u8> = Sha256::digest(token_raw.as_bytes()).to_vec();

    // UUID v4 from random bytes
    let mut uid = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut uid);
    uid[6] = (uid[6] & 0x0f) | 0x40;
    uid[8] = (uid[8] & 0x3f) | 0x80;
    let uuid = format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes(uid[0..4].try_into().unwrap()),
        u16::from_be_bytes(uid[4..6].try_into().unwrap()),
        u16::from_be_bytes(uid[6..8].try_into().unwrap()),
        u16::from_be_bytes(uid[8..10].try_into().unwrap()),
        {
            let mut b = [0u8; 8];
            b[2..8].copy_from_slice(&uid[10..16]);
            u64::from_be_bytes(b)
        }
    );

    let now = Utc::now();
    let expires_at = now + Duration::minutes(15);

    db.execute(Statement::from_sql_and_values(
        backend,
        "INSERT INTO magic_link_tokens (id, user_id, token_hash, expires_at, created_at) \
         VALUES (?, ?, ?, ?, ?)",
        [
            Value::String(Some(Box::new(uuid))),
            Value::BigInt(Some(user_id)),
            Value::Bytes(Some(Box::new(token_hash))),
            Value::String(Some(Box::new(
                expires_at.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string(),
            ))),
            Value::String(Some(Box::new(
                now.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string(),
            ))),
        ],
    ))
    .await
    .map_err(|e| format!("Failed to insert token: {e}"))?;

    let base = app_url.trim_end_matches('/');
    Ok(format!("{base}/auth/verify?token={token_raw}"))
}
