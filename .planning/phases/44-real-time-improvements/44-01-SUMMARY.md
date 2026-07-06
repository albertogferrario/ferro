---
phase: 44-real-time-improvements
plan: 01
subsystem: real-time
tags: [websocket, tokio-tungstenite, broadcast, whisper]

# Dependency graph
requires:
  - phase: 38
    provides: stabilized foundation
provides:
  - tokio-tungstenite 0.28 compatibility
  - WsMessage conversion helpers on ServerMessage/BroadcastMessage/ClientMessage
  - whisper() method on Broadcaster for client-to-client forwarding
affects: [44-02, 44-03]

# Tech tracking
tech-stack:
  added: [tokio-tungstenite 0.28]
  patterns: [WsMessage conversion layer in message types]

key-files:
  created: []
  modified:
    - ferro-broadcast/Cargo.toml
    - ferro-broadcast/src/message.rs
    - ferro-broadcast/src/broadcaster.rs

key-decisions:
  - "Conversion helpers on message types rather than in connection handler"
  - "whisper() drops DashMap guard before async send, consistent with subscribe pattern"

patterns-established:
  - "to_ws_message()/from_ws_text() pattern for WS payload conversion"

# Metrics
duration: 3min
completed: 2026-02-10
---

# Phase 44 Plan 01: tokio-tungstenite 0.28 + Message Helpers + Whisper Summary

**Bumped tokio-tungstenite from 0.26 to 0.28, added WsMessage conversion helpers on all message types, and implemented whisper forwarding on Broadcaster**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-10T06:18:37Z
- **Completed:** 2026-02-10T06:21:32Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- tokio-tungstenite bumped from 0.26 to 0.28 (Utf8Bytes/Bytes API)
- `to_ws_message()` convenience methods on `ServerMessage` and `BroadcastMessage` for converting to WebSocket text frames
- `from_ws_text()` on `ClientMessage` for parsing incoming WebSocket payloads
- `whisper()` method on `Broadcaster` for client-to-client event forwarding with proper DashMap guard handling
- 3 new unit tests for whisper: forwarding, disabled config rejection, unsubscribed client rejection

## Task Commits

Each task was committed atomically:

1. **Task 1: Bump tokio-tungstenite to 0.28 and adapt message types** - `b275099` (feat)
2. **Task 2: Add whisper forwarding to broadcaster** - `0ee5095` (feat)

## Files Created/Modified
- `ferro-broadcast/Cargo.toml` - Bumped tokio-tungstenite dependency from 0.26 to 0.28
- `ferro-broadcast/src/message.rs` - Added WsMessage import, to_ws_message() on BroadcastMessage and ServerMessage, from_ws_text() on ClientMessage
- `ferro-broadcast/src/broadcaster.rs` - Added whisper() method and 3 unit tests

## Decisions Made
- Placed WS conversion helpers on message types (not in connection handler) to keep handler code clean
- whisper() drops DashMap guard before async send_to_channel_except, consistent with existing subscribe pattern

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- ferro-broadcast compiles cleanly with tokio-tungstenite 0.28
- All 16 tests pass, no clippy warnings
- Full workspace compiles
- Ready for 44-02: WebSocket upgrade handler + connection message loop

---
*Phase: 44-real-time-improvements*
*Completed: 2026-02-10*
