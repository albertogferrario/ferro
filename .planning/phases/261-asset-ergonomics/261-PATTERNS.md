# Phase 261: `asset!()` ergonomics — Pattern Map

**Mapped:** 2026-07-26
**Files analyzed:** 9 new/modified files
**Analogs found:** 9 / 9

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-macros/src/asset.rs` | utility (proc-macro) | transform | `ferro-macros/src/memoize.rs` | exact |
| `ferro-macros/src/lib.rs` | config (macro registration) | transform | `ferro-macros/src/lib.rs` lines 305–308 | exact (same file) |
| `ferro-macros/tests/asset_macro.rs` | test | transform | `ferro-macros/tests/action_macro.rs` | exact |
| `ferro-macros/tests/ui/asset/pass/minimal.rs` | test fixture | transform | `ferro-macros/tests/ui/action/pass/minimal.rs` | exact |
| `ferro-bundle/src/lib.rs` | utility (library) | transform | `ferro-bundle/src/lib.rs` lines 89–106 | exact (same file, inverse function) |
| `ferro-bundle/Cargo.toml` | config | — | `ferro-bundle/Cargo.toml` line 18 (remove dep) | exact (same file, deletion) |
| `framework/src/lib.rs` | config (re-exports) | — | `framework/src/lib.rs` lines 205–213 (queue module) | role-match |
| `framework/Cargo.toml` | config | — | `framework/Cargo.toml` lines 41–48 (ferro-* deps block) | exact |
| `.github/workflows/publish.yml` | config | — | publish.yml lines 217, 335 | exact (same file, wave reassignment) |
| `ferro-cli/src/commands/assets.rs` | utility (CLI command) | request-response | `ferro-cli/src/commands/api_check.rs` | role-match |
| `ferro-cli/src/commands/mod.rs` | config | — | `ferro-cli/src/commands/mod.rs` lines 1–62 | exact (same file) |
| `ferro-cli/src/main.rs` | config (CLI dispatch) | — | `ferro-cli/src/main.rs` lines 458–543 (Mcp/Doctor variants) | exact (same file) |

---

## Pattern Assignments

### `ferro-macros/src/asset.rs` (NEW — proc-macro utility, transform)

**Analog:** `ferro-macros/src/memoize.rs`

**Imports pattern** (`ferro-macros/src/memoize.rs` lines 39–44):
```rust
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::sync::atomic::{AtomicUsize, Ordering};
use syn::{parse_macro_input, FnArg, ItemFn, Pat, ReturnType};

use crate::utils::ferro;
```

For `asset.rs`, the import set is simpler (no `FnArg`, `ItemFn`, etc.; uses `LitStr`):
```rust
use proc_macro::TokenStream;
use quote::quote;
use std::path::Path;
use syn::{parse_macro_input, LitStr};

use crate::utils::ferro;
```

**`ferro()` root-path helper usage** (`ferro-macros/src/memoize.rs` lines 58, 152):
```rust
let ferro = ferro();
// ... then in quote! { }:
#ferro::memo::MemoKey::new::<#marker_name, _>(...)
```
For `asset.rs`, the same pattern applies emitting `#ferro::bundle::Bundle` and `#ferro::bundle::mime_from_ext`.

**`crate::utils::ferro()` definition** (`ferro-macros/src/utils.rs` lines 55–60):
```rust
/// Returns the token stream for the ferro crate path: `::ferro`.
/// Emitted as the absolute path so generated code resolves correctly in
/// consumer crates that depend on `ferro-rs` under the `ferro` alias.
pub(crate) fn ferro() -> TokenStream2 {
    quote!(::ferro)
}
```
This is reused as-is. In workspace tests, `extern crate self as ferro;` (see `framework/src/lib.rs` line 10) makes `::ferro` resolve to `crate`. Downstream it resolves to the `ferro-rs` package. No changes needed to `utils.rs`.

**Core macro expansion pattern** (`ferro-macros/src/memoize.rs` lines 141–184 — OnceLock-style expansion):
```rust
let output = quote! {
    #(#fn_attrs)*
    #fn_vis async fn #fn_name #fn_generics(#(#all_inputs),*) #fn_output
    where
        ...
    {
        struct #marker_name;

        let __ferro_memo_key = #ferro::memo::MemoKey::new::<#marker_name, _>(
            &( #( &#value_arg_names, )* ),
        );

        if let ::std::option::Option::Some(__ferro_store) =
            #ferro::memo::current_memo_store()
        {
            let __ferro_slot = __ferro_store.get_or_insert(
                __ferro_memo_key,
                move || { ::std::boxed::Box::pin(async move { ... }) },
            );
            let __ferro_arc = __ferro_slot.await;
            return ::std::clone::Clone::clone(...);
        }
        { #fn_block }
    }
};
output.into()
```

For `asset.rs`, the expansion is simpler (a block expression, not a fn rewrite):
```rust
let output = quote! {
    {
        static __FERRO_ASSET_URL: ::std::sync::OnceLock<::std::string::String>
            = ::std::sync::OnceLock::new();
        __FERRO_ASSET_URL.get_or_init(|| {
            static __FERRO_ASSET_BYTES: &[u8] = include_bytes!(#path_lit);
            #ferro::bundle::Bundle::new(#bundle_name_str, __FERRO_ASSET_BYTES)
                .content_type(#ferro::bundle::mime_from_ext(#ext_str))
                .hashed_url()
        }).as_str()
    }
};
output.into()
```

**Input parsing pattern** — `memoize.rs` parses an `ItemFn`; `asset.rs` parses a single `LitStr`:
```rust
// memoize.rs line 57:
let input_fn = parse_macro_input!(input as ItemFn);

// asset.rs equivalent:
let path_lit = parse_macro_input!(input as LitStr);
let path_str = path_lit.value();
```

**Error emission pattern** (`ferro-macros/src/memoize.rs` lines 60–67):
```rust
if input_fn.sig.asyncness.is_none() {
    return syn::Error::new_spanned(
        &input_fn.sig,
        "#[memoize] can only be applied to `async fn`",
    )
    .to_compile_error()
    .into();
}
```
`asset.rs` does not need this guard (no function parsing), but the `syn::Error::new_spanned(...).to_compile_error().into()` pattern is the correct error return if a LitStr parse fails.

---

### `ferro-macros/src/lib.rs` (MODIFY — macro registration)

**Analog:** `ferro-macros/src/lib.rs` lines 13–30 (mod declarations) + lines 305–308 (`memoize` registration).

**Module declaration pattern** (lines 13–30):
```rust
mod action;
mod describe;
mod domain_error;
// ... (alphabetical list)
mod memoize;
mod utils;
// ...
```
Add `mod asset;` in alphabetical order (after `mod action;`, before `mod describe;`).

**`#[proc_macro]` registration pattern** (`ferro-macros/src/lib.rs` lines 305–308):
```rust
/// Mark an `async fn` or `async` impl method for request-scoped memoization.
///
/// [full doc comment]
#[proc_macro_attribute]
pub fn memoize(attr: TokenStream, input: TokenStream) -> TokenStream {
    memoize::memoize_impl(attr, input)
}
```
For `asset!()`, it is a `#[proc_macro]` (function-like, not attribute), matching the pattern of `inertia_response` and `redirect`:

```rust
// ferro-macros/src/lib.rs lines 70–73:
#[proc_macro]
pub fn inertia_response(input: TokenStream) -> TokenStream {
    inertia::inertia_response_impl(input)
}
```

Registration for `asset`:
```rust
/// Embed a static asset at compile time and register it as a content-hashed [`ferro::bundle::Bundle`].
///
/// Returns `&'static str` — the content-hashed URL (e.g. `/bundles/assets_app.a1b2c3.js`).
/// Path is resolved relative to the source file (call-site-source-relative, same as `include_bytes!`).
///
/// # Example
///
/// ```rust,ignore
/// let url: &'static str = ferro::asset!("assets/app.js");
/// ```
#[proc_macro]
pub fn asset(input: TokenStream) -> TokenStream {
    asset::asset_impl(input)
}
```

---

### `ferro-macros/tests/asset_macro.rs` (NEW — trybuild harness)

**Analog:** `ferro-macros/tests/action_macro.rs` (entire file, 15 lines):
```rust
//! Trybuild UI tests for the `#[action]` proc-macro.
//!
//! - `tests/ui/action/pass/*.rs` — fixtures that MUST compile cleanly.
//! - `tests/ui/action/fail/*.rs` + `*.stderr` — fixtures that MUST emit the
//!   exact compile error captured in the matching `.stderr` snapshot.
//!
//! Update `.stderr` snapshots after intentional message changes:
//!     TRYBUILD=overwrite cargo test -p ferro-macros --test action_macro

#[test]
fn action_macro_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/action/pass/*.rs");
    t.compile_fail("tests/ui/action/fail/*.rs");
}
```

For `asset_macro.rs`, Phase 261 ships pass fixtures only (no fail fixtures in scope):
```rust
//! Trybuild UI tests for the `asset!()` proc-macro.
//!
//! - `tests/ui/asset/pass/*.rs` — fixtures that MUST compile cleanly.
//!
//! Update snapshots after intentional message changes:
//!     TRYBUILD=overwrite cargo test -p ferro-macros --test asset_macro

#[test]
fn asset_macro_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/asset/pass/*.rs");
}
```

---

### `ferro-macros/tests/ui/asset/pass/minimal.rs` (NEW — trybuild pass fixture)

**Analog:** `ferro-macros/tests/ui/action/pass/minimal.rs` (entire file):
```rust
//! Compile-pass: minimal `#[action]` — `Ok(())` and no `?`.

#![allow(unused_imports)]

extern crate ferro_rs as ferro;

use ferro::{action, ActionResult, Request};

#[action(redirect_to = "/dashboard")]
pub async fn h(_req: Request) -> ActionResult {
    Ok(())
}

fn main() {}
```

Key points:
- `extern crate ferro_rs as ferro;` — resolves `::ferro` to `ferro-rs` in trybuild context.
- `fn main() {}` — required for trybuild pass fixtures (they are binary crates).
- The fixture must embed a real file on disk. Create `ferro-macros/tests/ui/asset/pass/fixture.js` as a companion (e.g., `console.log(1);`).

For `minimal.rs`:
```rust
//! Compile-pass: minimal `asset!()` — embeds a file and returns &'static str.

#![allow(unused_imports)]

extern crate ferro_rs as ferro;

fn main() {
    // asset!() returns &'static str
    let _url: &'static str = ferro::asset!("tests/ui/asset/pass/fixture.js");
}
```

---

### `ferro-bundle/src/lib.rs` (MODIFY — add `mime_from_ext` + decouple from `ferro-rs`)

**Analog:** `ferro-bundle/src/lib.rs` lines 89–106 (the existing `ext_from_content_type` function — direct inverse).

**Existing function to invert** (`ferro-bundle/src/lib.rs` lines 89–106):
```rust
fn ext_from_content_type(ct: &str) -> &'static str {
    match ct.split(';').next().unwrap_or(ct).trim() {
        "application/javascript" | "text/javascript" => "js",
        "text/css" => "css",
        "text/html" => "html",
        "text/plain" => "txt",
        "application/json" => "json",
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/svg+xml" => "svg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "font/woff2" => "woff2",
        "font/woff" => "woff",
        "application/wasm" => "wasm",
        _ => "",
    }
}
```

Add immediately after this function (before `fn hashed_url_for`):
```rust
/// Map a file extension to its MIME type string.
///
/// Used by the `asset!()` macro to infer content-type from the path extension.
/// Unknown extensions return `"application/octet-stream"`, preserving
/// byte-identical passthrough for unrecognized file types.
pub fn mime_from_ext(ext: &str) -> &'static str {
    match ext {
        "js" | "mjs"   => "application/javascript",
        "css"          => "text/css",
        "html" | "htm" => "text/html",
        "txt"          => "text/plain",
        "json"         => "application/json",
        "png"          => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg"          => "image/svg+xml",
        "gif"          => "image/gif",
        "webp"         => "image/webp",
        "woff2"        => "font/woff2",
        "woff"         => "font/woff",
        "wasm"         => "application/wasm",
        _              => "application/octet-stream",
    }
}
```

**D-06 decouple — `Bundle::serve` signature change** (`ferro-bundle/src/lib.rs` lines 34, 216–220):

Current (to remove):
```rust
// line 34:
use ferro_rs::{HttpResponse, Request};

// lines 216–220:
pub fn serve(req: Request) -> HttpResponse {
    let path = req.path().to_string();
    let if_none_match = req.header("if-none-match").map(|s| s.to_string());
    serve_inner(&path, if_none_match.as_deref())
}
```

The `serve_inner` function (`ferro-bundle/src/lib.rs` lines 228–257) already has the framework-agnostic signature `(path: &str, if_none_match: Option<&str>)`. The decoupling plan:
1. Remove the `use ferro_rs::{HttpResponse, Request};` import (line 34).
2. Remove or rename `Bundle::serve(req: Request) -> HttpResponse` — replace with `pub fn serve_path(path: &str, if_none_match: Option<&str>) -> BundleResponse` where `BundleResponse` is a new framework-agnostic struct, OR simply make `serve_inner` public (rename to `pub fn serve_raw`) and remove the `ferro-rs`-typed `serve` method.
3. The thin `Request → HttpResponse` adapter moves to `framework/src/lib.rs` (or a new `framework/src/bundle.rs` module).

Simplest approach matching the research recommendation: make `serve_inner` public as `Bundle::serve_raw(path: &str, if_none_match: Option<&str>) -> HttpResponse`. But `HttpResponse` is in `ferro-rs` — remove that too. Return a small `BundleResponse`:

```rust
// New framework-agnostic response type (add to ferro-bundle/src/lib.rs):
pub struct BundleResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Option<bytes::Bytes>,
}
```

Then `serve_inner` returns `BundleResponse` instead of `HttpResponse`. The `__test_internals` shim and the existing tests in `ferro-bundle/tests/` need updating accordingly.

**Alternative (simpler, less churn):** Expose `serve_inner` as a public method `Bundle::dispatch(path: &str, if_none_match: Option<&str>) -> (u16, Vec<(String, String)>, Option<&'static [u8]>)` — a plain tuple. The planner should choose the approach that minimizes test breakage; the `BundleResponse` struct is the cleanest API.

---

### `ferro-bundle/Cargo.toml` (MODIFY)

**Analog:** `ferro-bundle/Cargo.toml` line 18 (the line to remove):
```toml
ferro-rs = { path = "../framework", version = "0.2" }
```

After removing this line, `ferro-bundle` has no internal dependencies:
```toml
[dependencies]
sha2 = "0.10"
hex = "0.4"
dashmap = "6"
bytes = "1"
thiserror = "2"
# ferro-rs removed — ferro-bundle is now a leaf crate
```

Also remove the `__test_internals` module's `use ferro_rs::HttpResponse` once the serve signature is decoupled.

---

### `framework/src/lib.rs` + `framework/Cargo.toml` (MODIFY — re-export bundle)

**Analog:** `framework/src/lib.rs` lines 205–213 — the `queue` module re-export pattern (a `pub mod` with `pub use` inside):

```rust
// framework/src/lib.rs lines 205–213:
/// Background job queue. Use `ferro::queue::Job`, `ferro::queue::dispatch`, etc.
pub mod queue {
    pub use ferro_queue::{
        dispatch, dispatch_later, dispatch_to, register_tenant_capture_hook, CreateJobsTable,
        Error, FailedJobInfo, Job, JobInfo, JobPayload, JobState, PendingDispatch, Queue,
        QueueConfig, QueueStats, Queueable, SingleQueueStats, TenantScopeProvider, Worker,
        WorkerConfig, WorkerLoop,
    };
}
```

For bundle, add after the `queue` module block:
```rust
/// In-memory immutable byte blobs with content-hashed URLs and one-year immutable caching.
pub mod bundle {
    pub use ferro_bundle::{mime_from_ext, Bundle, BundleResponse};
}
```

**Additional**: a thin `Request → HttpResponse` adapter function in `framework` (can live in `framework/src/bundle.rs` or inline in `framework/src/lib.rs`):
```rust
// In framework (after ferro-bundle dep is added):
impl Bundle {
    // Framework-aware serve method — lives in framework, not ferro-bundle:
    pub fn serve(req: &crate::Request) -> crate::HttpResponse {
        let path = req.path().to_string();
        let if_none_match = req.header("if-none-match").map(|s| s.to_string());
        let resp = ferro_bundle::Bundle::dispatch(&path, if_none_match.as_deref());
        // convert BundleResponse → HttpResponse
        ...
    }
}
```

**`framework/Cargo.toml` addition** — copy the pattern from lines 41–48:
```toml
ferro-macros = { path = "../ferro-macros", version = "0.2" }
ferro-events = { path = "../ferro-events", version = "0.2" }
# ... (existing)
ferro-bundle = { path = "../ferro-bundle", version = "0.2" }  # ADD
```

**`framework/src/lib.rs` macro re-export** — copy the pattern from lines 351–362:
```rust
pub use ferro_macros::memoize;
// ADD:
pub use ferro_macros::asset;
```

---

### `.github/workflows/publish.yml` (MODIFY — wave reassignment)

**Analog:** publish.yml lines 217 and 335.

Current state (line 217):
```bash
WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet ferro-orm ferro-audit ferro-migration ferro-assets"
```

Current state (line 335):
```bash
WAVE3_CRATES="ferro-cli ferro-bundle"
```

After decoupling `ferro-bundle` from `ferro-rs`:
- Add `ferro-bundle` to `WAVE1A_CRATES` (leaf, no internal deps).
- Remove `ferro-bundle` from `WAVE3_CRATES`.

```bash
# WAVE1A (line 217, modified):
WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet ferro-orm ferro-audit ferro-migration ferro-assets ferro-bundle"

# WAVE3 (line 335, modified):
WAVE3_CRATES="ferro-cli"
```

---

### `ferro-cli/src/commands/assets.rs` (NEW — CLI command, request-response)

**Analog:** `ferro-cli/src/commands/api_check.rs` — reqwest blocking, error handling, file I/O.

**Imports pattern** (`ferro-cli/src/commands/api_check.rs` lines 1–4):
```rust
use console::Style;
use reqwest::blocking::Client;
use serde_json::Value;
use std::time::Duration;
```

For `assets.rs`:
```rust
use clap::Subcommand;
use reqwest::blocking::Client;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
```

**reqwest blocking client pattern** (`api_check.rs` lines 101–106):
```rust
let client = Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .expect("Failed to create HTTP client");
```

For fetch, use default client (no custom timeout needed for large font downloads):
```rust
let client = Client::new();
```

**HTTP GET + JSON parse pattern** (`api_check.rs` lines 124–134 + 159–169):
```rust
let spec_response = match client.get(&spec_url).send() {
    Ok(resp) => resp,
    Err(_) => { eprintln!("..."); return; }
};
let spec_json: Value = match spec_response.json() {
    Ok(v) => v,
    Err(_) => { eprintln!("..."); return; }
};
```

For `assets.rs`, use `?` with `anyhow::Result` instead of early returns, since the function returns `anyhow::Result<()>`. This matches how the research example was written:
```rust
fn fetch_iconify_set(client: &Client, set: &str, out_dir: &Path) -> anyhow::Result<()> {
    let url = format!("https://api.iconify.design/{set}.json");
    let meta: Value = client.get(&url).send()?.error_for_status()?.json()?;
    // ...
}
```

**Clap subcommand enum** — the nested `Subcommand` pattern is verified against `ferro-cli/src/main.rs` lines 263–279 (`MakeProjection` with a subcmd-style variant). The new `Assets` group follows the `Mcp` variant pattern (lines 458–462) for a simple group:

```rust
// In Commands enum (main.rs), pattern from existing variants:
/// Start the MCP server for AI-assisted development
Mcp {
    #[arg(long)]
    cwd: Option<String>,
},
```

For the nested subcommand group:
```rust
/// Download and manage project assets (Iconify icons, Fontsource fonts)
Assets {
    #[command(subcommand)]
    subcommand: commands::assets::AssetsCommand,
},
```

**`fn run()` signature** — all existing CLI commands use synchronous `fn run(...)`. From `api_check.rs` line 92: `pub fn run(url: String, api_key: Option<String>, spec_path: String)`. For `assets.rs`, the entry point called from `main.rs`:
```rust
pub fn run(subcommand: AssetsCommand) {
    if let Err(e) = run_inner(subcommand) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run_inner(subcommand: AssetsCommand) -> anyhow::Result<()> {
    // dispatch to fetch_iconify / fetch_fontsource
}
```

This `run` + inner `run_inner` error-wrapper pattern matches `json_ui_migrate_v1.rs` dispatch in `main.rs` lines 631–635:
```rust
Commands::JsonUiMigrateV1 { file, dry_run } => {
    if let Err(e) = commands::json_ui_migrate_v1::run(file, dry_run) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
```

---

### `ferro-cli/src/commands/mod.rs` (MODIFY)

**Analog:** `ferro-cli/src/commands/mod.rs` lines 1–62 (entire file — alphabetical `pub mod` list).

Add `pub mod assets;` in alphabetical position (after `pub mod api_check;`, before `pub mod auth_link;`):
```rust
pub mod api_check;
pub mod assets;       // ADD
pub mod auth_link;
```

---

### `ferro-cli/src/main.rs` (MODIFY — Commands enum + match arm)

**Analog:** `ferro-cli/src/main.rs` lines 458–462 (Mcp variant) for the enum addition; lines 788–790 (Mcp match arm) for the dispatch.

**Enum variant addition** (after `ApiCheck` variant at line 543, or in logical grouping near `StorageLink`):
```rust
/// Download and manage project assets (Iconify icons, Fontsource fonts)
Assets {
    #[command(subcommand)]
    subcommand: commands::assets::AssetsCommand,
},
```

**Match arm addition** (after `ApiCheck` match arm at line 816–820):
```rust
Commands::Assets { subcommand } => {
    commands::assets::run(subcommand);
}
```

---

## Shared Patterns

### OnceLock lazy-init (applied to `asset.rs` expansion)

**Source:** `ferro-bundle/src/lib.rs` lines 69–73 (process-global `OnceLock` pattern)
**Also:** `ferro-macros/src/memoize.rs` (per-call-site `OnceLock` in expansion)
**Apply to:** `ferro-macros/src/asset.rs` expansion code

```rust
// Process-global registry (ferro-bundle/src/lib.rs lines 69–73):
static BUNDLE_REGISTRY: OnceLock<DashMap<String, BundleEntry>> = OnceLock::new();
static NAME_INDEX: OnceLock<DashMap<String, String>> = OnceLock::new();

fn bundle_registry() -> &'static DashMap<String, BundleEntry> {
    BUNDLE_REGISTRY.get_or_init(DashMap::new)
}
```

The per-call-site static in the macro expansion follows the same `OnceLock::get_or_init` shape, scoped to a block instead of the module level.

### `anyhow::Result` error handling in CLI commands

**Source:** `ferro-cli/src/commands/json_ui_migrate_v1.rs` (returns `anyhow::Result<()>`), dispatch in `main.rs` lines 631–635.
**Apply to:** `ferro-cli/src/commands/assets.rs` `run_inner` function.

Pattern: public `run(args)` function is infallible (calls `process::exit(1)` on error); private `run_inner(args) -> anyhow::Result<()>` does the real work with `?` propagation.

### `crate::utils::ferro()` path helper

**Source:** `ferro-macros/src/utils.rs` lines 55–60
**Apply to:** `ferro-macros/src/asset.rs` (import and call `ferro()`)

All macros that emit `::ferro::...` paths call this helper. The `asset.rs` file must import and use it for `::ferro::bundle::Bundle` and `::ferro::bundle::mime_from_ext` to resolve correctly in both workspace and downstream contexts.

### `extern crate ferro_rs as ferro;` in trybuild fixtures

**Source:** `ferro-macros/tests/ui/action/pass/minimal.rs` line 5
**Apply to:** All new `ferro-macros/tests/ui/asset/pass/*.rs` fixtures

Every trybuild fixture that uses `ferro::` paths must declare this alias. The `ferro-macros` dev-dependency is `ferro-rs = { path = "../framework" }` (Cargo.toml line 25), which is the crate that maps to `ferro` in downstream consumers.

---

## No Analog Found

All files have clear analogs. No files require purely RESEARCH.md-derived patterns.

---

## Metadata

**Analog search scope:** `ferro-macros/src/`, `ferro-macros/tests/`, `ferro-bundle/src/`, `ferro-cli/src/commands/`, `framework/src/lib.rs`, `framework/Cargo.toml`, `.github/workflows/publish.yml`
**Files scanned:** 14 source files read in full
**Pattern extraction date:** 2026-07-26
