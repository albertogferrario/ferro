//! SECRET vs PLAIN env-key classifier. Pure naming-convention check.

/// A key is a SECRET when, uppercased:
///   - ends with `_KEY`, `_SECRET`, `_PASSWORD`, or `_TOKEN`, OR
///   - equals `DATABASE_URL`.
pub fn is_secret(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    if upper == "DATABASE_URL" {
        return true;
    }
    upper.ends_with("_KEY")
        || upper.ends_with("_SECRET")
        || upper.ends_with("_PASSWORD")
        || upper.ends_with("_TOKEN")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_suffix_is_secret() {
        assert!(is_secret("STRIPE_SECRET_KEY"));
    }

    #[test]
    fn token_suffix_is_secret() {
        assert!(is_secret("GITHUB_TOKEN"));
    }

    #[test]
    fn password_suffix_is_secret() {
        assert!(is_secret("DB_PASSWORD"));
    }

    #[test]
    fn secret_suffix_is_secret() {
        assert!(is_secret("APP_SECRET"));
    }

    #[test]
    fn database_url_exact_is_secret() {
        assert!(is_secret("DATABASE_URL"));
    }

    #[test]
    fn app_url_is_not_secret() {
        assert!(!is_secret("APP_URL"));
    }

    #[test]
    fn server_port_is_not_secret() {
        assert!(!is_secret("SERVER_PORT"));
    }

    #[test]
    fn lowercase_database_url_is_secret() {
        assert!(is_secret("database_url"));
    }
}
