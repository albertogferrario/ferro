//! `ferro make:api-key` command — generates API keys for authentication.
//!
//! Replicates the key generation logic from `framework/src/api/api_key.rs`
//! without depending on the framework crate.

use console::style;
use sha2::{Digest, Sha256};

const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Result of generating a new API key.
pub(crate) struct GeneratedApiKey {
    /// Full key (display once, never store).
    pub raw_key: String,
    /// First 16 characters for database lookup.
    pub prefix: String,
    /// SHA-256 hex digest for verification.
    pub hashed_key: String,
}

/// Generate a new API key for the given environment.
///
/// Returns `None` if the environment is not "live" or "test".
pub(crate) fn generate_api_key(env: &str) -> Option<GeneratedApiKey> {
    if env != "live" && env != "test" {
        return None;
    }

    let prefix_str = format!("fe_{env}_");
    let mut rng = rand::thread_rng();
    let random: String = (0..43)
        .map(|_| {
            let idx = rand::Rng::gen_range(&mut rng, 0..62);
            BASE62[idx] as char
        })
        .collect();

    let raw_key = format!("{prefix_str}{random}");
    let prefix = raw_key[..16].to_string();

    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    let hashed_key = format!("{:x}", hasher.finalize());

    Some(GeneratedApiKey {
        raw_key,
        prefix,
        hashed_key,
    })
}

/// Run the `make:api-key` command.
pub fn run(name: String, env: String) {
    let key = match generate_api_key(&env) {
        Some(k) => k,
        None => {
            eprintln!(
                "{} Invalid environment '{}'. Must be 'live' or 'test'.",
                style("Error:").red().bold(),
                env
            );
            std::process::exit(1);
        }
    };

    println!();
    println!("{}", style("API Key Generated!").green().bold());
    println!();
    println!(
        "{}",
        style("Raw Key (save this — shown only once):")
            .yellow()
            .bold()
    );
    println!("  {}", style(&key.raw_key).green());
    println!();
    println!("Database values:");
    println!("  Name:       {name}");
    println!("  Prefix:     {}", key.prefix);
    println!("  Hashed Key: {}", key.hashed_key);
    println!();
    println!("Insert SQL:");
    println!("  INSERT INTO api_keys (name, prefix, hashed_key, created_at)");
    println!(
        "  VALUES ('{name}', '{}', '{}', datetime('now'));",
        key.prefix, key.hashed_key
    );
    println!();
    println!("Rust snippet:");
    println!("  let key = api_keys::ActiveModel {{");
    println!("      name: Set(\"{name}\".to_string()),");
    println!("      prefix: Set(\"{}\".to_string()),", key.prefix);
    println!("      hashed_key: Set(\"{}\".to_string()),", key.hashed_key);
    println!("      ..Default::default()");
    println!("  }};");
    println!("  key.insert(db).await?;");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_format_live() {
        let key = generate_api_key("live").unwrap();
        assert!(key.raw_key.starts_with("fe_live_"));
        // fe_live_ = 8 chars + 43 random = 51 total
        assert_eq!(key.raw_key.len(), 51);
    }

    #[test]
    fn key_format_test() {
        let key = generate_api_key("test").unwrap();
        assert!(key.raw_key.starts_with("fe_test_"));
        assert_eq!(key.raw_key.len(), 51);
    }

    #[test]
    fn prefix_is_first_16_chars() {
        let key = generate_api_key("live").unwrap();
        assert_eq!(key.prefix.len(), 16);
        assert_eq!(key.prefix, &key.raw_key[..16]);
    }

    #[test]
    fn hash_is_valid_sha256_hex() {
        let key = generate_api_key("live").unwrap();
        assert_eq!(key.hashed_key.len(), 64);
        assert!(key.hashed_key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_matches_raw_key() {
        let key = generate_api_key("live").unwrap();
        let mut hasher = Sha256::new();
        hasher.update(key.raw_key.as_bytes());
        let expected = format!("{:x}", hasher.finalize());
        assert_eq!(key.hashed_key, expected);
    }

    #[test]
    fn consecutive_keys_are_unique() {
        let key1 = generate_api_key("live").unwrap();
        let key2 = generate_api_key("live").unwrap();
        assert_ne!(key1.raw_key, key2.raw_key);
        assert_ne!(key1.hashed_key, key2.hashed_key);
    }

    #[test]
    fn random_part_is_base62() {
        let key = generate_api_key("live").unwrap();
        let random_part = &key.raw_key[8..]; // skip "fe_live_"
        assert!(random_part.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn invalid_env_returns_none() {
        assert!(generate_api_key("staging").is_none());
        assert!(generate_api_key("production").is_none());
        assert!(generate_api_key("").is_none());
    }
}
