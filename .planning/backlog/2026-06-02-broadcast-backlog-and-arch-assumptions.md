# Feedback: ferro-broadcast backlog + framework assumption clarifications

**Source:** Downstream Inertia + WebSocket streaming consumer app (private, AI-native chat product), field assessment 2026-06-02
**Severity:** Mixed — one feature request (high impact), one documentation gap, three minor clarifications
**Ferro version inspected:** ferro-broadcast 0.2.41, framework HEAD as of 2026-06-02

## Planning Note

This document is a sketch from a downstream-app perspective, not an inside-Ferro design. When promoted from backlog to a phase, the Ferro planning agent should reconcile against `.planning/VISION.md` and existing conventions before drafting `PLAN.md`.

---

## 1. Feature request — ferro-broadcast: per-subscriber backlog + replay-from-cursor primitive

### Problem

A downstream app streaming LLM tokens over a WebSocket from a Ferro backend to a wrapped iOS WebView needs the WebSocket to survive iOS backgrounding (screen lock, app switcher) for 30 seconds and ideally up to 15 minutes, with **replay-from-cursor** recovery on resume.

Direct source inspection of ferro-broadcast 0.2.41 confirms: it ships **no backlog, no sequence numbers, no replay mechanism**. The transport is tokio's `broadcast::channel`, which silently drops on lag — fine for fan-out where a slow consumer is acceptable to lose events, fatal for any "what did I miss while I was gone?" use case.

### Why this should be a framework primitive

Every Ferro app that uses ferro-broadcast for client-facing streaming will hit this same gap. The hand-rolled solution (ring buffer keyed by channel + a `/since?seq=N` endpoint) is the same shape every time. Making it a framework feature:

- removes a 200+ LOC hand-roll per app
- ensures consistency in the on-wire schema for `{ seq, payload }` envelopes
- enables a documented "WebSocket survives backgrounding" recipe with reconnect/replay semantics

### Proposed API sketch

```rust
let broadcaster = Broadcaster::with_config(
    BroadcastConfig::from_env()
        .with_channel_backlog("chat.{tenant}.{session}", Backlog::Bounded {
            max_events: 256,
            max_age: Duration::from_secs(300),
        })
);

// Publish — returns the assigned monotonic seq for that channel
let seq: u64 = broadcaster
    .channel("chat.acme.s-42")
    .event("token")
    .data(json!({ "text": "Hello" }))
    .publish_seq()
    .await?;

// Catch-up — used by a /since?seq=N route or by client-driven replay
let missed: Vec<EventWithSeq> = broadcaster
    .channel("chat.acme.s-42")
    .since(after_seq)
    .await?;
```

### Acceptance criteria

- API: `Broadcaster::with_config(BroadcastConfig::with_channel_backlog(...))` — declarative per-channel-pattern backlog config
- API: `channel.publish_seq() -> u64` — returns assigned monotonic seq
- API: `channel.since(after_seq: u64) -> Vec<EventWithSeq>` — catch-up reads
- Ring buffer with bounded size + age eviction; evicted-seq detection so clients know "you fell off the end, reset"
- Tests: normal pub/sub; catch-up via `since()`; ring eviction; concurrent subscribers; multi-channel backlog isolation
- Doc update: ferro-broadcast README + a "WebSocket survives backgrounding" recipe

### Downstream impact if not built

Every app that wants the "graceful resume after iOS backgrounding" UX hand-rolls this. The downstream app in question will hand-roll it; the result will be available for donation back if the Ferro team prefers that path.

---

## 2. Documentation clarification — ferro-queue durability model

### Observation

A downstream app planned its durable-write queue assuming ferro-queue offered Postgres-backed persistence. Source inspection revealed ferro-queue is **Redis-backed**; durability is governed by Redis persistence configuration, not Postgres.

This is not a bug. It is a documentation gap: the README and crate docs don't make the backend explicit, and "queue with durability" reads (to a new user) as "DB-backed."

### Suggested action

- Make the storage backend a first-paragraph fact in the crate-level docs (`ferro-queue` README + `lib.rs` doc comment)
- Document the durability guarantees that follow from Redis AOF / RDB modes
- *(optional v-future)* Consider an opt-in Postgres backend for users who want strict durability without operating Redis

---

## 3. Documentation clarification — SQLite driver path

### Observation

ARCH-level docs in one downstream app's planning assumed `rusqlite` was Ferro's SQLite story. The actual path is `sqlx-sqlite` via SeaORM. Functionally equivalent for that app's needs, but the assumption document was wrong because no Ferro doc states this explicitly.

### Suggested action

In ferro-orm docs (or wherever SeaORM integration is described), state explicitly: "SQLite goes through `sqlx-sqlite` via SeaORM. Direct `rusqlite` use is not supported by Ferro's ORM layer."

---

## 4. Roadmap input — Inertia.js version upgrade evaluation

### Observation

`ferro-inertia` currently pins `@inertiajs/react@^1.0.0`. The downstream app's planning depended on Inertia v2 features — deferred props, partial reloads — for its chat-streaming UX. Those features don't exist in v1, so the app fell back to a custom WebSocket + React hook for streaming.

This is not a defect, but **Inertia v2's deferred props and partial reloads materially improve the streaming UX surface for AI / chat apps** built on Ferro. Worth a roadmap evaluation:

- Survey the ferro-inertia ecosystem dependents to gauge breaking-change blast radius
- Evaluate the cost of `@inertiajs/react@^2.0.0` migration in ferro-inertia
- If migration is reasonable, this could materially de-risk the v2.0+ multimodal / streaming direction the framework is heading

---

## 5. Roadmap input — `ferro-conversational` renderer crate

### Observation

The framework's `Renderer` trait is designed modality-agnostic. Today only `JsonUiRenderer` exists. The downstream app needs a *conversational* renderer — one that turns intents into `(text stream, citation chips, suggestion chips, action proposals)` instead of an HTML tree.

The downstream app intends to build `ConversationalRenderer` inside its own codebase first, then extract upward into a `ferro-conversational` crate once the API stabilizes. This is the "compressive first" investment path called out in Ferro's vision.

### Suggested coordination

- Tag this as a watching item on the v2.0+ roadmap
- The downstream app will open a separate proposal-doc PR when the renderer's API is ready for extraction
- No action required by Ferro maintainers yet; this is forward-notice

---

## Source / provenance

- Inspected ferro source at HEAD as of 2026-06-02
- Specifically: `framework/src/websocket.rs`, `ferro-broadcast/src/lib.rs`, `ferro-queue/Cargo.toml`, `ferro-inertia/package.json`, `ferro-orm/Cargo.toml`
- Verification cross-checked against framework CLAUDE.md vision anchors and the ferro MCP introspection surface

Filed by the downstream app per its dogfooding discipline rule.
