---
phase: 247-read-model-delta-broadcast-streaming
reviewed: 2026-08-14T00:00:00Z
depth: standard
files_reviewed: 5
files_reviewed_list:
  - framework/src/offload.rs
  - framework/src/app.rs
  - framework/tests/offload_delta_broadcast.rs
  - framework/Cargo.toml
  - docs/src/features/queues.md
findings:
  critical: 0
  warning: 2
  info: 3
  total: 5
status: clean
resolution: WR-01 and WR-02 fixed in commit 55b00768; info items assessed, no action required.
---

# Phase 247: Code Review Report

**Reviewed:** 2026-08-14
**Depth:** standard
**Files Reviewed:** 5
**Status:** clean (both warnings resolved in-phase — see Resolution)

## Resolution

- **WR-01 / WR-02** — Fixed in commit `55b00768`: `resolve()` now calls
  `broadcaster.remove_client` on both the timeout exit and the subscribe-failure
  exit, so no broadcaster client entry leaks on any early return. Verified by the
  green offload unit + `offload_delta_broadcast` integration suites plus the
  CI-exact `clippy --all --all-targets -D warnings` gate.
- **IN-01 / IN-02 / IN-03** — Assessed as non-actionable: IN-02's fallback path is
  unreachable (broadcast is skipped on persist failure), IN-01 is a comment-precision
  nit, IN-03 is a test-harness timing dependency inherited from Phase 246. No change.

## Summary

Phase 247 closes the fire-and-forward loop correctly: persist-before-broadcast ordering is honored (D-02), the failed delta carries no `error` field (D-05), `read_result_redacted` replaces raw errors with a fixed marker (D-10), the subscribe-before-read-back ordering in `resolve` is sound for the primary race (D-09), and the channel name is a constant-prefix + server-minted UUID with no injection surface. Two resource leaks in `resolve` — both involving `broadcaster.remove_client` not being called on error exit paths — are the material findings.

No critical findings (no auth bypass, no raw error leakage to clients, no SQL injection). The two warnings are resource leaks that could accumulate under retry or timeout load. Three informational items are noted below.

---

## Warnings

### WR-01: `resolve()` leaks broadcaster client on `Timeout`

**File:** `framework/src/offload.rs:488–492`

**Issue:** When `tokio::time::timeout` fires, the `?` operator propagates `ResolveError::Timeout` and exits the function immediately. The `broadcaster.remove_client(&client_id)` call at line 494 is unreachable on this path. The client — keyed by `"{handle_key}-resolve-{uuid}"` — remains in the broadcaster's `clients` DashMap with a live mpsc sender but no reader, and is never cleaned up unless the `Broadcaster` itself drops. Under any meaningful call rate with bounded timeouts, this leaks one entry per timed-out `resolve` call.

```rust
// Current — remove_client is unreachable on Timeout:
let out = match timeout {
    Some(d) => tokio::time::timeout(d, wait)
        .await
        .map_err(|_| ResolveError::Timeout)?,  // <-- early return; line 494 never runs
    None => wait.await,
};
broadcaster.remove_client(&client_id);  // line 494: skipped on Timeout
out
```

**Fix:** Remove the `?` on the timeout path; instead, let the outer match carry the cleanup:

```rust
let out = match timeout {
    Some(d) => match tokio::time::timeout(d, wait).await {
        Ok(result) => result,
        Err(_) => {
            broadcaster.remove_client(&client_id);
            return Err(ResolveError::Timeout);
        }
    },
    None => wait.await,
};
broadcaster.remove_client(&client_id);
out
```

---

### WR-02: `resolve()` leaks broadcaster client when `subscribe` fails

**File:** `framework/src/offload.rs:447–451`

**Issue:** `add_client` inserts the client into the broadcaster's `clients` map at line 447. If the subsequent `subscribe` call at line 448–451 returns an error (e.g., `Error::ChannelFull` or authorization failure), the `?` exits the function without calling `remove_client`. The client entry remains in the map, holding a live mpsc sender that nothing reads.

```rust
// Current — add_client runs, then subscribe may fail and return early:
broadcaster.add_client(client_id.clone(), tx);      // line 447: client inserted
broadcaster
    .subscribe(&client_id, &channel, None, None)
    .await
    .map_err(|e| ResolveError::Broadcast(e.to_string()))?;  // ? exits without cleanup
```

**Fix:** Clean up the client on subscribe failure:

```rust
broadcaster.add_client(client_id.clone(), tx);
if let Err(e) = broadcaster
    .subscribe(&client_id, &channel, None, None)
    .await
{
    broadcaster.remove_client(&client_id);
    return Err(ResolveError::Broadcast(e.to_string()));
}
```

---

## Info

### IN-01: Narrow internal race in `resolve()` between `add_client` and `subscribe` completion

**File:** `framework/src/offload.rs:447–451`

**Issue:** The `Broadcaster` subscription is completed in two separate steps: `add_client` adds the client to the `clients` map (line 447), then `subscribe` adds the socket id to `channel.subscribers` (line 448–451). A delta published in the window between these two operations would reach `send_to_channel` — which iterates `channel.subscribers` — before the resolve caller is listed there, and would not be delivered to the mpsc receiver.

The read-back at step 2 (line 454) guards against results that land before `subscribe` returns, so in practice this window is covered. The remaining concern is a theoretical scenario where the worker persists and broadcasts *after* `add_client` returns but *before* `subscribe` adds the socket to the channel's subscriber list — in which case both the delta and the read-back would be missed.

This window is extremely narrow (it spans a DashMap write inside the `subscribe` body, which holds no async yield points for the public channel case) and the read-back step provides a practical safety net. No code change is required, but the comment at line 444 ("Subscribe FIRST") slightly overstates the guarantee: the full subscription guarantee holds only after `subscribe` returns at line 451, not after `add_client` at line 447.

**Suggestion:** The comment could be tightened to "Subscribe-first guarantee holds after `subscribe` returns" to avoid implying that `add_client` alone is sufficient for the race safety.

---

### IN-02: `resolve()` returns `OffloadResult::Pending` on snapshot absence after delta wakeup

**File:** `framework/src/offload.rs:472–475`

**Issue:** When the delta wakes the `wait` loop, the authoritative snapshot is read back with:

```rust
.map(|opt| opt.unwrap_or(OffloadResult::Pending))
```

If the snapshot is absent at this point (which should not occur in the normal path, since `register_offload_hooks_with_broadcaster` skips broadcast when persist fails), the caller receives `OffloadResult::Pending` rather than an error. A caller matching on `OffloadResult::Pending` after a delta wakeup could misinterpret this as "still running" when the real cause is a transient persistence failure.

The guard at `offload.rs:357` (`return` on persist failure before broadcast) makes this path unreachable under normal operation. No change is required; this is noted for completeness.

---

### IN-03: Timing-based drain in integration test introduces a sleep dependency

**File:** `framework/tests/offload_delta_broadcast.rs:99`

**Issue:** The `drain()` helper sleeps 200 ms after `drain_for_test()` returns to allow spawned hook tasks time to complete snapshot writes and broadcasts. This is a fixed-duration wait, not a condition-based wait, so it is sensitive to system load. Under resource contention the hook writes may not complete in 200 ms, causing `offload_failed_delta_is_redacted` or `cross_replica_delta` to time out at their 5-second assertion windows rather than at the sleep.

This pattern is inherited from the Phase 246 drain harness and passes CI consistently. No change is required unless the test proves flaky in practice.

---

*Reviewed: 2026-08-14*
*Reviewer: Claude (gsd-code-reviewer)*
*Depth: standard*
