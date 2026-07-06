# Phase 152: ferro-orm GuardedUpdate — Context

**Gathered:** 2026-05-13
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults applied to every gray area)
**Milestone:** v11.11 Resource Reservation & Live Read-Model Primitives
**Driver:** gestiscilo-it inventory monitoring field test
**Killer feature (milestone):** Race-free reservations as a first-class framework primitive. Phase 152 is the foundational kernel that makes it correct by construction.

<domain>
## Phase Boundary

Create a new `ferro-orm` crate inside the ferro workspace that ships **`GuardedUpdate`** — a typed builder for atomic, conditional SQL `UPDATE` statements that race-free-by-construction replace the hand-rolled `read → check → write` pattern wherever a column's value is conditionally mutated.

The crate is intentionally minimal at v0: it is a thin SeaORM extension, not an extraction of `framework/src/database/`. It exposes:

- `GuardedUpdate<E: EntityTrait>` — the chainable builder
- `GuardedError` — `NoRowsAffected | TooManyRows { affected: u64 } | EmptyUpdate | Db(DbErr)`
- Targeted re-exports of the SeaORM symbols a consumer needs to call the builder (`EntityTrait`, `ColumnTrait`, `ConnectionTrait`, `IntoCondition`, `SimpleExpr`, `Value`, `DbErr`) — not a blanket `pub use sea_orm::*`

The phase delivers ONE foundational primitive that downstream phases (154 reservation, gestiscilo-it inventory) depend on. It does NOT extract the broader ORM into a standalone crate — that is out of scope and would derail v11.11.

**In scope:** crate scaffold, builder API, error model, multi-column atomic updates, SQLite-backed tests, rustdoc, one user-facing doc page, version bump + auto-publish.

**Out of scope (deferred):** `GuardedDelete`, `GuardedInsert`, broader extraction of `framework/src/database/` into `ferro-orm`, audit-log integration (Phase 153 territory), event emission on success, Postgres-specific integration tests in CI, `UPDATE … RETURNING` row return.
</domain>

<decisions>
## Implementation Decisions

### Crate placement & scope

- **D-01:** Ship as a new top-level workspace crate at `ferro-orm/` — NOT inside `framework/src/database/`. The roadmap explicitly names `ferro-orm::GuardedUpdate`; phases 154 and external consumer apps will import it as `use ferro_orm::GuardedUpdate;`. Putting it inside `framework` would force every consumer to depend on the full framework crate.
- **D-02:** Crate is thin and additive at v0. It does NOT take over `framework/src/database/` ownership. Migration of `query_builder.rs`, `model.rs`, `connection.rs`, etc. into `ferro-orm` is explicitly deferred — `ferro-orm v0.x` is the GuardedUpdate kernel only. Naming `ferro-orm` claims the future namespace without paying the extraction cost in this phase.
- **D-03:** Re-export only the SeaORM symbols the public API references: `EntityTrait`, `ColumnTrait`, `ConnectionTrait`, `IntoCondition`, `SimpleExpr`, `Value`, `DbErr`. Do NOT `pub use sea_orm::*`. Consumers that need the full SeaORM API can depend on `sea-orm` directly — keeps the `ferro-orm` surface inspectable in MCP and stable across SeaORM upgrades.
- **D-04:** Wave 1a publish (zero internal ferro-* deps). External deps: `sea-orm` (1.0, workspace version), `thiserror` (2), and whatever sea-orm pulls. Add to `.github/workflows/publish.yml` Wave 1a alongside `ferro-wallet`. New-crate-first-publish bootstrap from local terminal (CI token has publish-update only — see `project_ferro_publish_token_scoping.md`).

### Builder API

- **D-05:** Constructor: `GuardedUpdate::new(entity: E)`. `E` is the SeaORM entity (e.g. `inventory_units::Entity`).
- **D-06:** Filter API: `filter(self, f: impl IntoCondition) -> Self`. Multiple `.filter(...)` calls AND-combine onto an internal `Condition`. Matches SeaORM's `QueryFilter::filter` ergonomics so anyone fluent in sea-orm feels native.
- **D-07:** Set API: two methods, both chainable, both can be called multiple times to set multiple columns in one statement:
  - `set_expr(self, col: E::Column, expr: SimpleExpr) -> Self` — for value-derived updates (e.g. `Expr::col(Column::Quantity).sub(needed)`)
  - `set_value(self, col: E::Column, value: Value) -> Self` — for literal assignments (e.g. setting `updated_at`)
  - Internally stored as a `Vec<(E::Column, SetTarget)>` where `SetTarget` is an internal enum wrapping either form. Order of insertion preserved; later sets to the same column override earlier ones.
- **D-08:** Execution methods (ship both):
  - `exec_one<C: ConnectionTrait>(self, conn: &C) -> Result<(), GuardedError>` — succeeds iff exactly one row matched. `0 rows → NoRowsAffected`. `>1 rows → TooManyRows { affected }`. **This is the default callers should use for race-free counter mutations** — the predicate failure is the load-bearing signal that capacity was exhausted / pre-condition was unmet.
  - `exec_at_most_one<C: ConnectionTrait>(self, conn: &C) -> Result<bool, GuardedError>` — returns `Ok(true)` on 1 row, `Ok(false)` on 0 rows (predicate failure is a normal outcome), `Err(TooManyRows)` on >1 rows. For optimistic updates where "no match" is expected and shouldn't pollute error logs.
- **D-09:** Connection generic: `<C: ConnectionTrait>`. Works equally with `&DatabaseConnection`, `&DatabaseTransaction`, or any other SeaORM connection. **No global `DB::connection()` shortcut** — the caller passes the connection explicitly. This forces them to think about whether the update belongs inside a transaction and prevents accidental cross-connection race windows.
- **D-10:** No `UPDATE … RETURNING` support in v0. Cross-dialect portability (SQLite vs Postgres) requires dialect-specific code that SeaORM does not currently abstract cleanly. Callers who need the post-update row re-fetch it inside the same transaction. Documented as a known limitation.

### Error model

- **D-11:** `GuardedError` is a `thiserror`-derived enum, one error per crate, panics nowhere:
  ```rust
  pub enum GuardedError {
      #[error("guarded: predicate matched no rows")]
      NoRowsAffected,
      #[error("guarded: predicate matched {affected} rows (expected 1) — likely an index/uniqueness bug")]
      TooManyRows { affected: u64 },
      #[error("guarded: no columns to set — builder is empty")]
      EmptyUpdate,
      #[error("guarded: db error: {0}")]
      Db(#[from] sea_orm::DbErr),
  }
  ```
- **D-12:** `EmptyUpdate` is returned at `exec_*` time when `Vec<(Column, SetTarget)>` is empty. A builder with no `set_*` calls is a programming bug; refusing the call (vs producing invalid SQL) catches it loudly in tests. Cheaper than a compile-time `MissingSetMarker` type-state — a plain runtime error reads better in tracebacks for a tool aimed at AI-authored code.
- **D-13:** `TooManyRows` is preserved (the design doc has it; we keep it). Every guarded update is morally a unique-key-equivalent operation; if `affected > 1`, the filter is wrong or the index assumption is broken. Surfacing it makes the bug loud rather than letting two reservations both "succeed".

### Concurrency contract

- **D-14:** The crate's correctness claim: when the same `GuardedUpdate` is executed concurrently against the same row from N processes, **at most one of them sees `Ok(())` from `exec_one` for any given pre-condition violation; the rest see `NoRowsAffected`**. Mechanism: SeaORM compiles to a single `UPDATE … WHERE …` statement, which is atomic at the DB level on both SQLite (serial writer) and Postgres (`READ COMMITTED`). No application-side locks, no SELECT-then-UPDATE, no round-trip race window.
- **D-15:** Connection responsibility note in the rustdoc: the atomicity guarantee is per-statement, not per-builder. A caller building `.set_expr(qty - 1)` and reading the resulting `qty` value in a separate query without a transaction reintroduces a race. The crate's job is to make the conditional UPDATE race-free; bracketing it in a transaction is the caller's job. Documented explicitly to prevent misuse.

### Testing

- **D-16:** Unit tests in `ferro-orm/src/guarded.rs` (or a dedicated `#[cfg(test)] mod tests`) using in-memory SQLite via the existing framework testing pattern. Cover:
  1. Predicate matches → 1 row affected → `exec_one` returns `Ok(())`
  2. Predicate fails → 0 rows → `exec_one` returns `Err(NoRowsAffected)`, `exec_at_most_one` returns `Ok(false)`
  3. Predicate matches >1 row → both methods return `Err(TooManyRows { affected: 2 })`
  4. `EmptyUpdate` returned when no `set_*` called
  5. Multiple `.set_expr` / `.set_value` calls produce a single UPDATE that mutates all columns atomically
  6. Builder works inside `&DatabaseTransaction` (transaction rollback rolls back the guarded update)
  7. Multiple `.filter` calls AND-combine
- **D-17:** ONE integration test (`tests/concurrent_decrement.rs`) that proves the race-free claim: spin up N=10 tokio tasks all attempting `GuardedUpdate` on a counter starting at K=3, assert exactly 3 succeed with `Ok(())` and 7 fail with `NoRowsAffected`. Run against in-memory SQLite; the serial-writer model is sufficient to demonstrate the contract.
- **D-18:** Property tests are **not in scope** for Phase 152. The reservation crate (Phase 154) carries the property-test budget for the milestone (per `INVENTORY-PRIMITIVES.md` testing strategy). Phase 152 ships hand-written tests that fully cover the surface.
- **D-19:** Postgres integration tests deferred. Adding docker-Postgres to CI for one crate's race test is disproportionate; SQLite serial-writer + `READ COMMITTED` semantics of the underlying SQL pattern are documented as equivalent for this operation. Risk accepted.

### Documentation

- **D-20:** Module-level rustdoc on `lib.rs` with the canonical example (the inventory-decrement snippet from `INVENTORY-PRIMITIVES.md` §`ferro-orm::guarded`). Include the misuse footgun ("atomicity is per-statement, not per-builder").
- **D-21:** New user-facing doc page `docs/src/database/atomic-updates.md` covering: why race-free updates matter, the `read → check → write` anti-pattern this replaces, the GuardedUpdate API, common patterns (counter decrement, status transition, optimistic concurrency), the `exec_one` vs `exec_at_most_one` decision tree.
- **D-22:** ferro-mcp introspection: if any current MCP tool description, `code_templates`, or `generation_context` mentions ORM/UPDATE patterns, audit and update them in this phase. If none do, the MCP impact is zero — `application_info` will simply gain `ferro-orm` in the installed-crates list for free. Audit during execution, do not pre-scope.

### Release

- **D-23:** Workspace `[workspace.package] version` bumps one patch (from `0.2.24` to `0.2.25`) when Phase 152 verifies. Standard ferro release process.
- **D-24:** Add `ferro-orm` to Wave 1a of `.github/workflows/publish.yml`. New-crate bootstrap (first publish requires personal publish-new token from local terminal) is the only manual step — same as Phase 151 PLAN-09.
- **D-25:** CHANGELOG entry under `ferro-orm` (new section) summarising: new crate, `GuardedUpdate` builder, race-free conditional updates, multi-column set, two exec variants.

### Folded scope from todos

No pending todos matched Phase 152.

### Deferred (NOT in this phase)

- `GuardedDelete` / `GuardedInsert` — plausible follow-ups, not on v11.11 critical path
- Extraction of `framework/src/database/{query_builder,model,connection,...}` into `ferro-orm` — would derail v11.11
- `UPDATE … RETURNING` for returning the updated row
- Audit-log emission on success — Phase 153 owns audit; consumer wraps the call in `audit_log!()` at the call site
- Event emission on success — out of scope; reservation kernel (Phase 154) emits events at its level, not at the ORM primitive level
- Postgres CI integration tests
- Property-based tests (Phase 154 carries that budget for the milestone)

### Claude's Discretion

Within the boundaries set above, the planner/executor decides:

- Internal module layout of `ferro-orm/src/` (single `lib.rs` vs `lib.rs + guarded.rs`)
- Internal `SetTarget` enum shape (the public surface is the chainable `set_expr` / `set_value` methods)
- Exact rustdoc prose & code-block formatting
- Test file names within `ferro-orm/tests/`
- Whether to expose `into_query()` for diagnostics (probably no — keeps the surface tight)
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design source of truth

- `.planning/research/INVENTORY-PRIMITIVES.md` §`ferro-orm::guarded` — the original kernel design (API shape, error variants, concurrency claim). This is the spec.
- `.planning/research/INVENTORY-PRIMITIVES.md` §`Cross-crate relationships` — confirms `ferro-orm::guarded` is a leaf (no internal ferro deps), and is used by `ferro-reservation` (Phase 154).
- `.planning/research/INVENTORY-PRIMITIVES.md` §`Migration / rollout` — confirms `ferro-orm::guarded` ships first in v11.11, additive.

### Project conventions

- `CLAUDE.md` §`Architecture Principles` — project-agnostic crates rule (no hardcoded app identity); `ferro-orm` must not reference any consumer.
- `CLAUDE.md` §`Testing & Linting` — exact fmt + clippy + test commands required pre-commit.
- `.planning/PROJECT.md` — vision anchors; the projection/intent abstraction is the killer feature this milestone unblocks (via reservations + live read-models).
- `.planning/STATE.md` — current workspace version (`0.2.24`), next version is `0.2.25` after Phase 152 verifies.

### Patterns to mirror (template ferro-* crates)

- `ferro-wallet/Cargo.toml` — Wave 1a leaf-crate Cargo.toml shape (workspace inheritance, package metadata, dep style)
- `ferro-wallet/src/lib.rs` — module-level rustdoc tone for a v0 single-purpose crate
- `ferro-events/` — minimal crate with one primitive concept; closest structural analog to `ferro-orm` v0
- `.github/workflows/publish.yml` — Wave 1a crate list; new crate is added here
- `framework/src/database/query_builder.rs` — the SeaORM-fluent style the new builder should feel native next to (filter chaining, ConnectionTrait generics)

### Cross-phase coordination

- Phase 153 CONTEXT (when written) — ferro-audit ships in parallel; do NOT integrate audit emission into Phase 152
- Phase 154 CONTEXT (when written) — ferro-reservation depends on `ferro-orm::GuardedUpdate`; Phase 154 will exercise this API as its primary consumer

### Conventions repository

- `feedback_ci_clippy_command_match.md` — match CI's exact clippy command (`--all --all-targets -- -D warnings`) in pre-push checks
- `feedback_validate_scope_premises.md` — `ferro-orm` does not currently exist as a crate; this premise was verified before this CONTEXT was written

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable assets

- **SeaORM 1.0** is already a workspace dependency (`framework/Cargo.toml`). `ferro-orm` reuses it directly — no new SQL toolkit.
- **Workspace inheritance pattern** (`version.workspace = true`, `edition.workspace = true`, `license.workspace = true`) — copy from `ferro-wallet/Cargo.toml`.
- **`thiserror` 2** is the project standard error-derive — already used by every leaf crate.
- **In-memory SQLite testing harness** — `framework/src/database/testing.rs` exposes the pattern; `ferro-orm` tests reuse the same approach (spin up an in-memory SQLite connection, run migrations, exercise the builder).

### Established patterns

- **One Error enum per crate** (`thiserror` derive) — established convention across `ferro-wallet`, `ferro-stripe`, `ferro-events`, `ferro-notifications`. `GuardedError` follows the same shape.
- **Builder pattern: `with_*` taking `mut self` → `Self`** — used everywhere in framework. `GuardedUpdate` follows the same consuming-builder shape (`filter`, `set_expr`, `set_value` all take `self`).
- **Generic over `ConnectionTrait`** — `framework/src/database/query_builder.rs` and `framework/src/database/model.rs` both already accept `impl ConnectionTrait`; `GuardedUpdate::exec_*` matches.
- **`#[serde(rename_all = "snake_case")]`** on enums — N/A for Phase 152 (no serde-serialized types in this crate's public API; SeaORM types ride through unchanged).
- **Wave 1a Cargo.toml metadata fields** — `description`, `keywords`, `categories = ["database"]`, `repository`, `readme = "README.md"`, `homepage = "https://ferro-rs.dev"`. Copy from `ferro-wallet/Cargo.toml`.

### Integration points

- **Workspace `Cargo.toml`** — add `"ferro-orm"` to `[workspace.members]`
- **`.github/workflows/publish.yml`** — add `ferro-orm` to `WAVE1A_CRATES`
- **Workspace version bump** — `[workspace.package] version = "0.2.25"`
- **`framework/src/lib.rs`** — DO NOT add an automatic re-export of `ferro_orm`. Consumers depend on `ferro-orm` directly so framework users get an opt-in import. (Re-evaluate in a future phase if a `prelude` story emerges.)
- **`README.md` (workspace root)** — add `ferro-orm` to the workspace crates table (mirror how `ferro-wallet` was added in Phase 151).
- **`CLAUDE.md` "Workspace Structure" table** — add a row for `ferro-orm` so downstream agents see it immediately.
- **ferro-mcp `application_info` / `installed_crates`** — picks up `ferro-orm` automatically once it's a workspace member; no MCP code changes expected.

### Constraints surfaced by the scout

- `ferro-orm` is **a new top-level crate** — Phase 152 is the bootstrap. First publish requires manual personal-token bootstrap from local terminal (CI token is publish-update only) — same operational reality as `ferro-wallet` Phase 151 PLAN-09.
- The existing `framework/src/database/` is **not migrated** by this phase. Future-Alberto may want to fold it into `ferro-orm` later; today we just claim the name with a thin v0.

</code_context>

<specifics>
## Specific Ideas

- The canonical sample from the design doc, kept verbatim as the rustdoc top example:
  ```rust
  GuardedUpdate::new(inventory_units::Entity)
      .filter(inventory_units::Column::Id.eq(unit_id))
      .filter(inventory_units::Column::Quantity.gte(needed))
      .set_expr(inventory_units::Column::Quantity,
                Expr::col(Column::Quantity).sub(needed))
      .exec_one(&txn).await?;     // errors if rows_affected != 1
  ```
- The error-naming style across the workspace ("`config: …`", "`apple sign: …`") — `GuardedError` follows the same `"guarded: …"` Display prefix for grep-friendliness.
- The framing in the rustdoc: lead with the anti-pattern this replaces (`read → check → write`), then show the one-call replacement. Readers (humans and agents) should leave understanding *why* the type exists, not just *how*.

</specifics>

<deferred>
## Deferred Ideas

- **`GuardedDelete`** — conditional delete on a status column. Plausible v0.x.y addition but not on v11.11 critical path.
- **`GuardedInsert`** — conditional insert with `WHERE NOT EXISTS` semantics. Likely a future addition driven by a real use case, not pre-emptive.
- **Full `framework/src/database/` extraction into `ferro-orm`** — the long-term home for `query_builder.rs`, `model.rs`, `connection.rs`, eager loading, etc. Explicitly out of scope; would derail v11.11. The crate name is being claimed now with a thin v0 to make the future extraction less disruptive.
- **`UPDATE … RETURNING`** — return the updated row in one round-trip. Blocked on cross-dialect SeaORM support.
- **Audit-log emission** — Phase 153 (`ferro-audit`) is the canonical place for that. Consumers wrap `GuardedUpdate.exec_one()` in `audit_log!()` at their call site.
- **Event emission on success** — out of scope here; reservation kernel (Phase 154) emits domain events at its level.
- **Postgres CI integration tests** — would require docker-Postgres in CI. Disproportionate for one primitive; deferred until v11.11 wraps and we can decide pragmatically.
- **Property-based tests** — Phase 154 (`ferro-reservation`) carries the property-test budget for the milestone per `INVENTORY-PRIMITIVES.md` testing strategy.
- **`ferro::prelude` / framework re-export of `GuardedUpdate`** — leave consumers to import `ferro-orm` directly for now; revisit if a prelude story emerges.

### Reviewed Todos (not folded)

No todos matched this phase (cross_reference_todos returned zero matches).

</deferred>

---

*Phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-*
*Context gathered: 2026-05-13*
*Mode: --auto (single-pass, recommended defaults applied)*
