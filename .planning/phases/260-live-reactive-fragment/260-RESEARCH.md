# Phase 260: Live reactive fragment — Research

**Researched:** 2026-07-26
**Domain:** ferro-projection hook seam + ferro-json-ui renderer + ferro-broadcast transport + client runtime
**Confidence:** HIGH (all findings verified against actual source files in this session)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `ferro-projection` exposes a renderer-agnostic hook seam. The hook is type-erased at the `serde_json::Value` snapshot boundary. `ferro-projection` gains NO dependency on `ferro-json-ui`. One canonical renderer per projection name.
- **D-02:** Additive second broadcast — a distinct `"fragment"` event carrying `{ html }` on the SAME `projection.{name}.{key}` channel. Existing `"delta"` event untouched.
- **D-03:** New `ferro-json-ui/src/runtime/live_fragment.rs` contributing `setupLiveFragments`; `[data-live-fragment]` + `data-channel` container; opens `/_ferro/ws`; swaps innerHTML on `fragment`.
- **D-04:** First-paint with absent snapshot (`read` → `None`) still renders the marked container, bound to empty/default state `{}`. Never omit, never error.
- **D-05:** Reuse the existing `ferro-json-ui` expression/data-binding engine. One render function for both first paint and delta re-render.
- **D-06:** Register `LiveFragment` in `BUILTIN_TYPES` + `BUILTIN_SPECS` and bump the canonical count 52 → 53. Ferry-mcp mirror count deferred to Phase 262.

### Claude's Discretion

- Exact hook ordering: sync inside `apply_event` after the existing delta broadcast vs. second `ferro-events` listener.
- Internal hook-registry types (map shape, keying) and the exact trait/callback signature.
- The `fragment` event name spelling and the `{ html }` payload envelope shape.
- Container element/tag and wrapper classes.
- WebSocket connection sharing/reconnect details in the client runtime.
- Whether `read`-absent binds `{}` vs. a projection-declared default state.

### Deferred Ideas (OUT OF SCOPE)

- Keyed live lists / collection reconciliation.
- Delta-granular fragment patches.
- Multiple distinct fragment templates over the same projection.
- `generation_context` + ferro-mcp mirror count + `docs/src` — Phase 262.
- `asset!()` macro — Phase 261.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LIVE-02 | `LiveFragment` element + projection render hook + client runtime | Covered by all sections below: hook seam (§Architecture Patterns), render path (§Code Examples), transport (§Client Runtime Transport), catalog lockstep (§Don't Hand-Roll) |
</phase_requirements>

---

## Summary

Phase 260 joins two crates that have never been connected: `ferro-projection` (per-key snapshot + delta broadcast) and `ferro-json-ui` (builtin catalog + renderer). The join point is a type-erased render hook in the `ProjectionRuntime<P>`, fired synchronously inside `apply_event` after the existing delta broadcast. The hook serializes `P::State` → `serde_json::Value` (already required by `P::State: Serialize`) and calls a `dyn Fn(Value) -> String` renderer closure registered at app startup.

The `LiveFragment` element is a new builtin in `ferro-json-ui/src/render/containers.rs` (modeled on `render_card`). It reads three props: `projection: String`, `key: String`, and `template_spec: Value` (a child `Spec` JSON blob serialized as a prop). At first-paint time the renderer receives a pre-resolved snapshot `Value` (read async by the handler before calling `render_spec_to_html`) and renders the child spec against it, wrapping the result in a `<div data-live-fragment data-channel="projection.{name}.{key}">` container. The renderer is fully synchronous; async I/O happens upstream.

The re-render hook fires inside `apply_event` (step 6.5, after the existing `"delta"` broadcast, still inside the per-key Mutex), serializes the new state, calls the registered closure, and broadcasts a second `BroadcastMessage { event: "fragment", channel, data: json!({"html": rendered}) }` on the same channel using the already-available `self.broadcaster` handle. No new infrastructure.

The client runtime is a new `ferro-json-ui/src/runtime/live_fragment.rs` SOURCE constant following the exact shape of `sse.rs`. It opens one shared `WebSocket` to `/_ferro/ws`, sends `Subscribe` frames for each `[data-live-fragment]` channel found on `DOMContentLoaded`, and replaces the container's `innerHTML` when a `fragment` event arrives. The WS URL is hardcoded to `/_ferro/ws` (known-stable path, already in `framework/src/server.rs:209`).

**Primary recommendation:** Put the hook registry on `ProjectionRuntime<P>` as a field of type `Option<Arc<dyn Fn(serde_json::Value) + Send + Sync>>`. Register it at startup via a new `with_fragment_renderer` builder method. Fire sync after the existing delta broadcast in `apply_event`. Wire the glue closure in the app bootstrap, capturing an `Arc<ferro_broadcast::Broadcaster>` + the rendered channel name template.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Snapshot read at first paint | API / Backend (handler) | — | `ProjectionRuntime::read` is async; the renderer is sync — snapshot must be resolved upstream |
| Fragment HTML generation (first paint) | API / Backend (renderer) | — | `render_spec_to_html` is the sync server-side render path |
| Fragment HTML generation (on delta) | API / Backend (hook) | — | Hook fires inside `apply_event`; rendering is server-authoritative |
| Delta transport | API / Backend (`ferro-broadcast`) | Browser (WS receive) | Existing `projection.{name}.{key}` channel |
| DOM swap | Browser (client runtime JS) | — | `innerHTML` replace on `fragment` event; no state |
| Channel subscription | Browser (client runtime JS) | — | `ClientMessage::Subscribe` at DOMContentLoaded |
| Builtin catalog registration | API / Backend (`ferro-json-ui`) | — | BUILTIN_TYPES + BUILTIN_SPECS lockstep |

---

## Standard Stack

### Core — all already in-tree, no new dependencies

| Library | Crate | Purpose | Source |
|---------|-------|---------|--------|
| `ferro-projection` | `ferro-projection` | Hook seam, `ProjectionRuntime::apply_event` | [VERIFIED: ferro-projection/src/runtime.rs] |
| `ferro-broadcast` | `ferro-broadcast` | `Broadcaster` + `Broadcast` fluent API | [VERIFIED: ferro-broadcast/src/broadcast.rs] |
| `ferro-json-ui` | `ferro-json-ui` | Builtin catalog + `render_spec_to_html` | [VERIFIED: ferro-json-ui/src/render/mod.rs] |
| `serde_json::Value` | `serde_json` (already in both crates) | Type-erasure boundary for state | [VERIFIED: projection.rs `P::State: Serialize`] |

No new crate dependencies. `ferro-json-ui` does NOT need to depend on `ferro-projection` — the glue is a closure registered at app startup. `ferro-projection/Cargo.toml` already lists `ferro-broadcast` as a direct dependency, so the hook can call `Broadcast::new(self.broadcaster.clone())` directly.

---

## Architecture Patterns

### System Architecture Diagram

```
First paint (HTTP request):
  Handler
    │── runtime.read(&key) ──async──► Option<P::State>
    │── serde_json::to_value(state_or_default)
    │── render_live_fragment(el, spec, &snapshot_value, depth)
    │       └── render child spec against snapshot_value
    │       └── wrap in <div data-live-fragment data-channel="...">
    └── returns HTML in HTTP response

Delta cycle (event dispatch):
  P::Event::dispatch()
    └── ProjectionListener::handle()
          └── ProjectionRuntime::apply_event()
                ├── [steps 1–5: fold + upsert]
                ├── [step 6] Broadcast::delta on projection.{NAME}.{key}
                ├── [step 6.5] if let Some(hook) = &self.fragment_hook {
                │       let value = serde_json::to_value(&state);
                │       let html = (hook)(value);
                │       Broadcast::fragment { html } on same channel
                │   }
                └── [step 7] Mutex released

Client (browser):
  DOMContentLoaded
    └── setupLiveFragments()
          ├── scan [data-live-fragment] → collect channels
          ├── open single WebSocket to /_ferro/ws
          ├── send Subscribe { channel } for each
          └── on message { event: "fragment", data: { html } }
                └── container.innerHTML = html
```

### Recommended Project Structure (changes only)

```
ferro-projection/src/
└── runtime.rs              # add fragment_hook field + with_fragment_renderer() + fire after step 6

ferro-json-ui/src/
├── component.rs            # add LiveFragmentProps struct + #[derive(JsonSchema, Serialize, Deserialize)]
├── render/
│   ├── mod.rs              # add "LiveFragment" to BUILTIN_TYPES + dispatch arm
│   └── containers.rs       # add render_live_fragment()
├── catalog.rs              # add entry to BUILTIN_SPECS + bump count comment 52→53
└── runtime/
    ├── mod.rs              # add mod live_fragment; + s.push_str(live_fragment::SOURCE) + setupLiveFragments in dispatcher
    └── live_fragment.rs    # new: SOURCE const with setupLiveFragments JS
```

### Pattern 1: Hook field on `ProjectionRuntime<P>` (D-01)

The existing struct is in `ferro-projection/src/runtime.rs` at lines 37–42:

```rust
// Source: ferro-projection/src/runtime.rs (verified)
pub struct ProjectionRuntime<P: Projection> {
    pub(crate) db: DatabaseConnection,
    pub(crate) broadcaster: Arc<ferro_broadcast::Broadcaster>,
    pub(crate) projection: P,
    pub(crate) locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
    // ADD:
    pub(crate) fragment_hook: Option<Arc<dyn Fn(serde_json::Value) + Send + Sync>>,
}
```

`ProjectionRuntime::new` gains a default `fragment_hook: None`. A new builder method:

```rust
// ferro-projection/src/runtime.rs — new method
impl<P: Projection> ProjectionRuntime<P> {
    pub fn with_fragment_renderer(
        mut self,
        hook: impl Fn(serde_json::Value) + Send + Sync + 'static,
    ) -> Self {
        self.fragment_hook = Some(Arc::new(hook));
        self
    }
}
```

This keeps `new` unchanged and avoids breaking existing callers.

### Pattern 2: Hook firing inside `apply_event` (Claude's Discretion)

Recommendation: fire sync inside `apply_event` after step 6 (existing delta broadcast), still inside the per-key Mutex. This is simpler than a second `ferro-events` listener and guarantees the hook sees the freshly-persisted state without a race. The per-key Mutex already serializes applies; the hook cost (a `serde_json::to_value` + one render + one broadcast) is acceptable.

```rust
// ferro-projection/src/runtime.rs — inside apply_event, after step 6 (line ~161)
// Step 6.5: fragment re-render hook (D-01, D-02)
if let Some(ref hook) = self.fragment_hook {
    let snapshot_value = serde_json::to_value(&state)
        .unwrap_or(serde_json::Value::Null);
    hook(snapshot_value);
    // Note: hook is responsible for its own broadcast; it captures
    // Arc<Broadcaster> + channel_name template at registration time.
}
```

The hook captures everything it needs at registration time (see Pattern 4 below).

### Pattern 3: `LiveFragmentProps` in `component.rs`

Every builtin follows the `#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]` pattern. The child template is stored as a serialized `Spec` JSON blob:

```rust
// ferro-json-ui/src/component.rs — new struct
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Props for the LiveFragment builtin — binds a child template to a
/// ferro-projection per-key snapshot for server-push in-place re-render.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LiveFragmentProps {
    /// ferro-projection NAME (e.g. "inventory.dashboard").
    pub projection: String,
    /// Per-key channel selector (the `key` part of `projection.{name}.{key}`).
    pub key: String,
    /// Child template spec rendered against the snapshot as its data scope.
    /// Stored as a serde_json::Value (a ferro-json-ui/v2 Spec JSON object).
    pub template: Value,
}
```

Note: the child template as a `Value` prop sidesteps the need for `Element.children` slot-lookup. The renderer deserializes it as a `Spec` at render time. Slot fields array is `&[]` in `BUILTIN_SPECS`.

### Pattern 4: App-side wiring (the glue closure)

The app registers the hook at startup, capturing `Arc<Broadcaster>` + projection name. This is the only place `ferro-json-ui` and `ferro-projection` meet — in app bootstrap code, not in either crate:

```rust
// app/src/bootstrap.rs — example wiring
use ferro_json_ui::{render_spec_to_html, Spec};
use ferro_projection::{ProjectionRuntime, ProjectionKey};
use ferro_broadcast::Broadcaster;
use std::sync::Arc;

let broadcaster = Arc::clone(&broadcaster);
let runtime = Arc::new(
    ProjectionRuntime::new(db.clone(), broadcaster.clone(), MyProjection)
        .with_fragment_renderer(move |snapshot_value: serde_json::Value| {
            // The hook receives the serialized new state.
            // It must re-render the child template and broadcast the fragment.
            // The child template spec is stored by the app (e.g. a static Spec
            // built at startup), OR the hook resolves it from a registry.
            // For v17.0 (one canonical renderer per projection name), the
            // simplest approach: the app captures the child Spec at registration.
            let html = render_spec_to_html(&child_spec, &snapshot_value);
            // Channel: the hook is invoked inside apply_event which already
            // computed channel_name — OR the hook gets channel_name injected
            // at hook-call time (see Pattern 2 refinement below).
            let _ = ferro_broadcast::Broadcast::new(broadcaster.clone())
                .channel(channel_name.clone())  // captured at registration
                .event("fragment")
                .data(serde_json::json!({"html": html}))
                .send();  // spawn for async
        })
);
```

**Key design question (Claude's Discretion):** Does the hook signature receive the channel name and key at call time, or does it capture them? Since one registration is per-projection-name, capturing the channel template (`format!("projection.{}.{{}}", P::NAME)`) and formatting the key at call time is cleaner. This means the hook needs the key as well. Recommend the hook signature:

```rust
pub(crate) fragment_hook: Option<Arc<dyn Fn(&str, serde_json::Value) + Send + Sync>>,
//                                              ^^^  key string
```

The extra `&str` (the key) lets the hook construct the channel name `projection.{NAME}.{key}` correctly per apply. The hook fires as:

```rust
if let Some(ref hook) = self.fragment_hook {
    let snapshot_value = serde_json::to_value(&state).unwrap_or(serde_json::Value::Null);
    hook(key.as_str(), snapshot_value);
}
```

### Pattern 5: `render_live_fragment` in `containers.rs`

The renderer is synchronous. The snapshot is passed in as the `data: &Value` argument (it IS the data scope for the child template):

```rust
// ferro-json-ui/src/render/containers.rs — new function
pub(crate) fn render_live_fragment(
    el: &Element,
    spec: &Spec,
    data: &Value,    // ← snapshot Value passed in by caller
    depth: usize,
) -> String {
    let props: LiveFragmentProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => return format!(
            "<!-- ferro-json-ui: failed to decode LiveFragment props: {} -->",
            html_escape(&e.to_string())
        ),
    };

    // Deserialize the child template spec.
    let child_spec: Spec = match serde_json::from_value(props.template.clone()) {
        Ok(s) => s,
        Err(e) => return format!(
            "<!-- ferro-json-ui: LiveFragment template parse error: {} -->",
            html_escape(&e.to_string())
        ),
    };

    // Render child spec against snapshot as data scope.
    // render_spec_to_html returns a wrapped <div>; we use its inner body.
    let inner_html = render_spec_to_html(&child_spec, data);

    let channel = format!(
        "projection.{}.{}",
        html_escape(&props.projection),
        html_escape(&props.key)
    );

    format!(
        "<div data-live-fragment data-channel=\"{channel}\">{inner_html}</div>"
    )
}
```

**D-04 compliance:** The caller passes `data` as the pre-resolved snapshot. When `ProjectionRuntime::read` returns `None`, the handler passes `serde_json::Value::Object(Default::default())` (empty object `{}`). The container is always rendered.

**D-05 compliance:** The existing `render_spec_to_html(child_spec, snapshot_value)` call IS the expression/data-binding engine — `resolve_expressions` is called by the handler pipeline, and the binding vocabulary (`$data`, `$template`) works identically because the snapshot is the `data` root.

### Pattern 6: `setupLiveFragments` client runtime

Model on `sse.rs` exactly. The WS URL is the hardcoded known path `/_ferro/ws`. No `data-ws-url` body attribute is needed (unlike SSE which reads `data-sse-url` — the WS endpoint is always the same fixed path). The `ServerMessage` shape uses `#[serde(tag = "type")]` — the `Event` variant carries `BroadcastMessage { event, channel, data }`.

```javascript
// ferro-json-ui/src/runtime/live_fragment.rs — SOURCE constant

    // ── LiveFragment WebSocket subscriptions ──────────────────────────────────

    function setupLiveFragments() {
        var fragments = document.querySelectorAll('[data-live-fragment]');
        if (!fragments.length) return;

        // One shared socket for all fragments on the page.
        var ws = new WebSocket(
            (location.protocol === 'https:' ? 'wss://' : 'ws://') +
            location.host + '/_ferro/ws'
        );

        ws.addEventListener('open', function() {
            for (var i = 0; i < fragments.length; i++) {
                var ch = fragments[i].getAttribute('data-channel');
                if (ch) {
                    ws.send(JSON.stringify({ type: 'subscribe', channel: ch }));
                }
            }
        });

        ws.addEventListener('message', function(e) {
            try {
                var msg = JSON.parse(e.data);
                // ServerMessage::Event wraps BroadcastMessage with type: "event"
                if (msg.type === 'event' && msg.event === 'fragment' && msg.data && msg.data.html) {
                    var target = document.querySelector(
                        '[data-live-fragment][data-channel="' + msg.channel + '"]'
                    );
                    if (target) { target.innerHTML = msg.data.html; }
                }
            } catch (_) {}
        });

        ws.addEventListener('error', function() {
            // tungstenite will close; browser will not auto-reconnect a WS.
            // Reconnect logic deferred to Phase 262 / future work (D-03 posture).
        });
    }
```

**ServerMessage wire shape** (from `ferro-broadcast/src/message.rs` lines 82–95): `ServerMessage` uses `#[serde(tag = "type", rename_all = "snake_case")]`. The `Event(BroadcastMessage)` variant serializes as `{ "type": "event", "event": "...", "channel": "...", "data": {...} }`. The JS checks `msg.type === 'event'` to distinguish from `subscribed`, `connected`, `pong`, etc.

**WS URL construction:** The SSE module reads `data-sse-url` from `document.body` because SSE URLs are app-specific. The WS endpoint `/_ferro/ws` is framework-fixed (registered at `framework/src/server.rs:209`), so `setupLiveFragments` can construct it from `location.host` without a body attribute.

### Pattern 7: Runtime assembly changes in `mod.rs`

Three edit sites in `ferro-json-ui/src/runtime/mod.rs`:

1. `mod live_fragment;` declaration (after the existing mod list)
2. `s.push_str(live_fragment::SOURCE);` in `FERRO_RUNTIME_JS` lazy block (add after `hero_lazy`)
3. `setupLiveFragments,` in the `setups` array inside `ferroRuntime()`

The existing `bundle_contains_all_setup_functions` test (lines 196–221) must be updated to include `"setupLiveFragments"`. The `dispatcher_invokes_every_setup` test (lines 233–264) must include it too.

### Pattern 8: `BUILTIN_TYPES` and `BUILTIN_SPECS` lockstep (D-06)

Two edit sites:

**`ferro-json-ui/src/render/mod.rs`** — `BUILTIN_TYPES` (lines 44–101). `LiveFragment` is a container-category element. Add after `SelectionPanel` (last container entry):

```rust
"SelectionPanel",
// Live reactive primitive (containers.rs / Phase 260)
"LiveFragment",
```

And dispatch arm after the `"SelectionPanel"` arm:

```rust
"LiveFragment" => containers::render_live_fragment(el, spec, data, depth),
```

**`ferro-json-ui/src/catalog.rs`** — `BUILTIN_SPECS` (lines 126–446). Add after the `SelectionPanel` entry:

```rust
(
    "LiveFragment",
    "Binds a child template to a ferro-projection per-key snapshot; re-renders in place on each delta via server-push HTML over the ferro-broadcast WebSocket.",
    || to_value(schema_for!(LiveFragmentProps)).unwrap(),
    &[],
),
```

**Count drift guard** (`catalog.rs` line 1296): change `52` → `53` and update the history comment:

```rust
// → 52 (SelectionPanel) → 53 (LiveFragment).
assert_eq!(crate::render::BUILTIN_TYPES.len(), 53);
```

**`catalog.rs` import** (lines 29–40): add `LiveFragmentProps` to the component import list.

### Anti-Patterns to Avoid

- **Async renderer:** `render_element` (and all of `render_spec_to_html`) is synchronous. Do NOT make it async. Resolve the snapshot upstream, before calling the renderer.
- **Adding `ferro-json-ui` as a dependency of `ferro-projection`:** Violates the renderer-location rule and D-01. The glue is a plain `dyn Fn` closure, owned by the app.
- **A new crate for the bridge:** Violates the v17.0 no-new-crates constraint.
- **Child template in `Element.children`:** `Element.children` is a `Vec<String>` of IDs into the same `Spec`. A `LiveFragment` needs its own sub-spec for independent re-rendering by the hook. Store the child template as a `Spec` JSON blob in `LiveFragmentProps.template: Value`.
- **Rendering inside the mutex (long-held):** The hook fires inside the per-key Mutex. Rendering should be fast (sync, in-memory). Avoid IO inside the hook. If the child spec deserialization is slow, cache the parsed `Spec` in the closure capture.
- **One socket per fragment:** All fragments on a page share one WebSocket to avoid connection multiplication.
- **Calling `send()` on the hook's broadcast inside an async context without `.await`:** The hook signature is sync (`Fn(…)` not `async Fn`). Use `tokio::spawn` inside the closure to drive the async broadcast without blocking the Mutex holder, OR — simpler — use `Broadcaster::send_direct` if a sync send path exists. If not, `tokio::spawn` inside the `Fn` closure is the correct pattern.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Type-erased render hook | Custom vtable / trait object gymnastics | `Arc<dyn Fn(&str, Value) + Send + Sync>` | Standard Rust pattern; `Fn` trait is dyn-safe; already established for closures in this codebase |
| WebSocket transport | New WS endpoint | `/_ferro/ws` (`framework/src/websocket.rs`) | Already handles Subscribe/Unsubscribe/broadcast; adding a new endpoint would duplicate 200 lines |
| Broadcast on delta | Custom mpsc channel | `ferro_broadcast::Broadcast::new(…).channel(…).event(…).data(…).send().await` | Existing fluent API; already used in `apply_event` step 6 |
| JSON binding in child template | New binding syntax | `render_spec_to_html(child_spec, snapshot_value)` using existing `expression.rs` | No duplicate control surface; `$data`/`$template` vocab works unchanged |
| Count assertions | Ad-hoc count test | Update `assert_eq!(crate::render::BUILTIN_TYPES.len(), 53)` at `catalog.rs:1296` | The existing drift guard is the SINGLE source of truth; everything else is relational |

**Key insight:** The entire stack (broadcast, WS, render, binding engine) already exists. Phase 260 is assembly work: wiring the hook seam and implementing `render_live_fragment`.

---

## Common Pitfalls

### Pitfall 1: Sync hook calling an async `send()`

**What goes wrong:** `apply_event` fires the hook synchronously. The hook needs to call `Broadcast::send().await`. Calling `.await` inside a sync `Fn` closure is a compile error.

**Why it happens:** The hook signature is `dyn Fn(…)` (sync) but the send path is async.

**How to avoid:** Two clean options:
1. Use `tokio::spawn(async move { Broadcast::new(…).send().await; })` inside the closure — fire-and-forget, non-blocking.
2. Use `broadcaster.try_send_direct(message)` if `Broadcaster` exposes a sync path (check `ferro-broadcast/src/broadcaster.rs`). If not, option 1 is the implementation.

**Warning signs:** Compiler error "async operation in sync context" or a blocking `.block_on` call inside `apply_event`.

### Pitfall 2: Child spec deserialization on every delta

**What goes wrong:** `render_live_fragment` deserializes `props.template` into a `Spec` on every call. The hook also does this on every delta event. Repeated `serde_json::from_value` on a large spec is not free.

**How to avoid:** The hook closure (registered at startup) should capture a pre-parsed `Arc<Spec>` rather than deserializing on every delta. For first-paint, the renderer deserializes once per HTTP request.

**Warning signs:** CPU spike on high-frequency projection updates.

### Pitfall 3: `data-channel` attribute injection for first-paint

**What goes wrong:** If `render_live_fragment` emits the channel as `data-channel="projection.inventory.dashboard.{key}"` but the key contains characters unsafe for an HTML attribute (e.g. `&`, `<`, `"`), the JS selector `querySelector('[data-channel="…"]')` fails silently.

**How to avoid:** Pass `props.key` through `html_escape()` when emitting the attribute. The JS side reads `getAttribute('data-channel')` — the browser auto-unescapes HTML entities, so the raw value lands correctly in `ws.send(JSON.stringify({ channel: ch }))`.

### Pitfall 4: BUILTIN_TYPES / BUILTIN_SPECS count drift guard fails build

**What goes wrong:** Adding `LiveFragment` to `BUILTIN_TYPES` without adding it to `BUILTIN_SPECS` (or vice versa) causes `Catalog::build()` to return `Err` with the message "BUILTIN_SPECS has N entries but BUILTIN_TYPES has M". This panics `global_catalog()` at first call.

**How to avoid:** Edit both in lockstep; bump the count in `builtin_types_count_drift_guard` from 52 → 53 in the same commit.

**Warning signs:** `cargo test --all-features` fails with `catalog build failed` or the count assertion at `catalog.rs:1296` fires.

### Pitfall 5: `LiveFragmentProps` not imported in `catalog.rs`

**What goes wrong:** `catalog.rs` imports all Props structs at the top (lines 29–40). Forgetting to add `LiveFragmentProps` to the import list causes a compile error when the `schema_for!(LiveFragmentProps)` call in `BUILTIN_SPECS` is added.

**How to avoid:** Add `LiveFragmentProps` to the `use crate::component::{...}` import in `catalog.rs` in the same edit that adds the `BUILTIN_SPECS` entry.

### Pitfall 6: runtime `bundle_contains_all_setup_functions` / `dispatcher_invokes_every_setup` tests

**What goes wrong:** Both tests in `ferro-json-ui/src/runtime/mod.rs` enumerate all `setup*` names explicitly. Adding `setupLiveFragments` to the assembly without updating these tests causes test failures.

**How to avoid:** Add `"setupLiveFragments"` to both arrays in the tests (lines 199–213 and 237–254) in the same edit that modifies `mod.rs`.

### Pitfall 7: `ServerMessage::Event` wire shape in JS

**What goes wrong:** The JS runtime checks `msg.type === 'event'` to identify broadcast events. `ferro-broadcast`'s `ServerMessage` uses `#[serde(tag = "type", rename_all = "snake_case")]` — the `Event(BroadcastMessage)` variant serializes `type: "event"` (snake_case). If the JS checks the wrong type string (e.g. `"Event"` with capital E), fragment swaps silently fail.

**How to avoid:** Confirmed from `ferro-broadcast/src/message.rs:83-84`: the tag is `"type"` and rename is `snake_case`. `Event` → `"event"` (lowercase). The BroadcastMessage fields (`event`, `channel`, `data`) are flat because `Event(BroadcastMessage)` uses tuple variant — serde flattens them into the outer object alongside `type`.

**Warning signs:** Fragment swaps never happen in the browser; logging `msg` in the WS handler shows `type: "event"` correctly but the condition fails.

---

## Code Examples

### Full `apply_event` hook insertion point

```rust
// Source: ferro-projection/src/runtime.rs (verified lines 92–165)
// Insert between current step 6 and the closing Ok(())

// Step 6: broadcast delta (existing, unchanged)
let channel_name = format!("projection.{}.{}", P::NAME, key.as_str());
let event_name = self.projection.broadcast_event_name();
let send_result = ferro_broadcast::Broadcast::new(self.broadcaster.clone())
    .channel(channel_name.clone())
    .event(event_name)
    .data(delta)
    .send()
    .await;

if let Err(e) = send_result {
    tracing::warn!(error = %e, channel = %channel_name, "projection broadcast failed");
    return Err(ProjectionError::from(e));
}

// Step 6.5: fragment re-render hook (NEW — D-01, D-02)
if let Some(ref hook) = self.fragment_hook {
    let snapshot_value = serde_json::to_value(&state)
        .unwrap_or(serde_json::Value::Null);
    hook(key.as_str(), snapshot_value);
    // hook is responsible for its own async broadcast via tokio::spawn internally
}

// Step 7: Mutex released on drop of `_guard`
Ok(())
```

### `LiveFragmentProps` JsonSchema smoke test (component.rs pattern)

```rust
// Follow the pattern from catalog.rs tests::schema_smoke_tests (verified lines 1558–1601)
#[test]
fn live_fragment_props_schema_is_nonempty() {
    assert_schema_nonempty_object::<LiveFragmentProps>("LiveFragmentProps");
}
```

### First-paint handler pattern (caller's responsibility)

```rust
// In a ferro handler, before rendering a Spec that contains a LiveFragment:
let key = ProjectionKey::new("warehouse-a");
let snapshot: serde_json::Value = match runtime.read(&key).await? {
    Some(state) => serde_json::to_value(state)?,
    None => serde_json::Value::Object(Default::default()),  // D-04: empty object
};
// Inject snapshot into spec.data so the LiveFragment renderer sees it
// as the data scope for the child template.
// OR: pass it through a render-context extension (see Pattern 5 notes).
```

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| App hand-wires WS outside JSON-UI | `LiveFragment` builtin makes it agent-composable | `json_ui_catalog` / `generation_context` can now express reactive fragments |
| `render_spec_to_html` always sync | Still sync; async resolved upstream | No change to renderer contract |
| `ProjectionRuntime::new(db, broadcaster, projection)` | Same + optional `.with_fragment_renderer(hook)` | Backward-compatible; existing callers unchanged |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `Broadcaster` has no sync send path; hook needs `tokio::spawn` for the async `send()` | Pitfall 1 / Pattern 4 | If a sync path exists, no spawn needed; simpler code |
| A2 | `ServerMessage::Event(BroadcastMessage)` serializes to `{"type":"event","event":"...","channel":"...","data":{...}}` with fields flattened | Pattern 6 (JS) | If not flattened, JS needs `msg.data.event` instead of `msg.event` |
| A3 | `render_spec_to_html` is the right entry point for child-spec rendering (vs. calling `render_element` directly) | Pattern 5 | If the child spec needs the outer `<div class="flex flex-wrap...">` wrapper stripped, the caller must strip it |

All three are LOW-risk; A2 is directly verifiable from `ferro-broadcast/src/message.rs` lines 82–95 and the `ServerMessage::to_json` impl (both verified). A1 can be confirmed with a quick grep of `ferro-broadcast/src/broadcaster.rs`.

---

## Open Questions (RESOLVED)

All three were confirmed against the live code during pattern mapping (see `260-PATTERNS.md`)
and are reflected in the plans. None remain open.

1. **Broadcaster sync send path** — **RESOLVED: no sync path; use `tokio::spawn`.**
   - What we know: `Broadcaster::send` (internal) is async. `ferro_broadcast::Broadcast::send()` is async.
   - Resolution: `ferro-broadcast/src/broadcast.rs` (line ~77) confirms there is **no** `try_broadcast`/sync
     variant. The hook closure drives the `fragment` broadcast via
     `tokio::spawn(async move { … .send().await })` (PATTERNS.md §"No sync broadcast path", line ~466).
     Applied in Plan 01 (builder rustdoc) and Plan 04 (E2E test).

2. **Child template data scope: `spec.data` injection vs. `data` parameter** — **RESOLVED: pass as the `data` argument.**
   - What we know: `render_spec_to_html(spec, data)` passes `data` separately; `resolve_expressions`
     routes data-binding through this parameter.
   - Resolution: inject the snapshot as the `data` argument to `render_spec_to_html` — avoids mutating
     the `Arc<Spec>` cache. Applied in Plan 02 (`render_live_fragment` calls
     `super::render_spec_to_html(&child_spec, data)`).

3. **`render_spec_to_html` outer wrapper** — **RESOLVED: accept the wrapper as-is for v17.0.**
   - What we know: `render_spec_to_html` wraps output in `<div class="flex flex-wrap gap-4 [&>*]:w-full ...">...</div>`
     (render/mod.rs lines ~115–119).
   - Resolution: accept the flex wrapper for v17.0 (keeps the render path unchanged; the marked
     `data-live-fragment` container wraps it). Revisit only if a consumer hits a layout issue. Applied
     in Plan 02 / Plan 04 (tests assert on `data-live-fragment` + child content, not wrapper absence).

---

## Environment Availability

Step 2.6: SKIPPED — phase is purely in-tree code changes; no new external tools, databases, or CLI utilities required beyond the existing Rust toolchain and workspace.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `#[tokio::test]` |
| Config file | None — `cargo test` discovers by convention |
| Quick run command | `cargo test -p ferro-json-ui live_fragment` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LIVE-02-SC1 | `LiveFragment` renders current snapshot to HTML on first paint | Unit | `cargo test -p ferro-json-ui render_live_fragment` | ❌ Wave 0 |
| LIVE-02-SC2 | `event → ProjectionListener → delta` broadcasts re-rendered fragment HTML on `projection.{name}.{key}` | Integration | `cargo test -p ferro-projection live_fragment_hook` | ❌ Wave 0 |
| LIVE-02-SC3 | Client runtime adds no WASM and no client-side state | Static assertion | `cargo test -p ferro-json-ui live_fragment_no_wasm` | ❌ Wave 0 |
| LIVE-02-SC4 | Exactly one binding pattern ships (per-key snapshot) | Static assertion / doc | Verified by absence of list-reconciliation code | ❌ Wave 0 |
| D-04 | Absent snapshot renders empty container (never errors) | Unit | `cargo test -p ferro-json-ui render_live_fragment_absent_snapshot` | ❌ Wave 0 |
| D-06 | `BUILTIN_TYPES.len() == 53` | Drift guard | `cargo test -p ferro-json-ui builtin_types_count_drift_guard` | ❌ Wave 0 (update existing) |
| D-06 | `BUILTIN_SPECS.len() == BUILTIN_TYPES.len()` | Drift guard | `cargo test -p ferro-json-ui` (in `Catalog::build`) | ❌ Wave 0 (update existing) |

### Key Test Patterns (verified from existing codebase)

**First-paint render test** — model on `render/containers.rs` tests:

```rust
#[test]
fn render_live_fragment_renders_container_with_channel() {
    use crate::spec::{Element, Spec};
    use serde_json::json;
    // Build a minimal child spec
    let child_spec = Spec::builder()
        .element("content", Element::new("Text").prop("content", "hello"))
        .build()
        .expect("child spec");
    let template = serde_json::to_value(&child_spec).expect("serialize");

    let spec = Spec::builder()
        .element("root", Element::new("LiveFragment")
            .prop("projection", "inventory.dashboard")
            .prop("key", "warehouse-a")
            .prop("template", template))
        .build()
        .expect("spec");

    let snapshot = json!({"total": 42});
    let html = render_spec_to_html(&spec, &snapshot);
    assert!(html.contains("data-live-fragment"), "container must carry data-live-fragment");
    assert!(html.contains("data-channel=\"projection.inventory.dashboard.warehouse-a\""));
    assert!(html.contains("hello"), "child template must render");
}

#[test]
fn render_live_fragment_absent_snapshot_renders_container() {
    // D-04: empty object renders without error
    let html = render_spec_to_html(&spec, &json!({}));
    assert!(html.contains("data-live-fragment"));
    // must NOT contain an error comment
    assert!(!html.contains("<!-- ferro-json-ui:"));
}
```

**Integration test** — model on `ferro-projection/src/runtime.rs` tests (e.g. `apply_event_initial_writes_version_1`):

```rust
#[tokio::test]
async fn fragment_hook_fires_after_apply_event() {
    use std::sync::{Arc, Mutex};
    let received: Arc<Mutex<Vec<(String, serde_json::Value)>>> = Arc::new(Mutex::new(Vec::new()));
    let received_clone = received.clone();

    let rt = ProjectionRuntime::new(conn, broadcaster, CounterProjection)
        .with_fragment_renderer(move |key: &str, snapshot: serde_json::Value| {
            received_clone.lock().unwrap().push((key.to_string(), snapshot));
        });

    rt.apply_event(&CounterEvent { delta: 5 }).await.expect("apply");

    let calls = received.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "default-key");
    assert_eq!(calls[0].1["total"], 5);
}
```

**No-WASM assertion** (static source check, model on `runtime/mod.rs` tests):

```rust
#[test]
fn live_fragment_runtime_contains_no_wasm() {
    // SC-3: no WASM, no client-side state
    let src = super::live_fragment::SOURCE;
    assert!(!src.contains("WebAssembly"), "no WASM");
    assert!(!src.contains("useState"), "no React/signal state");
    assert!(src.contains("setupLiveFragments"));
    assert!(src.contains("data-live-fragment"));
    assert!(src.contains("/_ferro/ws"));
    assert!(src.contains("innerHTML"));
}
```

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-json-ui && cargo test -p ferro-projection`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-json-ui/src/component.rs` — `LiveFragmentProps` struct
- [ ] `ferro-json-ui/src/render/containers.rs` — `render_live_fragment` function + unit tests
- [ ] `ferro-json-ui/src/runtime/live_fragment.rs` — `SOURCE` const + JS tests
- [ ] `ferro-projection/src/runtime.rs` — `fragment_hook` field + `with_fragment_renderer` + hook invocation + integration test

*(No new test files needed — all additions land inside existing files.)*

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | — |
| V3 Session Management | No | — |
| V4 Access Control | Partial | Channel authorization is handled by `ferro-broadcast`'s existing `subscribe` auth check; `LiveFragment` consumers must set channel auth if fragments contain user-specific data |
| V5 Input Validation | Yes | `html_escape()` on `props.projection`, `props.key`, and any snapshot values interpolated into HTML; child spec validated by `Catalog::validate` before use |
| V6 Cryptography | No | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via snapshot data injected into innerHTML | Tampering | Server renders HTML — all data passes through `html_escape()` in the renderer; no unsanitized data reaches innerHTML |
| Channel spoofing (subscribe to private projection) | Elevation of Privilege | `ferro-broadcast` `subscribe(socket_id, channel, auth)` already validates auth tokens; `LiveFragment` channel names follow `projection.{name}.{key}` — scope is user-visible and not secret |
| Unbounded fragment re-render on high-frequency delta | Denial of Service | Mitigation: the hook fires inside the per-key Mutex, so same-key apply serializes; the render cost is bounded by the child spec size. High-frequency projections should throttle at the application layer |

---

## Sources

### Primary (HIGH confidence — all verified against source files in this session)

- `ferro-projection/src/runtime.rs` — `ProjectionRuntime<P>` struct, `apply_event` 7-step sequence, `read` signature, test harness pattern
- `ferro-projection/src/projection.rs` — `Projection` trait: `State: Serialize + Default`, `Delta: Serialize`, `NAME`, `broadcast_event_name()` default `"delta"`
- `ferro-projection/src/listener.rs` — `ProjectionListener<P>` as `ferro_events::Listener<P::Event>`
- `ferro-json-ui/src/render/mod.rs` — `BUILTIN_TYPES` (52 entries, lines 44–101), `render_element` dispatch, `render_spec_to_html` signature (sync)
- `ferro-json-ui/src/render/containers.rs` — `render_card` pattern for container renderers
- `ferro-json-ui/src/catalog.rs` — `BUILTIN_SPECS` (52 entries), drift guard at line 1296 (`assert_eq!(..., 52)`), `Catalog::build` runtime check
- `ferro-json-ui/src/runtime/mod.rs` — `FERRO_RUNTIME_JS` assembly, `ferroRuntime()` dispatcher, test assertions
- `ferro-json-ui/src/runtime/sse.rs` — `SOURCE` constant shape as prior-art for `live_fragment.rs`
- `ferro-broadcast/src/message.rs` — `BroadcastMessage`, `ClientMessage::Subscribe`, `ServerMessage` (`#[serde(tag = "type", rename_all = "snake_case")]`)
- `ferro-broadcast/src/broadcast.rs` — `Broadcast::new(broadcaster).channel().event().data().send().await` fluent API
- `framework/src/websocket.rs` — `/_ferro/ws` upgrade path, `handle_client_message` Subscribe handling
- `ferro-json-ui/src/layout.rs` — `data-sse-url` injection pattern (body attr), confirming WS has no equivalent body attr today
- `ferro-json-ui/Cargo.toml` — no `ferro-projection` dependency; `projections` feature = `ferro-projections` (plural, unrelated)
- `ferro-projection/Cargo.toml` — already depends on `ferro-broadcast`

### Secondary (MEDIUM confidence)

- Design spec: `docs/superpowers/specs/2026-07-21-live-projection-surface-design.md` — authoritative contract, alternatives, non-goals
- CONTEXT.md: `.planning/phases/260-live-reactive-fragment/260-CONTEXT.md` — locked decisions D-01 through D-06

---

## Metadata

**Confidence breakdown:**
- Hook seam mechanics: HIGH — verified against actual `apply_event` source
- Render path: HIGH — verified `render_spec_to_html` is sync, `render_card` pattern is exact prior-art
- Catalog lockstep: HIGH — verified BUILTIN_TYPES length (52), drift guard line (1296), import list
- Client runtime: HIGH — verified `ServerMessage` wire shape, WS path, SSE prior-art pattern
- Transport ordering: HIGH — `Broadcast::new(self.broadcaster.clone())` is the same pattern already in step 6

**Research date:** 2026-07-26
**Valid until:** 2026-08-26 (stable Rust codebase; no external dependencies changing)
