---
phase: 44-real-time-improvements
plan: 02
subsystem: real-time
tags: [websocket, hyper-tungstenite, connection-handler, heartbeat, upgrade]

# Dependency graph
requires:
  - phase: 44-01
    provides: tokio-tungstenite 0.28 + WsMessage conversion helpers + whisper forwarding
provides:
  - WebSocket upgrade at /_ferro/ws
  - Connection handler with heartbeat/timeout
  - Client message dispatch (subscribe/unsubscribe/whisper/ping)
affects: [44-03, 44-04]

# Tech tracking
tech-stack:
  added: [hyper-tungstenite 0.19, uuid 1, futures-util 0.3]
  patterns: [HTTP upgrade interception before middleware, tokio::select! message loop]

key-files:
  created:
    - framework/src/websocket.rs
  modified:
    - framework/Cargo.toml
    - framework/src/server.rs
    - framework/src/lib.rs

key-decisions:
  - "WS upgrade intercept before middleware chain (raw hyper request needed for upgrade)"
  - "hyper_tungstenite::tungstenite re-export instead of direct tokio-tungstenite dep in framework"
  - "Generic SinkExt bound on handle_client_message for testability"

patterns-established:
  - "/_ferro/ws as framework WebSocket endpoint"
  - "tokio::select! loop for concurrent frame/server-msg/heartbeat handling"

# Metrics
duration: 5min
completed: 2026-02-10
---

# Phase 44 Plan 02: WebSocket Upgrade Handler + Connection Message Loop Summary

**HTTP-to-WebSocket upgrade at /_ferro/ws with hyper-tungstenite, heartbeat/timeout message loop, and client message dispatch to Broadcaster**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-10T06:23:49Z
- **Completed:** 2026-02-10T06:28:55Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- WebSocket upgrade support via hyper-tungstenite 0.19 at `/_ferro/ws`
- `.with_upgrades()` enabled on HTTP/1 serve_connection for WebSocket protocol switch
- Connection handler with `tokio::select!` loop handling incoming frames, server messages, and heartbeat ticks
- Client message dispatch: Subscribe, Unsubscribe, Whisper, Ping all dispatched to Broadcaster
- Clean disconnect with Close frame and Broadcaster cleanup

## Task Commits

Each task was committed atomically:

1. **Task 1: Add hyper-tungstenite dependency and enable upgrades in server.rs** - `fbfdeeb` (feat)
2. **Task 2: Create WebSocket connection handler module** - `d1a5c86` (feat)

## Files Created/Modified
- `framework/Cargo.toml` - Added hyper-tungstenite 0.19, uuid 1, futures-util 0.3 dependencies
- `framework/src/server.rs` - Added .with_upgrades(), WS upgrade intercept at /_ferro/ws before middleware
- `framework/src/lib.rs` - Added mod websocket declaration
- `framework/src/websocket.rs` - WebSocket upgrade handler, connection message loop, client message dispatch

## Decisions Made
- WebSocket upgrade intercept runs before middleware chain because the upgrade requires the raw hyper Request, not a framework Request
- Used `hyper_tungstenite::tungstenite` re-export for Message/Error types instead of adding tokio-tungstenite as a direct framework dependency
- Generic `SinkExt` bound on `handle_client_message` for cleaner type signature and testability

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- WebSocket upgrade working at /_ferro/ws with full message loop
- All workspace tests pass, no clippy warnings
- Ready for 44-03: Broadcasting auth endpoint + Broadcaster::check_auth

---
*Phase: 44-real-time-improvements*
*Completed: 2026-02-10*
