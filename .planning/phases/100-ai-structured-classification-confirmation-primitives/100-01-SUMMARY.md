---
phase: 100-ai-structured-classification-confirmation-primitives
plan: 01
subsystem: ai
tags: [anthropic, classification, structured-output, reqwest, async-trait, ferro-ai]

requires: []
provides:
  - "ferro-ai crate with ClassificationProvider trait, AnthropicProvider, Classifier<T> facade, ClassifierConfig, ClassificationResult"
  - "Retry logic partitioning permanent (400/401/403/404/422) vs transient (429/500/503/529) HTTP errors"
  - "output_config.format.type=json_schema request body for Anthropic structured outputs API"
affects: [100-02, 100-03]

tech-stack:
  added:
    - "ferro-ai crate (new, Wave 1)"
    - "reqwest 0.12 (async HTTP client)"
    - "dashmap 6 (already in workspace, now in ferro-ai)"
    - "chrono 0.4 (already in workspace, now in ferro-ai)"
  patterns:
    - "Classifier<T: DeserializeOwned> generic facade pattern mirroring Stripe facade"
    - "ClassificationProvider async trait (object-safe, Arc<dyn ClassificationProvider>)"
    - "build_request_body() pub(crate) helper for testable request construction without HTTP calls"
    - "is_permanent_error() / is_transient_error() status code partitioning for retry logic"
    - "confidence field extracted from raw_json if schema includes it (Anthropic does not return metadata outside schema)"

key-files:
  created:
    - "ferro-ai/Cargo.toml"
    - "ferro-ai/src/lib.rs"
    - "ferro-ai/src/error.rs"
    - "ferro-ai/src/classifier/mod.rs"
    - "ferro-ai/src/classifier/provider.rs"
    - "ferro-ai/src/classifier/anthropic.rs"
  modified:
    - "Cargo.toml (workspace members)"

key-decisions:
  - "Single ferro-ai crate for both classification and confirmation (Plan 02) — they share no code but co-deploy under one feature flag"
  - "ClassifierConfig confidence_threshold check happens after deserialization — callers must include 'confidence: f64' in their schema to get this signal"
  - "Retry loop only retries on non-permanent errors; is_permanent_provider_error checks the error message string for HTTP status code digits"
  - "AnthropicProvider::build_request_body is pub(crate) for unit testability without making real HTTP calls"
  - "Confidence extracted from raw_json.get('confidence') before deserialization — schema must include this field"

patterns-established:
  - "Provider trait pattern: ClassificationProvider mirrors TenantResolver/ThemeResolver — async_trait, Send + Sync, object-safe"
  - "Generic facade: Classifier<T> holds Arc<dyn ClassificationProvider> + ClassifierConfig + PhantomData<T>"
  - "Retry with permanent/transient split: loop up to max_retries+1, sleep on transient, immediate fail on permanent"

requirements-completed: [AI-01, AI-02, AI-03]

duration: 4min
completed: 2026-03-22
---

# Phase 100 Plan 01: AI Classification Subsystem Summary

**ferro-ai crate with provider-abstracted Anthropic structured JSON classification — ClassificationProvider trait, AnthropicProvider using output_config.format.type=json_schema, Classifier<T> facade with retry logic, 10 unit tests**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-22T13:37:18Z
- **Completed:** 2026-03-22T13:41:03Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- ferro-ai crate added to workspace with classification module structure
- ClassificationProvider object-safe async trait enabling provider swap (Anthropic, OpenAI, local)
- AnthropicProvider with `output_config.format.type=json_schema` for guaranteed schema-compliant responses
- Classifier<T> facade with retry logic distinguishing permanent (400/401/403/404/422) from transient (429/500/503/529) HTTP errors
- Confidence threshold check — schema must include `confidence: f64` field; extracted from raw JSON

## Task Commits

1. **Task 1: Create ferro-ai crate with classification types and traits** - `398d731` (feat)
2. **Task 2: Implement AnthropicProvider with retry logic** - `52eaa75` (feat)

## Files Created/Modified

- `ferro-ai/Cargo.toml` — Crate manifest with reqwest, serde_json, dashmap, tokio, async-trait, thiserror, serde, tracing, ferro-events, chrono
- `ferro-ai/src/lib.rs` — Public re-exports: Classifier, ClassifierConfig, ClassificationResult, ClassificationProvider, AnthropicProvider, Error
- `ferro-ai/src/error.rs` — Error enum: Config, Provider, LowConfidence{best_guess, confidence}, Deserialization, Timeout
- `ferro-ai/src/classifier/provider.rs` — ClassificationProvider async trait (object-safe)
- `ferro-ai/src/classifier/mod.rs` — Classifier<T>, ClassifierConfig (with defaults), ClassificationResult, retry loop
- `ferro-ai/src/classifier/anthropic.rs` — AnthropicProvider: new(), from_env(), build_request_body(), classify_raw()
- `Cargo.toml` — Added "ferro-ai" to workspace members

## Decisions Made

- `is_permanent_provider_error()` checks error message string for HTTP status digits — provider wraps status in the message string; this keeps the retry logic in `Classifier::classify()` without needing to distinguish error subtypes at the Error enum level
- Confidence is extracted from `raw_json.get("confidence")` before deserialization — callers must include it in the schema; Anthropic structured output returns only schema-compliant JSON, no metadata
- `build_request_body` is `pub(crate)` — allows unit testing request construction without HTTP mocking infrastructure

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. Formatting fixed automatically (`cargo fmt`). The `dead_code` warning on the `WithConfidence` test struct was suppressed with `#[allow(dead_code)]`.

## User Setup Required

None - no external service configuration required. `ANTHROPIC_API_KEY` is read at runtime by `AnthropicProvider::from_env()`.

## Next Phase Readiness

- ferro-ai crate structure is ready for Plan 02 (confirmation module: ConfirmationStore trait, InMemoryConfirmationStore, ConfirmationExpired event)
- `pub mod classifier; pub mod error;` in lib.rs is structured so `pub mod confirmation;` can be added in Plan 02

---
*Phase: 100-ai-structured-classification-confirmation-primitives*
*Completed: 2026-03-22*
