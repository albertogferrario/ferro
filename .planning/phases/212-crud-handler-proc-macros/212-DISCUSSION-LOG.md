# Phase 212: CRUD Handler Proc Macros - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-13
**Phase:** 212-crud-handler-proc-macros
**Mode:** `--auto` (recommended defaults selected without interactive prompts)
**Areas discussed:** the seven open design questions from the scoping doc

The recommended option was selected for each. Choices favor reusing an existing ferro surface
over adding a new one (continuous conceptual coherence / no duplicate control surface), grounded
in a scout of `ferro-macros`, the tenant layer, the validation layer, and the action surface.

---

## Q1 — Tenant resolution coupling

| Option | Description | Selected |
|--------|-------------|----------|
| A: new `TenantResolver` trait on Request | Consumers implement a new trait | |
| B: macro arg `tenant = "expr"` | Rust expression per call site | (escape hatch only) |
| C: runtime extension type | Look up `req.extensions::<TenantResolver>()` | |
| **Reuse existing `current_tenant()`/`TenantScopeProvider`** | Bind to the tenant layer ferro already owns (v12.6); `tenant = expr` as the escape hatch | ✓ |

**Choice:** Reuse the existing layer (D-01) + optional `tenant = expr` escape hatch (D-02).
**Why:** ferro already decides tenant resolution; a new `TenantResolver` would duplicate that
control surface. Scout: `framework/src/lib.rs:133` (`current_tenant`, `TenantScopeProvider`).

---

## Q2 — Resource lookup trait shape

| Option | Description | Selected |
|--------|-------------|----------|
| Required `find_for_tenant` trait | Fixed signature | (default, generalized) |
| `find = "Customer::find_for_tenant"` arg | Function-pointer override | (escape hatch) |

**Choice:** `TenantScoped` trait with assoc `Id: FromStr` + `find_for_tenant` as default (D-03);
`find = "expr"` override (D-04). Generalized `Id` so it isn't hardcoded to `i64`.

---

## Q3 — 404 / miss strategy

| Option | Description | Selected |
|--------|-------------|----------|
| A: plain `Response::not_found()` + consumer 404 middleware | Generic only | (the omitted-arg default) |
| B: `on_miss = url` redirect / handler | Configurable | ✓ (both) |

**Choice (D-05):** `on_miss = "/url"` → redirect; omitted → generic `Response::not_found()` /
`ActionError::not_found`. Macro emits NO consumer styling (project-agnostic crate rule).

---

## Q4 — Macro composition with `#[handler]`/`#[action]`

**Choice (D-06):** `#[resource_get]` emits `#[ferro::handler]`; `#[resource_post]` emits
`#[ferro::action]`; single attribute at the call site; user body → named inner fn for
`cargo expand` readability. Scout: `ferro-macros/src/lib.rs:232`/`:265`.

---

## Q5 — `validate_or_redirect` shape

| Option | Description | Selected |
|--------|-------------|----------|
| Method on `Validator` | Composes existing `into_action_error` | ✓ |
| Method on `ValidationError` | | |
| Free function | | |

**Choice (D-07):** method on `Validator`, returns `Result<(), ActionError>`, reuses
`with_old_input` + `into_action_error` (`framework/src/validation/error.rs:160`). No new error path.

---

## Q6 — Form-URL synthesis

**Choice (D-08):** `{param}` placeholders in `redirect_to`/`form_url`/`on_miss` resolve from the
macro-extracted path params via `format!`; a non-path placeholder is a compile error (no
query/body/session magic).

---

## Q7 — Editor experience

**Choice (D-09):** tenant/resource are REAL typed params (autocomplete + jump-to-def work); body
in a named inner fn; rustdoc `cargo expand` walkthroughs; plan includes an explicit IDE-experience
verification.

---

## Claude's Discretion

- Exact `TenantScoped` method names / bounds beyond `Id: FromStr`.
- `path = "{id:i64}"` vs positional resource-type arg for id extraction.
- Whether the gestiscilo thread-local → `current_tenant()` bridge is docs-only or a tiny adapter.

## Deferred Ideas

- `#[confirm_page]` macro (no recurring shape — dropped).
- Multipart upload macros (4 distinct domains — future phases).
- gestiscilo Phase 202b (consumer-side macro adoption, ships after 212 publishes; cross-repo).
