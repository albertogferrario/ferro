# Project Milestones: Ferro Framework

## v14.0 Channel Projection — Non-Visual Rendering (Shipped: 2026-06-13)

**Phases completed:** 99 phases, 373 plans, 307 tasks

**Key accomplishments:**

- Flat Spec/Element type foundation with parse-time structural validation (duplicate, ID format, root, dangling, cycle, depth) and an 18-fixture test corpus — additive alongside v1, zero regressions.
- Strip v1 types (JsonUiView, Component enum, ComponentNode, PluginProps, ~200 LoC custom ser/de), replace render.rs/resolve.rs/projection internals with Spec-native equivalents, and enforce D-32's runtime JsonSchema contract with a 42-test schema_for! smoke suite.
- Migrate the framework's JSON-UI integration from v1 (`JsonUiView` + `ComponentNode` + `Component` enum) to v2 (`Spec` + `Element`), port all 30 inline tests in `framework/src/json_ui/mod.rs`, and swap re-exports in `framework/src/lib.rs`. After this plan, any handler importing `ferro_rs::{JsonUi, Spec, Element}` compiles and renders against the v2 surface.
- Signature-and-string-swap migration of ferro-mcp (8 files) and ferro-cli (3 files) against the ferro-json-ui v2 surface produced by Plan 02. All live-code `JsonUiView` / `ComponentNode` references removed; template strings across 6 files now emit `Spec::builder()` / `Element::new()`. Two v1 scanners (json_ui_inspect.rs, application_info.rs) kept as v1-only with `TODO(Phase 120)` markers per D-19 and the plan's Warning 3.
- Phase 115 closes green: fmt + clippy + `cargo test --all-features` all zero-exit, 7/7 ROADMAP success criteria confirmed (SC-6 verified at runtime with 42 passing schema_for_ tests, well above the ≥14 floor), no v1 type leaks outside the documented Phase 120 scanner quarantine.
- 23 atom renderers ported verbatim from v1 render.rs into the new (el, spec, data, depth) walker signature, with D-12 prop-decode diagnostics, D-15/D-16 action URL handling, and XSS regression coverage.
- Ported all 9 container renderer bodies (Card, Modal, Tabs, KanbanBoard, PageHeader, Grid, Collapsible, FormSection, ButtonGroup) verbatim from v1 render.rs into the Phase 116 walker, routing child rendering through super::render_element for ID-keyed lookup per CONTEXT D-05/D-06.
- 1. [Rule 1 — Plan wrong about value precedence] Input/Select default_value > data_path.
- 1. [Rule 1 - Bug] Suppressed dead_code warning on Catalog struct fields
- `ferro-json-ui/src/catalog.rs`
- `ferro-json-ui/src/catalog.rs`
- `ferro-json-ui/src/catalog.rs`
- 1. [Rule 1 - Bug] Switched all new tests from `Catalog::build()` to `Catalog::build_builtins_only()`
- `Catalog::prompt()`
- Typed meaning→component dispatch with catalog drift-guard and intent-layout baseline for the Spec::from_service_def pipeline.
- `Spec::from_service_def` realized — every output spec is validated against the catalog by construction (ROADMAP success criteria 1, 3, 4).
- Legacy v1 mapping files deleted; JsonUiRenderer::render is a one-line delegate; ProjectionError exposed at crate root; full workspace quality gate is green.
- Location:
- `make_json_view.rs`
- 1. [Rule 2 - Missing critical functionality] Updated generation_context.rs v1 reference
- `JsonUiCatalog` struct additions
- ai.rs additions:
- getting-started.md
- components.md
- One-liner:
- expressions.md
- v2 JSON spec dashboard + data-only handler prove the render_file pipeline end-to-end in the sample app
- 1. [Rule 3 - Blocker] Added `#![allow(dead_code)]` to project.rs
- 1. [Rule 3 - Blocker] `deploy/mod.rs` re-exports trip `unused-imports` lint
- `ferro-cli/src/templates/files/docker/Dockerfile.tpl`
- `ferro-cli/src/main.rs`
- `ferro-cli/src/templates/files/do/app.yaml.tpl`
- Deleted (8 source files + golden suite):
- Stripped the Phase 122 docker_init/do_init commands, their renderers, and Dockerfile/app.yaml templates down to compile-only stubs so Plans 06 and 07 can rewrite them against the new SCOPE design from a clean slate.
- [Rule 3 - Blocker] Added `anyhow` dependency to ferro-cli.
- [Rule 3 - Blocker] Promoted `templates::docker` from private to `pub mod`.
- [Rule 3 - Blocker] Module ident `do_` mapped to file `do.rs` via `#[path]`.
- 1. LOC accounting variance — methodology, not regression
- ferro-cli side (public surface promotion):
- New tool module `ferro-mcp/src/tools/deploy_check.rs`:
- New tool module `ferro-mcp/src/tools/deploy_diff_env.rs`:
- New tool module `ferro-mcp/src/tools/runtime_requirements.rs`:
- Stable JSON schema emitter for `ferro generate-routes` consumed by ferro-mcp, with a documented additive-only contract and full serde round-trip test coverage.
- 1. [Rule 3 - Blocker] Replaced YAML-parse test with structural-anchor test
- `SLACK_WEBHOOK_URL` classifies as non-secret.
- Dockerfile.tpl:
- 1. [Rule 3 — Blocker] Renderer takes `AppYamlContext`, not `&Project`
- `docker:init`
- CheckCategory enum + category() default on DoctorCheck trait; single read_path_dep_version helper in deploy::mod replacing two private duplicates
- Two deploy-specific doctor checks (`copy_dirs_dockerignore_collision`, `ferro_version_skew`) plus `--deploy` filter flag on `ferro doctor`; all three checks categorized as `CheckCategory::Deploy` and registered in `default_checks()` at positions 7-8 of 11.
- Interactive `ferro deploy:init` command that writes `[package.metadata.ferro.deploy]` into root Cargo.toml via toml_edit, with --dry-run preview, --yes bypass, and Abort/Overwrite/Merge collision policy
- `deploy_check` MCP tool registered on FerroMcpService (shells out to `ferro doctor --deploy --json`); doctor.md updated with all Phase 128 surfaces: 11-check table, `--deploy` filter, preflight check descriptions, `ferro deploy:init` section.
- Library-change gate in check-version job: non-library pushes (ferro-cli, docs, CI config, planning) set should_publish=none and skip all downstream jobs
- Deleted (3 files, net -1038 lines across both commits):
- Gap 1: Comment header
- Wave 0 finding confirmed:
- S3Driver struct
- VisualContext replaces RenderContext in ferro-mcp and framework re-exports, enabling full workspace compilation after the Renderer trait refactor in Plan 01.
- JsonUiRenderer, VisualContext, RenderMode, field_map, and relationship_map relocated from ferro-projections into ferro-json-ui behind a `projections` feature flag, establishing the output-crate ownership pattern
- ferro-projections stripped to renderer-trait-only: visual feature removed, three relocated files deleted, ferro-mcp and framework re-exports updated to ferro-json-ui.
- Task 1 — DataType::from_column_type()
- 1. [Rule 1 - Bug] Clippy uninlined format args
- dashmap = "6" added to ferro-stripe/Cargo.toml
- Mode enum
- 1. [Rule 1 - Bug] testing.rs imported deleted SubscriptionInfo/SubscriptionStatus
- 1. [Rule 3 - Blocker] framework/Cargo.toml had `ferro-stripe = "^0.2"` dependency constraint
- 1. [Rule 1 - Bug] Invoice.id is String, not Option<String>
- 1. [Rule 1 - Bug] mock_checkout_completed_event missing required CheckoutSession fields
- 1. [Rule 1 - Bug] async-stripe 0.41 requires more fields than RESEARCH Pattern 4 documented
- 1. [Rule 1 - Bug] ferro_queue::Error::JobFailed is a struct variant, not tuple
- Commit:
- Pure-string path-combination helper centralizing group-prefix + route-path semantics for both group implementations, with 8-row D-09 matrix test.
- Five `pub(crate)` alias methods on Router plus full GroupDef::register_with_inherited reshape using the Plan 01 helper — fixes `get!("/", h)` inside a group reaching the handler at both `/prefix` and `/prefix/`.
- Builder-API `Router::group()` now uses the Plan 01 helper to register canonical + optional alternate path pairs, matching the macro-based `group!()` behavior fixed in Plan 02 — restoring the D-11 lockstep invariant.
- Five serial integration tests proving D-07/D-10 RouteInfo deduplication, T-144-12 Strategy A middleware coverage, gestiscilo URL-shape routing, and Pitfall 6 regression guard against the public ferro API.
- Stable pure-function contracts (render_banner, classify_key(KeyCode, KeyModifiers), format_trigger_source, should_spawn_keyboard), 7-test inline skeleton with exact spec-banner literal oracle, 4-test integration scaffold, and a standalone-buildable minimal-serve fixture — all landing before any BackendSupervisor code is written.
- Removed the external `cargo-watch` dependency from `ferro serve`, added the `--watch` opt-in flag on the clap surface, extracted `spawn_child_with_prefix` as a shared piping helper, and filled four pure-helper bodies (`render_banner` / `classify_key` / `format_trigger_source` / `should_spawn_keyboard`) against the spec-verbatim banner oracle — four unit tests un-ignored and passing.
- `ferro serve` now runs an in-process BackendSupervisor that owns the backend `cargo run` child exclusively — kill/regenerate-types/respawn on every trigger (r key or debounced file save), shutdown ordering enforced via explicit JoinHandles, and all seven inline unit tests un-ignored and passing on every commit.
- 1. [Rule 1 - Bug] Fixed missing `compact` field in 8 SwitchProps struct initializers
- Wave-0 RED scaffold: 29 unit tests asserting DetailForm contract (EditMode parsing, DetailFormProps serde, render HTML invariants, resolver participation) + ferro-mcp catalog exhaustive-list bump 39→41 including KeyValueEditor backfill from Phase 146.
- Gate design per 147-02-PLAN.md `<wave_1_green_expectation>`:
- 1. [Rule 2 — Accessibility] aria-label wrapper inside Edit-mode `<dd>`
- Locked-signature skeleton types for WhatsApp / InApp / Sms / Push channels in ferro-notifications, wired through channels/mod.rs and the crate's top-level re-exports — downstream plans 02-07 now compile their tests and adapter code against fixed contracts.
- Public surface of `ferro-notifications` extended with `Channel::WhatsApp` + `Channel::InApp` (with explicit serde renames closing the lowercase-rule trap), four default-None `Notification` trait methods (D-02 + ARCH-FINDING-03), and three new `Error` variants (`WhatsApp(#[from])`, `Broadcast(String)`, `AttachmentTooLarge {..}`) — plans 03-06 now compile their dispatcher and adapter logic against locked types.
- MailMessage gains `attachments: Vec<MailAttachment>` and a fallible `attachment()` builder enforcing the 25 MB per-attachment cap from CONTEXT.md D-11 — Plan 04's SMTP multipart and Resend base64 emitters now have the typed payload to consume.
- Both mail drivers now ship MailMessage attachment support — SMTP via `MultiPart::mixed` + per-part `Attachment::new` + `ContentType::parse` fault-tolerance, Resend via a new `ResendAttachment` struct base64-encoding bytes through the standard alphabet — closing CONTEXT.md D-12 in one wave with zero regression on the no-attachment path.
- `Channel::WhatsApp` dispatch is end-to-end functional and gated: `NotificationConfig::whatsapp_enabled` (default `false`, env-driven via `WHATSAPP_ENABLED`) controls a `send_whatsapp` adapter that calls `ferro_whatsapp::WhatsApp::send` directly through the static facade — no client injection, no panic risk for default configurations, full propagation of `ferro_whatsapp::Error` via the `#[from]` chain landed in Plan 02.
- `Channel::InApp` dispatch is end-to-end functional: `NotificationConfig::in_app: Option<InAppConfig>` (combining `Arc<Broadcaster>` and `Arc<dyn DatabaseNotificationStore>`) gates `send_in_app`, which writes the DB-store leg first and broadcasts to `user.{id}` second — either failure bubbles up. `send_database` now routes through `DatabaseNotificationStore::store(...)` when `database_store` is configured, closing ARCH-FINDING-02 while preserving the placeholder log path for the unconfigured (backward-compat) case.
- Phase 149 ships: every new public type from plans 01-06 (`WhatsAppMessage`, `InAppMessage`, `InAppSeverity`, `MailAttachment`, `InAppConfig`, `SmsMessage`, `PushMessage`) is re-exported from both `ferro_notifications` and the framework crate (with `WhatsAppRawMessage` rename resolving the cross-crate name collision); `ferro-notifications` moves to publish Wave 1b (ARCH-FINDING-05 closed); ROADMAP success criterion #3 reflects D-04 static-facade reality (ARCH-FINDING-01 closed); consumer docs cover all three new surfaces with end-to-end usage examples; and a default-skip Mailpit-backed SMTP integration test verifies binary attachment round-trip on demand. Final workspace `cargo fmt + clippy + test --all-features` all green — phase is publish-ready.
- 1. [Rule 1 - Style] cargo fmt applied as separate commit
- 1. [Rule 2 - Dead Code] Added `#[allow(dead_code)]` to four pub(crate) constants
- 1. [Rule 3 - Blocking] resolve.rs needed RichTextEditor match arms
- 1. [Rule 1 - Style] mod declaration alphabetical order
- ferro-orm crate scaffolded as a Wave 1a leaf with GuardedError complete, GuardedUpdate forward-declared, and targeted SeaORM re-exports establishing the compile boundary for plans 02-06.
- ferro-orm registered in publish.yml Wave 1a and CLAUDE.md Workspace Structure table; root Cargo.toml entry was already in place from plan 01's Rule 3 deviation (idempotent no-op, verified).
- Race-free atomic conditional UPDATE primitive landed: GuardedUpdate<E> builder body with chainable filter/set/exec surface, EmptyUpdate guard against the sea-orm is_noop() short-circuit, and seven D-16 regression tests that lock the rows-affected → GuardedError mapping forever.
- Integration test (`ten_tasks_against_capacity_three_exactly_three_succeed`) empirically demonstrates the GuardedUpdate race-free claim under real SQL-level contention — 10 tokio tasks vs counter K=3, exactly 3 Ok(()) + 7 NoRowsAffected, final quantity 0.
- User-facing mdBook page for GuardedUpdate — walks the reader from the read → check → write anti-pattern through the builder API to the atomicity-per-statement contract, registered in the # Features sidebar.
- Workspace pre-release gate green; `## ferro-orm` section opened in CHANGELOG at `[0.2.30] — 2026-05-13`; first publish to crates.io awaits a local-terminal bootstrap with a `publish-new`-scoped token (RESEARCH Pitfall 5).
- Rule:
- Cargo.toml
- Chainable `AuditEntry::record(action).actor(…).target(…).write(&conn)` builder with mandatory post-INSERT UUID re-fetch for DB-stamped `created_at`, and 5 unit tests proving the write path end-to-end.
- One-liner:
- One-liner:
- 1. [Rule 3 - Blocking] Added ferro-reservation to Cargo.toml [workspace.members] in plan 01
- Workspace version bumped 0.2.31 → 0.2.32; ferro-reservation registered in WAVE1B_CRATES, CLAUDE.md table, and README.md crates list — cargo build -p ferro-reservation now works without --manifest-path workaround
- 1. [Rule 1 - Bug] Missing JsonValue import in entity.rs
- Resource trait, ReservationContext builder, ReservationEvent+ReleaseReason serde+Event impl, and ReservationHandle serde tests — the four pure-Rust foundation types plan 05 (kernel) will compose
- ReservationKernel<R> with four race-free state-transition methods composing GuardedUpdate + AuditEntry + ferro_events, 8 unit tests green including audit smoke test
- Full `run_sweep_once` body on `ReservationKernel<R>` + 3 test files proving the v11.11 correctness claim (D-48 concurrent invariant, D-49 property tests, D-50 cross-crate showcase) — 33 tests total, all green
- ferro-reservation v0.2.32 published to crates.io — race-free resource reservation kernel composing GuardedUpdate + AuditEntry + domain events
- 1. [Rule 3 - Blocking] Added ferro-projection to workspace Cargo.toml in plan 01
- 1. [No-op] Workspace member registration already done by Plan 01
- `ferro-projection/src/migration.rs`
- key::tests (3):
- 1. [Rule 1 - Bug] Removed unused imports (`self`, `ActiveModelTrait`)
- 1. [Rule 1 - Bug] Clippy uninlined_format_args in test files
- ferro-projection v0.2.33 documentation and CHANGELOG complete — first-publish pending operator manual action (Task 5)
- Reference app generated TS files untracked from git, gitignore.tpl annotated as load-bearing, and generate_types.rs header corrected to point to frontend/src/lib/types/
- One-liner:
- `DockerContext.ferro_version: String`
- `docker_init.rs` call site wired
- One-liner:
- One-liner:
- One-liner:
- `req.file("avatar").await?` and `req.multipart().await?` wired as consuming Request methods backed by 13 passing unit tests covering every behavior in the CONTEXT.md spec.
- One-liner:
- File:
- Rewrote 30+ doc-comment sites across `render/{mod,atoms,containers,form,data}.rs`, `projection/builder.rs`, and `layout.rs` in present-tense voice, eliminating every `v1`, `Port of`, `Differences from v1`, `Phase 116`, `Per CONTEXT D-XX`, and `render.rs line-range` framing while preserving all documented behavior; deleted dead `_plan_02_reserved` placeholder.
- ferro-mcp `code_templates` tool no longer advertises a v1→v2 migration category; 230 lines (registration + 7-template function + integration test) removed in a single coordinated diff.
- `test_ignores_non_json_files` fixture renamed in-place from v1-coded names (`old_view.rs`, `// old v1 file`, `pub mod old;`) to neutral identifiers (`stale_artifact.rs`, `// non-JSON artifact`, `pub mod stale_artifact;`); scanner-ignores-non-JSON behavior assertion preserved unchanged.
- Replaced the v1 JsonUiView/LayoutComponent README example with the v2 surface (Spec::builder + JsonUi::render_file), clearing the Phase 161 crates.io publish blocker.
- Quick Start example in docs/src/features/projections.md rewritten to mirror ferro-json-ui/src/projection/mod.rs:79-97 — VisualContext replaces RenderContext, spec.schema/spec.elements replace json["$schema"]/json["components"], and the v1 schema string is gone.
- Verification gate closes Phase 160: all D-10 grep gates clean, ferro workspace 2697/2697 tests green, gestiscilo build green and 530/538 tests green (the 8 failures triaged as gestiscilo-internal regressions unrelated to ferro), ferro-code descope recorded per OQ-2, no publish performed per D-11.
- CheckboxListProps struct + render_checkbox_list with data-driven options and XSS-safe HTML, registered in BUILTIN_TYPES=40 and catalog
- One-liner:
- Re-added two blast-radius props (D-16/D-17): SwitchProps.compact emits scale-75/origin-left CSS, ImageProps.inline_svg emits verbatim SVG in an aria-labelled div with no img tag
- 1. [Rule 1 - Bug] BUILTIN_SPECS catalog entry would break drift guard
- AuthLayout card wrapper removed: layout is now structural only (centering + max-width), specs must declare their own Card root (D-05)
- One-liner:
- 1. [Rule 2 - Missing derives] Added Clone + PartialEq to RouteInfo
- Migration guide (493 lines, 7 sections), plugins.md RichTextEditor + catalog docs, components.md v1→v2 banner + Card+children example + inline view/edit pattern, and 7 migration_v1_to_v2 code templates in ferro-mcp
- Adds `EachDirective` struct and `Element.each: Option<EachDirective>` to `ferro-json-ui::spec`, enabling JSON specs to carry `"$each": { "path": "/orders", "as": "order" }` through serde round-trip; resolver expansion is the next step.
- Adds `Element.if_: Option<Visibility>` to `ferro-json-ui::spec`, enabling JSON specs to carry `"$if": { "path": "/can_advance", "operator": "eq", "value": true }` (or compound `and`/`or`/`not` forms) through serde round-trip; resolve-time deletion is the next step (Plan 03).
- Ships the killer feature for Phase 163: a single resolve-time pass that materializes `$each` directives into N concrete elements with auto-suffixed IDs and removes `$if`-falsy elements from the spec map entirely. Wired BEFORE `resolve_actions` / `resolve_expressions` in `JsonUi::resolve` so all downstream resolution operates on the expanded element set. The wire-format types from Plans 01 + 02 are no longer inert — they now do something.
- 1. [Rule 1 - Bug] Fixed incorrect Visibility/Action type names in test 6
- Four-quadrant decision rubric for spec construction (Static / `$each` / `$if` / `SpecBuilder`) and `$each` / `$if` directive reference in expressions.md, locking the `eq` operator name and the element-level vs prop-level namespace split.
- One-liner:
- One-liner:
- One-liner:
- `ferro-cli/tests/fixtures/migrate_v1/in_all_verbs.rs`
- Task 1 (data_path on Image and DescriptionList):
- 1. [Rule 1 - Bug] projection/builder.rs missing variant field
- Task 1 (KanbanBoardProps serde):
- Alert.variant="" gated by `visible` no longer blocks server startup: load_cached downgrades catalog validation to tracing::warn; per-request resolve() enforces after expand_directives via tracing::error + continue
- One-liner:
- 1. [Rule 1 - Bug] Stale depth-3 mention in migration guide
- File:
- One-liner:
- 1. [Rule 1 - Dead Code] Removed is_permanent_error / is_transient_error from classifier/anthropic.rs
- 1. [Rule 2 - Missing critical functionality] `Message` import placement
- Task 1 — AiConfig::from_env() factory + lib.rs re-exports (SC#3, SC#6)
- Verified schema shapes for Plan 02/03 closing algorithm:
- Clean seam for Plan 03:
- Two schemars anyOf shapes handled:
- complete_with_tools as a separate trait method (D-14):
- WAVE1B reorder, no wave promotion (Task 1):
- `ferro_ai::cosine_similarity(a, b)`
- `ferro_ai::pgvector::PgVectorStore`
- `framework/src/http/body.rs`
- `framework/src/http/sse.rs`
- 1. [Rule 1 - Bug] rustfmt formatting diff
- 1. [Rule 1 - Bug] catalog.rs count assertions still at 44
- `ferro ai:make <description>` ships: loads ferro-mcp introspection in-process, lexically filters to description-relevant context, produces a typed ServiceDef via `complete_with::<ServiceDef>()`, and writes exactly one `src/projections/<name>.rs` builder file — the produce half of the milestone killer feature
- `ferro ai:explain <target>` ships: resolves any route/model/service in-process, builds a projection-framed prompt from inspect_projection's parsed vocabulary when a ServiceDef projection exists, and produces prose via a raw schema:None LLM call — the consume half of the milestone killer feature
- Task 2: Human-verify live ai:make and ai:explain quality (SC#6, SC#4)
- One-liner:
- One-liner:
- One-liner:
- One-liner:
- 1. [Rule 1 - Bug] Unused import `ferro_json_ui::global_catalog`
- 1. [Rule 1 - Bug] rustfmt formatting in projection_roundtrip.rs
- CardProps extended with `badge: Option<String>` and `subtitle: Option<String>` slots — render_card emits Badge-styled span in flex title-row for badge and muted paragraph for subtitle, both html_escaped; eleven new tests pin slot semantics and serde round-trips
- F9 closes as Outcome A (no-repro): three Grid visibility regression tests pass green on first run against current ferro master; visibility evaluator architecture is correct; consumer chip-strip Grid should render correctly when rebuilt against patched ferro runtime
- Commit:
- Commit:
- Commit:
- No clash.
- Decision: raise `handle_action_result` to `pub #[doc(hidden)]`.
- 1. [Rule 2 - Missing critical functionality] Cross-link added to controllers.md instead of handlers.md
- Added new test
- 1. [Rule 1 - Bug] cargo fmt failures from Wave 2 commits
- 1. [Rule 1 - Bug] Incorrect `redirect_back` signature in plan template
- 1. [Rule 1 - Bug] `runtime_contains_lazy_hero_setup` assertion literal mismatch
- 1. [Rule 1 - Bug] Plan's verbatim content did not satisfy its own acceptance criteria for `data-lazy-hero-margin` count
- New `ferro-bundle` workspace crate scaffolded with locked deps + Cargo.toml + lib.rs stub + bundle-vs-filesystem README; workspace bumped 0.2.42 -> 0.2.43; publish.yml Wave 3 renamed to framework-consumers and converted to a `ferro-cli ferro-bundle` for-loop.
- Bundle struct + two OnceLock<DashMap<...>> registries + pub(crate) serve_inner dispatcher implementing content-hashed URLs, ETag-quoted 304 fast-path, and 301 alias redirects; 5 unit tests pin SHA-256 determinism and registration semantics.
- Three single-test binaries under `ferro-bundle/tests/` verify BUNDLE-02 (cold 200 + 304 fast-path) and BUNDLE-03 (alias 301 redirect) against the Plan 02 dispatcher, reached through a new `#[doc(hidden)] pub mod __test_internals` wrapper that bridges integration-test binaries to the crate-private `serve_inner`.
- Workspace-wide lint + test gate green; `cargo publish -p ferro-bundle --dry-run` exit 0; real publish to crates.io deferred to user (per explicit instruction); manual bootstrap command + recovery procedure + new-crate runbook recorded for the post-merge action.
- Telemetry module foundation — Sample/Decision/InlineBudgetState types, OnceLock<DashMap> process-global ring buffer (cap 128), AppConfig inline_budget_threshold_bytes field, crate-root re-exports of Decision/RequestTelemetry/Sample (InlineBudget intentionally hidden).
- Inline-budget state machine + Request integration — record_and_decide pure method, decide(req) thin wrapper with borrow-safe ordering, three Request methods (inline_budget / telemetry_record / telemetry_record_scoped). Pre-commit gate green.
- Closes Phase 184: integration smoke test exercising both Phase 184 primitives via a real Request, public docs page for InlineBudget + RequestTelemetry under 'The Basics', workspace bump 0.2.43 to 0.2.44, ship-gate green (pre-commit + publish dry-run + cargo doc).
- 1. [Rule 3 - Blocking] Removed Queue/worker from lib.rs to allow compilation
- 1. [Rule 1 - Bug] Reaper park step used wrong placeholder index
- DB-backed WorkerLoop with reaper/claim/spawn cycle, catch_unwind panic isolation, SIGTERM graceful shutdown with drain+requeue, and dispatcher wired to db::enqueue — ferro-queue is now fully Redis-free.
- Namespaced `ferro::queue` module, WorkerLoop auto-start in `Application::run`, DB-backed debug endpoints, and corrected ferro-mcp failed-jobs query — the queue now "just works" with a single binary.
- SC-1 SQLite race test proves two concurrent workers claim N=20 jobs exactly once on a shared NamedTempFile database; SC-4b shutdown test proves requeue_claimed_by resets claimed rows; docs rewritten for DB backend with spawn_blocking guidance and old→new migration table.
- 1. [Rule 3 - Blocking] Stub modules required for Task 1 build
- 1. [Rule 1 - Bug] promote_rejects_deleted_artifact test needed conn.clone()
- 1. [Rule 1 - Bug] ferro-storage Memory driver double-slash breaks files() and delete_directory()
- 1. [Rule 1 - Bug] DeploymentStorage trait not imported in doc-test hidden section
- `swc = "66.0.0"`
- 1. [Rule 3 - Blocking] lol_html text!() replace() requires ContentType argument
- 1. [Rule 1 - Bug] ravif with_quality takes f32 not f64
- 1. [Rule 1 - Bug] AVIF decode via image::load_from_memory fails under --all-features
- 1. [Rule 1 - Bug] Partial-move compiler error in create()
- `StripePaymentIntentAmountCapturableUpdated`
- Task 1:
- Task 1:
- Task 1:
- Task 1:
- 1. [Rule 3 - Blocking] `sqlx::Error` not available as top-level name
- SQLite integration suite proves the full ConstraintMap defensive-layer contract: TOCTOU simulation, message-parse identity, and passthrough, all against real in-memory SQLite driver errors.
- One-liner:
- One-liner:
- 1. [Rule 1 - Bug] Updated framework/Cargo.toml ferro-stripe version pin
- 1. [Rule 2 - Missing critical functionality] Implemented seam 2 and aggregation logic in Task 1
- 1. [Rule 1 - Bug] DataType::Text is unknown; fixtures using it produced Warn instead of intended Fail/Pass
- Task 1 — aggregate_status (4 tests):
- Task 1 — Async conversion + seam-name reconciliation (source, tests, docs)
- 1. [Rule 1 - Bug] Non-exhaustive match in action_to_route_seam
- Files exist:
- 1. [Rule 2 - Missing critical cleanup] Removed `#[allow(dead_code)]` from `read_ambient_status`
- One-liner:
- `ferro-mcp/src/service.rs`
- 1. [Rule 1 - Bug] rmcp feature set adjusted from `["schemars"]` to `["server", "macros", "base64"]`
- 1. [Rule 3 - Blocking] ferro_projections::field module is private
- One-time manual bootstrap publish:
- One-liner:
- One-liner:
- 1. [Rule 2 - Dead code warning] Suppressed `sanitized_app_url` dead_code for stub phase
- 1. [Rule 3 - Blocking] ferro dependency must be aliased as 'ferro' for #[handler] macro
- 1. [Rule 1 - Bug] Redundant guard pattern rejected by clippy -D warnings
- 1. [Rule 1 - Bug] Test assertion used wrong string for HTML doctype check
- 1. [Rule 1 - Bug] Task 1 and Task 2 could not be committed separately
- 1. [Rule 3 - Blocking] Updated existing integration test call sites for new dispatch signature
- BearerAuthMiddleware
- Inline bearer validation removed.
- Bidirectional two-tenant isolation (SC-1) and JwtClaimResolver middleware-chain parity (SC-3) proven by three integration tests against a real in-memory SQLite fixture with seeded acme/globex tenants.
- Status:
- 1. [Rule 2 - Missing critical functionality] Added `with_test_session` to framework
- 1. [Rule 2 - Missing dependency] Added tracing as direct app dep
- login_confirm.json corrected: dev_link Button visibility and navigation moved from unsupported Button props to element-level visible (is_true condition) + action (ActionHandler::Binding) so the verify URL is never in production HTML.
- 1. [Rule 1 - Bug] Removed useless `.into()` on `Cache::get` return value
- One-liner:
- One-liner:
- One-liner:
- One-liner:
- One-liner:
- One-liner:
- `ferro-storage/src/error.rs`
- `ferro-storage/src/config.rs`
- `Cargo.toml` (workspace root)
- Replaced bare-row `content` array in `handle_tools_call` with `CallToolResult::structured(payload)`, yielding a valid MCP envelope (one `type:text` content block + `structuredContent` + `isError:false`) that strict MCP clients parse without Zod errors.
- Plan:
- `ferro-projections/tests/catalog.rs`
- 1. [Rule 3 - Blocking] voice.rs and mobile.rs created before Task 1 commit
- 1. [Rule 1 - Bug] Intent table used bold formatting, failing the plan's acceptance criterion grep
- Task 1:
- Dev-dependency delta (`ferro-mcp/Cargo.toml`):
- `TierResult` struct:
- Task 1 — In-process rmcp transport + tool dispatch:
- One-liner:
- One-liner:
- One-liner:
- One-liner:
- One-liner:
- `emit_actions_placeholder` (Focus / Process / Track — `actions` slot):
- `StatCardProps.value_path: Option<String>`
- `ColumnFormat::Image` added; ImageUrl fields render as html-escaped `<img>` thumbnails in DataTable columns instead of being excluded
- One-liner:
- Found during:
- 1. [Rule 1 - Bug] 51 compile errors in scaffolded app detected by the test
- File:
- File:
- 1. [Rule 1 - Bug] Test imports used private ferro-projections module paths
- 1. [Rule 1 - Bug] rustfmt drift in ferro-text/src/lib.rs from Plan 02

---

## v12.7 Passwordless MCP Auth (Shipped: 2026-06-12)

**Phases completed:** 2 phases (202–203), 10 plans

**Delivered:** Passwordless and cross-device authentication for the consumer-app MCP
surface. A magic-link (async) ferro app now completes the OAuth/MCP browser-login flow
by resuming the in-flight authorize request, and `ferro-mcp-oauth` gains the OAuth 2.0
Device Authorization Grant (RFC 8628) for passwordless, cross-device, and headless/CLI
MCP clients — both reusing the v12.6 consent + tenant-scoping surfaces and the single
existing token issuer (no second token path).

**Key accomplishments:**

- Phase 202 — Login-resume contract + magic-link sample app: a documented helper
  (`oauth_resume_redirect` / `take_oauth_return_to`) any login handler calls to obtain
  the post-login redirect target from the session `oauth_return_to`; the bundled sample
  app login converted from password to magic-link as the golden-path exemplar, with an
  async-flow acceptance test (unauthenticated `/authorize` → login → verify → resume →
  consent). Verified 5/5.

- Phase 203 — OAuth Device Authorization Grant (RFC 8628): `device_authorization`
  endpoint returning RFC-8628 §3.2 fields, a user-code verification page bound to the
  existing consent + `(user, tenant)` scoping (login-resume reused), and the §3.5
  device-code token polling state machine whose Approved arm mints through the same
  `build_claims` + `mint_token` path as the authorization-code arm. Discovery advertises
  `device_authorization_endpoint` + the device-code grant. Verified 5/5 (13-test SC-5 matrix).

**Verification:** both phases passed 5/5. Milestone audit `v12.7-MILESTONE-AUDIT.md`:
status tech_debt (no blockers) — cross-phase integration 8/8 wired, single-token-issuer
invariant held. Deferred edge cases: WR-02/WR-03 (resume not triggered for an
already-authenticated tab clicking a magic-link mid-flow, or for `POST /auth/register`).

## v12.5 Projection Checkpoint (Shipped: 2026-06-10)

**Phases completed:** 3 phases (194–196), 11 plans

**Delivered:** An agent-facing write→verify loop for projections. The
`checkpoint_projection` MCP tool walks a five-seam spine, owns the
projection-field→model-column seam (the one check no existing validator covered),
delegates the other seams to existing validators, and returns a single
`pass`/`warn`/`fail` verdict with ranked next steps — honest about coverage,
closing by default after generation.

**Key accomplishments:**

- Phase 194 — Core checkpoint tool: structured verdict (per-seam results + ranked,
  deduplicated `next_steps`); the field→column seam resolves projection→model and
  flags dangling fields; `not_checked` is a distinct status never coerced to `pass`
  (coverage honesty).

- Phase 195 — Close the loop by default: wrapper seams 1/3/4/5 dispatch to existing
  validators (no logic reimplemented; each finding names its source);
  `generate_projection`/`json_ui_generate` embed the verdict inline;
  `application_info`/`projection_coverage` surface per-projection checkpoint status
  from the cache.

- Phase 196 — Dogfood acceptance + hardening: a deliberately-poisoned synthetic
  fixture proves the field→column seam; the in-repo `app/` live consumer produced
  20 findings (seam 3 `action_to_route` the genuine driver — unregistered actions);
  `next_steps` capped 10→5; the one zero-finding wrapper seam (`props_to_contract`)
  demoted to `not_checked`-by-default and documented.

**Requirements:** CHK-01 … CHK-10 all complete.

**Acceptance:** GO — the checkpoint surfaced a real seam defect in a real project
(recorded in `196-ACCEPTANCE.md`).

---

## v12.4 Form Validation DX (Shipped: 2026-06-09)

**Phases 190-192** (4+2+2 plans). Async DB-backed uniqueness validation as a first-class ferro form primitive, two layers:

- **190 — proactive:** `AsyncRule` trait + `AsyncValidator`/`validate_async` + `unique(table, col)` with `.ignore(id)` exclude-self; fails before the write with a field-level error via the existing `ValidationError` → redirect-back flow. SQLite + Postgres (live-PG gate test).
- **191 — defensive:** `ConstraintMap` + `try_map` + `MapConstraintExt::map_constraint` — maps a DB UNIQUE violation at the write site to the same field error, closing the TOCTOU race; portable detection (`sql_err()` + Postgres `constraint()` / SQLite message parse); unmatched `DbErr` falls through unchanged. Live-PG gate test.
- **192 — surface:** ferro-mcp `action_handler` template + `validation.md` show both layers together (no surface shows one without the other).

All requirements VALID-01..06 Complete. Both live-Postgres manual gates closed via `#[ignore]`d tests.

## v12.1 AI — ferro-ai SDK & AI as Projection Consumer (Shipped: 2026-06-09)

**Phases 165-173** (9 phases). AI as a first-class consumer of the projection/intent core: `LlmClient` trait + providers (165), structured outputs + ServiceDef-aware schema normalizer + tool calling (166), embeddings + pgvector (167), framework SSE primitives (168), `StreamText` component (169), ferro-cli SDK migration (170), `ai:make`/`ai:explain` killer-feature CLI commands producing typed `ServiceDef` (171), MCP tool wrappers (172). **Capstone (173):** `make:json-view` consumes a `ServiceDef` via the existing `Spec::from_service_def` renderer + the offline projection-roundtrip proof test (NL → ServiceDef → rendered JSON-UI, pinned to the ServiceDef-aware path via the `Money → currency` assertion) — the structural proof that AI feeds the projection core, not a parallel scaffolder.

## v11.6.2 ferro-stripe Refund Event Completeness + 0.7.0 Release (Code complete: 2026-06-09)

**Phase 193** (1 plan). Adds `StripeChargeRefunded::refund_id: Option<String>` parsed from the charge's refunds list (`charge.refunds.data[].id` — corrected from the roadmap's mistaken `EventObject::Refund`); golden-JSON fixture + parser-contract test; ferro-stripe `0.5.0 → 0.7.0` + CHANGELOG bundling the Phase 189 manual-capture work. **Publish pending:** the 0.7.0 crates.io release fires on the operator's `git push` (GH Actions), which unblocks gestiscilo Phase 99. Requirements STRIPE-REFUND-01/02 (code) complete.

## v12.0 JSON-UI v2 — Spec-Driven Rendering (Shipped: 2026-05-19)

**Phases completed:** 115-121, 159-164 (incl. friction loop with gestiscilo)

**Key accomplishments:**

- Spec-driven rendering pipeline: `Spec` JSON is now the public wire contract; the renderer walks the spec, resolves expressions, applies visibility rules, and emits HTML deterministically
- Component catalog grew to 42 built-in components including DataTable, KanbanBoard, DetailPage, PageHeader, EmptyState, RichTextEditor, Calendar, NotificationDropdown, CheckboxList, RawHtml — each with structured props, JSON Schema, and catalog entries surfaced via `mcp__ferro__json_ui_catalog`
- JSON Schema contract for `Spec` with `json_ui_validate_spec` and `json_ui_verify_action` MCP tools; round-trip and reject test fixtures enforce the schema
- Expression engine: `{$data: "/path"}` bindings, `{$template: "..."}` interpolation, `$each` iteration, `IsTrue`/`IsFalse` visibility operators, `Action.handler` accepting `{$data}` bindings for per-row navigation
- Renderer ergonomics: `JsonUi::render_file`, back-aware redirects via `Redirect::back(&req, fallback)`, `Request::back_or(fallback)` with same-origin host enforcement, `scroll_preserve` runtime capturing `<main>.scrollTop`
- Visual polish: translucent backdrop-blur toasts with auto-dismiss, popover dropdowns, anchored Buttons, kanban with column caps and full-bleed cards, DataTable density (`px-4 py-2`), DetailPage shape
- v1 view/component materialization API fully removed (Phase 160)
- Production-validated via gestiscilo v7.0 integration loop — friction Phases 138-143 absorbed by ferro Phases 162-164

**Stats:** 491 commits, 13 phases (115-121, 159-164), single-publish cadence at merge

---

## v11.7 Tailwind Static CSS Pipeline (Shipped: 2026-04-20)

**Phases completed:** 1 phase (143), 4 plans

**Key accomplishments:**

- Pre-built `ferro-base.css` (36 KB) embedded at compile time via `include_str!`, eliminating the in-browser Tailwind JIT runtime that failed on Safari/WebKit
- Framework serves `/_ferro/ferro-base.css` automatically with `Cache-Control: public, max-age=86400`; CI drift check enforces the committed file stays in sync with Tailwind CLI output
- `JsonUiConfig::stylesheet_urls: Vec<String>` added (default `["/_ferro/ferro-base.css"]`); `tailwind_cdn` default flipped to `false`
- Theme injection migrated from `<style type="text/tailwindcss">` (Tailwind-CDN-specific magic MIME) to plain `<style>` with `:root { }` CSS variable overrides
- `ferro make:theme` scaffolder updated to emit plain CSS `:root { }` blocks instead of Tailwind `@theme { }` syntax

**Known deferred tech debt:** D-08 — no test for "app appends own token URL alongside ferro-base default" via `stylesheet_urls`; mechanism verified correct, coverage gap only.

---

## v11.1 Template Renderer (Shipped: 2026-04-07)

**Phases completed:** 1 phases, 1 plans, 2 tasks

**Key accomplishments:**

- TemplateRenderer struct implementing Renderer trait: produces intent-agnostic serde_json::Value context with fields (object), actions (array with inputs), and state_machine (object or null)

---

## v11.0 Framework Consolidation Audit (Shipped: 2026-04-07)

**Phases completed:** 7 phases, 13 plans, 14 tasks

**Key accomplishments:**

- 24 stale `ferro_rs::` import paths corrected to `ferro::` across multi-tenancy, actions, and data-binding docs
- CLI reference examples now use real logic (tracing + SeaORM patterns), S3 marked shipped, and README presents JSON-UI as a delivered feature with a corrected crate badge
- All 65 MCP tool descriptions audited — one doc bug fixed (CodeTemplatesParams missing 'api' category), three cross-references added for newer tools
- FerroModel and ValidateRules documented with complete worked examples on a dedicated derive-macros.md page linked between Database and Validation in SUMMARY.md
- introduction.md rewritten with agent-first identity and MCP callouts; new Working with Agents guide covers ferro-mcp setup for Claude Desktop, Claude Code, and generic stdio with discovery loop and agent-to-CLI workflow
- Standardized 22 documentation files to use explicit crate-root imports, #[handler] attributes, and ? / .expect() error propagation instead of glob imports and .unwrap()
- COMPONENT_CATALOG moved from two identical 100+ line local constants to a single pub const in ferro-json-ui, with ferro-cli and ferro-mcp importing it via direct dependency.

---

## --help --help (Shipped: 2026-04-07)

**Phases completed:** 7 phases, 13 plans, 14 tasks

**Key accomplishments:**

- 24 stale `ferro_rs::` import paths corrected to `ferro::` across multi-tenancy, actions, and data-binding docs
- CLI reference examples now use real logic (tracing + SeaORM patterns), S3 marked shipped, and README presents JSON-UI as a delivered feature with a corrected crate badge
- All 65 MCP tool descriptions audited — one doc bug fixed (CodeTemplatesParams missing 'api' category), three cross-references added for newer tools
- FerroModel and ValidateRules documented with complete worked examples on a dedicated derive-macros.md page linked between Database and Validation in SUMMARY.md
- introduction.md rewritten with agent-first identity and MCP callouts; new Working with Agents guide covers ferro-mcp setup for Claude Desktop, Claude Code, and generic stdio with discovery loop and agent-to-CLI workflow
- Standardized 22 documentation files to use explicit crate-root imports, #[handler] attributes, and ? / .expect() error propagation instead of glob imports and .unwrap()
- COMPONENT_CATALOG moved from two identical 100+ line local constants to a single pub const in ferro-json-ui, with ferro-cli and ferro-mcp importing it via direct dependency.

---

## v10.0 JSON-UI Visual Overhaul (Shipped: 2026-03-26)

**Delivered:** Professional visual quality uplift across all JSON-UI components — Inter font, surface elevation, typography scale, form polish, interactive states, and SVG icon refinements.

**Phases completed:** 102-107 (8 plans total)

**Key accomplishments:**

- Inter Variable font loaded via Bunny Fonts CDN with correct Tailwind v4 --font-sans token namespace (Phase 102)
- Three-tier surface elevation hierarchy (background → surface → card) with WCAG 4.5:1 dark mode contrast verification (Phase 103)
- Typography scale: H1/H2 tight tracking, H3 snug, body relaxed line-height across all text elements (Phase 104)
- Form polish: inline SVG select chevron, destructive error focus rings, 150ms transitions with reduced-motion, disabled states (Phase 105)
- Focus-visible rings and hover states on all interactive elements (buttons, tabs, pagination, breadcrumbs, sidebar, table rows) (Phase 106)
- SVG icons for alerts/bell/breadcrumb/collapsible, CSS shimmer animation for skeleton, font-semibold active tabs (Phase 107)

**Stats:**

- 39 files changed (+6,847, -204 lines)
- 6 phases, 8 plans, 46 commits
- 2 days (2026-03-24 → 2026-03-26)

**Git range:** `9d906347` → `67d74d51`

**What's next:** Planning next milestone.

---

## v8.1 API DX Polish (Shipped: 2026-02-28)

**Delivered:** Closed the DX gaps between `ferro make:api` scaffold and a working MCP integration with five targeted improvements.

**Phases completed:** 83 (5 plans total)

**Key accomplishments:**

- `ferro make:api-key` CLI command generates API keys with SHA-256 hashing, SQL/Rust code snippets (8 tests)
- Route-level x-MCP builder API: .mcp_tool_name(), .mcp_description(), .mcp_hint(), .mcp_hidden() with group-level defaults (5 tests)
- Sensitive field auto-exclusion in make:api with --exclude/--include-all flags and 8 known patterns (8 tests)
- `ferro api:check` validates server connectivity, OpenAPI spec, and API key auth with actionable error messages (7 tests)
- Enhanced post-scaffold guidance with setup steps, MCP config snippets for Claude Desktop and Claude Code
- Complete API-to-MCP documentation: Quick Start Workflow and Route Customization guides

**Stats:**

- 20 files changed (+1,995, -78 lines)
- 1 phase, 5 plans, 10 tasks
- 1 day (2026-02-28)

**Git range:** `495edd9` → `7aae50e`

**What's next:** Planning next milestone.

---

## v6.0 ferro-lang — Localization (Shipped: 2026-02-13)

**Delivered:** Added localization infrastructure via new ferro-lang crate with JSON translations, per-request locale detection, validation message localization, CLI scaffolding, MCP introspection, and comprehensive test coverage.

**Phases completed:** 58-66 (11 plans total)

**Key accomplishments:**

- Created ferro-lang crate with JSON translation loading, :param interpolation, and pipe-separated pluralization with range syntax
- Per-request locale detection via task_local! with LangMiddleware (Accept-Language + query param)
- OnceLock-based validation bridge decoupling all 22 rules from ferro-lang with English fallback
- Framework integration with t()/trans()/choice() helpers auto-booted in Application::run()
- CLI scaffolding: make:lang command + ferro new templates with localization defaults
- MCP introspection (list_lang_files) + comprehensive documentation page (253 lines)

**Stats:**

- 69 files changed (+6,811, -85 lines)
- 9 phases, 11 plans, 48 commits
- 1 day (2026-02-13)

**Git range:** `d99fbcd` → `5073fc2`

**What's next:** Publish to crates.io and public announcement.

---

## v5.1 Housekeeping (Shipped: 2026-02-13)

**Delivered:** Resolved technical debt and improved project hygiene: fixed deployment templates, split oversized template files, updated env defaults, and audited concerns.

**Phases completed:** 54-57 (5 plans total)

**Key accomplishments:**

- Updated env.example.tpl to match all 63 framework env vars (removed 8 phantom, added 23 missing)
- Split templates/mod.rs from 2,987 to 831 lines across 7 focused modules
- Audited CONCERNS.md: resolved 6/8 items, rebuilt priority matrix to 4 remaining
- Fixed deployment templates: health check path, Rust image version, deployment tip text

**Stats:**

- 26 files changed (+4,821, -3,669 lines)
- 4 phases, 5 plans, 17 commits
- 1 day (2026-02-13)

**Git range:** `3f5e0e1` → `fa1375f`

**What's next:** v6.0 ferro-lang — Localization.

---

## v5.0 Proximity — JSON-UI Field Test (Shipped: 2026-02-10)

**Delivered:** Built a complete map-based social network app (app-proximity) as the first real-world validation of JSON-UI and v4.0 features, including a plugin system, geospatial queries, real-time presence, and end-to-end UI polish.

**Phases completed:** 47-53 (20 plans total)

**Key accomplishments:**

- JSON-UI plugin system with trait-based extensibility, global registry, and Map plugin with Leaflet rendering
- app-proximity workspace crate — complete social network with auth, geo profiles, location posts, and nearby feeds
- Geospatial proximity queries with bounding-box + Haversine filtering, nearby users map, and nearby posts feed
- Real-time presence via WebSocket broadcasting with channel authorization, presence data, and live location/post events
- UI polish with ProximityLayout navigation, Avatar/Badge/DescriptionList components, and relative timestamps
- JSON-UI field validation: discovered and fixed issues (Div/Section variants, SQLite Haversine in Rust, input step attribute)

**Stats:**

- 104 files changed (+11,900, -77 lines)
- 3,042 lines of Rust (app-proximity)
- 7 phases, 20 plans, 82 commits
- 1 day (2026-02-10)

**Git range:** `dbdb0f0` → `24fecfe`

**What's next:** Publish to crates.io and public announcement.

---

## v4.0 Production Readiness (Shipped: 2026-02-10)

**Delivered:** Authentication, API resources, rate limiting, real-time WebSocket broadcasting, and DX polish to make Ferro production-ready.

**Phases completed:** 38-46 (24 plans total)

**Key accomplishments:**

- Complete session-based authentication system with bcrypt hashing, Auth facade, AuthUser/OptionalUser extractors, middleware guards, and `ferro make:auth` CLI scaffolding
- Production-ready API Resources with derive macro, ResourceMap builder, pagination envelope, collection mapping, and batch-loaded relationship support
- Cache-backed rate limiting with RateLimiter::define() and Throttle middleware supporting named limiters, multiple limits per route, and fail-open behavior
- Real-time WebSocket broadcasting with upgrade handler, heartbeat/timeout, channel authorization, and whisper support for client-to-client messaging
- Enhanced DX with actionable error hints, comprehensive MCP introspection (list_resources, list_policies, list_rate_limiters, list_broadcast_channels), and v4.0 feature documentation
- Stabilized foundation: fixed flaky tests, replaced S3 driver panics, removed CDN dependencies, added 100+ unit tests across all new features

**Stats:**

- 128 files changed (+16,105, -878 lines)
- ~80,900 lines of Rust (total codebase)
- 9 phases, 24 plans
- 2 days (2026-02-09 → 2026-02-10)

**Git range:** `94c73c1` → `a9dcd8a`

**What's next:** Publish to crates.io and public announcement.

---

## v3.0 JSON-UI (Shipped: 2026-02-09)

**Delivered:** JSON-based UI rendering system as an alternative to Inertia, enabling rapid UI without frontend builds.

**Phases completed:** 23-32 (24 plans total)

**Key accomplishments:**

- Created ferro-json-ui crate with 20-component catalog (Card, Table, Form, Modal, Tabs, etc.) using serde-tagged enums and shadcn/ui-aligned variants
- Built complete Rust HTML renderer with Tailwind CSS output, XSS prevention, and progressive enhancement (no-JS modals, SSR tabs)
- Integrated data binding with slash-separated JSON paths, 11 visibility operators with And/Or/Not composition, and automatic validation error propagation
- Implemented action system with builder API, callback-based URL resolution, and confirmation/outcome chaining
- Added layout system with trait-based registry, 3 default layouts (Default/App/Auth), and composable partial functions
- Built AI-powered `ferro make:json-view` CLI command with Anthropic API and 3 MCP tools (catalog, inspect, generate) for agent-driven development
- Created comprehensive documentation: getting-started guide, component reference (all 20), actions, data binding, layouts, and CLI reference updates

**Stats:**

- 336 files changed (+39,266, -1,297 lines)
- 7,203 lines of Rust (ferro-json-ui crate)
- 2,134 lines of documentation (6 pages)
- 10 phases, 24 plans, 241 commits
- 24 days (2026-01-16 → 2026-02-09)

**Git range:** `2cd48df` → `45e5487`

**What's next:** Publish to crates.io and public announcement.

---

## v2.2 CLI Improvements (Shipped: 2026-02-09)

**Delivered:** CLI commands for database workflows, gitignore for generated types, and typed UpdateBuilder pattern for model updates.

**Phases completed:** 35-37 (5 plans total)

**Key accomplishments:**

- Added `ferro db:seed` CLI command completing the seeder workflow
- Unified all database commands under `db:` namespace (db:migrate, db:rollback, db:status, db:fresh, db:seed)
- Excluded generated TypeScript types directory from version control in project template
- Implemented typed UpdateBuilder with selective field tracking via `model.update().set_field(v).save().await`
- Updated scaffold templates, MCP code templates, and documentation with builder pattern

**Stats:**

- 40 files modified (+2098, -310 lines)
- 3 phases, 5 plans, ~11 tasks
- 22 days (2026-01-18 to 2026-02-09)

**Git range:** `09e01d3` → `3c7dcfb`

**What's next:** v3.0 JSON-UI for JSON-based UI rendering without frontend builds.

---

## v2.1 Inertia DX & Fixes (Shipped: 2026-01-17)

**Delivered:** Improved Inertia developer experience with JSON API fallback, auto type generation, utility types, and fixed documentation URLs.

**Phases completed:** 33-34 (4 plans total)

**Key accomplishments:**

- Added JSON Accept header fallback for API testing via `render_with_json_fallback()` method
- Enhanced SavedInertiaContext documentation with Common Patterns and Troubleshooting sections
- Enabled auto type generation by default in `ferro serve` with file watching
- Added `JsonValue` and `ValidationErrors` utility types to generated TypeScript
- Fixed documentation URLs to use docs.ferro-rs.dev subdomain

**Stats:**

- 34 files modified (+1165, -219 lines)
- 2 phases, 4 plans, ~12 tasks
- Same day completion (2026-01-17)

**Git range:** `e69749d` → `556eee7`

**What's next:** v3.0 JSON-UI for JSON-based UI rendering without frontend builds.

---

## v2.0.3 DO Apps Deploy (Shipped: 2026-01-17)

**Delivered:** One-click deployment to DigitalOcean App Platform with `ferro do:init` CLI command.

**Phases completed:** 22.10 (1 plan total)

**Key accomplishments:**

- Created DO App Platform spec template with service, database, and redis configuration
- Implemented `ferro do:init --repo owner/repo` command following docker_init pattern
- Generated YAML includes GitHub integration with deploy-on-push
- Health check endpoint, environment variables, and database bindings pre-configured

**Stats:**

- 9 files modified (606 insertions)
- 1 phase, 1 plan, 4 tasks
- Same day completion (2026-01-17)

**Git range:** `87bd781` → `705750d`

**What's next:** v2.1 JSON-UI milestone for JSON-based UI rendering.

---

## v2.0.2 Type Generator Fixes (Shipped: 2026-01-17)

**Delivered:** TypeScript type generation fixes for production-ready frontend integration.

**Phases completed:** 22.4-22.9 (6 plans total)

**Key accomplishments:**

- Fixed serde case handling with enum-based approach
- Resolved prop naming collisions with namespaced names
- Added contract validation CLI command
- Implemented datetime type recognition for chrono types
- Added nested types generation with fixed-point iteration
- Mapped ValidationErrors to Record<string, string[]>

**Stats:**

- 6 phases, 6 plans
- Same day completion (2026-01-17)

**Git range:** Phase 22.4 → Phase 22.9

**What's next:** v2.0.3 DO Apps Deploy

---

## v2.0.1 Macro Fix (Shipped: 2026-01-17)

**Delivered:** Fixed hardcoded macro crate paths from `::ferro_rs::` to canonical `ferro::`.

**Phases completed:** 22.1-22.3 (6 plans total)

**Key accomplishments:**

- Fixed proc macro crate path generation
- Simplified macro path handling
- Completed remaining rebrand items

**Stats:**

- 3 phases, 6 plans
- Same day completion (2026-01-17)

**Git range:** Phase 22.1 → Phase 22.3

**What's next:** v2.0.2 Type Generator Fixes

---

## v2.0 Rebrand (Shipped: 2026-01-16)

**Delivered:** Complete framework rebrand from "cancer" to "ferro" for crates.io publication and public release.

**Phases completed:** 13-22 (13 plans total)

**Key accomplishments:**

- Renamed all 11 crates from cancer-* to ferro-* (framework, CLI, MCP, events, queue, etc.)
- Updated all documentation, READMEs, and code comments to use "ferro" branding
- Created comprehensive migration guide for existing users at docs/src/upgrading/migration-guide.md
- Prepared crates.io metadata and publishing checklist (PUBLISHING.md)
- Updated repository URLs to ferroframework/ferro
- Migrated sample app to use ferro imports

**Stats:**

- 321 files modified
- 60,000 lines of Rust (total codebase)
- 10 phases, 13 plans
- 1 day (intensive single-day rebrand)

**Git range:** `docs(13-01)` -> `docs(phase-22)`

**What's next:** Publish crates to crates.io using PUBLISHING.md checklist, then announce public release.

---

## v1.0 DX Overhaul (Shipped: 2026-01-16)

**Delivered:** Agent-first developer experience transformation with reduced boilerplate, expanded MCP introspection, and improved CLI scaffolding.

**Phases completed:** 1-12 (18 plans total)

**Key accomplishments:**

- Simplified handler definitions with #[handler] macro reducing ceremony
- Created FerroModel derive macro for automatic SeaORM trait implementations
- Added ValidateRules derive macro for concise validation rule definitions
- Expanded MCP to 30+ introspection tools including domain glossary, relationship graphs, and generation hints
- Added CLI feature scaffolding with smart defaults and FK detection
- Implemented actionable error messages with fix suggestions

**Stats:**

- 200+ files modified
- 60,000 lines of Rust
- 12 phases, 18 plans
- 2 days from start to ship

**Git range:** `feat(01-01)` -> `feat(12-05)`

**What's next:** v2.0 Rebrand (cancer -> ferro for crates.io publication)

---
