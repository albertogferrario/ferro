---
gsd_state_version: 1.0
milestone: v11.0
milestone_name: Framework Consolidation Audit
status: executing
stopped_at: Phase 152 PLAN-06 Task 3 — first-publish bootstrap human-action checkpoint
last_updated: "2026-05-13T16:01:00Z"
last_activity: 2026-05-13 -- Phase 152 PLAN-06 executor portion complete; awaiting operator bootstrap of ferro-orm to crates.io
progress:
  total_phases: 156
  completed_phases: 138
  total_plans: 369
  completed_plans: 348
  percent: 94
---

# Project State

## Project Reference

See: .planning/PROJECT.md and .planning/VISION.md

**Core value:** Ferro is a Rust web framework optimized for AI-assisted authoring, with projection / intent (`ferro-projections`) as its core abstraction.
**Current focus:** Phase 152 — ferro-orm-guardedupdate-atomic-conditional-updates-for-race-

## Current Position

Phase: 152 (ferro-orm-guardedupdate-atomic-conditional-updates-for-race-) — AWAITING HUMAN-ACTION CHECKPOINT (plan 06 Task 3)
Plan: 6 of 6 — Task 3 (first-publish bootstrap) awaits user; Tasks 1 (gate) + 2 (CHANGELOG e38536cc) complete
Plans: 5 of 6 fully complete; plan 06 executor portion done, pending operator bootstrap of ferro-orm to crates.io
Workspace version: 0.2.30 (unchanged — CONTEXT D-23's 0.2.25 superseded by RESEARCH Open Question 1)
Status: Executing Phase 152 — paused at PLAN-06 Task 3 human-action checkpoint
Last activity: 2026-05-13 -- Phase 152 plan 06 executor portion complete; human-action checkpoint surfaced for first-publish bootstrap
Next milestone: v12.0 JSON-UI v2 (Phase 115 — Spec v2 Data Structures)

Progress: [██████████] 96%

## Performance Metrics

**Velocity:**

- Total plans completed: 45
- Average duration: —
- Total execution time: —

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 140 | 5 | - | - |
| 141 | 4 | - | - |
| 143 | 4 | - | - |
| 144 | 5 | - | - |
| 146 | 3 | - | - |
| 145 | 5 | - | - |
| 148 | 3 | - | - |
| 149 | 7 | - | - |
| 150 | 5 | - | - |

*Updated after each plan completion*
| Phase 108-p0-accuracy-fixes P01 | 3 | 1 tasks | 3 files |
| Phase 108-p0-accuracy-fixes P02 | 12min | 2 tasks | 3 files |
| Phase 109-cli-reference-completeness P01 | 148s | 2 tasks | 1 files |
| Phase 110-mcp-tool-accuracy P02 | 8min | 1 tasks | 1 files |
| Phase 110-mcp-tool-accuracy P01 | 15min | 2 tasks | 2 files |
| Phase 111-documentation-coverage P01 | 106s | 2 tasks | 2 files |
| Phase 111-documentation-coverage P02 | 2min | 2 tasks | 2 files |
| Phase 112-agent-first-philosophy P01 | 2min | 2 tasks | 3 files |
| Phase 112-agent-first-philosophy PP02 | 248s | 2 tasks | 19 files |
| Phase 113-pattern-coherence P02 | 12min | 2 tasks | 5 files |
| Phase 113-pattern-coherence P01 | 85 | 2 tasks | 22 files |
| Phase 114.1-template-renderer P01 | 10min | 2 tasks | 3 files |
| Phase 122-deploy-scaffold-core-rewrite P01 | 7min | 1 tasks | 2 files |
| Phase 122-deploy-scaffold-core-rewrite P02 | 6min | 2 tasks | 5 files |
| Phase 122-deploy-scaffold-core-rewrite P03 | 5min | 2 tasks | 2 files |
| Phase 122-deploy-scaffold-core-rewrite P04 | 5min | 2 tasks | 5 files |
| Phase 122-deploy-scaffold-core-rewrite P05 | ~6min | 2 tasks | 5 files |
| Phase 122 P06 | 3m | 1 tasks | 1 files |
| Phase 123-deploy-mcp-tools P02 | 8min | 2 tasks | 8 files |
| Phase 123-deploy-mcp-tools P05 | 6min | 2 tasks | 3 files |
| Phase 124-doctor-introspection-and-ci-scaffold P02 | 15min | 2 tasks | 4 files |
| Phase 124 P03 | 25min | 2 tasks | 8 files |
| Phase 124 P05 | 5min | 1 tasks | 2 files |
| Phase 122.1 P02 | 6min | 2 tasks | 2 files |
| Phase 122.1 P04 | ~8min | 2 tasks | 7 files |
| Phase 122.2 P01 | 3min | 2 tasks | 6 files |
| Phase 122.2 P03 | 12min | 3 tasks | 18 files |
| Phase 122.2 P07 | 8min | 2 tasks | 4 files |
| Phase 122.2 P08 | 14m | 2 tasks | 10 files |
| Phase 127 P01 | 25min | 3 tasks | 7 files |
| Phase 127-generated-artifact-polish P02 | 10min | 2 tasks | 4 files |
| Phase 127-generated-artifact-polish P03 | 8min | 1 tasks | 3 files |
| Phase 127-generated-artifact-polish P04 | 15min | 3 tasks | 5 files |
| Phase 128-deploy-preflight P01 | 5min | 2 tasks | 4 files |
| Phase 128-deploy-preflight P03 | 2min | 2 tasks | 3 files |
| Phase 128-deploy-preflight P02 | 5min | 3 tasks | 8 files |
| Phase 128-deploy-preflight P04 | 4min | 2 tasks | 4 files |
| Phase 129-publish-workflow-refinement P01 | 2 | 2 tasks | 1 files |
| Phase 129-publish-workflow-refinement P02 | 2min | 3 tasks | 2 files |
| Phase 129 P03 | 2min | 2 tasks | 1 files |
| Phase 131 P01 | 20min | 2 tasks | 7 files |
| Phase 131-scaffolder-multibin-copydirs-runtime-apt P02 | 9min | 2 tasks | 11 files |
| Phase 131 P03 | 8min | 1 tasks | 6 files |
| Phase 132 P01 | 11min | 2 tasks | 4 files |
| Phase 133-generalize-renderer-trait P01 | 3.5min | 1 tasks | 5 files |
| Phase 133-generalize-renderer-trait P02 | 5min | 1 tasks | 4 files |
| Phase 134-relocate-renderers-to-output-crates P01 | 15min | 1 tasks | 6 files |
| Phase 134-relocate-renderers-to-output-crates P02 | 4min | 2 tasks | 10 files |
| Phase 135-servicedef-derivation-bridge P01 | 8min | 2 tasks | 3 files |
| Phase 135-servicedef-derivation-bridge P02 | 6min | 2 tasks | 3 files |
| Phase 141 P02 | 15min | 2 tasks | 5 files |
| Phase 145 P01 | 11min | 3 tasks | 7 files |
| Phase 145 P02a | 8min | 2 tasks | 3 files |
| Phase 145 P02b | 21min | 2 tasks | 1 files |
| Phase 148 P01 | 221s | 2 tasks | 2 files |
| Phase 149 P01 | 9min | 3 tasks | 6 files |
| Phase 149 P02 | 4m 12s | 3 tasks | 4 files |
| Phase 149 P03 | 5m 1s | 2 tasks | 3 files |
| Phase 149 P04 | 4m 41s | 3 tasks | 2 files |
| Phase 149 P05 | 4m 7s | 2 tasks | 1 files |
| Phase 149 P06 | 9m 17s | 3 tasks | 2 files |
| Phase 149 P07 | 8m 2s | 7 tasks | 8 files |
| Phase 151 P02 | 3min | 2 tasks | 2 files |
| Phase 151 P03 | 4m 15s | 2 tasks | 2 files |
| Phase 151-ferro-wallet-crate P04 | 2m 46s | 2 tasks | 2 files |
| Phase 151-ferro-wallet-crate P05 | 4m 34s | 4 tasks | 5 files |
| Phase 151 P07 | 10min | 3 tasks | 4 files |
| Phase 151-ferro-wallet-crate P06 | 4min | 1 tasks | 1 files |
| Phase 151-ferro-wallet-crate P151-08 | 94s | 1 tasks | 1 files |
| Phase 151 P09 | 2min | 2 tasks | 3 files |

## Accumulated Context

### Key Decisions

See PROJECT.md Key Decisions table for full history.

Recent decisions affecting current work:

- Research established strict ordering: P0 accuracy → CLI/MCP → completeness → philosophy → metadata
- COMPONENT_CATALOG duplication requires a design decision before implementation (Phase 113)
- ferro-stripe phantom stubs: classify as incomplete, add callout — do not implement in v11.0
- `#![warn(missing_docs)]` on framework crate only — not workspace-wide (avoids mass failures)
- [110-01] All ferro imports use explicit crate-root exports — no ferro::prelude or ferro::validation:: module paths
- [110-01] Status codes use .status(u16) pattern — StatusCode enum not re-exported from ferro crate
- [110-01] Validation rule functions imported at crate root: ferro::{Validator, required, email, min, ...}
- [112-01] introduction.md leads with "agent-first" in sentence 1 — MCP mentioned before any framework comparison or Laravel reference
- [112-01] Working with Agents guide covers ferro-mcp only — ferro-api-mcp remains on its dedicated api-mcp.md page
- [112-01] Agent-to-CLI workflow documented within working-with-agents.md as a section, not a separate page
- [112-01] MCP config command is `ferro mcp` — not a standalone ferro-mcp binary
- [145-01] Test-fixture crates under workspace root need an empty [workspace] table in their Cargo.toml to opt out of the enclosing workspace and build standalone
- [145-01] classify_key signature declared with final crossterm types (KeyCode, KeyModifiers) at Wave 0 — no Plan-02 signature rewrite needed
- [145-02b] BackendSupervisor owns backend child in its own thread; main thread holds JoinHandle for deterministic shutdown ordering per D-29
- [145-02b] drop(reload_tx) after cloning to producers lets the supervisor's recv_timeout see Disconnected — belt-and-braces termination path in addition to the AtomicBool shutdown flag
- [145-02b] debouncer_coalesces_burst uses 500ms production window and "strictly fewer events than raw writes" invariant; plan's 50ms + "exactly one" was flaky under macOS FSEvents + parallel-test CPU contention
- [145-02b] ProcessManager::any_exited deleted entirely (D-12: backend child exits are not grounds for shutdown); also deleted spawn_with_prefix convenience wrapper since only spawn_with_prefix_env is still called
- [149-01] Pulled lib.rs top-level re-exports of new channel types forward from plan 07 (Rule 3 deviation) — needed because CI -D warnings rejects unused-import warnings produced by mod.rs re-exports without crate-level re-exports
- [149-01] SmsMessage and PushMessage placeholders live in shared channels/future.rs (not separate sms.rs/push.rs files) since they share the unimplemented-but-signature-stable lifecycle
- [149-02] Per-variant `#[serde(rename = "in_app")]` overrides the enum-level `lowercase` rule on Channel to lock the wire form to "in_app" not "inapp"; regression-guard test rejects literal "inapp" deserialization (closes ARCH-FINDING-05 trap, T-149-W1A-01)
- [149-02] Pulled forward dispatcher exhaustive-match arm fix from plans 04/05 (Rule 3 blocking deviation) — adding new Channel variants forced match exhaustiveness; placeholder arm (`Channel::WhatsApp | Channel::InApp | Channel::Sms | Channel::Push => info!("not implemented")`) is the minimal compile fix, real adapter arms land in plans 04/05
- [149-02] Pulled forward Error::Broadcast(String) + Error::broadcast helper from plan 06's scope (Rule 2 deviation) — load-bearing primitive for InApp adapter error mapping; one logical error.rs commit avoids re-touching the file in plan 06
- [149-03] 25 MB per-attachment cap is inclusive (D-11): exactly MAX_ATTACHMENT_BYTES bytes succeeds; one byte over fails with Error::AttachmentTooLarge. No cumulative cap — Resend's 40 MB total is carrier responsibility per CONTEXT.md.
- [149-03] MailMessage.attachments field carries `#[serde(default)]` so pre-existing JSON payloads (queue jobs, retry envelopes) continue to deserialize after the field is added — backward-compat for already-persisted MailMessage envelopes.
- [149-03] Pulled forward MailAttachment lib.rs re-export (Rule 3 blocking deviation, identical pattern to Plan 01) — adding `pub use` in mod.rs without a crate-level re-export trips unused-imports under -D warnings.
- [149-04] SMTP body branches on `message.attachments.is_empty()` — empty path preserves byte-identical wire format (no Content-Type: multipart/mixed header on simple emails), non-empty path uses MultiPart::mixed with Attachment::new(filename).body(content, ContentType::parse(content_type)?) per part. Existing 8 dispatcher tests pass unchanged as proof of zero regression.
- [149-04] Resend driver uses base64 standard alphabet (NOT URL-safe) for attachment encoding — locked by test_base64_encoding_uses_standard_alphabet against pangram fixture "Many hands make light work." → "TWFueSBoYW5kcyBtYWtlIGxpZ2h0IHdvcmsu" (closes T-149-W2-03; URL-safe would corrupt binary content).
- [149-04] ResendEmailPayload.attachments uses `#[serde(skip_serializing_if = "Vec::is_empty")]` so the no-attachment JSON wire payload contains NO "attachments" key — byte-identical to today. Existing test_resend_payload_serialization tightened with `assert!(json.get("attachments").is_none())` so it doubles as a backward-compat regression guard alongside the dedicated test_resend_payload_no_attachments_omits_field.
- [149-04] Function-scoped `use lettre::message::{Attachment, MultiPart, SinglePart};` and `use base64::Engine;` inside the dispatcher's send_mail_smtp / send_mail_resend — matches the existing function-local pattern for header::ContentType + Mailbox; keeps dispatcher.rs's module-level imports tidy.
- [149-05] NotificationConfig::whatsapp_enabled (default false) gates the WhatsApp adapter; from_env reads WHATSAPP_ENABLED and falls back to false on parse failure (matches the legacy SLACK_WEBHOOK_URL / MAIL_FROM_ADDRESS optional-read shape).
- [149-05] send_whatsapp calls ferro_whatsapp::WhatsApp::send via the static facade (D-04 / ARCH-FINDING-01) — no client object injection. CONFIG.get().map(|c| c.whatsapp_enabled).unwrap_or(false) gate keeps the static-facade panic-on-uninit-init path unreachable for default configurations.
- [149-05] Channel::InApp arm split out of the shared placeholder collapse as a transitional placeholder ("Channel not configured" wording aligns with the eventual NotificationConfig::in_app: Option<InAppConfig> gate). Plan 06 diff is now a body replacement, not a surrounding-scaffolding edit.
- [149-06] InApp adapter writes both legs (DB-store first, broadcast second) per D-08; ferro_broadcast::Error mapped via Error::broadcast(e.to_string()) helper since no #[from] is available. Persistence-first ordering means the broker can replay on reconnect from the store; the inverse order would risk silent loss.
- [149-06] send_database now routes through DatabaseNotificationStore::store(...) when CONFIG.database_store is configured; unconfigured path retains placeholder log for backward-compat (closes ARCH-FINDING-02). The Database channel and the InApp channel both share the same Arc<dyn DatabaseNotificationStore> — no duplicate persistence path.
- [149-06] inapp_to_database_message normalizes the type-shape mismatch between InAppMessage.data (serde_json::Value, any shape) and DatabaseMessage.data (HashMap<String, Value>, object only): object inputs flatten to fields directly; non-object inputs wrap under the 'payload' key (lossless round-trip).
- [151-03] APP_NAME / APP_URL fallbacks hardcoded to framework::config::AppConfig defaults ("Ferro Application" / "http://localhost:8080") rather than via framework dep — keeps ferro-wallet a true leaf crate per spec §5; any future framework-side default change must be mirrored here, not coupled.
- [151-03] AppleConfig / GoogleConfig cluster fails to None on ANY missing required var (D-02 permissive) — partial cluster is treated as a misconfiguration and forces the caller to surface it at startup-feature-gate rather than at pass-issuance time. from_env_apple_partial_returns_none locks this in.
- [151-03] WalletConfig::from_env signature is Result<Self, WalletError> even though the current impl never returns Err — forward-compat for non-wallet validation (e.g. URL parse) without SemVer break. Downstream PLAN-05 / PLAN-07 already use `?` to chain the call.
- [151-03] EnvGuard RAII + static Mutex<()> chosen over serial_test dev-dep — keeps ferro-wallet's dep set minimal AND provides panic-safe env restore that ferro-stripe's manual save/restore pattern lacks. Std::io::Error is not UnwindSafe so the plan's catch_unwind suggestion was uncompilable.
- [151-05] labelColor tracks foregroundColor in v1 (D-06): both fields receive the same BT.601-derived value; per-pass label/foreground separation deferred to a future phase.
- [151-05] Apple manifest map ordering uses BTreeMap<String,String> for byte-stable JSON output (RESEARCH.md Risk 7) — load-bearing for PKCS#7 signature determinism across re-invocations of the same subject.
- [151-05] manifest.rs imports super::ApplePassBuilder to borrow pass_type_id/team_id/app_name; kept as the only cross-file Apple coupling to avoid threading 3 parameters through the pipeline.
- [151-05] Per-task cargo-build verifies relaxed to chain-level verify (Rule 3 deviation) — the four apple/* files share data dependencies the plan itself documents; atomic commits preserved, full verification (build + test + clippy -D warnings + fmt) runs after Task 4.
- [151-08] Pitfall-3 mitigation: `Validation::new(Algorithm::RS256)` requires `exp` by default; for save JWTs (which carry no `exp`), explicitly set `validate_exp = false` AND `required_spec_claims = HashSet::new()`, then re-arm `set_audience(&[expected])` to keep aud-check active. Reusable pattern across the workspace for any exp-less token (OIDC id-token bearer assertions, Google save JWTs, custom service-to-service JWTs).
- [151-08] Runtime-mint RSA keypair pattern (D-09 applied): tests/google_jwt.rs uses `openssl::rsa::Rsa::generate(2048)` → `PKey::from_rsa` → `private_key_to_pem_pkcs8` + `public_key_to_pem`. Pairs with production builder's PEM-loading path so the test exercises real parse-and-sign code. No committed credentials; key material discarded when test ends.
- [151-08] Phase 151 now publish-ready: 38 lib + 1 apple_integration + 2 google_jwt = 41 green tests in ferro-wallet. Plans 01–08 done; only 151-09 (version bump + CHANGELOG + auto-publish) remains.
- [152-06] Workspace pre-release gate green across all 22 workspace crates at version 0.2.30: fmt + clippy (-D warnings) + build + test (--all-features) + doc (-p ferro-orm) all exit 0 with zero warnings. ferro-orm contributes 12 tests (11 unit + 1 integration / concurrent_decrement).
- [152-06] CONTEXT D-23's `0.2.25` target was superseded by reality — workspace already advanced to 0.2.30 across earlier phases without re-tagging (RESEARCH Open Question 1). CHANGELOG records `### [0.2.30] — 2026-05-13`; no manual bump performed; the actually-published version is whatever Cargo.toml records at bootstrap time.
- [152-06] Task 3 returned as human-action checkpoint (Pitfall 5; same pattern as Phase 151 PLAN-09): first publish of a new crate requires personal `publish-new`-scoped crates.io token from a local terminal; CI's `publish-update`-scoped token cannot create a new crate. Auth/credential gates cannot be automated.
- [152-06] CHANGELOG placement convention reinforced: newest crate at top — `## ferro-orm` inserted above `## ferro-wallet` (which itself sits above `## ferro-rs` from Phase 151 PLAN-09).
- [152-06] Release plan structure for new workspace crates is now stable across two phases: (1) workspace-wide pre-release gate, (2) CHANGELOG entry under per-crate section, (3) manual first-publish bootstrap from local terminal, (4) push to master so CI auto-publish takes over for subsequent versions.

### Pending Todos

- Push workspace to origin/master to publish v0.2.0 (627 commits ahead).
- Ferro doctor `db_connection` and `migrations_pending` checks should auto-resolve `--bin <pkg>` for multi-bin projects without `default-run`. Tracked in `.planning/phases/122.2-deploy-simplification/122.2-VERIFICATION.md`.

### Blockers/Concerns

- [Research flag] Phase 113: COMPONENT_CATALOG resolution needs design decision evaluation (shared data file vs build script vs new crate) — evaluate options before scoping
- Phase 152 PLAN-06: awaiting user manual first-publish bootstrap of ferro-orm to crates.io at the version Cargo.toml records (currently 0.2.30). CI token has publish-update only; first publish needs personal publish-new token from local terminal (RESEARCH Pitfall 5). Task 2 committed (e38536cc — CHANGELOG entry). Resume signal: user replies 'published' with resolved version after crates.io confirms; subsequent versions auto-publish via existing GH Actions workflow. Mirror pattern: Phase 151 PLAN-09 ferro-wallet 0.2.24 bootstrap.

### Roadmap Evolution

- Phase 147 added: DetailForm component for inline edit — ferro-json-ui
- Phase 146 added: Add KeyValueEditor component to ferro-json-ui
- Phase 122 added: Deploy scaffold core rewrite (docker_init/do_init/templates rewrite, path→git ferro dep handling, multi-bin + worker support) — driven by gestiscilo deployment work
- Phase 123 added: Deploy MCP tools (deploy_check, deploy_diff_env, runtime_requirements) — read-only deploy diagnostics surfaced via ferro-mcp
- Phase 124 added: Doctor, introspection, CI scaffold (ferro doctor, routes --json, ci.yml generation, ignore_patterns sync)
- Phase 125 added: Module scaffolder + ferro-json-ui runtime split (make:module convention, runtime IIFE refactor)
- [CLI bug] `gsd-tools phase add` assigned 115 four times in one batch — does not detect previously added phases when computing next integer; also collided with an unrelated active milestone (JSON-UI v2 already at 115-121). Manually renumbered to 122-125. File against gsd-tools.
- Phase 126 added (2026-04-08): Deploy experience feedback triage — analysis-only phase pointing the next agent at `phases/126-deploy-experience-feedback/REPORT.md` (field notes from first end-to-end gestiscilo deploy: 2 fixed bugs already shipped in 0.2.1, 9 sharp edges still present, 6 DX improvements). Agent must produce `PROPOSAL.md` classifying every item before any new ferro work is scoped.
- Phase 131 added (2026-04-09): Scaffolder multi-bin, copy_dirs, runtime_apt, DO app.yaml robustness, drift detection — promoted from `.planning/backlog/gestiscilo-scaffolder-multibin-gap.md` (gestiscilo-it Phase 75 field test gap). CLI bug recurred again (returned phase 1); manually renumbered.
- Phase 130 added (2026-04-09): Invert dep convention (simple) — retire `Cargo.docker.toml` and `cargo_docker_toml_staleness` doctor check; Docker builds use `Cargo.toml` directly; local ferro dev via uncommitted `[patch.crates-io]`. Source: `.planning/proposals/dep-override-convention.md` (simplified per user direction — no new CLI verbs, no new doctor check). CLI bug recurred: `gsd-tools phase add` returned phase 1 instead of 130; manually renumbered.
- Phase 143 inserted (2026-04-20): Tailwind static CSS pipeline (URGENT) — opened new milestone v11.7. Source: gestiscilo-it production field report — `@tailwindcss/browser@4` runtime JIT fails on Safari, renders login page as unstyled HTML. Replace with pre-built static CSS. Manually scaffolded (gsd-tools phase insert rejected because STATE.md milestone field still says v11.0 but v11.6 and earlier have shipped — STATE drift is a separate cleanup). Context: `.planning/phases/143-tailwind-static-css-pipeline/143-CONTEXT.md`.
- Phase 144 added (2026-04-21): Fix root path routing in group routes — `get!("/", ...)` inside a group does not match the trailing-slash URL. Source: gestiscilo-it field test — `/s/{slug}/` returns 404; `/s/{slug}/index.html` works. The `serve_root` handler is unreachable via the canonical URL.
- Phase 145 added (2026-04-22): ferro serve manual reload key and watch supervisor — replace external `cargo-watch` with in-process supervisor, flip auto-watch to opt-in via `--watch`, add runtime `r` key for cancel-and-restart rebuilds, unify backend recompile + types regen under one debounced loop. Source: field report — rapid file saves produce compounding stale rebuilds; thermal cost on MacBook. Spec: `docs/superpowers/specs/2026-04-22-ferro-serve-reload-key-design.md`.
- Phases 152-155 added (2026-05-13): v11.11 Resource Reservation & Live Read-Model Primitives milestone created. Source: gestiscilo-it inventory monitoring field test. Four domain-neutral horizontal primitives — 152 `ferro-orm::GuardedUpdate` (atomic conditional updates), 153 `ferro-audit` (structured before/after log), 154 `ferro-reservation` (generic hold/commit/release with TTL, depends on 152+153), 155 `ferro-projection` (live read-model from domain events with broadcast deltas, uses existing ferro-events + ferro-broadcast). Unblocks gestiscilo-it v6.3 Online Checkout (slot hold during Stripe payment) and v6.7 Inventory Monitoring. Design: `research/INVENTORY-PRIMITIVES.md`.

## Session Continuity

Last session: 2026-05-13T16:01:00Z
Stopped at: Phase 152 PLAN-06 Task 3 — first-publish bootstrap human-action checkpoint
Resume file: .planning/phases/152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-/152-06-SUMMARY.md
Next action: Operator runs `cargo publish -p ferro-orm --token <PERSONAL_PUBLISH_TOKEN>` from repo root, then `git push origin master`. Reply "published" with resolved version to close Phase 152 and advance to Phase 153.
