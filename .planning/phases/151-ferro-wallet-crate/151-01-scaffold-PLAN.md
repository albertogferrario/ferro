---
phase: 151
plan: 151-01
slug: scaffold
wave: 1
depends_on: []
files_modified:
  - ferro-wallet/Cargo.toml
  - ferro-wallet/README.md
  - ferro-wallet/src/lib.rs
  - ferro-wallet/src/error.rs
  - ferro-wallet/src/subject.rs
  - ferro-wallet/src/config.rs
  - ferro-wallet/src/images.rs
  - ferro-wallet/src/qr.rs
  - ferro-wallet/src/apple/mod.rs
  - ferro-wallet/src/apple/manifest.rs
  - ferro-wallet/src/apple/sign.rs
  - ferro-wallet/src/apple/package.rs
  - ferro-wallet/src/google/mod.rs
  - ferro-wallet/src/google/object.rs
  - ferro-wallet/src/google/jwt.rs
  - Cargo.toml
  - .github/workflows/publish.yml
autonomous: true
requirements: [ACC-2]
must_haves:
  truths:
    - "Workspace builds green with new `ferro-wallet` crate registered"
    - "All module files exist as placeholders so subsequent waves can land independently without breaking `cargo check`"
    - "Auto-publish workflow knows about the new crate (Wave 1a)"
  artifacts:
    - path: "ferro-wallet/Cargo.toml"
      provides: "Crate manifest with locked dep versions (openssl, zip, jsonwebtoken, image, qrcode-generator, sha1, base64, serde, serde_json, thiserror, chrono)"
      contains: "[package]"
    - path: "ferro-wallet/src/lib.rs"
      provides: "Module declarations + WalletError re-export (apple/google re-exports stripped per D-11)"
      contains: "pub mod apple;"
    - path: "ferro-wallet/src/error.rs"
      provides: "WalletError enum with all variants from D-04"
      contains: "pub enum WalletError"
    - path: "Cargo.toml"
      provides: "Workspace registers ferro-wallet as a member"
      contains: "\"ferro-wallet\","
    - path: ".github/workflows/publish.yml"
      provides: "Wave 1a list includes ferro-wallet"
      contains: "ferro-wallet"
  key_links:
    - from: "Cargo.toml [workspace] members"
      to: "ferro-wallet/Cargo.toml"
      via: "workspace member registration"
      pattern: "\"ferro-wallet\","
    - from: "ferro-wallet/src/lib.rs"
      to: "ferro-wallet/src/error.rs"
      via: "pub mod error + pub use error::WalletError"
      pattern: "pub use error::WalletError"
---

<objective>
Scaffold the `ferro-wallet` crate: manifest, README, module stubs (with `// placeholder` lines), `WalletError` enum, workspace member registration, and Wave 1a entry in `publish.yml`. This plan is sequential and blocks every other plan in Phase 151 (D-11).
</objective>

<context>
@CLAUDE.md
@.planning/phases/151-ferro-wallet-crate/151-CONTEXT.md
@.planning/phases/151-ferro-wallet-crate/151-PATTERNS.md
@.planning/phases/151-ferro-wallet-crate/151-RESEARCH.md
@.planning/phases/151-ferro-wallet-crate/151-VALIDATION.md
@docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md
@ferro-whatsapp/Cargo.toml
@ferro-stripe/Cargo.toml
@ferro-stripe/README.md
@ferro-stripe/src/lib.rs
@ferro-stripe/src/error.rs
@ferro-whatsapp/src/error.rs
@Cargo.toml
@.github/workflows/publish.yml
</context>

<must_haves>
- `ferro-wallet/Cargo.toml` declares the exact dependency set from spec §5.
- Every module file referenced by `lib.rs` exists (no broken `pub mod` declarations).
- `WalletError` enum has all 8 variants per D-04 with name-prefixed `#[error("…")]` strings.
- A `#[cfg(test)] mod tests` block in `error.rs` covers one `to_string()` assertion per variant (mirrors ferro-whatsapp/src/error.rs:38–92).
- Workspace root `Cargo.toml` `[workspace] members` ends with `"ferro-wallet",`.
- `.github/workflows/publish.yml` Wave 1a list ends with ` ferro-wallet`.
- `cargo build --workspace` exits 0.
- `cargo test -p ferro-wallet --lib` exits 0 (only the error tests run; everything else is placeholder).
- `cargo clippy --all --all-targets -- -D warnings` exits 0.
- `cargo fmt --all -- --check` exits 0.
</must_haves>

<tasks>

<task type="auto">
  <name>Task 1: Create crate manifest, README, lib.rs, and module stubs</name>
  <files>
    ferro-wallet/Cargo.toml,
    ferro-wallet/README.md,
    ferro-wallet/src/lib.rs,
    ferro-wallet/src/subject.rs,
    ferro-wallet/src/config.rs,
    ferro-wallet/src/images.rs,
    ferro-wallet/src/qr.rs,
    ferro-wallet/src/apple/mod.rs,
    ferro-wallet/src/apple/manifest.rs,
    ferro-wallet/src/apple/sign.rs,
    ferro-wallet/src/apple/package.rs,
    ferro-wallet/src/google/mod.rs,
    ferro-wallet/src/google/object.rs,
    ferro-wallet/src/google/jwt.rs
  </files>
  <read_first>
    - ferro-whatsapp/Cargo.toml (full file — pattern for `version.workspace = true`, line 3)
    - ferro-stripe/README.md (full file — short README pattern, ~10 lines)
    - ferro-stripe/src/lib.rs lines 1–58 (module declaration + re-export pattern)
    - 151-PATTERNS.md §"ferro-wallet/Cargo.toml (crate manifest)" — exact `[package]` and `[dependencies]` blocks
    - 151-PATTERNS.md §"ferro-wallet/src/lib.rs (crate root + re-exports)" — D-11 stripped-reexports pattern
    - 151-RESEARCH.md §"Standard Stack" (dep version table) and §"Common Pitfalls" Pitfall 5 (openssl-sys system dep)
    - 151-CONTEXT.md decisions D-10 (workspace placement) and D-11 (scaffold order)
  </read_first>
  <action>
    Create `ferro-wallet/Cargo.toml` matching the newer `ferro-whatsapp` pattern (NOT `ferro-stripe`'s standalone-version pattern):

    ```toml
    [package]
    name = "ferro-wallet"
    version.workspace = true
    edition.workspace = true
    license.workspace = true
    description = "Digital wallet pass issuance (Apple .pkpass + Google Wallet) for the Ferro framework"
    repository = "https://github.com/albertogferrario/ferro"
    keywords = ["wallet", "pkpass", "google-wallet", "apple-wallet", "ferro"]
    categories = ["web-programming"]
    readme = "README.md"
    homepage = "https://ferro-rs.dev"

    [dependencies]
    openssl = "0.10"
    zip = "2"
    jsonwebtoken = "9"
    image = "0.25"
    qrcode-generator = "5"
    sha1 = "0.10"
    base64 = "0.22"
    serde = { version = "1", features = ["derive"] }
    serde_json = "1"
    thiserror = "2"
    chrono = { version = "0.4", features = ["serde"] }
    ```

    No `[dev-dependencies]` — integration tests reuse openssl + jsonwebtoken (already deps). Per Open Question 2 / Pitfall 5: start without `features = ["vendored"]`; if CI fails on libssl, PLAN-09 can revisit.

    Create `ferro-wallet/README.md` (~10 lines, neutral tone per CLAUDE.md):

    ```markdown
    # ferro-wallet

    Digital wallet pass issuance for the Ferro framework — Apple `.pkpass` files and Google Wallet save-links.

    The crate exposes a `WalletSubject` trait (the content contract any downstream model implements for its domain object), an `ApplePassBuilder` (PKCS#7-signed `.pkpass` ZIP), and a `GoogleWalletBuilder` (RS256-signed save JWT pointing at `pay.google.com/gp/v/save/{jwt}`). Image normalisation and QR generation helpers are bundled. Reads `APP_NAME` / `APP_URL` from environment, matching the project-agnostic convention shared by every `ferro-*` crate.

    Status: part of the [ferro](https://github.com/albertogferrario/ferro) framework workspace.

    Documentation: https://docs.rs/ferro-wallet

    License: MIT
    ```

    Create `ferro-wallet/src/lib.rs` per D-11 — modules declared, error types re-exported, but `apple::ApplePassBuilder` / `google::GoogleWalletBuilder` re-exports are commented out with a comment pointing at the plan that restores them:

    ```rust
    //! Digital wallet pass issuance (Apple .pkpass + Google Wallet) for the Ferro framework.
    //!
    //! See the crate-level docs in the [README](https://crates.io/crates/ferro-wallet) and
    //! the design spec at `docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md` in the
    //! ferro repo for the public API surface.

    pub mod apple;
    pub mod config;
    pub mod error;
    pub mod google;
    pub mod images;
    pub mod qr;
    pub mod subject;

    // pub use apple::ApplePassBuilder;   // Restored in PLAN-05 (apple builder body lands)
    // pub use config::{AppleConfig, GoogleConfig, WalletConfig};   // Restored in PLAN-03
    pub use error::WalletError;
    // pub use google::GoogleWalletBuilder;   // Restored in PLAN-07 (google builder body lands)
    // pub use subject::{
    //     Branding, Field, FieldAlignment, GeoPoint, PassKind, RgbColor, TextColorMode, WalletSubject,
    // };   // Restored in PLAN-02
    ```

    Each of the following files must contain a single `// placeholder` line so `cargo check` succeeds (per D-11):

    - `ferro-wallet/src/subject.rs`
    - `ferro-wallet/src/config.rs`
    - `ferro-wallet/src/images.rs`
    - `ferro-wallet/src/qr.rs`
    - `ferro-wallet/src/apple/manifest.rs`
    - `ferro-wallet/src/apple/sign.rs`
    - `ferro-wallet/src/apple/package.rs`
    - `ferro-wallet/src/google/object.rs`
    - `ferro-wallet/src/google/jwt.rs`

    `ferro-wallet/src/apple/mod.rs` contains:
    ```rust
    // placeholder — body lands in PLAN-05
    pub mod manifest;
    pub mod package;
    pub mod sign;
    ```

    `ferro-wallet/src/google/mod.rs` contains:
    ```rust
    // placeholder — body lands in PLAN-07
    pub mod jwt;
    pub mod object;
    ```
  </action>
  <verify>
    <automated>test -f ferro-wallet/Cargo.toml &amp;&amp; test -f ferro-wallet/README.md &amp;&amp; test -f ferro-wallet/src/lib.rs &amp;&amp; test -f ferro-wallet/src/apple/mod.rs &amp;&amp; test -f ferro-wallet/src/google/mod.rs &amp;&amp; grep -F 'version.workspace = true' ferro-wallet/Cargo.toml &amp;&amp; grep -F 'openssl = "0.10"' ferro-wallet/Cargo.toml &amp;&amp; grep -F 'qrcode-generator = "5"' ferro-wallet/Cargo.toml &amp;&amp; grep -F 'pub mod apple;' ferro-wallet/src/lib.rs &amp;&amp; grep -F '// pub use apple::ApplePassBuilder;' ferro-wallet/src/lib.rs</automated>
  </verify>
  <done>All 14 source files + Cargo.toml + README.md exist. `lib.rs` declares all 7 top-level modules. `apple::ApplePassBuilder`, `google::GoogleWalletBuilder`, `config::*`, and `subject::*` re-exports are commented out with a pointer to their restoring plan. Each stub file contains either `// placeholder` or a `pub mod ...;` declaration; no other code yet.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement WalletError enum + exhaustive Display tests</name>
  <files>ferro-wallet/src/error.rs</files>
  <read_first>
    - ferro-stripe/src/error.rs lines 1–27 (variant + name-prefix `Display` pattern)
    - ferro-whatsapp/src/error.rs lines 38–92 (exhaustive per-variant `#[test]` pattern)
    - 151-PATTERNS.md §"ferro-wallet/src/error.rs (WalletError enum, thiserror)"
    - 151-CONTEXT.md decision D-04 (variant list + name-prefix convention)
    - docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md §6 (variant enumeration)
  </read_first>
  <behavior>
    - `WalletError::Config("x".into()).to_string() == "config: x"`
    - `WalletError::AppleSign("x".into()).to_string() == "apple sign: x"`
    - `WalletError::ApplePackage("x".into()).to_string() == "apple package: x"`
    - `WalletError::GoogleJwt("x".into()).to_string() == "google jwt: x"`
    - `WalletError::Image("x".into()).to_string() == "image: x"`
    - `WalletError::Qr("x".into()).to_string() == "qr: x"`
    - `WalletError::InvalidInput("x".into()).to_string() == "invalid input: x"`
    - `WalletError::Io(std::io::Error::new(std::io::ErrorKind::Other, "x")).to_string() == "io: x"`
    - `std::io::Error` converts to `WalletError::Io` via `?` (i.e. `From<std::io::Error>`)
  </behavior>
  <action>
    Replace the placeholder line. Create the canonical `WalletError` enum per D-04 — every variant `#[derive(Debug, thiserror::Error)]` with name-prefixed `Display`:

    ```rust
    //! `WalletError` — the single error type for the ferro-wallet crate.
    //!
    //! Each variant's `Display` impl prefixes its name (`"config: …"`, `"apple sign: …"`)
    //! so production log greps stay surgical. `Io(#[from] std::io::Error)` covers zip + io
    //! plumbing in the apple package path.

    #[derive(Debug, thiserror::Error)]
    pub enum WalletError {
        #[error("config: {0}")]
        Config(String),

        #[error("apple sign: {0}")]
        AppleSign(String),

        #[error("apple package: {0}")]
        ApplePackage(String),

        #[error("google jwt: {0}")]
        GoogleJwt(String),

        #[error("image: {0}")]
        Image(String),

        #[error("qr: {0}")]
        Qr(String),

        #[error("invalid input: {0}")]
        InvalidInput(String),

        #[error("io: {0}")]
        Io(#[from] std::io::Error),
    }
    ```

    Append a `#[cfg(test)] mod tests {}` block with one `#[test]` per variant (8 total). Mirror the `ferro-whatsapp/src/error.rs:38–92` shape exactly — each test asserts the `to_string()` output. Include a final test `io_from_std_io_error` that constructs a `std::io::Error` and converts via `WalletError::from(io_err)`, asserting the resulting variant matches `WalletError::Io(_)`.
  </action>
  <verify>
    <automated>cargo build -p ferro-wallet &amp;&amp; cargo test -p ferro-wallet --lib error::tests &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; grep -F '#[error("config: {0}")]' ferro-wallet/src/error.rs &amp;&amp; grep -F '#[error("apple sign: {0}")]' ferro-wallet/src/error.rs &amp;&amp; grep -F '#[error("io: {0}")]' ferro-wallet/src/error.rs &amp;&amp; grep -F 'Io(#[from] std::io::Error)' ferro-wallet/src/error.rs</automated>
  </verify>
  <done>`WalletError` enum has all 8 variants per D-04. `#[cfg(test)] mod tests` contains 8 tests covering every variant's `Display`. `From<std::io::Error>` derive works. Pre-commit gate green.</done>
</task>

<task type="auto">
  <name>Task 3: Register `ferro-wallet` in workspace members and publish workflow Wave 1a</name>
  <files>Cargo.toml, .github/workflows/publish.yml</files>
  <read_first>
    - Cargo.toml lines 1–24 (workspace members array)
    - .github/workflows/publish.yml lines 195–220 (Wave 1a block)
    - 151-PATTERNS.md §"Workspace Edits — Exact Insertion Points" (lines 952–1010)
    - 151-RESEARCH.md §"Workspace Integration" (Edit 1, Edit 3)
    - User memory: "When adding a new crate to the workspace, always add it to `.github/workflows/publish.yml` in the correct wave."
  </read_first>
  <action>
    Edit `/Users/alberto/repositories/albertogferrario/ferro/Cargo.toml`. Append `"ferro-wallet",` after the existing last member (`"ferro-whatsapp",` on line 23). Do NOT alphabetise — the array follows phase-introduction order. Resulting members tail:

    ```toml
        "ferro-whatsapp",
        "ferro-wallet",
    ]
    ```

    Edit `/Users/alberto/repositories/albertogferrario/ferro/.github/workflows/publish.yml`. On the line that currently reads:

    ```
              WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp"
    ```

    append ` ferro-wallet` inside the quoted value so it becomes:

    ```
              WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet"
    ```

    Rationale: ferro-wallet has zero internal `ferro-*` workspace deps (spec §5: "No dependency on `framework` — the crate stays pure"), so Wave 1a is the correct placement. Do NOT touch the Wave 1b list.

    Do NOT bump `[workspace.package] version` here — that lives in PLAN-09 (D-10).
  </action>
  <verify>
    <automated>grep -F '"ferro-wallet",' Cargo.toml &amp;&amp; grep -F 'ferro-api-mcp ferro-wallet"' .github/workflows/publish.yml &amp;&amp; cargo build --workspace &amp;&amp; cargo fmt --all -- --check &amp;&amp; cargo clippy --all --all-targets -- -D warnings &amp;&amp; cargo test -p ferro-wallet --lib</automated>
  </verify>
  <done>`Cargo.toml` workspace members array contains `"ferro-wallet",`. `publish.yml` Wave 1a list ends with ` ferro-wallet`. Full workspace builds, lints, and tests pass. Scaffold ready for waves 2–4.</done>
</task>

</tasks>

<threat_model>
This plan introduces no secret-handling or cryptographic code. `WalletError` is declarative; module stubs are `// placeholder` lines. The workspace + publish.yml edits are build-infrastructure changes with no runtime impact.

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-151-DEFAULT-CRED | I | `Cargo.toml` deps | accept | No secrets introduced; openssl + jsonwebtoken pulled but unused until later plans (D-09 covers test-time crypto minting). |

No further STRIDE entries — this plan is structural only.
</threat_model>

<verification>
- `cargo build --workspace` exits 0 (ACC-2).
- `cargo test -p ferro-wallet --lib` runs the 8 `WalletError` tests, all pass.
- `cargo clippy --all --all-targets -- -D warnings` exits 0.
- `cargo fmt --all -- --check` exits 0.
- `grep -F '"ferro-wallet",' Cargo.toml` returns one match.
- `grep -F 'ferro-api-mcp ferro-wallet"' .github/workflows/publish.yml` returns one match.
- 14 source files exist under `ferro-wallet/src/`.
</verification>

<success_criteria>
The crate is scaffolded such that PLAN-02 (subject), PLAN-03 (config), and PLAN-04 (images + qr) can each land as a separate atomic commit without breaking `cargo check`. The `lib.rs` comment-out pattern for `apple` / `google` / `config` / `subject` re-exports means the wave-by-wave landings stay green between plans (D-11).
</success_criteria>

<output>
After completion, create `.planning/phases/151-ferro-wallet-crate/151-01-SUMMARY.md` documenting what landed (file list + workspace edits) and any deviations from the plan.
</output>

## PLANNING COMPLETE
