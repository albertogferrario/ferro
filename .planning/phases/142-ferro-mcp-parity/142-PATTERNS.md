# Phase 142: ferro-mcp Parity - Pattern Map

**Mapped:** 2026-04-20
**Files analyzed:** 3 (2 modified source files + 1 Cargo.toml version bump)
**Analogs found:** 3 / 3

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-mcp/src/tools/stripe.rs` | tool / static-analyzer | file-I/O + transform | `ferro-mcp/src/tools/stripe.rs` (self — evolving in place) | exact (self-mutation) |
| `ferro-mcp/src/service.rs` | service / MCP registration | request-response | `ferro-mcp/src/service.rs` lines 1542–1595 (three existing registrations) | exact (string-only edits) |
| `Cargo.toml` (workspace root) | config | — | `Cargo.toml` `[workspace.package] version` field | exact |

The phase touches exactly these three files. All changes are in-place mutations — no new files are created.

---

## Pattern Assignments

### `ferro-mcp/src/tools/stripe.rs` — `WebhookEventInfo` struct (D-03)

**Analog:** same file, lines 107–115 (current struct definition to replace)

**Current shape (lines 107–115) — DELETE this:**
```rust
#[derive(Debug, Serialize)]
pub struct WebhookEventInfo {
    /// The Ferro event type (e.g., "StripeSubscriptionUpdated").
    pub event_type: String,
    /// The listener struct name (e.g., "SyncSubscriptionPlan").
    pub listener: String,
    /// Relative file path where the listener is defined.
    pub file: String,
}
```

**New shape — REPLACE WITH:**
```rust
#[derive(Debug, Serialize)]
pub struct WebhookEventInfo {
    /// The event type name (e.g., "StripeSubscriptionUpdated").
    pub event_type: String,
    /// Relative file path where the handler registration appears.
    pub file: String,
    /// 1-based line number of the `.on(...)` call.
    pub line: u32,
}
```

`listener: String` is hard-removed. No `Option<String>` wrapper — feature branch, no backward compat needed.

---

### `ferro-mcp/src/tools/stripe.rs` — `stripe_webhook_events` function (D-01, D-02, D-05)

**Analog:** `ferro-mcp/src/introspection/events.rs` lines 113–137 (WalkDir + `filter_map` + `fs::read_to_string` idiom)

**WalkDir traversal pattern to copy from `ferro-mcp/src/introspection/events.rs` lines 113–137:**
```rust
for entry in WalkDir::new(&events_dir)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
{
    if let Ok(content) = fs::read_to_string(entry.path()) {
        let relative_path = entry
            .path()
            .strip_prefix(project_root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();
        // ... process content
    }
}
```

**Regex capture pattern to copy from `ferro-mcp/src/tools/stripe.rs` lines 137–147 (old, adapt for new regexes):**
```rust
let re = Regex::new(r"impl\s+Listener<(\w+)>\s+for\s+(\w+)").unwrap();
let events: Vec<WebhookEventInfo> = re
    .captures_iter(&content)
    .map(|cap| WebhookEventInfo { ... })
    .collect();
```

**New function skeleton — replace `stripe_webhook_events` (lines 124–149) with:**
```rust
pub fn stripe_webhook_events(project_root: &Path) -> StripeWebhookEvents {
    let src_dir = project_root.join("src");
    if !src_dir.is_dir() {
        return StripeWebhookEvents { events: Vec::new() };
    }

    // Primary: closure form  .on(|ident: EventType| ...)
    let re_closure = Regex::new(r"\.on\(\s*\|[a-zA-Z_]+:\s*(\w+)\s*\|").unwrap();
    // Secondary: turbofish form  .on::<EventType
    let re_turbofish = Regex::new(r"\.on::<(\w+)").unwrap();

    let mut events = Vec::new();

    for entry in WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        let content = match fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let relative = entry
            .path()
            .strip_prefix(project_root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();

        for cap in re_closure.captures_iter(&content) {
            let byte_offset = cap.get(0).unwrap().start();
            let line = (content[..byte_offset].lines().count() + 1) as u32;
            events.push(WebhookEventInfo {
                event_type: cap[1].to_string(),
                file: relative.clone(),
                line,
            });
        }

        for cap in re_turbofish.captures_iter(&content) {
            let byte_offset = cap.get(0).unwrap().start();
            let line = (content[..byte_offset].lines().count() + 1) as u32;
            events.push(WebhookEventInfo {
                event_type: cap[1].to_string(),
                file: relative.clone(),
                line,
            });
        }
    }

    StripeWebhookEvents { events }
}
```

**Line number formula** (standard Rust pattern, no analog in codebase — use as shown):
```rust
let byte_offset = cap.get(0).unwrap().start();
let line = (content[..byte_offset].lines().count() + 1) as u32;
```
The `+ 1` is required: `lines().count()` on the prefix is 0-based.

**Required import addition** — `walkdir` is already in `ferro-mcp/Cargo.toml` as a direct dep (line 21). Add at top of `stripe.rs`:
```rust
use walkdir::WalkDir;
```

---

### `ferro-mcp/src/tools/stripe.rs` — `StripeConfigStatus` struct (D-07, D-08)

**Analog:** same file, lines 18–30 (current struct to extend)

**Current shape (lines 18–30) — extend by appending four fields:**
```rust
#[derive(Debug, Serialize)]
pub struct StripeConfigStatus {
    pub configured: bool,
    pub keys_present: Vec<String>,
    pub keys_missing: Vec<String>,
    pub scaffold_exists: bool,
    pub scaffold_files: Vec<String>,
    // New: capability-axis layout detection (D-07)
    pub checkout_exists: bool,
    pub refund_exists: bool,
    pub account_exists: bool,
    pub webhook_dir_exists: bool,
}
```

**`stripe_config_status` function — add after existing `scaffold_files` computation (after line 89), before struct construction:**
```rust
let checkout_exists = scaffold_dir.join("checkout.rs").is_file();
let refund_exists = scaffold_dir.join("refund.rs").is_file();
let account_exists = scaffold_dir.join("account.rs").is_file();
let webhook_dir_exists = scaffold_dir.join("webhook").is_dir();
```

**Pattern for `Path::is_file()` / `Path::is_dir()` checks** — copy from existing `scaffold_exists` check (lines 71–72):
```rust
let scaffold_dir = project_root.join("src/stripe");
let scaffold_exists = scaffold_dir.is_dir();
```

**`scaffold_files` flat listing (D-08) — no change.** The existing `fs::read_dir` on `src/stripe/` (lines 74–89) already excludes directories because it filters on `.extension() == "rs"`. Directories have no `.rs` extension; `webhook/` subdir entries are naturally excluded.

**Struct constructor — add four new fields:**
```rust
StripeConfigStatus {
    configured,
    keys_present,
    keys_missing,
    scaffold_exists,
    scaffold_files,
    checkout_exists,
    refund_exists,
    account_exists,
    webhook_dir_exists,
}
```

---

### `ferro-mcp/src/tools/stripe.rs` — Tests (D-16, D-17, D-18, D-19)

**Analog:** existing tests in same file, lines 326–587. All new tests copy the `TempDir` setup pattern.

**TempDir pattern to copy from lines 332–347:**
```rust
#[test]
fn test_config_status_scaffold_exists() {
    let tmp = TempDir::new().unwrap();
    let stripe_dir = tmp.path().join("src/stripe");
    fs::create_dir_all(&stripe_dir).unwrap();
    fs::write(stripe_dir.join("mod.rs"), "// mod").unwrap();
    // ...
    let status = stripe_config_status(tmp.path());
    assert!(status.scaffold_exists);
}
```

**D-16 — Update `test_webhook_events_parses_listeners` (lines 404–451):**

Replace the entire fixture content and assertions. The file path written can be any `.rs` under `src/`:
```rust
#[test]
fn test_webhook_events_parses_listeners() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("src/stripe")).unwrap();

    let content = r#"
use ferro_stripe::{SyncDispatcher, StripeSubscriptionUpdated};
use std::sync::Arc;

let dispatcher = SyncDispatcher::new()
    .on(|event: StripeSubscriptionUpdated| async move {
        Ok(())
    });
"#;
    fs::write(tmp.path().join("src/stripe/mod.rs"), content).unwrap();

    let result = stripe_webhook_events(tmp.path());
    assert_eq!(result.events.len(), 1);
    assert_eq!(result.events[0].event_type, "StripeSubscriptionUpdated");
    assert!(result.events[0].line > 0);
    assert!(result.events[0].file.contains("src/stripe/mod.rs"));
}
```

**D-17 — New test `test_webhook_events_turbofish`:**
```rust
#[test]
fn test_webhook_events_turbofish() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("src/stripe")).unwrap();

    let content = r#"
dispatcher.on::<StripeCheckoutCompleted, _, _>(handler)
"#;
    fs::write(tmp.path().join("src/stripe/checkout.rs"), content).unwrap();

    let result = stripe_webhook_events(tmp.path());
    assert_eq!(result.events.len(), 1);
    assert_eq!(result.events[0].event_type, "StripeCheckoutCompleted");
    assert!(result.events[0].line > 0);
}
```

**D-18 — New test `test_config_status_capability_axis_fields`:**
```rust
#[test]
fn test_config_status_capability_axis_fields() {
    let tmp = TempDir::new().unwrap();
    let stripe_dir = tmp.path().join("src/stripe");
    fs::create_dir_all(&stripe_dir).unwrap();
    fs::write(stripe_dir.join("checkout.rs"), "// checkout").unwrap();
    fs::write(stripe_dir.join("refund.rs"), "// refund").unwrap();
    // account.rs absent intentionally
    fs::create_dir_all(stripe_dir.join("webhook")).unwrap();

    let status = stripe_config_status(tmp.path());
    assert!(status.checkout_exists);
    assert!(status.refund_exists);
    assert!(!status.account_exists);
    assert!(status.webhook_dir_exists);
}
```

**D-19 — Update `test_webhook_events_serializes` (lines 454–464):**

Remove `listener` field from the `WebhookEventInfo` construction. Replace with:
```rust
#[test]
fn test_webhook_events_serializes() {
    let info = WebhookEventInfo {
        event_type: "StripeSubscriptionUpdated".to_string(),
        file: "src/stripe/mod.rs".to_string(),
        line: 6,
    };
    let events = StripeWebhookEvents { events: vec![info] };
    let json = serde_json::to_string(&events).unwrap();
    assert!(json.contains("StripeSubscriptionUpdated"));
    assert!(json.contains("\"line\":6"));
}
```

Also update `test_config_status_serializes` (lines 360–373) — add the four new fields to the struct literal:
```rust
#[test]
fn test_config_status_serializes() {
    let status = StripeConfigStatus {
        configured: false,
        keys_present: vec!["STRIPE_SECRET_KEY".to_string()],
        keys_missing: vec!["STRIPE_WEBHOOK_SECRET".to_string()],
        scaffold_exists: false,
        scaffold_files: Vec::new(),
        checkout_exists: false,
        refund_exists: false,
        account_exists: false,
        webhook_dir_exists: false,
    };
    // ... rest unchanged
}
```

---

### `ferro-mcp/src/service.rs` — MCP tool descriptions (D-13, D-14, D-15)

**Analog:** same file, lines 1542–1595 (current three `#[tool(...)]` registrations)

**Pattern to copy — `#[tool(...)]` attribute shape (lines 1543–1552):**
```rust
#[tool(
    name = "stripe_config_status",
    description = "Report Stripe configuration status for the current project.\n\n\
        **When to use:** ...\n\n\
        **Returns:** ...\n\n\
        **Combine with:** ..."
)]
pub async fn stripe_config_status(
    &self,
    #[allow(unused_variables)] _params: Parameters<StripeConfigStatusParams>,
) -> String {
    let status = tools::stripe::stripe_config_status(&self.project_root);
    serde_json::to_string_pretty(&status).unwrap_or_else(|_| "{}".to_string())
}
```

Function bodies are unchanged for all three registrations. Only the `description` string inside each `#[tool(...)]` is updated.

**D-13 — `stripe_webhook_events` description (lines 1564–1569), replace with:**
```
"Scan project source for SyncDispatcher webhook handler registrations. \
Returns event types and file locations for all `.on(|event: EventType| ...)` \
calls found in `src/`.\n\n\
**When to use:** Understanding which Stripe events the app handles, \
checking handler coverage, debugging missing event handling.\n\n\
**Returns:** events array with event_type, file path, and line number.\n\n\
**Combine with:** `stripe_config_status` to verify setup, \
`list_jobs` to see ProcessStripeWebhook job."
```

**D-14 — `stripe_config_status` description (lines 1545–1551), replace with:**
```
"Report Stripe configuration status for the current project.\n\n\
**When to use:** Verifying Stripe is configured before running the app, \
checking which env vars are set, confirming the scaffold and capability-axis \
module layout (checkout, refund, account, webhook) exist.\n\n\
**Returns:** configured (bool), keys_present, keys_missing, scaffold_exists, \
scaffold_files, checkout_exists, refund_exists, account_exists, webhook_dir_exists.\n\n\
**Combine with:** `stripe_webhook_events` to check event handlers, \
`stripe_subscription_info` to inspect the billing table schema, \
`get_config` to view Stripe env var values."
```

**D-15 — `stripe_subscription_info` description (lines 1582–1587), replace with:**
```
"Report the tenant_billing table schema parsed from app migration files.\n\n\
**When to use:** Checking the app billing table structure, understanding column types \
and nullability, verifying the migration was generated. Scans app migrations for \
`tenant_billing` — not the ferro-stripe framework module.\n\n\
**Returns:** table_exists, migration_file path, columns (name, sql_type, nullable, default), indexes.\n\n\
**Combine with:** `list_migrations` to see migration status, \
`db_schema` for live table introspection after migration."
```

---

### `Cargo.toml` (workspace root) — Version Bump (D-20)

**Analog:** `Cargo.toml` line 27 (current value)

**Current (line 27):**
```toml
version = "0.2.2"
```

**Replace with:**
```toml
version = "0.2.3"
```

Single field change in `[workspace.package]`. All crates use `version.workspace = true` so the bump propagates automatically.

---

## Shared Patterns

### File I/O: WalkDir for recursive `.rs` source traversal
**Source:** `ferro-mcp/src/introspection/events.rs` lines 114–137
**Apply to:** `stripe_webhook_events` function only (other functions use flat `fs::read_dir`)
```rust
for entry in WalkDir::new(&src_dir)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
{
    let content = match fs::read_to_string(entry.path()) {
        Ok(c) => c,
        Err(_) => continue,
    };
    let relative = entry
        .path()
        .strip_prefix(project_root)
        .unwrap_or(entry.path())
        .to_string_lossy()
        .to_string();
}
```

### Regex capture iteration
**Source:** `ferro-mcp/src/tools/stripe.rs` lines 137–147 (old pattern — adapt, do not copy verbatim)
```rust
let re = Regex::new(r"some_pattern").unwrap();
for cap in re.captures_iter(&content) {
    // cap[1] is first capture group
}
```

### TempDir test setup
**Source:** `ferro-mcp/src/tools/stripe.rs` lines 332–347
```rust
let tmp = TempDir::new().unwrap();
let some_dir = tmp.path().join("src/stripe");
fs::create_dir_all(&some_dir).unwrap();
fs::write(some_dir.join("file.rs"), content).unwrap();
// ... call function with tmp.path()
// TempDir drops and cleans up automatically
```

### MCP tool return pattern
**Source:** `ferro-mcp/src/service.rs` lines 1557–1559
```rust
serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
```
All three tool methods return `String`; this one-liner is the established pattern.

---

## No Analog Found

No files in this phase lack a codebase analog. All patterns are present in existing ferro-mcp source.

---

## Implementation Order

The three changes within `stripe.rs` are coupled: struct field removal (`WebhookEventInfo.listener`) breaks the two existing tests immediately. The recommended order within a single task:

1. Update `WebhookEventInfo` struct (removes `listener`, adds `line`)
2. Update `StripeConfigStatus` struct (adds four booleans)
3. Rewrite `stripe_webhook_events` function body
4. Update `stripe_config_status` function body (add four checks + struct fields)
5. Update / remove tests (D-16, D-17, D-18, D-19) — compile check at this point
6. Update `service.rs` description strings (D-13, D-14, D-15) — independent, safe last
7. Bump workspace `Cargo.toml` version

---

## Metadata

**Analog search scope:** `ferro-mcp/src/tools/`, `ferro-mcp/src/introspection/`, `ferro-mcp/src/service.rs`, `ferro-stripe/src/webhook/`
**Files read:** 6 (`stripe.rs`, `service.rs`, `events.rs` introspection, `sync.rs`, `events.rs` stripe, `Cargo.toml`)
**Pattern extraction date:** 2026-04-20
