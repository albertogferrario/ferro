# Phase 253 — Deferred Items

## Pre-existing cargo-deny failures (out of scope for phase 253)

The `Security & Dependency Check` CI job (cargo-deny) was already failing before
this phase. The same failure pattern appeared on the previous push (CI run
28486489730, 2026-07-01). Phase 253 added no new external crates.

**Advisories flagged:**

| Advisory | Crate | Severity | Fixed in |
|---|---|---|---|
| RUSTSEC-2026-0190 | anyhow 1.0.100 | Unsound (UB in Error::downcast_mut) | anyhow >= next patch |
| RUSTSEC-2026-0189 | rmcp 0.12.0 | Vulnerability (DNS rebinding in Streamable HTTP) | rmcp >= 1.4.0 |

**Action required (next session):**
1. Bump `anyhow` to a version without RUSTSEC-2026-0190.
2. Evaluate `rmcp` 1.4.0 upgrade (major API change from 0.12 → 1.4; ferry-mcp-server impact needs assessment).
3. Add ignores in `deny.toml` if the crates cannot yet be upgraded (with justification).

**Not blocking publish:** The Publish CI workflow does not run cargo-deny.
ferro-rs 0.2.85 is live on crates.io.
