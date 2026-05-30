---
phase: 180
slug: declarative-action-handler-primitive-typed-result-return-so
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-30
---

# Phase 180 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution of the `#[action]` proc-macro + `ActionError` / `ActionOk` / `ActionResult` / `IntoActionError` runtime types.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust stable) |
| **Config file** | none — workspace-level `Cargo.toml` already configures targets |
| **Quick run command** | `cargo test -p ferro-macros action && cargo test -p framework http::action` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~90-180 seconds (incremental) / ~5-8 min (clean) |

CI parity: full suite is the exact command listed in CLAUDE.md "Testing & Linting (MUST run before every commit)".

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p {crate-just-modified}` quick targeted run (`ferro-macros` for macro work, `framework` for runtime types, `ferro` smoke test for re-exports).
- **After every plan wave:** Run full suite (fmt + clippy + test).
- **Before `/gsd-verify-work`:** Full suite must be green AND the consumer acceptance diff (publish_by_id rewrite from CONTEXT.md) must compile against ferro local-path.
- **Max feedback latency:** ~30s per targeted crate test, ~6 min for full suite.

Sampling continuity rule: no 3 consecutive task commits without at least one targeted `cargo test` run.

---

## Per-Task Verification Map

> Filled by the planner as plans are produced. Schema:

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 180-XX-YY | XX | N | D-01..D-10 | T-180-01..T-180-03 | see threat row | unit / integration / trybuild | exact `cargo test ...` invocation | ✅ / ❌ W0 | ⬜ pending |

**Threat references (carried from RESEARCH.md §6):**

| Threat | Description | Mitigation under test |
|--------|-------------|------------------------|
| T-180-01 | Flash message injection — untrusted error text rendered into HTML by consumer templates | Document escape requirement in rustdoc; unit test that `ActionError::msg("<script>")` is stored verbatim (escaping is template's responsibility) but rustdoc presence is grep-verifiable |
| T-180-02 | Open redirect via `redirect_override` containing attacker-controlled URL | Validate `redirect_override` with `is_same_origin` (reused from `framework/src/validation/error.rs`); integration test sends `redirect_override = "https://evil.example"` and asserts 303 falls back to configured `redirect_to` |
| T-180-03 | Log injection via control characters in `err.message` reaching `tracing::error!` | Sanitize `\r\n\x00` from message field before `tracing::error!` call; unit test that `ActionError::msg("a\nfake-log-line")` produces single-line log output |

**Required test categories (from RESEARCH.md §8):**

1. **ActionError builder + From impls** — unit tests for `::msg`, `::not_found`, `::forbidden`, `::unauthorized`, `.with_flash(...)`, `.redirect_to(...)`, plus each concrete `From<{FrameworkError, String, &str, sea_orm::DbErr}>` impl.
2. **Macro happy-path** — integration test: handler returns `Ok(())` → 303 to `redirect_to` with success flash written to session.
3. **Macro error-path with no override** — integration test: handler returns `Err(ActionError::msg("boom"))` → 303 to `redirect_to` + `?error=...&msg=boom` + flash `{variant: Error, message: "boom"}` + `tracing::error!` captured.
4. **Macro error-path with redirect_override** — integration test: handler returns `Err(ActionError::unauthorized().redirect_to("/login"))` → 303 to `/login` (not `redirect_to`).
5. **Open-redirect mitigation (T-180-02)** — integration test: `Err(ActionError::msg("x").redirect_to("https://evil.example/"))` → 303 falls back to configured `redirect_to`, attacker URL ignored.
6. **Log injection mitigation (T-180-03)** — unit test on the sanitizer helper: `\r`, `\n`, `\x00` are stripped/escaped before reaching tracing.
7. **`?` ergonomics** — integration test that compiles a handler using `?` on `String`, `FrameworkError`, `sea_orm::DbErr`, and `anyhow::Error`-wrapped-via-`.action_err()`. trybuild UI test confirms helpful compile error when the user tries to `?` an unsupported type without the shim.
8. **Public API smoke test** — trybuild UI test that `use ferro::{action, ActionError, ActionOk, ActionResult, IntoActionError};` compiles in a downstream crate.

---

## Wave 0 Requirements

Files Wave 0 must create as stubs so later tasks can land tests incrementally:

- [ ] `framework/tests/action_handler.rs` — integration test harness (new file). Hosts test cases 2-6 from the category list above.
- [ ] `framework/src/http/action.rs` — module skeleton: bare `pub struct ActionError;`, `pub struct ActionOk;`, `pub type ActionResult = Result<ActionOk, ActionError>;` with `todo!()` bodies so other crates can name the types while the runtime is being filled in.
- [ ] `ferro-macros/tests/action_macro.rs` — trybuild + integration test harness for the `#[action]` proc-macro (new file).
- [ ] `ferro-macros/tests/ui/action/` — directory for trybuild UI tests (compile-fail + pass cases).
- [ ] `framework/Cargo.toml` — confirm `tracing` is in dependencies (RESEARCH §6 OQ-B); add if missing. Confirm `form_urlencoded` is exposed for runtime use.

No framework install needed — Rust toolchain is already configured per workspace `rust-toolchain.toml` / `Cargo.toml`.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Consumer acceptance diff (publish_by_id rewrite from CONTEXT.md "Acceptance test") | D-10 — sweep deliverable proven against real code | Lives in the gestiscilo-it repo, not in ferro; cargo cannot run consumer tests | After ferro plans complete: in `~/repositories/gestiscilo-it`, replace `src/controllers/pages.rs::publish_by_id` with the `#[action]` form from CONTEXT.md, run `cargo check` against the local-path ferro, manually trigger the publish action in a dev session, observe 303 to `/dashboard/pagine?error=publish&msg=...` on simulated failure |
| Flash rendering across dashboard templates | Consumer sweep | Template rendering is in gestiscilo-it views | Trigger an action error → confirm consumer template reads `session.get_flash("_action")` and displays the message |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify command OR a Wave 0 dependency
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all stub files above
- [ ] No watch-mode flags (`cargo watch`, `bacon` ok for local dev only — not in CI test commands)
- [ ] Feedback latency target: targeted crate test < 60s
- [ ] `nyquist_compliant: true` set in frontmatter once plans assign tests to every task
- [ ] Threat tests T-180-01, T-180-02, T-180-03 each have a corresponding test row

**Approval:** pending
