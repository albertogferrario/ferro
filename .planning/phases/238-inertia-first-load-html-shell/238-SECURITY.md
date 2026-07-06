---
phase: 238-inertia-first-load-html-shell
audited: 2026-06-21
asvs_level: 1
threats_total: 7
threats_closed: 7
threats_open: 0
status: SECURED
---

# Phase 238 — Security Audit

## Result: SECURED

All 7 registered threats are closed. No open threats. No unregistered flags requiring escalation.

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-238-01 | Tampering (XSS) — `data-page` attribute | mitigate | CLOSED | `escape_html()` helper at `ferro-inertia/src/response.rs:11-18` covers all 5 chars (`& < > " '`). Applied to `page_json` at `:402`, to `csrf` at `:404`, to `title_text` at `:418-423`, and to `mount_id` at `:425` (WR-01 fix from commit `e09d71b1`). `html_data_page_equals_json_contract` test at `:556-588` round-trips through unescaping and asserts equality with the JSON contract. |
| T-238-02 | Tampering (XSS) — `head_extras` raw HTML | accept (documented) | CLOSED | Field docstring at `ferro-inertia/src/config.rs:41-44` states "SECURITY: developer-controlled config only — never populate from request data." No request-data path reaches the field anywhere in `ferro-inertia` or `framework`. Injection-site comment at `response.rs:416-417` reiterates the trust boundary. `head_extras` is explicitly not escaped (by design). |
| T-238-03 | Information disclosure — dev tags leaking to prod | mitigate | CLOSED | Dev-mode Vite tags (`/@vite/client`, `@react-refresh` preamble) gated behind `if self.config.development` at `response.rs:428`. `prod_mode_does_not_leak_dev_server` test at `:659-673` asserts both strings absent in the prod branch. |
| T-238-04 | Tampering/race — `OnceLock<InertiaConfig>` set after serving | accept (documented) | CLOSED | `framework/src/inertia/global.rs:10` declares `static INERTIA_CONFIG: OnceLock<InertiaConfig> = OnceLock::new()`. Module docstring at `:3-5` and function docstring at `:12-14` state the set-once-before-server-start contract. `App::set_inertia_config` docstring at `framework/src/container/mod.rs:408` repeats "Call once from bootstrap.rs before the server starts accepting requests." Reads are immutable clones: `get_inertia_config()` calls `.cloned()` at `global.rs:25`. No RwLock present (`grep -c "RwLock" global.rs` = 0). |
| T-238-05 | DoS/misconfig — second `set_inertia_config` silently dropped | mitigate | CLOSED | `framework/src/inertia/global.rs:16-18`: `if INERTIA_CONFIG.set(config).is_err() { eprintln!("Warning: InertiaConfig already set; second call ignored"); }`. Visible warning on second call, not a silent no-op. |
| T-238-06 | Information disclosure (guidance) — docs implying `head_extras` accepts user input | mitigate | CLOSED | `docs/src/features/inertia.md:85-87` (First-Load section): "`head_extras` adds raw HTML into `<head>` ... and is developer-controlled config — it must not be populated from request data to avoid XSS." Also at `:141-143` (Manual Configuration section). |
| T-238-07 | Spoofing/session theft (guidance) — proxy recipe insecure cookie/Origin setup | mitigate | CLOSED | `docs/src/features/inertia.md:62-81`: `changeOrigin: false` shown in both proxy targets with explicit comment "changeOrigin: false preserves the Origin header so CSRF and session validation on the backend receives the browser's actual origin." SameSite implications documented at `:72-80`. MEDIUM-confidence validation marker at `:52`. |

## WR-01/WR-02 Hardening (Review-Fix Beyond Original Threat Model)

Commit `e09d71b1` added hardening beyond what the original threat register declared:

- **WR-01**: `title_text` and `mount_id` now routed through `escape_html()` before template interpolation (`response.rs:418-425`). Previously raw; a `"` or `<` in developer config would have broken HTML structure (not XSS, but structural correctness).
- **WR-02**: `csrf` token now routed through `escape_html()` at `response.rs:404`. Provides defense against future CSRF token formats that might include `"` or `&`.

Both fixes use the same `escape_html()` helper as the existing `page_json` path, ensuring consistent treatment of the 5-character set (`& < > " '`).

## Leaf-Crate Invariant Check

`ferro-inertia/Cargo.toml` dependencies: `serde`, `serde_json` only. No ferro-* internal dependencies. Leaf-crate invariant holds.

## Accepted-Risk Log

| ID | Finding | Rationale |
|----|---------|-----------|
| T-238-02 | `head_extras` injected raw into `<head>` | Developer-controlled config, not request input. Documented trust boundary in field docstring and injection-site comment. No request-data path exists. |
| T-238-04 | `OnceLock<InertiaConfig>` has no enforcement of set-before-serve ordering at the type level | Rust's `OnceLock` guarantees single-write, immutable-reads, and thread-safety. The contract is documented in two docstrings. Type-level enforcement is not achievable without a builder-pattern server bootstrap, which is out of scope for this phase. |

## Unregistered Flags

None. All threat flags from the four SUMMARY.md files map directly to registered threats T-238-02, T-238-04, T-238-05, T-238-06, and T-238-07.
