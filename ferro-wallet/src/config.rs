//! Permissive env-driven configuration for ferro-wallet.
//!
//! Per D-02: missing Apple cluster ⇒ `apple: None`, missing Google cluster ⇒ `google: None`.
//! `WalletConfig::from_env` never returns `Err` for absent wallet env vars — callers
//! gate features on `apple.is_some()` / `google.is_some()`. `APP_NAME` / `APP_URL`
//! fall back to the same defaults as `framework::config::AppConfig`
//! (`"Ferro Application"` / `"http://localhost:8080"`).
//!
//! `Debug` is derived on every config struct for ergonomics; it includes raw PEM
//! strings for [`AppleConfig`] / [`GoogleConfig`]. Callers MUST NOT log these
//! configs in production — the exposure parity is identical to
//! `ferro_stripe::config::StripeConfig`'s `api_key` exposure.

use crate::WalletError;

/// Top-level wallet configuration.
///
/// `apple` and `google` are independently optional — a deployment can ship
/// Apple-only, Google-only, or neither without errors.
#[derive(Debug, Clone)]
pub struct WalletConfig {
    /// Application name. Defaults to `"Ferro Application"` when `APP_NAME` is unset.
    pub app_name: String,
    /// Application URL. Defaults to `"http://localhost:8080"` when `APP_URL` is unset.
    pub app_url: String,
    /// Apple Wallet signing cluster, or `None` if any required Apple env var is missing.
    pub apple: Option<AppleConfig>,
    /// Google Wallet signing cluster, or `None` if any required Google env var is missing.
    pub google: Option<GoogleConfig>,
}

/// Apple `.pkpass` signing material (PEM-encoded).
///
/// All five required fields must be present in the environment for the cluster
/// to populate — any missing required var ⇒ `WalletConfig.apple = None`.
#[derive(Debug, Clone)]
pub struct AppleConfig {
    /// `APPLE_WALLET_PASS_TYPE_ID` — Apple-issued pass type identifier.
    pub pass_type_id: String,
    /// `APPLE_WALLET_TEAM_ID` — Apple Developer team identifier.
    pub team_id: String,
    /// `APPLE_WALLET_CERT_PEM` — pass-type signing certificate (PEM).
    pub cert_pem: String,
    /// `APPLE_WALLET_KEY_PEM` — pass-type signing private key (PEM).
    pub key_pem: String,
    /// `APPLE_WALLET_KEY_PASSWORD` — optional passphrase for the encrypted key.
    pub key_password: Option<String>,
    /// `APPLE_WALLET_WWDR_PEM` — Apple WWDR intermediate certificate (PEM).
    pub wwdr_pem: String,
}

/// Google Wallet service-account credentials.
///
/// All three required fields must be present in the environment for the cluster
/// to populate — any missing required var ⇒ `WalletConfig.google = None`.
#[derive(Debug, Clone)]
pub struct GoogleConfig {
    /// `GOOGLE_WALLET_ISSUER_ID` — Google Wallet issuer ID (numeric string).
    pub issuer_id: String,
    /// `GOOGLE_WALLET_SERVICE_ACCOUNT_EMAIL` — service-account email.
    pub service_account_email: String,
    /// `GOOGLE_WALLET_SERVICE_ACCOUNT_KEY_PEM` — RSA private key (PEM, PKCS#8 or PKCS#1).
    pub service_account_private_key_pem: String,
}

impl WalletConfig {
    /// Loads wallet configuration from environment variables.
    ///
    /// `APP_NAME` and `APP_URL` fall back to the same defaults as
    /// `framework::config::AppConfig::from_env`: `"Ferro Application"` and
    /// `"http://localhost:8080"`.
    ///
    /// Per D-02 (permissive semantics): missing Apple or Google env vars NEVER
    /// produce an error. Callers gate downstream functionality on
    /// `wallet_cfg.apple.is_some()` / `.google.is_some()`.
    ///
    /// # Errors
    ///
    /// Returns `Result` for forward compatibility — additional non-wallet
    /// validation may be added in future versions. The current implementation
    /// never returns `Err`.
    pub fn from_env() -> Result<Self, WalletError> {
        // Defaults mirror framework::config::providers::app.rs lines 20 + 23.
        let app_name =
            std::env::var("APP_NAME").unwrap_or_else(|_| "Ferro Application".to_string());
        let app_url =
            std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

        let apple = AppleConfig::from_env_optional()?;
        let google = GoogleConfig::from_env_optional()?;

        Ok(Self {
            app_name,
            app_url,
            apple,
            google,
        })
    }
}

impl AppleConfig {
    /// Returns `Ok(Some(cfg))` only when all five required Apple env vars are set;
    /// `Ok(None)` if ANY of them is missing. Never returns `Err`.
    ///
    /// Required vars: `APPLE_WALLET_PASS_TYPE_ID`, `APPLE_WALLET_TEAM_ID`,
    /// `APPLE_WALLET_CERT_PEM`, `APPLE_WALLET_KEY_PEM`, `APPLE_WALLET_WWDR_PEM`.
    /// Optional: `APPLE_WALLET_KEY_PASSWORD`.
    pub fn from_env_optional() -> Result<Option<Self>, WalletError> {
        let pass_type_id = match non_empty_env("APPLE_WALLET_PASS_TYPE_ID") {
            Some(v) => v,
            None => return Ok(None),
        };
        let team_id = match non_empty_env("APPLE_WALLET_TEAM_ID") {
            Some(v) => v,
            None => return Ok(None),
        };
        let cert_pem = match non_empty_env("APPLE_WALLET_CERT_PEM") {
            Some(v) => v,
            None => return Ok(None),
        };
        let key_pem = match non_empty_env("APPLE_WALLET_KEY_PEM") {
            Some(v) => v,
            None => return Ok(None),
        };
        let wwdr_pem = match non_empty_env("APPLE_WALLET_WWDR_PEM") {
            Some(v) => v,
            None => return Ok(None),
        };
        let key_password = non_empty_env("APPLE_WALLET_KEY_PASSWORD");

        Ok(Some(Self {
            pass_type_id,
            team_id,
            cert_pem,
            key_pem,
            key_password,
            wwdr_pem,
        }))
    }
}

impl GoogleConfig {
    /// Returns `Ok(Some(cfg))` only when all three required Google env vars are set
    /// AND non-empty; `Ok(None)` if ANY of them is missing or set to "". Never returns `Err`.
    ///
    /// Required vars: `GOOGLE_WALLET_ISSUER_ID`,
    /// `GOOGLE_WALLET_SERVICE_ACCOUNT_EMAIL`, `GOOGLE_WALLET_SERVICE_ACCOUNT_KEY_PEM`.
    pub fn from_env_optional() -> Result<Option<Self>, WalletError> {
        let issuer_id = match non_empty_env("GOOGLE_WALLET_ISSUER_ID") {
            Some(v) => v,
            None => return Ok(None),
        };
        let service_account_email = match non_empty_env("GOOGLE_WALLET_SERVICE_ACCOUNT_EMAIL") {
            Some(v) => v,
            None => return Ok(None),
        };
        let service_account_private_key_pem =
            match non_empty_env("GOOGLE_WALLET_SERVICE_ACCOUNT_KEY_PEM") {
                Some(v) => v,
                None => return Ok(None),
            };

        Ok(Some(Self {
            issuer_id,
            service_account_email,
            service_account_private_key_pem,
        }))
    }
}

/// Read `name` from the environment, returning `None` for both missing AND empty
/// values. Empty-as-missing makes the wallet env block in `.env.example`
/// scaffolds safe to ship without forcing the builder to fail at startup when
/// an operator copies the example to `.env` without filling values.
fn non_empty_env(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// All env-touching tests in this module serialize through this mutex to avoid
    /// process-global env-var races under `cargo test`'s default parallel execution.
    /// Without this guard, e.g. `from_env_apple_all_set_returns_some` could set
    /// `APPLE_WALLET_PASS_TYPE_ID` while `from_env_apple_missing_is_none` is mid-assert,
    /// producing flaky behaviour. ferro-stripe sidesteps this by having only one
    /// env-touching test in its module; ferro-wallet has seven, so an in-process
    /// mutex is the minimal correctness fix.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Names of every wallet env var the tests in this module read or mutate.
    const APP_VARS: &[&str] = &["APP_NAME", "APP_URL"];
    const APPLE_VARS: &[&str] = &[
        "APPLE_WALLET_PASS_TYPE_ID",
        "APPLE_WALLET_TEAM_ID",
        "APPLE_WALLET_CERT_PEM",
        "APPLE_WALLET_KEY_PEM",
        "APPLE_WALLET_WWDR_PEM",
        "APPLE_WALLET_KEY_PASSWORD",
    ];
    const GOOGLE_VARS: &[&str] = &[
        "GOOGLE_WALLET_ISSUER_ID",
        "GOOGLE_WALLET_SERVICE_ACCOUNT_EMAIL",
        "GOOGLE_WALLET_SERVICE_ACCOUNT_KEY_PEM",
    ];

    /// RAII guard that snapshots a set of env vars on construction and restores
    /// them on drop, so even an assertion panic leaves the process env clean for
    /// the next test. Implements the save→remove→assert→restore pattern
    /// (RESEARCH.md Pitfall 6) in panic-safe form.
    struct EnvGuard {
        saved: Vec<(&'static str, Option<String>)>,
    }

    impl EnvGuard {
        fn capture(vars: &[&'static str]) -> Self {
            let saved = vars
                .iter()
                .map(|v| (*v, std::env::var(v).ok()))
                .collect::<Vec<_>>();
            for v in vars {
                std::env::remove_var(v);
            }
            Self { saved }
        }

        /// Capture every wallet-relevant env var into a single guard. Order of
        /// concatenation is irrelevant — restoration happens uniformly on drop.
        fn capture_all() -> Self {
            let mut all = Vec::with_capacity(APP_VARS.len() + APPLE_VARS.len() + GOOGLE_VARS.len());
            all.extend(APP_VARS);
            all.extend(APPLE_VARS);
            all.extend(GOOGLE_VARS);
            Self::capture(&all)
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (name, value) in &self.saved {
                match value {
                    Some(v) => std::env::set_var(name, v),
                    None => std::env::remove_var(name),
                }
            }
        }
    }

    /// Acquire the env-serialization lock, recovering from a poisoned mutex
    /// (a previous test panicked) so subsequent tests still run deterministically.
    fn lock_env() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// ACC-1a: Missing Apple cluster ⇒ `apple: None`. Never errors.
    #[test]
    fn from_env_apple_missing_is_none() {
        let _lock = lock_env();
        let _env = EnvGuard::capture_all();

        let cfg =
            WalletConfig::from_env().expect("from_env must not error when wallet vars are absent");

        assert!(
            cfg.apple.is_none(),
            "expected apple cluster to be None when APPLE_WALLET_* vars are absent"
        );
    }

    /// ACC-1b: Missing Google cluster ⇒ `google: None`. Never errors.
    #[test]
    fn from_env_google_missing_is_none() {
        let _lock = lock_env();
        let _env = EnvGuard::capture_all();

        let cfg =
            WalletConfig::from_env().expect("from_env must not error when wallet vars are absent");

        assert!(
            cfg.google.is_none(),
            "expected google cluster to be None when GOOGLE_WALLET_* vars are absent"
        );
    }

    /// ACC-1c: APP_NAME / APP_URL defaults must match
    /// `framework::config::providers::app.rs` lines 20 + 23.
    #[test]
    fn from_env_defaults_match_appconfig() {
        let _lock = lock_env();
        let _env = EnvGuard::capture_all();

        let cfg =
            WalletConfig::from_env().expect("from_env must not error when wallet vars are absent");

        assert_eq!(
            cfg.app_name, "Ferro Application",
            "APP_NAME default must match framework::config::AppConfig::from_env"
        );
        assert_eq!(
            cfg.app_url, "http://localhost:8080",
            "APP_URL default must match framework::config::AppConfig::from_env"
        );
    }

    /// Setting four of five required Apple vars must still resolve to `apple: None`.
    /// Guards against accidental partial-cluster acceptance.
    #[test]
    fn from_env_apple_partial_returns_none() {
        let _lock = lock_env();
        let _env = EnvGuard::capture_all();

        std::env::set_var("APPLE_WALLET_PASS_TYPE_ID", "pass.com.example.test");
        std::env::set_var("APPLE_WALLET_TEAM_ID", "TEAMID1234");
        std::env::set_var(
            "APPLE_WALLET_CERT_PEM",
            "-----BEGIN CERTIFICATE-----\nx\n-----END CERTIFICATE-----\n",
        );
        std::env::set_var(
            "APPLE_WALLET_KEY_PEM",
            "-----BEGIN PRIVATE KEY-----\nx\n-----END PRIVATE KEY-----\n",
        );
        // APPLE_WALLET_WWDR_PEM intentionally unset.

        let cfg =
            WalletConfig::from_env().expect("from_env must not error on partial Apple cluster");

        assert!(
            cfg.apple.is_none(),
            "expected apple cluster to be None when WWDR_PEM is unset"
        );
    }

    /// All five required Apple vars set + optional password ⇒ populated cluster
    /// with matching field values.
    #[test]
    fn from_env_apple_all_set_returns_some() {
        let _lock = lock_env();
        let _env = EnvGuard::capture_all();

        std::env::set_var("APPLE_WALLET_PASS_TYPE_ID", "pass.com.example.test");
        std::env::set_var("APPLE_WALLET_TEAM_ID", "TEAMID1234");
        std::env::set_var("APPLE_WALLET_CERT_PEM", "cert-pem-bytes");
        std::env::set_var("APPLE_WALLET_KEY_PEM", "key-pem-bytes");
        std::env::set_var("APPLE_WALLET_WWDR_PEM", "wwdr-pem-bytes");
        std::env::set_var("APPLE_WALLET_KEY_PASSWORD", "secret");

        let cfg =
            WalletConfig::from_env().expect("from_env must not error when wallet vars are present");
        let apple = cfg
            .apple
            .expect("apple cluster must populate when all required vars set");

        assert_eq!(apple.pass_type_id, "pass.com.example.test");
        assert_eq!(apple.team_id, "TEAMID1234");
        assert_eq!(apple.cert_pem, "cert-pem-bytes");
        assert_eq!(apple.key_pem, "key-pem-bytes");
        assert_eq!(apple.wwdr_pem, "wwdr-pem-bytes");
        assert_eq!(apple.key_password.as_deref(), Some("secret"));
    }

    /// All three required Google vars set ⇒ populated cluster with matching field values.
    #[test]
    fn from_env_google_all_set_returns_some() {
        let _lock = lock_env();
        let _env = EnvGuard::capture_all();

        std::env::set_var("GOOGLE_WALLET_ISSUER_ID", "3388000000000000000");
        std::env::set_var(
            "GOOGLE_WALLET_SERVICE_ACCOUNT_EMAIL",
            "sa@example.iam.gserviceaccount.com",
        );
        std::env::set_var(
            "GOOGLE_WALLET_SERVICE_ACCOUNT_KEY_PEM",
            "private-key-pem-bytes",
        );

        let cfg =
            WalletConfig::from_env().expect("from_env must not error when wallet vars are present");
        let google = cfg
            .google
            .expect("google cluster must populate when all required vars set");

        assert_eq!(google.issuer_id, "3388000000000000000");
        assert_eq!(
            google.service_account_email,
            "sa@example.iam.gserviceaccount.com"
        );
        assert_eq!(
            google.service_account_private_key_pem,
            "private-key-pem-bytes"
        );
    }

    /// D-02 invariant: regardless of which wallet env vars are present or absent,
    /// `from_env` MUST return `Ok`. This is the load-bearing permissive guarantee.
    #[test]
    fn from_env_never_errors_on_missing_wallet_vars() {
        let _lock = lock_env();
        let _env = EnvGuard::capture_all();

        assert!(
            WalletConfig::from_env().is_ok(),
            "from_env must never error when all wallet env vars are absent"
        );
    }
}
