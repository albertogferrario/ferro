//! Secret-shaped env key classifier (D-08).
//!
//! Case-insensitive substring match against a fixed vocabulary. Keys ending
//! in `_URL` are non-secret UNLESS they also match another substring hit
//! (e.g. `DATABASE_URL` → false, `STRIPE_SECRET_URL` → true).
//!
//! Known behavior (documented, not a bug for this phase): plain webhook URL
//! envs like `SLACK_WEBHOOK_URL` classify as non-secret because the `_URL`
//! carve-out trumps the lack of other substring hits. Extending the
//! heuristic is out of scope for Phase 127.

const SECRET_SUBSTRINGS: &[&str] = &[
    "secret",
    "password",
    "passwd",
    "token",
    "key",
    "api_key",
    "dsn",
    "private",
    "credential",
];

/// Classify an env var key as secret-shaped per D-08.
pub fn is_secret_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    let non_url_hit = SECRET_SUBSTRINGS.iter().any(|n| lower.contains(n));
    if lower.ends_with("_url") {
        // _URL carve-out: non-secret unless another substring matches the
        // portion before the trailing `_url`.
        let head = &lower[..lower.len() - 4];
        return SECRET_SUBSTRINGS.iter().any(|n| head.contains(n));
    }
    non_url_hit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_secret_key_stripe_secret_key() {
        assert!(is_secret_key("STRIPE_SECRET_KEY"));
    }
    #[test]
    fn is_secret_key_api_token() {
        assert!(is_secret_key("API_TOKEN"));
    }
    #[test]
    fn is_secret_key_db_password() {
        assert!(is_secret_key("DB_PASSWORD"));
    }
    #[test]
    fn is_secret_key_passwd() {
        assert!(is_secret_key("USER_PASSWD"));
    }
    #[test]
    fn is_secret_key_dsn() {
        assert!(is_secret_key("SENTRY_DSN"));
    }
    #[test]
    fn is_secret_key_private() {
        assert!(is_secret_key("PRIVATE_RSA"));
    }
    #[test]
    fn is_secret_key_credential() {
        assert!(is_secret_key("DB_CREDENTIAL"));
    }
    #[test]
    fn is_secret_key_api_key() {
        assert!(is_secret_key("MY_API_KEY"));
    }
    #[test]
    fn is_secret_key_database_url_carve_out() {
        assert!(!is_secret_key("DATABASE_URL"));
    }
    #[test]
    fn is_secret_key_redis_url_carve_out() {
        assert!(!is_secret_key("REDIS_URL"));
    }
    #[test]
    fn is_secret_key_stripe_secret_url_still_secret() {
        assert!(is_secret_key("STRIPE_SECRET_URL"));
    }
    #[test]
    fn is_secret_key_app_name_not_secret() {
        assert!(!is_secret_key("APP_NAME"));
    }
}
