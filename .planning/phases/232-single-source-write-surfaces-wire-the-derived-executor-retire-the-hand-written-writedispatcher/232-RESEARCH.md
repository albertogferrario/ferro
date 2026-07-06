# Phase 232: Single-Source Write Surfaces — Research

**Researched:** 2026-06-16
**Domain:** Cross-surface write dispatch in `ferro-mcp-server` + the (absent) visual/form write path (Rust, SeaORM, JSON-UI)
**Confidence:** HIGH (every claim grounded in workspace files at 0.2.65; verified against code, not summaries)

## Scope Reality Check

This is the single most important section. Phase 231 did **more than half** of what EXEC-05's wording implies, and the remaining work is **not** what the phase title literally says ("wire across BOTH surfaces, retire the WriteDispatcher").

### Q1 — What did 231 ALREADY do on the MCP side? (VERIFIED — already satisfied)

The MCP half of EXEC-05 is **done**. Confirmed against code, not summaries:

- `grep -rn 'match action_name' app/src` → **empty** (one comment mention only, `app/src/tests/mcp_write_dispatch.rs:292`). The hand-written `match action_name { "submit" => "submitted", ... }` is **deleted everywhere** [VERIFIED: grep].
- `derive_transition_plan` is wired into the MCP write path at **three** call sites: `ferro-mcp-server/src/write_dispatch.rs:541` (`handle_write_call`), `:678` (`handle_request_confirm`), `:839` (`handle_confirm`) [VERIFIED: grep + read]. All three derive the transition guard from the declared `StateMachine` and feed it into `merged_guards` (`write_dispatch.rs:137`).
- The app executor derives `to_state` from the plan: `app/src/controllers/mcp.rs:108` (`ferro::derive_transition_plan(svc, &action_name)`) — facade path only, no `ferro_projections::` in `app/` [VERIFIED: read].
- EXEC-02 (live transition-guard re-eval) and EXEC-03 (post-persist override registry) shipped in 231 Plan 02 (`write_dispatch.rs:84` `OverrideFn`, `:453` post-persist hook).

**Conclusion:** the MCP write surface (`AMCP-04`) already runs every state-transition write through the derived executor with no per-action match. For EXEC-05's MCP clause, **Phase 232 is a verification point, not new work.** Do not re-derive, re-wire, or re-test the MCP path beyond a regression assertion.

### Q2 — Does a visual/form write path that should use ServiceDef transitions actually EXIST? (VERIFIED — NO. This is the load-bearing finding.)

**There is no visual/form transition-write executor.** The "visual write path" EXEC-05 names is, today, a **dangling contract with nothing behind it**:

- The projection renderer emits action buttons whose URLs follow the convention `POST /{service.name}/{action.name}`:
  - Detail/Process actions slot: `ferro-json-ui/src/projection/builder.rs:685` — `Action::new(format!("/{}/{}", service.name, a.name))`.
  - Browse/Track row actions: `builder.rs:309` — `Action::new(format!("/{}/{{row_key}}/{}", service.name, a.name))`.
- The emitting code's own doc comment says the executor does not exist: *"`ActionDef` has no route field, so the consumer's route table must match this convention for the buttons to resolve (documented as the projection action-route contract, Risk 4)"* (`builder.rs:669-671`) [VERIFIED: read].
- `grep` for any handler receiving `POST /{service}/{action}` in `app/`, `framework/`, or `ferro-json-ui/` → **none exists** [VERIFIED: grep]. The visual action buttons POST to routes the app has never been given a way to wire to the derived executor.
- JSON-UI actions are intentionally decoupled string handlers (`ActionHandler::Literal("controller.method")` or `/url`, `ferro-json-ui/src/action.rs:88-100`) — they target an arbitrary app handler, **not** the transition kernel. There is no path from a JSON-UI form submit into `derive_transition_plan` + guard re-eval + persist.
- The write dispatch path is consumed **exclusively** by MCP surfaces: `app/src/controllers/mcp.rs`, `mcp_chat.rs`, `ferro-mcp-server/src/{jsonrpc,intent}.rs` (`handle_write_call`). No Inertia, JSON-UI, or plain HTTP form controller calls `dispatch_write`/`handle_write_call`/`WriteDispatcher` [VERIFIED: grep across app/src, ferro-inertia/src, ferro-json-ui/src, framework/src].
- `ferro-inertia` has **zero** references to `ActionDef`/`transition`/`dispatch_write`/`ServiceDef` [VERIFIED: grep].

**The honest restatement of EXEC-05's real remaining work:** EXEC-05 is **not** "wire an existing visual path into the derived executor" (no such path exists) and **not** "retire the WriteDispatcher" (it is the load-bearing runtime envelope and must stay — see Q4). The real, only remaining work is:

> **Build the visual/form transition-write surface.** Lift the channel-agnostic transition-execution kernel out of `ferro-mcp-server`'s MCP framing so a non-MCP HTTP form handler (receiving the `POST /{service}/{action}` the projection already emits) can drive the same derived executor — same guard re-eval, same plan derivation, same audit/idempotency envelope, same override hook — with no second executor.

This is a **scope expansion relative to the phase title's literal reading**, but it is exactly what the milestone goal and EXEC-05 require: *"one declaration backs writes in every modality with no per-channel executor."* Today writes work in **one** modality (MCP). EXEC-05 is satisfied only when the **visual** modality also routes through the shared kernel. Manufacturing less scope ("just verify MCP, mark done") would leave the visual buttons (`builder.rs:685`) pointing at a void and the milestone's cross-modal claim false.

### What's left, concretely

1. **Extract a channel-agnostic transition-execution kernel** so it is callable outside MCP framing (the `dispatch_write` body is already 95% channel-agnostic — see Q3).
2. **Add a visual/form write entry point** (an HTTP handler in `framework` or app that receives `POST /{service}/{action}`, resolves the `ServiceDef`+`ActionDef`, authenticates the tenant, and calls the shared kernel).
3. **Prove single-source:** a synthetic test driving the **same** `submit` transition through **both** the MCP path and the visual path, asserting identical guard re-eval, identical `to_state`, identical audit, and that **no** second `match`/executor exists.
4. **(Verification only)** Confirm the MCP path still routes through the now-shared kernel (regression).

---

## User Constraints

No `CONTEXT.md` exists for Phase 232 (verified: `.planning/phases/232-.../` is empty). Constraints are taken from `REQUIREMENTS.md` and the binding `CLAUDE.md` / `feedback_*` memory. `/gsd-discuss-phase` should confirm the Q3 architecture decision and the scope-expansion framing in Q2 before planning.

### Locked Decisions (from REQUIREMENTS.md + CLAUDE.md)
- **No parallel control surface / second source of truth for transitions** (REQUIREMENTS.md:7,42; `feedback_no_duplicate_control_surface`). The visual path must call the SAME derived executor — it must not get its own `match` or its own transition kernel.
- **No new crate for executor derivation** (REQUIREMENTS.md:43). Derivation stays in `ferro-projections`. *Note: this constrains the derivation, not the runtime kernel — the runtime envelope already lives in `ferro-mcp-server` and the open question is only WHERE the shared runtime kernel lands, see Q3.*
- **Reads FROM the existing `StateMachine`/`ActionDef`** — no second declaration form (REQUIREMENTS.md:42).
- **No StateMachine/guard redesign** (REQUIREMENTS.md:45).
- **`ferro-projections` stays schema-only** — no `sea-orm`/`tokio`/closures (`ferro-projections/CLAUDE.md`, `Cargo.toml:18-21`). The shared kernel (DB I/O) therefore CANNOT live in `ferro-projections`.
- **Project-agnostic `ferro-*` crates** (`CLAUDE.md`) — any shared kernel must not hardcode app identity; the MCP-specific audit prefix `"mcp.action.{}"` (`write_dispatch.rs:436`) must become channel-parameterized when the kernel is shared.
- **gestiscilo / consumer migration is OUT** (REQUIREMENTS.md:44; `feedback_cross_repo_phase_split`) — ferro delivers the framework capability + synthetic validation only.

### Claude's Discretion
- WHERE the shared transition-execution kernel lands (Q3 — `framework` vs a new home vs staying in `ferro-mcp-server` with a re-export). RECOMMENDED below.
- The exact shape of the visual/form HTTP entry point (a `framework` route helper vs an app-level controller pattern).
- Whether the confirmation seam (`#[cfg(feature = "confirmation")]`) applies to the visual path in v16.0 or is MCP-only for now.

### Deferred Ideas (OUT OF SCOPE)
- Derived executor for **non-transition** plain CRUD writes (REQUIREMENTS.md:34).
- Operating-AX / NL description quality (REQUIREMENTS.md:35).
- Projection `body` slot (REQUIREMENTS.md:36).
- gestiscilo adoption (REQUIREMENTS.md:44).

---

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| EXEC-05 | The derived executor drives writes from the single `ServiceDef` across the MCP write dispatch (AMCP-04) AND the visual/form write path — one declaration backs writes in every modality, no per-channel executor. | MCP half DONE in 231 (`write_dispatch.rs:541/678/839`, `mcp.rs:108`, `match action_name` empty). Visual half is UNBUILT — projection emits `POST /{service}/{action}` (`builder.rs:685,309`) with no handler. Remaining work = extract a channel-agnostic kernel from `dispatch_write` (`write_dispatch.rs:336-458`) + add a visual entry point that calls it. |

---

## Architecture Decision (Q3 — the load-bearing design choice)

### The question
The security envelope — derive plan → re-eval guard (live) → idempotency → confirmation seam → execute/persist → audit → override — currently lives in `ferro-mcp-server::dispatch_write` (`write_dispatch.rs:336-458`). EXEC-05 says BOTH surfaces must execute "through the same derived executor" with "no per-channel executor." So: where does the shared kernel live, and how does the visual path reach it without depending on the MCP crate?

### Key code-grounded observation: `dispatch_write` is ALREADY channel-agnostic

Read `dispatch_write` (`write_dispatch.rs:336-458`). Its signature takes only `(&ActionDef, &Value inputs, i64 tenant_id, &DatabaseConnection, &WriteDispatcher, Option<&str> transition_guard, [is_confirmed])`. **None of that is MCP-specific.** The MCP/JSON-RPC framing lives entirely ABOVE it, in `handle_write_call` (`write_dispatch.rs:470+`):
- `call_params["name"]` tool-name parsing (`:484`),
- JSON-RPC error envelopes `-32603` / `-32601` (`:528`, `:534`),
- `request_confirm_` / `confirm_` prefix routing (`:490-518`),
- `McpContext` (`:474`).

The **only** MCP couplings inside `dispatch_write` itself are:
1. `crate::Error` / `crate::Result` — the `ferro-mcp-server` error type (`error.rs:3-6`).
2. The audit reason prefix `"mcp.action.{}"` (`write_dispatch.rs:436`) — hardcoded channel identity, violates project-agnostic intent once shared.
3. `ferro_ai::ConfirmationStore` in the confirm helpers (not in `dispatch_write` core; `:477`).

This means the extraction is **mechanical, not architectural** — the kernel is already separated from the wire format by the `handle_write_call` ↔ `dispatch_write` split. 231 (perhaps unknowingly) left the seam in the right place.

### Options

**Option A — Move the kernel to `framework`, MCP and visual both call it. (RECOMMENDED)**
- `dispatch_write` + `WriteDispatcher` + `ExecutorFn`/`GuardEvaluatorFn`/`OverrideFn` + `merged_guards` + idempotency/audit helpers move to a new `framework` module (e.g. `framework::write::{dispatch_write, WriteDispatcher, ...}`), parameterized on an `audit channel` string so the prefix is `"{channel}.action.{name}"` not `"mcp.action.{name}"` (`write_dispatch.rs:436`).
- `ferro-mcp-server::handle_write_call` keeps the JSON-RPC framing and calls `framework::write::dispatch_write`.
- The new visual/form HTTP handler (a `framework` route or app controller) parses `POST /{service}/{action}`, authenticates the tenant via the existing auth middleware, derives the plan via `ferro::derive_transition_plan`, and calls the **same** `framework::write::dispatch_write`.
- *Why ferro-idiomatic:* `framework` is the shared home for cross-cutting runtime (`framework/src/lib.rs` is the public API; HTTP, middleware, database all live there per `CLAUDE.md` File Locations). Both `ferro-mcp-server` and an HTTP form handler legitimately depend on `framework`. It satisfies "no per-channel executor" literally — one `dispatch_write`, two callers. It respects project-agnostic crates once the audit prefix is parameterized.
- *Cost:* moves ~120 lines + helpers and updates `ferro-mcp-server` imports; the `crate::Error` → `framework::Error` (or a shared error) bridge must be reconciled (`ferro-mcp-server/error.rs`).

**Option B — Visual path calls into `ferro-mcp-server::dispatch_write`.**
- *Rejected.* The visual/JSON-UI/form layer would depend on the MCP server crate purely to perform a write. That inverts the dependency intent (`ferro-json-ui` and HTTP controllers should not pull in the MCP transport) and embeds the channel name `mcp.action.*` in non-MCP audit trails. It also makes "no per-channel executor" technically true but architecturally backwards.

**Option C — Keep two executors, share only `derive_transition_plan`.**
- *Rejected — violates EXEC-05 directly.* "no per-channel executor" forbids a separate visual executor. Sharing only the pure plan but duplicating the guard-reeval/audit/idempotency envelope re-creates the "declare the envelope twice" bug class one layer up. This is precisely the `feedback_no_duplicate_control_surface` failure.

### Recommendation

**Option A.** Extract the already-channel-agnostic `dispatch_write` + `WriteDispatcher` envelope into `framework` (parameterizing the audit channel prefix to honor project-agnostic crates), leave the MCP/JSON-RPC framing in `ferro-mcp-server` calling into it, and add a `framework`-level visual/form write entry point that receives the `POST /{service}/{action}` the projection already emits (`builder.rs:685`) and calls the same kernel. One executor, two callers — EXEC-05 satisfied by construction.

*The kernel does NOT go in `ferro-projections`* (it touches `sea-orm`/`tokio`/closures, forbidden there — `ferro-projections/Cargo.toml:18-21`, `CLAUDE.md`). The REQUIREMENTS.md "no new crate" rule (line 43) is about **derivation**, which already lives in `ferro-projections`; the **runtime envelope** is a different concern and `framework` is its idiomatic home.

### Open design sub-questions for `/gsd-discuss-phase`
- Confirm `framework` (vs. leaving the kernel in `ferro-mcp-server` and having the visual handler depend on it) — Option A vs B.
- Reconcile `ferro-mcp-server::Error` ↔ `framework::Error` when the kernel moves (a shared error or a `From` bridge).
- Whether the confirmation seam (`#[cfg(feature = "confirmation")]`) is exercised by the visual path in v16.0 or deferred (forms have their own confirm UX).

---

## Q4 — What "retire the hand-written WriteDispatcher" means precisely

**Two readings — the planner must pick the right one or it will delete load-bearing infrastructure.**

1. **"Delete the `match action_name`"** — the per-action transition target re-encoding. **Already done in 231** (`grep -rn 'match action_name' app/src` empty; `app/src/controllers/mcp.rs:108` derives `to_state`). Nothing left to retire here.

2. **"Dismantle the `WriteDispatcher` / `ExecutorFn` registry"** — **DO NOT.** `WriteDispatcher` (`write_dispatch.rs:102-126`) is the runtime envelope: it holds the app-registered `ExecutorFn` (the SeaORM persist closure), the `GuardEvaluatorFn` (the live guard runner — the load-bearing security gate, `:357-366`), and the `OverrideFn` registry (EXEC-03). It is NOT the thing being retired; it is the thing being SHARED. The phase title's "retire the hand-written WriteDispatcher" is misleading shorthand for "retire the hand-written per-action transition logic, which 231 already did" — **not** "delete the dispatcher infrastructure."

**Precise instruction for the planner:** the `match` is gone; the `WriteDispatcher`/`ExecutorFn`/`GuardEvaluatorFn`/`OverrideFn` infrastructure STAYS and gets relocated (Option A) so both channels share it. Deleting it would remove the guard re-eval and persist machinery — a security regression.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Transition fact derivation (event→to_state, guard) | `ferro-projections` (schema) | — | Owns `StateMachine`/`ActionDef`; pure `derive_transition_plan` (done in 231) |
| Transition-execution kernel (plan→guard re-eval→persist→audit→idempotency→override) | `framework` (shared runtime) — RECOMMENDED, currently in `ferro-mcp-server` | — | Channel-agnostic; touches `sea-orm`/`tokio` so cannot live in `ferro-projections`; must be shared so there is no per-channel executor (EXEC-05) |
| MCP/JSON-RPC framing (tool-name parse, RPC error codes, confirm prefix routing) | `ferro-mcp-server` | — | MCP transport detail; calls the shared kernel (`handle_write_call`, `write_dispatch.rs:470+`) |
| Visual/form framing (HTTP `POST /{service}/{action}`, form parse, redirect/notify outcome) | `framework` HTTP layer / app controller — NEW | `ferro-json-ui` (emits the URLs, `builder.rs:685`) | The unbuilt half of EXEC-05; receives the projection's action-route contract and calls the shared kernel |
| Action button URL emission | `ferro-json-ui::projection::builder` | — | Already emits `POST /{service}/{action}` (`builder.rs:309,685`); the receiving handler is what's missing |
| Tenant scoping (find_for_tenant) | app `ExecutorFn` (registered) | — | Stays in the app-supplied executor closure (`mcp.rs:86-95`); the kernel is tenant-agnostic plumbing |

---

## Standard Stack

### Core (no new external deps)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| (none new) | — | Phase 232 relocates existing code + adds an HTTP handler | The kernel already exists in `ferro-mcp-server`; derivation already exists in `ferro-projections` [VERIFIED: grep — no new crate needed] |
| `sea-orm` | workspace | Kernel reads/persists the entity via the registered `ExecutorFn` | Already a `ferro-mcp-server` dep; moves with the kernel into `framework` (which already depends on it) [VERIFIED: write_dispatch.rs:13] |
| `ferro-audit` | workspace | Audit envelope (`AuditEntry::record`, `write_dispatch.rs:436`) | Existing; channel prefix must be parameterized when shared [VERIFIED] |
| `ferro_projections::derive_transition_plan` | workspace (0.2.65) | Plan derivation feeding both channels | Shipped in 231 (`ferro-projections/src/executor.rs`); reached via `ferro::derive_transition_plan` facade [VERIFIED: mcp.rs:108] |

**Installation:** No new crates. Phase 232 relocates a module and adds an HTTP entry point. `framework/Cargo.toml` already depends on `sea-orm`, `ferro-audit`, and (via facade) `ferro-projections` — verify before planning with `cargo tree -p ferro-rs | grep -E 'sea-orm|ferro-audit|ferro-projections'`.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Shared kernel in `framework` (Option A) | Visual handler depends on `ferro-mcp-server` (Option B) | Rejected — inverts crate-dependency intent; embeds `mcp.action.*` in non-MCP audit trails |
| One shared kernel | Two executors sharing only the plan (Option C) | Rejected — violates "no per-channel executor"; re-creates the duplicate-envelope bug class |
| Relocate to `framework` | New `ferro-write` crate | Avoid unless discuss-phase prefers crate isolation; `framework` is the established home for cross-cutting runtime and avoids a crate proliferation the milestone explicitly resists (REQUIREMENTS.md:43 spirit) |

---

## Architecture Patterns

### System Architecture Diagram

```
  ┌─────────────────────────┐         ┌──────────────────────────────────┐
  │ MCP caller (agent)      │         │ Visual caller (form / projection │
  │ tools/call {name, args} │         │ action button — UNBUILT receiver)│
  └───────────┬─────────────┘         └──────────────┬───────────────────┘
              │                                       │ POST /{service}/{action}
              ▼                                       ▼ (builder.rs:685 emits this URL;
  ferro-mcp-server::handle_write_call        framework HTTP handler (NEW) parses it,
   (write_dispatch.rs:470)                     authenticates tenant, resolves Action)
   - parse call_params["name"]                       │
   - JSON-RPC error envelope (-32601/-32603)         │
   - confirm_ / request_confirm_ routing             │
              │   derive_transition_plan(svc,name)    │   derive_transition_plan(svc,name)
              │   (ferro-projections, pure)           │   (same pure derivation)
              └───────────────┬───────────────────────┘
                              ▼
        ┌──────────────────────────────────────────────────────────────┐
        │  SHARED transition-execution kernel  (move to framework::write)│
        │  = today's dispatch_write (write_dispatch.rs:336-458)          │
        │   1. guard re-eval — merged_guards, LIVE (:357)  EXEC-02        │
        │   2. idempotency check (tenant+key) (:376)                      │
        │   3. confirmation seam (feature) (:401)                        │
        │   4. execute registered ExecutorFn → persist to_state (:410)   │
        │   5. store idempotency result (:422)                           │
        │   6. audit — "{channel}.action.{name}" (:436, parameterize)    │
        │   7. post-persist OverrideFn (:453)  EXEC-03                    │
        └──────────────────────────────────────────────────────────────┘
                              │ one declaration → one executor → both channels
                              ▼
                      CallToolResult (MCP)  /  redirect+notify (visual)
```

### Anti-Patterns to Avoid
- **A second transition kernel for the visual path** — directly violates EXEC-05 "no per-channel executor" and `feedback_no_duplicate_control_surface`. The visual handler must call the SAME `dispatch_write`.
- **Deleting `WriteDispatcher`/`ExecutorFn`/`GuardEvaluatorFn`** — that is the security envelope (guard re-eval at `:357`), not the thing being retired (Q4).
- **Leaving the audit prefix `"mcp.action.{}"` hardcoded after sharing** — a visual write would be audited as an MCP action; violates project-agnostic crates (`CLAUDE.md`). Parameterize the channel.
- **A new `match action_name` in the visual handler** — re-introduces the exact duplication 231 deleted. The visual handler resolves the action by name from `ServiceDef.actions` (mirror `find_action`, `write_dispatch.rs:150`) and derives `to_state` from the plan.
- **The visual handler reading the transition target from the form payload** — the `to_state` comes ONLY from `derive_transition_plan(...).to_state` (`Transition.to`), never from client input.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Plan derivation for the visual path | A second derive | `ferro::derive_transition_plan` (231) | Already pure + shared; the visual handler calls the identical function (`mcp.rs:108` pattern) |
| Guard re-eval for visual writes | A new guard loop | The kernel's `merged_guards` + live loop (`write_dispatch.rs:137,357`) | Moving the kernel brings this for free; it is the audited, fail-closed security gate |
| Idempotency / audit for visual writes | New plumbing | The kernel's steps 2,5,6 (`write_dispatch.rs:376,422,436`) | Shared by construction once the kernel moves; parameterize the channel prefix |
| Action lookup by name | A new map | `find_action` pattern (`write_dispatch.rs:150`) | Resolves `(&ServiceDef, &ActionDef)` across mcp-exposed services; the visual variant resolves across all projection-exposed services |
| Action button URLs | New emission | `builder.rs:309,685` already emit `POST /{service}/{action}` | The contract exists; only the receiving handler is missing |

**Key insight:** Phase 232 builds almost no new logic — it **relocates** the existing channel-agnostic kernel and **adds one HTTP entry point** that calls it. The hard parts (derivation, guard re-eval, audit, idempotency, override) all exist and are tested. The genuinely new code is: (1) the module move + audit-channel parameterization, (2) the visual/form HTTP handler.

---

## Common Pitfalls

### Pitfall 1: Treating EXEC-05 as "verify MCP, done"
**What goes wrong:** Reading the Scope Reality Check Q1, concluding the MCP path satisfies EXEC-05, and marking the phase complete without building the visual surface.
**Why it happens:** 231 did the visible, named work (deleted the `match`); the remaining work is the absence of a visual handler, which is invisible to a grep.
**How to avoid:** EXEC-05 says "**every** modality." Visual writes today hit a void (`builder.rs:685` → no handler). The phase is incomplete until a visual write drives the shared kernel.
**Warning signs:** A plan with only MCP regression tests and no `POST /{service}/{action}` handler.

### Pitfall 2: Deleting the WriteDispatcher infrastructure (Q4 misread)
**What goes wrong:** Interpreting "retire the hand-written WriteDispatcher" literally and removing `ExecutorFn`/`GuardEvaluatorFn`, dropping the live guard gate.
**Why it happens:** The phase title's shorthand.
**How to avoid:** The `match` is what's retired (done in 231). The dispatcher is the security envelope — it STAYS and gets shared.
**Warning signs:** A diff deleting `WriteDispatcher` or `dispatch_write`.

### Pitfall 3: Visual path gets its own executor
**What goes wrong:** Building a parallel visual write function that re-derives the plan, re-runs guards, re-audits — a second envelope.
**Why it happens:** It feels simpler than relocating a `ferro-mcp-server` module.
**How to avoid:** Option A — one `dispatch_write`, two callers. The visual handler is framing only.
**Warning signs:** Two functions both calling `AuditEntry::record(...action...)` or both running a guard loop.

### Pitfall 4: Audit channel stays `mcp.action.*` for visual writes
**What goes wrong:** A form-submitted transition is audited as an MCP action (`write_dispatch.rs:436`).
**Why it happens:** The prefix is hardcoded; moving the kernel without parameterizing carries it.
**How to avoid:** Thread the channel name (`"mcp"` / `"web"`) into the kernel; audit `"{channel}.action.{name}"`. Also honors project-agnostic crates.
**Warning signs:** A visual-path audit assertion expecting `mcp.action.submit`.

### Pitfall 5: Cross-tenant write through the new visual handler
**What goes wrong:** The visual handler trusts a tenant id from the form, or skips tenant scoping.
**Why it happens:** New entry point, easy to forget the MCP path's `tenant_id` discipline (`write_dispatch.rs:339` — "from auth, never from payload").
**How to avoid:** The visual handler authenticates the tenant via the existing auth middleware (same principal source as MCP) and passes it to the kernel; the registered `ExecutorFn` keeps `find_for_tenant` scoping (`mcp.rs:86-95`).
**Warning signs:** A visual handler reading `tenant_id` from `req.input()` / form body.

---

## Code Examples

### The visual write entry point this phase must add (shape)
```rust
// NEW (framework HTTP layer or app controller). Receives the URL the projection
// already emits: builder.rs:685 -> Action::new("/{service}/{action}").
// Calls the SAME kernel as handle_write_call — no second executor.
// Source pattern: ferro-mcp-server/src/write_dispatch.rs:470-545 (find_action,
// derive_transition_plan, dispatch_write), with HTTP framing instead of JSON-RPC.
async fn handle_visual_action(req: Request /* {service}, {action} path params */) -> Response {
    let tenant_id = req.authenticated_tenant()?;           // from auth, never the form body
    let (svc, action) = find_projection_action(&service, &action_name)?;
    let plan = ferro::derive_transition_plan(svc, &action.name).ok();   // same derivation
    let transition_guard = plan.as_ref().and_then(|p| p.guard.as_deref());
    let inputs = req.form_inputs().await?;                  // validated form fields
    let result = framework::write::dispatch_write(          // SHARED kernel (relocated)
        action, &inputs, tenant_id, db, &dispatcher,
        transition_guard, /* channel: */ "web",
    ).await?;
    // outcome: redirect + notify (ActionOutcome::Redirect/Notify, action.rs:71-83)
}
```

### The kernel that gets relocated (already channel-agnostic — only 3 MCP couplings)
```rust
// Source: ferro-mcp-server/src/write_dispatch.rs:336-458 (today). Moves to
// framework::write. The signature has NO MCP type — only the audit prefix
// (:436 "mcp.action.{}") and crate::Error need parameterizing/bridging.
pub async fn dispatch_write(
    action: &ActionDef, inputs: &Value, tenant_id: i64,
    db: &DatabaseConnection, dispatcher: &WriteDispatcher,
    transition_guard: Option<&str>,
    // ADD: channel: &str   // "mcp" | "web" — replaces the hardcoded "mcp" prefix at :436
    #[cfg(feature = "confirmation")] is_confirmed: bool,
) -> Result<Value> { /* steps 1-7 unchanged */ }
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| App hand-writes `match action_name => new_status` per channel | Derived `to_state` from `Transition.to`; MCP path uses it | Phase 231 | MCP write surface single-sourced (`mcp.rs:108`) |
| Write kernel lives in `ferro-mcp-server`, callable only via MCP framing | Channel-agnostic kernel in `framework`, called by MCP + visual framing | Phase 232 (this) | Visual writes route through the same executor — EXEC-05 |
| Projection action buttons POST to a route with no handler | `POST /{service}/{action}` handled by the shared visual entry point | Phase 232 (this) | `builder.rs:685` contract becomes functional, not dangling |

**Deprecated/outdated:** The phase-title phrase "retire the hand-written WriteDispatcher" is misleading — the `match` is already retired (231); the `WriteDispatcher` infrastructure is relocated and shared, not removed.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `framework` already depends on `sea-orm` + `ferro-audit` + (facade) `ferro-projections`, so the kernel can move there without a new edge | Standard Stack, Q3 | If `framework` lacks one, either add the dep or pick a different home; verify with `cargo tree -p ferro-rs` before planning |
| A2 | The visual/form transition-write surface is genuinely UNBUILT (not built elsewhere I didn't grep) | Q2 | Verified by grep across app/src, framework/src, ferro-inertia/src, ferro-json-ui/src — no `POST /{service}/{action}` handler. Low risk |
| A3 | EXEC-05's "visual/form write path" means the projection-emitted action route (`builder.rs:685`), not some other surface | Q2, Q3 | If the milestone owner means Inertia-specific writes or a different surface, the entry-point shape changes; confirm in discuss-phase |
| A4 | The confirmation seam (`#[cfg(feature="confirmation")]`) need not be wired to the visual path in v16.0 | Q3 sub-questions | If forms require the same destructive-confirm UX, scope grows; confirm in discuss-phase |
| A5 | Relocating the kernel to `framework` is acceptable vs. the "no new crate" rule (which targets derivation, not runtime) | Q3 | If the owner insists the runtime kernel stay in `ferro-mcp-server`, fall back to Option B with the dependency-inversion cost noted |

---

## Open Questions

1. **Where the shared kernel lives (Q3).**
   - What we know: `dispatch_write` is already channel-agnostic (`write_dispatch.rs:336`); `framework` is the idiomatic cross-cutting runtime home.
   - What's unclear: whether discuss-phase prefers `framework` (Option A) or a dedicated crate; how `ferro-mcp-server::Error` reconciles with `framework::Error`.
   - Recommendation: Option A (`framework`), bridge the error type, parameterize the audit channel.

2. **What the "visual/form write path" concretely is (A3).**
   - What we know: the projection emits `POST /{service}/{action}` (`builder.rs:685`); no handler exists.
   - What's unclear: whether EXEC-05 also intends Inertia form writes or only the projection action route.
   - Recommendation: target the projection action-route contract first (it is the named, emitted surface); confirm in discuss-phase.

3. **Confirmation seam on the visual path (A4).**
   - What we know: MCP gates destructive transitions behind `request_confirm_`/`confirm_` (`write_dispatch.rs:490-518`).
   - What's unclear: whether forms reuse that or have their own confirm UX.
   - Recommendation: ship the visual path with the seam wired but the form confirm-UX deferred unless discuss-phase requires it.

---

## Environment Availability

Not applicable — Phase 232 is a pure-Rust relocation + new HTTP handler within the existing workspace. No external tools/services beyond the standard `cargo` toolchain. SeaORM/tokio/ferro-audit are already present where the kernel lands [VERIFIED: write_dispatch.rs:13, framework deps to confirm in A1].

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` / `#[tokio::test]` (+ `insta` snapshots where used) |
| Config file | none — `cargo test` (workspace) |
| Quick run command | `cargo test -p ferro-mcp-server` (MCP regression) + the new visual-path test crate |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| EXEC-05 | A `submit` transition driven via the **visual** `POST /{service}/{action}` handler persists the derived `to_state` ("submitted") through the shared kernel | integration | `cargo test -p app visual_action_persists_derived_to_state` | ❌ Wave 0 (new handler + test) |
| EXEC-05 | The **same** transition via MCP and via visual produces identical guard re-eval + identical `to_state` + identical (channel-distinct) audit | integration | `cargo test -p app single_source_both_channels` | ❌ Wave 0 |
| EXEC-05 | A guard-failing transition is rejected on the **visual** path (live re-eval, not trusted from form) | integration | `cargo test -p app visual_guard_rejects_illegal_transition` | ❌ Wave 0 |
| EXEC-05 | No second executor exists — `grep` finds exactly one `dispatch_write` definition; no `match action_name` anywhere | structural | `grep -rn 'pub async fn dispatch_write' . ; grep -rn 'match action_name' app/src` (one / empty) | ✅ assert in test or CI step |
| EXEC-05 | Visual write is audited with a `web`/non-mcp channel prefix, not `mcp.action.*` | integration | `cargo test -p app visual_audit_channel_is_web` | ❌ Wave 0 |
| EXEC-05 (regression) | MCP path still routes through the relocated shared kernel | integration | `cargo test -p ferro-mcp-server` (existing `submit_persists_derived_to_state`, `guard_rejects_illegal_transition`) | ✅ exists (231) — must stay green after the move |
| EXEC-05 | Cross-tenant visual write denied (tenant from auth, not form) | integration | `cargo test -p app visual_cross_tenant_denied` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-mcp-server` (regression — kernel move must not break MCP) + the touched crate.
- **Per wave merge:** `cargo test -p ferro-mcp-server -p framework -p app`.
- **Phase gate:** Full suite green (`fmt + clippy --all --all-targets -D warnings + test --all-features`) before `/gsd-verify-work`. NOTE: `cargo test --all-features` recurrently disk-full-fails on this host (`project_ferro_disk_full_test_gate`) — check `df` and clean `target/` first.

### Wave 0 Gaps
- [ ] Relocate `dispatch_write` + `WriteDispatcher` + fn types to `framework::write` (or chosen home); parameterize audit channel — covers the EXEC-05 single-kernel requirement.
- [ ] New visual/form HTTP handler receiving `POST /{service}/{action}` calling the shared kernel — the unbuilt half of EXEC-05.
- [ ] `app` integration tests: `visual_action_persists_derived_to_state`, `single_source_both_channels`, `visual_guard_rejects_illegal_transition`, `visual_audit_channel_is_web`, `visual_cross_tenant_denied`.
- [ ] Structural assertion: exactly one `dispatch_write`, `match action_name` empty.
- Framework install: none — all tooling present.

---

## Security Domain

`security_enforcement` not explicitly `false` → included. Phase 232 puts a **new write entry point** (the visual handler) directly on the authorization path, so the security surface is load-bearing.

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V1 Architecture | yes | One shared transition kernel — no parallel control surface; the visual path calls the SAME `dispatch_write` (EXEC-05, coherence constraint) |
| V4 Access Control | yes | Live guard re-eval on the visual path via the shared `merged_guards` loop (`write_dispatch.rs:357`); never trust a form-supplied transition target or guard outcome |
| V5 Input Validation | yes | `validate_action_inputs` (`write_dispatch.rs:169`) and the plan's `from_states` assertion apply to visual inputs too; `to_state` comes ONLY from `Transition.to` |
| V7 Error Handling | yes | The new handler must not leak SQL/table names — reuse the existing redaction pattern (`write_dispatch.rs:497`) |
| V11 Business Logic | yes | The StateMachine is the business-logic guard; both channels deriving from it prevents a visual gap driving an illegal transition |

### Known Threat Patterns for the cross-surface write path
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Visual handler trusts `to_state`/`status` from the form payload | Tampering / EoP | Derive `to_state` from `Transition.to` via `derive_transition_plan` only; never from input |
| Guard bypass on the new visual path (skip the kernel) | Elevation of Privilege | Visual handler MUST call the shared `dispatch_write`; live guard re-eval (`:357`), no second executor |
| Cross-tenant write via the new entry point | Information Disclosure / Tampering | Tenant id from auth middleware (never form body, mirror `write_dispatch.rs:339`); `ExecutorFn` keeps `find_for_tenant` |
| Audit-trail confusion (visual writes logged as MCP) | Repudiation | Parameterize the audit channel prefix (`:436`) — `web.action.*` vs `mcp.action.*` |
| Idempotency/replay on the visual path | Tampering | The kernel's tenant+key-scoped idempotency (`:376,422`) applies once shared; the visual handler passes through any `idempotency_key` |

---

## Sources

### Primary (HIGH confidence — all read/grepped at 0.2.65)
- `ferro-mcp-server/src/write_dispatch.rs` — `dispatch_write` (:336-458, the channel-agnostic kernel), `WriteDispatcher`/`ExecutorFn`/`GuardEvaluatorFn`/`OverrideFn` (:38-126), `merged_guards` (:137), `find_action` (:150), live guard loop (:357), audit prefix `"mcp.action.{}"` (:436), override hook (:453), `handle_write_call` MCP framing (:470-545), `derive_transition_plan` call sites (:541, :678, :839), JSON-RPC error codes (:528,:534)
- `app/src/controllers/mcp.rs` — derived `to_state` (:108), tenant-scoped executor (:86-95), `make_write_dispatcher` (:68-71), the only `WriteDispatcher` consumer + `mcp_chat.rs:87`
- `ferro-json-ui/src/projection/builder.rs` — projection emits `POST /{service}/{action}` action URLs (:309 row actions, :685 actions slot), Risk-4 contract comment (:669-671)
- `ferro-json-ui/src/action.rs` — `ActionHandler` decoupled string handlers (:88-100), `ActionOutcome` (:71-83), `HttpMethod` (:35)
- `app/src/projections/order.rs` — declared StateMachine + ActionDefs (single source of truth)
- `.planning/REQUIREMENTS.md` — EXEC-01..05, milestone goal, coherence constraint (:7,42,43), traceability (:51-59)
- `.planning/phases/231-.../231-01-SUMMARY.md`, `231-02-SUMMARY.md`, `231-RESEARCH.md` — what 231 built
- grep evidence: `match action_name` empty in app/src; write-dispatch consumers all MCP; `ferro-inertia` has zero ActionDef/transition refs; no `POST /{service}/{action}` handler anywhere

### Secondary (MEDIUM confidence)
- None — all claims grounded in primary source files.

### Tertiary (LOW confidence)
- None.

---

## Metadata

**Confidence breakdown:**
- Scope Reality Check (Q1/Q2): HIGH — MCP wiring and the absent visual handler both verified by direct grep + read, not inferred from summaries.
- Architecture Decision (Q3): HIGH — `dispatch_write`'s channel-agnostic signature and its 3 MCP couplings were read line-by-line.
- Q4 (retire meaning): HIGH — `WriteDispatcher` is the guard/persist envelope (`:357,410`), not the retired `match`.
- Validation strategy: MEDIUM — the visual-path tests are designed against the recommended Option A; exact file paths depend on the chosen kernel home.

**Research date:** 2026-06-16
**Valid until:** 2026-07-16 (stable — internal workspace code, no fast-moving external deps)

## Recommended Approach (summary)

1. **Do not redo the MCP path** — EXEC-05's MCP clause is satisfied by 231 (verify-only).
2. **Do not delete `WriteDispatcher`** — it is the security envelope; relocate and share it.
3. **Relocate the channel-agnostic kernel** (`dispatch_write` + `WriteDispatcher`) to `framework::write`, parameterizing the audit channel prefix (Option A).
4. **Build the visual/form write entry point** that receives the projection's `POST /{service}/{action}` (`builder.rs:685`), authenticates the tenant, derives the plan, and calls the shared kernel — no second executor, no `match`, `to_state` from `Transition.to` only.
5. **Prove single-source** with a both-channels integration test asserting identical guard re-eval, identical `to_state`, and a channel-distinct audit entry.
