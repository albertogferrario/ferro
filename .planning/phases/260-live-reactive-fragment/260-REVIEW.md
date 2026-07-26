---
phase: 260-live-reactive-fragment
reviewed: 2026-07-26T00:00:00Z
depth: standard
files_reviewed: 8
files_reviewed_list:
  - ferro-projection/src/runtime.rs
  - ferro-json-ui/src/component.rs
  - ferro-json-ui/src/render/containers.rs
  - ferro-json-ui/src/render/mod.rs
  - ferro-json-ui/src/catalog.rs
  - ferro-json-ui/src/runtime/live_fragment.rs
  - ferro-json-ui/src/runtime/mod.rs
  - ferro-mcp/src/tools/json_ui_catalog.rs
findings:
  critical: 0
  warning: 3
  high: 0
  medium: 0
  low: 1
  total: 4
status: resolved
resolution: "All 4 findings (WR-01/02/03, IN-01) fixed inline in commit f8ded454; verified green (ferro-projection 28 tests, ferro-json-ui live_fragment/render/runtime 22 tests, fmt + scoped clippy -D warnings clean)."
---

# Phase 260: Code Review Report

**Reviewed:** 2026-07-26
**Depth:** standard
**Files Reviewed:** 8
**Status:** resolved (all findings fixed in commit f8ded454)

## Summary

Phase 260 delivers the `LiveFragment` builtin: server-rendered HTML fragments
that subscribe over the existing `/_ferro/ws` WebSocket and swap `innerHTML`
on each `fragment` event. The implementation is clean overall and the
priority-1 concerns (XSS escaping, single-render-path, catalog lockstep) are
largely correct. No critical issues were found.

Three warnings are worth fixing before the Phase 262 publish:

1. **CSS selector injection via `msg.channel` in the client JS** — the
   server-pushed channel string is concatenated into a `querySelector`
   attribute-value selector without escaping, which can misdirect DOM
   targeting (see WR-01).
2. **Fragment hook is silently skipped when the delta broadcast fails** — this
   is a correctness divergence from D-02's "additive" contract and is
   undocumented (WR-02).
3. **`serde_json::to_value(&state).unwrap_or(Value::Null)` passes `Null` to
   the hook silently** — a serialization failure hands the hook a `null`
   snapshot with no diagnostics, producing a blank re-render instead of
   revealing the real fault (WR-03).

One low-priority finding: `inner_html` from `render_spec_to_html` is
interpolated into the container `format!` string without going through
`html_escape`, which is correct but relies on `render_spec_to_html`'s own
escaping guarantees being complete — that assumption is valid today but
deserves a comment (IN-01).

---

## Warnings

### WR-01: CSS selector injection via server-pushed `msg.channel`

**File:** `ferro-json-ui/src/runtime/live_fragment.rs:38-40`

**Issue:** The `message` handler builds a CSS attribute-value selector by
direct string concatenation:

```js
var target = document.querySelector(
    '[data-live-fragment][data-channel="' + msg.channel + '"]'
);
```

`msg.channel` is a string that arrives over the WebSocket from the server.
The server constructs the channel as `projection.{NAME}.{key}` where both
segments come from props set at authoring time — not from end-user input —
so this is not an injection path from untrusted data in the normal usage.

However the security model becomes fragile in two situations:

1. **If the server ever echoes a channel derived from user-controlled input**
   (e.g. a `key` that includes a per-user identifier constructed from a query
   parameter), a `"` or `]` in `msg.channel` breaks out of the attribute
   selector, either matching unintended containers or causing a `querySelector`
   exception (which is caught and silently swallowed by the outer `try/catch`).
   The result is either the wrong container's `innerHTML` being replaced, or
   the update being silently dropped.

2. The `try/catch` around the entire message handler means a malformed channel
   string causes a *silent no-op*, making this failure mode invisible in
   production.

Because `msg.channel` is server-controlled, this is not an XSS vector for
arbitrary untrusted users. But it is a correctness and robustness issue:
the selector should be constructed from the DOM directly rather than from the
message.

**Fix:** Replace the server-echoed channel lookup with a DOM-driven lookup.
The subscribed channels are already known at subscription time from the
`[data-live-fragment]` containers in the DOM. Build and maintain a JS `Map`
from `channel → element` at `open` time, then look up by `msg.channel` key —
no CSS selector construction needed at all:

```js
ws.addEventListener('open', function() {
    for (var i = 0; i < fragments.length; i++) {
        var ch = fragments[i].getAttribute('data-channel');
        if (ch) {
            channelMap[ch] = fragments[i];            // direct reference
            ws.send(JSON.stringify({ type: 'subscribe', channel: ch }));
        }
    }
});

ws.addEventListener('message', function(e) {
    try {
        var msg = JSON.parse(e.data);
        if (msg.type === 'event' && msg.event === 'fragment' &&
            msg.data && msg.data.html) {
            var target = channelMap[msg.channel];     // O(1), no selector
            if (target) { target.innerHTML = msg.data.html; }
        }
    } catch (_) {}
});
```

This also eliminates the CSS selector character-class concern entirely, since
no CSS selector string is ever built from `msg.channel`.

---

### WR-02: Fragment hook skipped when delta broadcast returns `Err`

**File:** `ferro-projection/src/runtime.rs:177-183, 186-192`

**Issue:** In `apply_event`, the delta broadcast error path returns early with
`Err` before the fragment hook fires:

```rust
if let Err(e) = send_result {
    tracing::warn!(...);
    return Err(ProjectionError::from(e));   // ← early return HERE
}

// Step 6.5: fragment re-render hook — NEVER REACHED if delta broadcast fails
if let Some(ref hook) = self.fragment_hook {
    ...
    hook(key.as_str(), snapshot_value);
}
```

D-02 documents the fragment event as *additive* — a second broadcast on the
same channel, independent of the delta broadcast. A caller that registers a
fragment hook reasonably expects it to fire whenever the state changes,
regardless of whether the raw-delta broadcast succeeded.

The current behavior means: if the broadcaster has no active subscribers
(common during startup, or when the broadcaster drops all clients), the delta
`send` may fail (depending on broadcaster semantics), and the hook is skipped
silently. The state is persisted, the hook is not fired. The live fragment
never receives its first render.

**Fix:** Either (a) fire the hook unconditionally after the upsert regardless
of delta broadcast result, or (b) document the dependency explicitly in the
crate docs and in `with_fragment_renderer`'s rustdoc. If the hook *should*
fire even when the delta broadcast fails (which D-02 implies), restructure:

```rust
// Step 6: broadcast — failure does NOT roll back state (D-21)
let send_result = ferro_broadcast::Broadcast::new(self.broadcaster.clone())
    ...
    .send()
    .await;

let broadcast_error = if let Err(e) = send_result {
    tracing::warn!(...);
    Some(ProjectionError::from(e))
} else {
    None
};

// Step 6.5: fragment hook fires regardless of delta broadcast result (D-02).
if let Some(ref hook) = self.fragment_hook {
    let snapshot_value = serde_json::to_value(&state)
        .unwrap_or(serde_json::Value::Null);
    hook(key.as_str(), snapshot_value);
}

// Return broadcast error after the hook has had a chance to fire.
if let Some(e) = broadcast_error {
    return Err(e);
}
```

If the intended behavior is that the hook only fires when the delta broadcast
succeeds, document that decision in `with_fragment_renderer`'s rustdoc.

---

### WR-03: Silent `Null` snapshot passed to hook on serialization failure

**File:** `ferro-projection/src/runtime.rs:190`

**Issue:**

```rust
let snapshot_value = serde_json::to_value(&state).unwrap_or(serde_json::Value::Null);
hook(key.as_str(), snapshot_value);
```

`P::State: Serialize` is required by the `Projection` trait. `to_value` can
fail only if the `Serialize` implementation returns an error — which is
theoretically impossible for a well-formed derived `Serialize`, but is
possible for hand-implemented `Serialize` impls that return errors for certain
states.

If `to_value` does fail, the hook receives `Value::Null`. The hook will then
call `render_spec_to_html` against a `null` data scope and push blank HTML to
the live fragment. The projection key's state has been persisted correctly
(step 5 uses `state_json` computed earlier on line 148), but the rendered live
fragment goes blank with no diagnostic anywhere.

The same state was already successfully serialized at line 148
(`let state_json = serde_json::to_value(&state)?`). Reusing that value
eliminates both the redundant serialization and the failure path:

```rust
// Reuse the state_json already computed at step 5 — avoids double
// serialization and eliminates the silent-Null failure path.
if let Some(ref hook) = self.fragment_hook {
    hook(key.as_str(), state_json.clone());
}
```

If reuse is not preferred for some reason, at minimum add a `tracing::warn!`
before the `unwrap_or` so the failure is observable:

```rust
let snapshot_value = serde_json::to_value(&state)
    .inspect_err(|e| tracing::warn!(error = %e, "fragment hook: state serialization failed; hook receives Null"))
    .unwrap_or(serde_json::Value::Null);
```

---

## Info

### IN-01: `inner_html` in container format string relies on implicit escaping guarantee

**File:** `ferro-json-ui/src/render/containers.rs:1675`

**Issue:**

```rust
format!(r#"<div data-live-fragment data-channel="{channel}">{inner_html}</div>"#)
```

`inner_html` is the output of `render_spec_to_html`, which owns all escaping
for the child template content. This is correct today: every string that
crosses from a `Spec` or data into HTML output goes through `html_escape` at
the atom/container level. However `inner_html` is placed inside the outer
format string without any explicit annotation that it is pre-escaped trusted
HTML.

This is not a bug — `render_spec_to_html` is infallible and always returns
well-formed HTML — but it is a pattern that future maintainers could
incorrectly copy for a different value that is *not* pre-escaped.

Consider adding a comment:

```rust
// inner_html is produced by render_spec_to_html which owns all escaping
// for the child content — it is already-escaped trusted HTML, not raw data.
format!(r#"<div data-live-fragment data-channel="{channel}">{inner_html}</div>"#)
```

---

## Confirmed Clean (no issues found)

The following priority areas were inspected and are clean:

**XSS / HTML injection in `render_live_fragment`:**
- `props.projection` and `props.key` are both passed through `html_escape`
  before interpolation into the `data-channel` attribute value (containers.rs:1671-1672).
  A key containing `"` or `>` cannot break out of the attribute.
- The child template's rendered output (`inner_html`) comes from
  `render_spec_to_html`, which routes all expression-bound values through the
  existing binding engine that applies `html_escape` at every text-content
  interpolation. Snapshot string values cannot inject markup through this path.
- Error comment strings in the decode/parse failure paths also pass through
  `html_escape` (lines 1650, 1660).

**Client JS channel provenance:**
- The `message` handler filters on `msg.type === 'event' && msg.event === 'fragment'`
  before acting, so `delta` events are ignored as required by D-02.
- The `innerHTML` swap only targets a container that (a) is already in the DOM
  with `data-live-fragment` and (b) has a `data-channel` matching the message
  channel. A message for channel A cannot overwrite a container bound to channel B
  (modulo the CSS selector issue documented in WR-01 when the channel string contains
  selector metacharacters).
- No `eval`, `new Function`, `WebAssembly`, or `importScripts` usage. SC3 is
  satisfied.
- No reconnect loop implemented; the `error` handler is empty and documented as
  deferred.

**Concurrency in `apply_event`:**
- The hook receives the post-apply snapshot: `state` at step 6.5 is the
  mutated state after `P::apply` (step 4), persisted (step 5), and
  delta-broadcast (step 6). The snapshot is current.
- The hook is a synchronous call that immediately returns; the actual async
  broadcast is pushed into `tokio::spawn` by the hook closure (as shown in the
  SC2 integration test). A hook panic would propagate up through `apply_event`
  and unwind the per-key Mutex guard via RAII — `tokio::sync::Mutex` is not
  poisoned by panics (it uses `PoisonError`-free `tokio` semantics), so the
  mutex is released cleanly and subsequent applies on the same key are
  unaffected. No panic isolation is needed because `tokio::sync::Mutex` does
  not poison.
- The per-key Mutex is held across the synchronous hook call. Because the hook
  immediately dispatches to `tokio::spawn` (async work is off-path), it returns
  in nanoseconds. Other keys are not affected (independent DashMap shards).
  Deadlock is not possible because the hook does not attempt to re-acquire the
  same per-key Mutex.
- Spawned broadcast failure: the `tokio::spawn` result is discarded (the `let _`
  pattern in the SC2 test is representative). This is consistent with the
  crate's documented posture: "broadcast failure does not roll back state"
  (runtime.rs line 17). The fragment broadcast is a best-effort delivery.

**`render_spec_to_html` as the single render path (D-05):**
- Both first-paint (containers.rs:1667) and delta re-render (hook closure in
  the app) call `render_spec_to_html` on the same child `Spec` with different
  snapshot `Value`s. One render function, two call sites — deterministic and
  consistent.

**Catalog / dispatch lockstep (D-06):**
- `BUILTIN_TYPES` count is 53 (render/mod.rs lines 44-103).
- `BUILTIN_SPECS` count is 53 (catalog.rs drift guard at line 1303 pins the count).
- `ferro-mcp` mirror count is 53 (json_ui_catalog.rs line 419).
- Dispatch arm routes `"LiveFragment"` to `render_live_fragment` (mod.rs:229).
- No duplicate in `BUILTIN_TYPES` (the `builtin_types_have_no_duplicates` test covers this).
- No dead dispatch arms left (each entry in `BUILTIN_TYPES` has a corresponding match arm).

---

_Reviewed: 2026-07-26_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
