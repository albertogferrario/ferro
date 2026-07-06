---
phase: 238-inertia-first-load-html-shell
reviewed: 2026-06-21T00:00:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - ferro-inertia/src/config.rs
  - ferro-inertia/src/response.rs
  - framework/src/container/mod.rs
  - framework/src/inertia/context.rs
  - framework/src/inertia/global.rs
  - framework/src/inertia/mod.rs
  - docs/src/features/inertia.md
findings:
  critical: 0
  warning: 4
  info: 2
  total: 6
status: issues_found
---

# Phase 238: Code Review Report

**Reviewed:** 2026-06-21
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

Phase 238 wires the Inertia first-load HTML shell — `to_html_response`, the new
`InertiaConfig` fields (`title`, `head_extras`, `mount_id`), the process-global
`OnceLock<InertiaConfig>` in `framework/src/inertia/global.rs`, and the updated
documentation in `docs/src/features/inertia.md`.

The security-critical path (`data-page` JSON escaping, dev-mode Vite tag gating, and
`head_extras` trust-boundary documentation) is all correct. No critical issues found.

Four warnings are present:

1. `title_text` and `mount_id` are interpolated raw into the HTML template without
   HTML-escaping — a developer-supplied value containing `<`, `>`, or `"` would break
   the document structure.
2. The `csrf` token from `csrf_token()` is interpolated into
   `content="..."` without HTML-escaping — same class of break if the token ever
   contains `"` or `&` (unlikely but not architecturally excluded).
3. `ferro-inertia`'s own `Inertia::render`, `render_with_json_fallback`, and
   `render_with_shared` all call `InertiaConfig::default()` directly. These are
   bypassed by the framework layer (which calls `render_with_options` with the global
   config), but they remain on the `ferro-inertia` public surface and silently ignore
   the global config when called directly.
4. Docs use `SavedInertiaContext::from_request(&req)` in six places, but that method
   does not exist — the API is `SavedInertiaContext::from(&req)` or
   `SavedInertiaContext::new(req)`. Every example would fail to compile.

Two info items round out the report.

## Warnings

### WR-01: `title_text` and `mount_id` not HTML-escaped before interpolation

**File:** `ferro-inertia/src/response.rs:403-409, 437, 443, 463, 469, 473, 475`
**Issue:** `title_text` (from `config.title` or `config.app_name`) and `mount_id`
are interpolated directly into the HTML template without going through the same
`&amp;`/`&lt;`/`&gt;`/`&quot;` escaping applied to `page_json`. Both are
developer-controlled config values (not request data), so this is not an injection
vector today. However:

- `title_text` is interpolated between `<title>` tags: an unescaped `<` or `>`
  breaks the document.
- `mount_id` is interpolated into `id="..."`: an unescaped `"` would close the
  attribute and allow attribute injection (e.g. `mount_id = r#"app" class="evil"#`
  would produce `id="app" class="evil"`).

Because config values come from developer code (not users), this is not XSS. It is
a structural correctness bug — a malformed config silently corrupts the HTML output
rather than failing loudly.

**Fix:** Apply a minimal HTML-attribute escape to `title_text` and `mount_id` at the
interpolation site. A simple helper is sufficient since these are known-bounded strings:

```rust
fn html_attr_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('"', "&quot;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
}

// Then at use sites:
let title_text = html_attr_escape(
    self.config.title.as_deref().unwrap_or(&self.config.app_name)
);
let mount_id = html_attr_escape(&self.config.mount_id);
```

For `<title>` content the escaping of `<`/`>` is sufficient; for `id="..."` the `"`
escape is the load-bearing one.

---

### WR-02: `csrf` token not HTML-escaped in `content="..."` attribute

**File:** `ferro-inertia/src/response.rs:391, 419, 462`
**Issue:** The CSRF token is interpolated unescaped into
`<meta name="csrf-token" content="{}">`. The token is cryptographically generated
and unlikely to contain `"`, but the framework makes no contractual guarantee that
CSRF tokens are `"`-free. If a future token format or custom provider emits a
base64-standard token (which can include `+`, `/`, `=` — none harmful) or a
URL-safe variant with `%` sequences, the `"` risk remains theoretical. The `&`
character appearing in a token would produce a well-formedness warning in strict
parsers.

**Fix:** Apply `html_attr_escape` (same helper as WR-01) to `csrf` before
interpolation:

```rust
let csrf_escaped = html_attr_escape(csrf);
// dev branch:
//   <meta name="csrf-token" content="{}"> → pass csrf_escaped
// prod branch:
//   <meta name="csrf-token" content="{csrf}"> → use csrf_escaped
```

---

### WR-03: `ferro-inertia` public render methods silently ignore the global config

**File:** `ferro-inertia/src/response.rs:134, 168, 187, 351`
**Issue:** `Inertia::render`, `Inertia::render_with_json_fallback`, and
`Inertia::render_with_shared` (lines 134, 168, 187) all pass
`InertiaConfig::default()` (= `from_env()`) to `render_internal`. `InertiaResponse::new`
(line 351) also initialises with `InertiaConfig::default()`.

The framework layer in `framework/src/inertia/context.rs` correctly calls
`render_with_options` with the global config fetched from `get_inertia_config()`. But
the three convenience methods on the `ferro-inertia` `Inertia` struct are part of its
public API. Any consumer of `ferro-inertia` calling them directly will silently receive
env-derived defaults for `title`, `head_extras`, and `mount_id` regardless of what was
set via `App::set_inertia_config`.

This is a latent correctness trap: docs show `Inertia::render(&req, "Home", props)` as
the primary API, and framework users will call the framework wrapper — but the
underlying crate's namesake methods are broken relative to the global config contract.

**Fix:** Either deprecate `ferro-inertia::Inertia::render` / `render_with_json_fallback`
/ `render_with_shared` (directing users to `render_with_config` or `render_with_options`)
or document clearly in their rustdoc that they always use env defaults and are not
global-config-aware. Adding a `#[deprecated]` hint is the safer choice to prevent
accidental use by framework embedders:

```rust
/// Render an Inertia response using `InertiaConfig::from_env()` defaults.
///
/// When embedding ferro-inertia inside a framework that sets a global config,
/// use `render_with_config` or `render_with_options` instead.
#[deprecated(note = "Does not respect a globally-set InertiaConfig. \
    Use render_with_config or render_with_options.")]
pub fn render<R, P>(req: &R, component: &str, props: P) -> InertiaHttpResponse
```

---

### WR-04: Docs use non-existent `SavedInertiaContext::from_request` throughout

**File:** `docs/src/features/inertia.md:359, 398, 669, 925, 952, 1043`
**Issue:** Six code examples call `SavedInertiaContext::from_request(&req)`. That method
does not exist. The actual API exposed in `framework/src/inertia/context.rs` is:

- `SavedInertiaContext::new(req: &Request)` (line 60)
- `SavedInertiaContext::from(&req)` via the `From<&Request>` impl (line 82)

Every one of these examples would fail to compile if copy-pasted by a user.

**Fix:** Replace all six occurrences of `from_request(&req)` with `from(&req)`:

```rust
// Wrong (six times in the docs):
let ctx = SavedInertiaContext::from_request(&req);

// Correct:
let ctx = SavedInertiaContext::from(&req);
```

`From<&Request>` is the idiomatic Rust conversion API and is already implemented;
`from_request` is a Laravel convention name that was not carried forward.

---

## Info

### IN-01: Default `app_name` fallback of `"Ferro"` is a framework identity string in a leaf crate

**File:** `ferro-inertia/src/config.rs:63`
**Issue:** `from_env()` falls back to `"Ferro"` when `APP_NAME` is not set. Per
CLAUDE.md, `ferro-*` crates must not hardcode any application identity — they must
read `APP_NAME`/`APP_URL` from env. `"Ferro"` is the framework name, not a tenant
identifier, so this is a borderline case. But it does appear in user-visible HTML
(`<title>Ferro</title>`) when `APP_NAME` is absent, which is the framework name
leaking into app output.

A more appropriate fallback is either an empty string (forcing the user to see a blank
title until they configure it) or omitting the `<title>` tag when neither `APP_NAME`
nor `config.title` is set. The current behavior silently produces branded output.

**Fix:** Change the fallback to an empty string and omit the `<title>` tag when the
resolved title is empty, or keep `"Ferro"` but document it as the deliberate default
in the docstring:

```rust
// Option A: empty fallback, explicit intent
let app_name = std::env::var("APP_NAME").unwrap_or_default();

// Option B: document the current behavior
/// Falls back to `"Ferro"` when `APP_NAME` is not set. Set `APP_NAME` in your
/// `.env` to override this.
let app_name = std::env::var("APP_NAME").unwrap_or_else(|_| "Ferro".to_string());
```

---

### IN-02: `OnceLock` second-set warning goes to stderr with no structured context

**File:** `framework/src/inertia/global.rs:16-18`
**Issue:** When `set_inertia_config` is called a second time, the ignored config is
silently discarded and a bare `eprintln!` warning is emitted. In a production
environment, `stderr` output is often discarded or routed through a logging
framework. The warning gives no indication of where the second call originated
(no source location, no timestamp), making it hard to trace in practice.

**Fix:** If the framework has a structured logging layer (e.g. `tracing`), prefer
`tracing::warn!` so the message is captured by the same sink as other framework logs.
If not, the current `eprintln!` is acceptable but should include a hint to search
`bootstrap.rs`:

```rust
if INERTIA_CONFIG.set(config).is_err() {
    eprintln!(
        "[ferro] Warning: InertiaConfig already set; second call to \
         App::set_inertia_config ignored. Check bootstrap.rs for duplicate calls."
    );
}
```

---

_Reviewed: 2026-06-21_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
