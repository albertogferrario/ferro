---
phase: 151
plan: 151-03
slug: config
wave: 2
depends_on: [151-01]
files_modified:
  - ferro-wallet/src/config.rs
  - ferro-wallet/src/lib.rs
autonomous: true
requirements: [ACC-1a, ACC-1b, ACC-1c]
must_haves:
  truths:
    - "`WalletConfig::from_env()` never errors on missing Apple or Google wallet env vars"
    - "`APP_NAME` and `APP_URL` fall back to `\"Ferro Application\"` / `\"http://localhost:8080\"` (matches `framework::config::AppConfig`)"
    - "Missing or partially-set Apple cluster ⇒ `apple: None`; same for Google (D-02)"
    - "All env-touching tests follow the save→remove→assert→restore pattern (no cross-test env pollution)"
  artifacts:
    - path: "ferro-wallet/src/config.rs"
      provides: "WalletConfig + AppleConfig + GoogleConfig + permissive from_env"
      contains: "pub fn from_env() -> Result<Self, WalletError>"
      min_lines: 150
    - path: "ferro-wallet/src/lib.rs"
      provides: "Restored `pub use config::{AppleConfig, GoogleConfig, WalletConfig};`"
      contains: "pub use config::"
  key_links:
    - from: "WalletConfig::from_env"
      to: "framework::config::AppConfig defaults"
      via: "matching default strings"
      pattern: "\"Ferro Application\""
    - from: "AppleConfig::from_env_optional"
      to: "Option<AppleConfig>"
      via: "any missing required var → Ok(None)"
      pattern: "fn from_env_optional"
---

<objective>
Implement `WalletConfig::from_env` with D-02's permissive semantics — missing wallet env vars never error. Mirror `ferro-stripe::StripeConfig::from_env`'s env-pollution-safe test pattern but invert assertions to confirm permissiveness. Restore the `pub use config::{...}` re-exports in `lib.rs`.
</objective>

<context>
@.planning/phases/151-ferro-wallet-crate/151-CONTEXT.md
@.planning/phases/151-ferro-wallet-crate/151-PATTERNS.md
@.planning/phases/151-ferro-wallet-crate/151-RESEARCH.md
@.planning/phases/151-ferro-wallet-crate/151-VALIDATION.md
@docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md
@ferro-stripe/src/config.rs
@framework/src/config/providers/app.rs
@ferro-wallet/src/lib.rs
@ferro-wallet/src/error.rs

<interfaces>
Public API per spec §3.2:
```rust
pub struct WalletConfig {
    pub app_name: String,
    pub app_url: String,
    pub apple: Option<AppleConfig>,
    pub google: Option<GoogleConfig>,
}

pub struct AppleConfig {
    pub pass_type_id: String,        // APPLE_WALLET_PASS_TYPE_ID
    pub team_id: String,             // APPLE_WALLET_TEAM_ID
    pub cert_pem: String,            // APPLE_WALLET_CERT_PEM
    pub key_pem: String,             // APPLE_WALLET_KEY_PEM
    pub key_password: Option<String>,// APPLE_WALLET_KEY_PASSWORD
    pub wwdr_pem: String,            // APPLE_WALLET_WWDR_PEM
}

pub struct GoogleConfig {
    pub issuer_id: String,                          // GOOGLE_WALLET_ISSUER_ID
    pub service_account_email: String,              // GOOGLE_WALLET_SERVICE_ACCOUNT_EMAIL
    pub service_account_private_key_pem: String,    // GOOGLE_WALLET_SERVICE_ACCOUNT_KEY_PEM
}

impl WalletConfig {
    pub fn from_env() -> Result<Self, WalletError>;
}
```

Defaults source-of-truth: `framework/src/config/providers/app.rs` lines 16–26 — `APP_NAME` → `"Ferro Application"`, `APP_URL` → `"http://localhost:8080"`.
</interfaces>
</context>

<must_haves>
- `WalletConfig`, `AppleConfig`, `GoogleConfig` structs match spec §3.2 field names + types exactly.
- All three derive `Debug, Clone`.
- `AppleConfig::from_env_optional()` returns `Ok(None)` if ANY of the 5 required vars (`APPLE_WALLET_PASS_TYPE_ID`, `APPLE_WALLET_TEAM_ID`, `APPLE_WALLET_CERT_PEM`, `APPLE_WALLET_KEY_PEM`, `APPLE_WALLET_WWDR_PEM`) is missing. `key_password` is always optional.
- `GoogleConfig::from_env_optional()` returns `Ok(None)` if ANY of the 3 required vars is missing.
- `WalletConfig::from_env` defaults `APP_NAME` to `"Ferro Application"` and `APP_URL` to `"http://localhost:8080"`.
- Tests cover ACC-1a, ACC-1b, ACC-1c with the env-pollution-safe save→remove→assert→restore pattern (RESEARCH.md Pitfall 6).
- `lib.rs` re-export block restored.
</must_haves>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Implement WalletConfig + AppleConfig + GoogleConfig + permissive from_env + tests</name>
  <files>ferro-wallet/src/config.rs</files>
  <read_first>
    - docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md §3.2 (env var name list + struct fields)
    - ferro-stripe/src/config.rs lines 1–90 (struct shape + env-pollution-safe test pattern)
    - framework/src/config/providers/app.rs lines 16–26 (default values for APP_NAME / APP_URL)
    - 151-PATTERNS.md §"ferro-wallet/src/config.rs"
    - 151-CONTEXT.md D-02 (permissive semantics)
    - 151-RESEARCH.md §"Common Pitfalls" Pitfall 6 (env-pollution-safe test pattern)
    - 151-VALIDATION.md rows for ACC-1a / ACC-1b / ACC-1c (test names)
  </read_first>
  <behavior>
    - With `APPLE_WALLET_PASS_TYPE_ID` unset (other Apple vars unset too), `WalletConfig::from_env().unwrap().apple.is_none()` (ACC-1a).
    - With `GOOGLE_WALLET_ISSUER_ID` unset, `WalletConfig::from_env().unwrap().google.is_none()` (ACC-1b).
    - With `APP_NAME` and `APP_URL` both unset, `from_env().unwrap()` returns `app_name == "Ferro Application"` and `app_url == "http://localhost:8080"` (ACC-1c).
    - With all 5 required Apple vars set, `apple.is_some()` and its fields match the env values.
    - With all 3 required Google vars set, `google.is_some()` and its fields match.
    - With Apple set but only 4 of 5 required vars present, `apple.is_none()`.
    - `from_env` never returns `Err` for any combination of present/absent wallet env vars.
  </behavior>
  <action>
    Replace the `// placeholder` line. Implement:

    ```rust
    //! Permissive env-driven configuration for ferro-wallet.
    //!
    //! Per D-02: missing Apple cluster ⇒ `apple: None`, missing Google cluster ⇒ `google: None`.
    //! `WalletConfig::from_env` never returns `Err` for absent wallet env vars — callers
    //! gate features on `apple.is_some()` / `google.is_some()`. `APP_NAME` / `APP_URL`
    //! fall back to the same defaults as `framework::config::AppConfig`.

    use crate::WalletError;

    #[derive(Debug, Clone)]
    pub struct WalletConfig {
        pub app_name: String,
        pub app_url: String,
        pub apple: Option<AppleConfig>,
        pub google: Option<GoogleConfig>,
    }

    #[derive(Debug, Clone)]
    pub struct AppleConfig {
        pub pass_type_id: String,
        pub team_id: String,
        pub cert_pem: String,
        pub key_pem: String,
        pub key_password: Option<String>,
        pub wwdr_pem: String,
    }

    #[derive(Debug, Clone)]
    pub struct GoogleConfig {
        pub issuer_id: String,
        pub service_account_email: String,
        pub service_account_private_key_pem: String,
    }

    impl WalletConfig {
        pub fn from_env() -> Result<Self, WalletError> {
            // Defaults mirror framework::config::providers::app.rs lines 20 + 23.
            let app_name = std::env::var("APP_NAME")
                .unwrap_or_else(|_| "Ferro Application".to_string());
            let app_url = std::env::var("APP_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string());

            let apple = AppleConfig::from_env_optional()?;
            let google = GoogleConfig::from_env_optional()?;

            Ok(Self { app_name, app_url, apple, google })
        }
    }

    impl AppleConfig {
        /// Returns `Ok(None)` if ANY of the 5 required Apple env vars is missing.
        /// Never returns `Err` (D-02 — wallet config is permissive).
        pub fn from_env_optional() -> Result<Option<Self>, WalletError> {
            let pass_type_id = match std::env::var("APPLE_WALLET_PASS_TYPE_ID") {
                Ok(v) => v,
                Err(_) => return Ok(None),
            };
            let team_id = match std::env::var("APPLE_WALLET_TEAM_ID") {
                Ok(v) => v,
                Err(_) => return Ok(None),
            };
            let cert_pem = match std::env::var("APPLE_WALLET_CERT_PEM") {
                Ok(v) => v,
                Err(_) => return Ok(None),
            };
            let key_pem = match std::env::var("APPLE_WALLET_KEY_PEM") {
                Ok(v) => v,
                Err(_) => return Ok(None),
            };
            let wwdr_pem = match std::env::var("APPLE_WALLET_WWDR_PEM") {
                Ok(v) => v,
                Err(_) => return Ok(None),
            };
            let key_password = std::env::var("APPLE_WALLET_KEY_PASSWORD").ok();
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
        pub fn from_env_optional() -> Result<Option<Self>, WalletError> {
            let issuer_id = match std::env::var("GOOGLE_WALLET_ISSUER_ID") {
                Ok(v) => v,
                Err(_) => return Ok(None),
            };
            let service_account_email =
                match std::env::var("GOOGLE_WALLET_SERVICE_ACCOUNT_EMAIL") {
                    Ok(v) => v,
                    Err(_) => return Ok(None),
                };
            let service_account_private_key_pem =
                match std::env::var("GOOGLE_WALLET_SERVICE_ACCOUNT_KEY_PEM") {
                    Ok(v) => v,
                    Err(_) => return Ok(None),
                };
            Ok(Some(Self {
                issuer_id,
                service_account_email,
                service_account_private_key_pem,
            }))
        }
    }
    ```

    Append `#[cfg(test)] mod tests` block. EVERY test that touches env vars MUST follow the save-remove-assert-restore pattern (RESEARCH.md Pitfall 6 / ferro-stripe/src/config.rs lines 49–73). Define a helper `with_env_clean` if it reduces duplication, or inline per-test. Required tests:

    - `from_env_apple_missing_is_none` (ACC-1a) — remove all 5 Apple env vars + key_password, call `from_env`, assert `apple.is_none()`, restore.
    - `from_env_google_missing_is_none` (ACC-1b) — remove all 3 Google env vars, call `from_env`, assert `google.is_none()`, restore.
    - `from_env_defaults_match_appconfig` (ACC-1c) — remove `APP_NAME` and `APP_URL`, call `from_env`, assert `app_name == "Ferro Application"` and `app_url == "http://localhost:8080"`, restore.
    - `from_env_apple_partial_returns_none` — set 4 of 5 required Apple vars, leave one unset, assert `apple.is_none()`. Restore env.
    - `from_env_apple_all_set_returns_some` — set all 5 required Apple vars + optional password, assert `apple.is_some()` and field values match. Restore env.
    - `from_env_google_all_set_returns_some` — set all 3 Google vars, assert `google.is_some()` and fields match. Restore env.
    - `from_env_never_errors_on_missing_wallet_vars` — remove every APPLE_WALLET_* and GOOGLE_WALLET_* var, assert `from_env().is_ok()`.

    Each test's restore section MUST restore every env var the test touched, even if the assertion panics — use a small drop-guard pattern OR explicit `set_var` of saved values at end of test body. (A panic in the middle leaves env dirty; ferro-stripe accepts this risk by ordering save→remove→assert→restore and not using a guard. Match that pattern.)
  </action>
  <verify>
    <automated>cargo build -p ferro-wallet &amp;&amp; cargo test -p ferro-wallet --lib config::tests::from_env_apple_missing_is_none &amp;&amp; cargo test -p ferro-wallet --lib config::tests::from_env_google_missing_is_none &amp;&amp; cargo test -p ferro-wallet --lib config::tests::from_env_defaults_match_appconfig &amp;&amp; cargo test -p ferro-wallet --lib config::tests &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; grep -F 'pub fn from_env() -&gt; Result&lt;Self, WalletError&gt;' ferro-wallet/src/config.rs &amp;&amp; grep -F '"Ferro Application"' ferro-wallet/src/config.rs &amp;&amp; grep -F '"http://localhost:8080"' ferro-wallet/src/config.rs</automated>
  </verify>
  <done>Three structs + permissive `from_env` + 7 tests land. ACC-1a, ACC-1b, ACC-1c test names exist and pass. Defaults match `framework::config::AppConfig`. Tests do not leak env state.</done>
</task>

<task type="auto">
  <name>Task 2: Restore config re-exports in lib.rs</name>
  <files>ferro-wallet/src/lib.rs</files>
  <read_first>
    - ferro-wallet/src/lib.rs (commented-out `pub use config::` line from PLAN-01)
    - 151-CONTEXT.md D-11
  </read_first>
  <action>
    Uncomment the `pub use config::{AppleConfig, GoogleConfig, WalletConfig};` line. Leave `apple::ApplePassBuilder` and `google::GoogleWalletBuilder` commented out — those restore in PLAN-05 and PLAN-07.
  </action>
  <verify>
    <automated>cargo build -p ferro-wallet &amp;&amp; cargo test -p ferro-wallet --lib config::tests &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; grep -F 'pub use config::{AppleConfig, GoogleConfig, WalletConfig};' ferro-wallet/src/lib.rs</automated>
  </verify>
  <done>`ferro_wallet::WalletConfig`, `ferro_wallet::AppleConfig`, `ferro_wallet::GoogleConfig` resolve from outside the crate. Build + tests green.</done>
</task>

</tasks>

<threat_model>
This plan reads env vars but never writes them outside `#[cfg(test)]` scopes. No crypto material is loaded — config stores PEM strings inertly; parsing happens later in PLAN-05 (`SigningMaterial::parse`) and PLAN-07 (`EncodingKey::from_rsa_pem`).

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-151-Apple-SIGN | I | `AppleConfig` stores `cert_pem`, `key_pem`, `wwdr_pem` as `String` | accept (partial) | `AppleConfig` derives `Debug` — `Debug` output WILL include the PEM string. Production code should not `dbg!(config)` in logs. This is identical to `StripeConfig`'s exposure of `api_key` via `Debug`. Mitigation: document in module doc-comment that `Debug` is included for ergonomics and callers must not log it. |
| T-151-DEFAULT-CRED | I | Defaults `"Ferro Application"`, `"http://localhost:8080"` | accept | Defaults match `framework::config::AppConfig` exactly. They are project-agnostic placeholders, not credentials. |
</threat_model>

<verification>
- `cargo test -p ferro-wallet --lib config::tests` runs ≥7 tests, all pass.
- `cargo build -p ferro-wallet` exits 0.
- `cargo clippy -p ferro-wallet --all-targets -- -D warnings` exits 0.
- `cargo fmt -p ferro-wallet -- --check` exits 0.
- `grep -F 'pub use config::{AppleConfig, GoogleConfig, WalletConfig};' ferro-wallet/src/lib.rs` returns one match.
- ACC-1a, ACC-1b, ACC-1c verifiable via the exact test commands in VALIDATION.md.
</verification>

<success_criteria>
PLAN-05 (`ApplePassBuilder::new(cfg: AppleConfig, app_name: String)`) and PLAN-07 (`GoogleWalletBuilder::new(cfg: GoogleConfig, app_name: String, app_url: String)`) can construct their builders from a `WalletConfig` obtained via `from_env`. Downstream apps can gate features on `wallet_cfg.apple.is_some()`.
</success_criteria>

<output>
After completion, create `.planning/phases/151-ferro-wallet-crate/151-03-SUMMARY.md` listing the env var names, the default-fallback strings, and the exact test names that ACC-1a/1b/1c map to.
</output>

## PLANNING COMPLETE
