# Phase 260: Live reactive fragment — Pattern Map

**Mapped:** 2026-07-26
**Files analyzed:** 7 new/modified files
**Analogs found:** 7 / 7

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-projection/src/runtime.rs` | service | event-driven | itself (extend existing `apply_event`) | exact |
| `ferro-json-ui/src/component.rs` | model/props | transform | `StreamTextProps` (new props struct on an existing props file) | exact |
| `ferro-json-ui/src/render/containers.rs` | renderer | request-response | `render_card` (container with child rendering + props decode) | exact |
| `ferro-json-ui/src/render/mod.rs` | config/dispatch | request-response | the existing `"SelectionPanel"` arm + `BUILTIN_TYPES` slice | exact |
| `ferro-json-ui/src/catalog.rs` | config | transform | the existing `SelectionPanel` BUILTIN_SPECS entry + count guard | exact |
| `ferro-json-ui/src/runtime/live_fragment.rs` (CREATE) | utility | event-driven | `ferro-json-ui/src/runtime/sse.rs` (`SOURCE` const + `setup*` function) | exact |
| `ferro-json-ui/src/runtime/mod.rs` | config/assembly | transform | itself (add `mod`, `push_str`, add to dispatcher and tests) | exact |

---

## Pattern Assignments

### `ferro-projection/src/runtime.rs` — extend `ProjectionRuntime<P>`

**Analog:** itself (the current struct and `apply_event`)

**Existing struct shape** (`runtime.rs` lines 37–42):
```rust
pub struct ProjectionRuntime<P: Projection> {
    pub(crate) db: DatabaseConnection,
    pub(crate) broadcaster: Arc<ferro_broadcast::Broadcaster>,
    pub(crate) projection: P,
    pub(crate) locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
}
```

**Add one field** — place after `locks`:
```rust
    pub(crate) fragment_hook: Option<Arc<dyn Fn(&str, serde_json::Value) + Send + Sync>>,
```

**`new` fix** — add `fragment_hook: None` to the struct literal inside `new` (`runtime.rs` lines 53–59). Existing constructor is:
```rust
Self {
    db,
    broadcaster,
    projection,
    locks: DashMap::new(),
}
```
Becomes:
```rust
Self {
    db,
    broadcaster,
    projection,
    locks: DashMap::new(),
    fragment_hook: None,
}
```

**New builder method** — add to the `impl<P: Projection> ProjectionRuntime<P>` block after `new`:
```rust
/// Register a renderer-agnostic re-render hook fired after each `apply_event`
/// step 6 broadcast (D-01, D-02). The hook receives the key and the newly
/// persisted state serialized as `serde_json::Value`. It is responsible for
/// its own async broadcast (typically via `tokio::spawn` — see Pitfall 1).
pub fn with_fragment_renderer(
    mut self,
    hook: impl Fn(&str, serde_json::Value) + Send + Sync + 'static,
) -> Self {
    self.fragment_hook = Some(Arc::new(hook));
    self
}
```

**Hook firing inside `apply_event`** — insert between the existing step-6 broadcast error check and the final `Ok(())` (`runtime.rs` lines 154–164). The existing step-6 end:
```rust
        if let Err(e) = send_result {
            tracing::warn!(
                error = %e,
                channel = %channel_name,
                "projection broadcast failed; snapshot persisted"
            );
            return Err(ProjectionError::from(e));
        }

        // Step 7: Mutex released on drop of `_guard` after this return
        Ok(())
```
Insert between the `if let Err` block and `Ok(())`:
```rust
        // Step 6.5: fragment re-render hook (D-01, D-02)
        if let Some(ref hook) = self.fragment_hook {
            let snapshot_value = serde_json::to_value(&state)
                .unwrap_or(serde_json::Value::Null);
            hook(key.as_str(), snapshot_value);
            // hook captures Arc<Broadcaster> + channel template; drives its
            // own async broadcast via tokio::spawn internally.
        }
```

**Integration test pattern** — model on existing tests in `runtime.rs` (`fresh_runtime` at line 350, `apply_event_initial_writes_version_1` at line 368):
```rust
#[tokio::test]
async fn fragment_hook_fires_after_apply_event() {
    use std::sync::{Arc, Mutex};
    let received: Arc<Mutex<Vec<(String, serde_json::Value)>>> =
        Arc::new(Mutex::new(Vec::new()));
    let received_clone = received.clone();

    let rt = fresh_runtime(CounterProjection).await
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

---

### `ferro-json-ui/src/component.rs` — `LiveFragmentProps` struct

**Analog:** `StreamTextProps` (`component.rs` lines 728–740) — props struct with doc comment + derive block + `serde_json::Value` field pattern

**Derive block to copy** (from `StreamTextProps`):
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StreamTextProps {
    #[serde(default)]
    pub sse_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    ...
}
```

**New struct** — add after `StreamTextProps` or at end of the atoms section:
```rust
/// Props for the `LiveFragment` builtin — binds a child template to a
/// `ferro-projection` per-key snapshot for server-push in-place re-render.
///
/// First paint: the handler resolves `projection` + `key` via
/// `ProjectionRuntime::read`, serializes the state (or uses `{}` when absent),
/// and passes the `Value` as the data scope for `template`.
///
/// On delta: the registered fragment hook re-renders `template` against the
/// new snapshot and broadcasts `{ html }` on the same
/// `projection.{name}.{key}` channel. The client runtime swaps `innerHTML`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LiveFragmentProps {
    /// ferro-projection NAME — the `Projection::NAME` const of the target projection.
    #[serde(default)]
    pub projection: String,
    /// Per-key channel selector (the `key` segment of `projection.{name}.{key}`).
    #[serde(default)]
    pub key: String,
    /// Child template spec rendered against the snapshot as its data scope.
    /// Store as a `serde_json::Value` encoding a valid ferro-json-ui `Spec`.
    pub template: serde_json::Value,
}
```

Note: `PartialEq + Eq` are not derived because `serde_json::Value` implements `PartialEq` but not `Eq`. Drop to `#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]` (no `Eq`).

---

### `ferro-json-ui/src/render/containers.rs` — `render_live_fragment`

**Analog:** `render_card` (`containers.rs` lines 39–138) — the canonical container-with-props-decode pattern

**Props decode pattern** (copy verbatim from `render_card` lines 40–48):
```rust
pub(crate) fn render_live_fragment(
    el: &Element,
    spec: &Spec,
    data: &Value,
    depth: usize,
) -> String {
    let props: LiveFragmentProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode LiveFragment props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };
```

**`html_escape` import** — already in scope at top of `containers.rs` line 30: `use super::{html_escape, render_element};`

**`LiveFragmentProps` import** — add to the `use crate::component::{...}` block at `containers.rs` lines 15–21.

**Core render body** — after props decode:
```rust
    // Deserialize the child template spec.
    let child_spec: crate::spec::Spec = match serde_json::from_value(props.template.clone()) {
        Ok(s) => s,
        Err(e) => return format!(
            "<!-- ferro-json-ui: LiveFragment template parse error: {} -->",
            html_escape(&e.to_string())
        ),
    };

    // data IS the snapshot Value; render_spec_to_html routes it through the
    // existing expression/data-binding engine (D-05).
    let inner_html = super::render_spec_to_html(&child_spec, data);

    let channel = format!(
        "projection.{}.{}",
        html_escape(&props.projection),
        html_escape(&props.key)
    );

    format!(r#"<div data-live-fragment data-channel="{channel}">{inner_html}</div>"#)
}
```

D-04 compliance: the caller passes `data` — when the snapshot is absent the handler passes `serde_json::Value::Object(Default::default())`. The container is always rendered; `render_live_fragment` itself never reads from the DB.

**Unit tests** — add at the bottom of `containers.rs` in a `#[cfg(test)]` block, modeling on the existing test structure in the file. Key assertions:
- `html.contains("data-live-fragment")`
- `html.contains(r#"data-channel="projection.inventory.dashboard.warehouse-a""#)`
- `html.contains("hello")` (child template rendered)
- Absent snapshot (`json!({})`) still emits the container without an error comment

---

### `ferro-json-ui/src/render/mod.rs` — `BUILTIN_TYPES` + dispatch arm

**Analog:** `"SelectionPanel"` entry and arm (lines 88, 226)

**BUILTIN_TYPES edit** (`render/mod.rs` lines 86–101) — add after `"SelectionPanel"`:
```rust
    "SelectionPanel",
    // Live reactive primitive — ferro-projection per-key snapshot binding (Phase 260)
    "LiveFragment",
    // Form controls (form.rs)
```

**Dispatch arm** (`render/mod.rs` line 226, after the `SelectionPanel` arm):
```rust
        "SelectionPanel" => containers::render_selection_panel(el, spec, data, depth),
        "LiveFragment" => containers::render_live_fragment(el, spec, data, depth),
```

`BUILTIN_TYPES` currently has 52 entries (lines 44–101). After adding `"LiveFragment"` it will have 53 — which is what the count guard tests verify.

---

### `ferro-json-ui/src/catalog.rs` — `BUILTIN_SPECS` entry + count bump + import

**Analog:** `SelectionPanel` entry (`catalog.rs` lines 375–380)

**Existing SelectionPanel entry** (exact form to copy):
```rust
    (
        "SelectionPanel",
        "Live client-side view of the register form state: ...",
        || to_value(schema_for!(SelectionPanelProps)).unwrap(),
        &[],
    ),
```

**New entry** — add immediately after the `SelectionPanel` tuple, before the `// === Form controls` comment:
```rust
    (
        "LiveFragment",
        "Binds a child template to a ferro-projection per-key snapshot; re-renders \
         in place on each delta via server-push HTML over the ferro-broadcast WebSocket.",
        || to_value(schema_for!(LiveFragmentProps)).unwrap(),
        &[],
    ),
```

**Import edit** (`catalog.rs` lines 29–40) — add `LiveFragmentProps` to the `use crate::component::{...}` block. Current last line of block:
```rust
    ToastProps,
```
Becomes:
```rust
    LiveFragmentProps, ToastProps,
```

**Count drift guard** (`catalog.rs` lines 1292–1296):
```rust
        // History: ... → 52 (SelectionPanel) → 53 (LiveFragment).
        assert_eq!(crate::render::BUILTIN_TYPES.len(), 53);
```
Change `52` to `53` and append `→ 53 (LiveFragment)` to the history comment.

The `catalog.rs` line 2222 comment also says `// SelectionPanel added — 52 components total)` — update to `53` if present in the test that references it.

---

### `ferro-json-ui/src/runtime/live_fragment.rs` (CREATE)

**Analog:** `ferro-json-ui/src/runtime/sse.rs` (entire file — the `pub(super) const SOURCE: &str` pattern)

**Exact SOURCE const shape from `sse.rs`** (lines 1–56):
```rust
pub(super) const SOURCE: &str = r#"
    // ── SSE connection ─────...

    function setupSSE() {
        var sseUrl = document.body && document.body.getAttribute('data-sse-url');
        if (!sseUrl) return;
        var es = new EventSource(sseUrl);
        es.onmessage = function(event) { ... };
        es.onerror = function() { ... };
    }
"#;
```

**New file full pattern:**
```rust
pub(super) const SOURCE: &str = r#"
    // ── LiveFragment WebSocket subscriptions ──────────────────────────────────

    function setupLiveFragments() {
        var fragments = document.querySelectorAll('[data-live-fragment]');
        if (!fragments.length) return;

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
                // ServerMessage::Event serializes as { type: "event", event: "...",
                // channel: "...", data: {...} } — see ferro-broadcast/src/message.rs:83.
                if (msg.type === 'event' && msg.event === 'fragment' &&
                    msg.data && msg.data.html) {
                    var target = document.querySelector(
                        '[data-live-fragment][data-channel="' + msg.channel + '"]'
                    );
                    if (target) { target.innerHTML = msg.data.html; }
                }
            } catch (_) {}
        });

        ws.addEventListener('error', function() {
            // Browser does not auto-reconnect a closed WebSocket.
            // Reconnect strategy deferred to Phase 262 / future work.
        });
    }
"#;
```

**Key design facts confirmed:**
- `ServerMessage` uses `#[serde(tag = "type", rename_all = "snake_case")]` (`message.rs` line 83). `Event(BroadcastMessage)` serializes as `{ "type": "event" }` with `BroadcastMessage` fields (`event`, `channel`, `data`) flattened alongside `type` by serde's adjacently-tagged enum.
- `/_ferro/ws` path is hardcoded in `framework/src/server.rs` line 209. No body attribute needed (unlike SSE which reads `data-sse-url` from `document.body`).
- `ClientMessage::Subscribe` serializes as `{ "type": "subscribe", "channel": "..." }` (`message.rs` line 52–62).
- One shared socket for all fragments (D-03 contract). No per-fragment socket.

---

### `ferro-json-ui/src/runtime/mod.rs` — assembly edits

**Analog:** the existing `mod sse;` + `s.push_str(sse::SOURCE)` + `setupSSE` pattern (lines 8–70)

**Three edit sites:**

**1. Module declaration** — add after line 21 (`mod hero_lazy;`):
```rust
mod live_fragment;
```

**2. SOURCE push** (`FERRO_RUNTIME_JS` lazy block, lines 30–80) — add after `hero_lazy` push (line 47):
```rust
    s.push_str(hero_lazy::SOURCE);
    s.push_str(live_fragment::SOURCE);  // ADD
```

**3. Dispatcher** — add `setupLiveFragments,` to the `setups` array (lines 54–71), after `setupLazyHeroes`:
```rust
         \x20           setupLazyHeroes,\n\
         \x20           setupLiveFragments\n\
```
(Keep `setupLazyHeroes` as the second-to-last; `setupLiveFragments` becomes the last entry, replacing the trailing `\n` after `setupLazyHeroes`.)

**4. Test updates** — two tests enumerate all setup names explicitly:

`bundle_contains_all_setup_functions` (lines 197–221) — add `"setupLiveFragments"` to the array:
```rust
            "setupLazyHeroes",
            "setupLiveFragments",
```

`dispatcher_invokes_every_setup` (lines 233–264) — add to the names array:
```rust
            "setupLazyHeroes",
            "setupLiveFragments",
```

Add a focused SC-3 no-WASM test:
```rust
#[test]
fn live_fragment_runtime_no_wasm_no_state() {
    let src = super::live_fragment::SOURCE;
    assert!(src.contains("setupLiveFragments"), "must define setup fn");
    assert!(src.contains("data-live-fragment"), "must scan for marker");
    assert!(src.contains("/_ferro/ws"), "must use the fixed WS path");
    assert!(src.contains("innerHTML"), "must swap innerHTML");
    assert!(!src.contains("WebAssembly"), "SC-3: no WASM");
    assert!(!src.contains("useState"), "SC-3: no client-side state");
}
```

---

### App bootstrap glue (sample `app/` — pattern only)

The glue lives outside both crates. No existing analog in `app/src/` uses `ProjectionRuntime` in a handler context with broadcaster wiring, but the pattern assembles directly from the two analogs above.

**Pattern:**
```rust
// At app startup, after constructing ProjectionRuntime:
let broadcaster = Arc::clone(&app_broadcaster);
let child_spec: Arc<ferro_json_ui::spec::Spec> = Arc::new(
    // parse or build the child Spec once at startup
    serde_json::from_value(template_json).expect("child spec"),
);

let runtime = Arc::new(
    ProjectionRuntime::new(db.clone(), broadcaster.clone(), MyProjection)
        .with_fragment_renderer(move |key: &str, snapshot_value: serde_json::Value| {
            let spec = Arc::clone(&child_spec);
            let bc = Arc::clone(&broadcaster);
            let channel = format!("projection.{}.{}", MyProjection::NAME, key);
            tokio::spawn(async move {
                let html = ferro_json_ui::render_spec_to_html(&spec, &snapshot_value);
                let _ = ferro_broadcast::Broadcast::new(bc)
                    .channel(channel)
                    .event("fragment")
                    .data(serde_json::json!({ "html": html }))
                    .send()
                    .await;
            });
        })
);
runtime.clone().register();
```

**Key facts confirmed from codebase:**
- `Broadcast::send()` is `async fn` (`broadcast.rs` line 77). There is NO sync `try_send` path on `Broadcaster` — `tokio::spawn` inside the hook closure is the correct approach (Pitfall 1 from RESEARCH.md confirmed).
- `Broadcast::new(broadcaster).channel(…).event(…).data(…).send().await` is the fluent API (`broadcast.rs` lines 27–91). `.data<T: Serialize>` accepts any serializable type.

---

## Shared Patterns

### Props decode error pattern
**Source:** `ferro-json-ui/src/render/containers.rs` lines 40–48 (`render_card`)
**Apply to:** `render_live_fragment` in `containers.rs`
```rust
let props: XxxProps = match serde_json::from_value(el.props.clone()) {
    Ok(p) => p,
    Err(e) => {
        return format!(
            "<!-- ferro-json-ui: failed to decode Xxx props: {} -->",
            html_escape(&e.to_string())
        );
    }
};
```

### Builtin lockstep (BUILTIN_TYPES + BUILTIN_SPECS + count guard)
**Sources:**
- `ferro-json-ui/src/render/mod.rs` lines 44–101 (BUILTIN_TYPES)
- `ferro-json-ui/src/catalog.rs` lines 126–446 (BUILTIN_SPECS), line 1296 (count guard)
**Apply to:** All three must change in the same commit. Drift guard at `catalog.rs:1296` asserts the pinned absolute count; `catalog.rs:1496` asserts `BUILTIN_SPECS.len() == BUILTIN_TYPES.len()`. Both must pass.

### Runtime module assembly
**Source:** `ferro-json-ui/src/runtime/mod.rs` lines 8–80
**Apply to:** `live_fragment.rs` SOURCE const + 4 edit sites in `mod.rs`
**Rule:** every `setup*` function must appear in: `mod` declaration, `push_str`, `setups` array, AND both test arrays (`bundle_contains_all_setup_functions`, `dispatcher_invokes_every_setup`).

### Arc + DashMap Mutex shard-drop pattern
**Source:** `ferro-projection/src/runtime.rs` lines 100–106
**Apply to:** hook fires inside `apply_event` while `_guard` is held — this is intentional (serializes same-key applies). The hook must be fast (no I/O); async broadcast must run via `tokio::spawn`.

---

## No Analog Found

All files have close analogs. No entries.

---

## Metadata

**Analog search scope:** `ferro-projection/src/`, `ferro-json-ui/src/`, `ferro-broadcast/src/`, `framework/src/`
**Files scanned:** 12 source files read directly, 8 additional via grep
**Pattern extraction date:** 2026-07-26
