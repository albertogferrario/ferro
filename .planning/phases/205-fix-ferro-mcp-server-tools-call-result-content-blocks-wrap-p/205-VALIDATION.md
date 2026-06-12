---
phase: 205
slug: fix-ferro-mcp-server-tools-call-result-content-blocks-wrap-p
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-12
---

# Phase 205 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (tokio::test for async) |
| **Config file** | none — workspace Cargo.toml; `tokio` features `full,macros` already in `ferro-mcp-server` dev-deps |
| **Quick run command** | `cargo test -p ferro-mcp-server` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~30s (quick) / minutes (full) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp-server && cargo test -p app`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite green AND D-06 manual dogfood GO
- **Max feedback latency:** ~30 seconds (quick run)
- **Disk note:** `cargo test --all-features` has known ENOSPC risk on this machine — run `df -h` and clean `target/` before the full gate; use `-p ferro-mcp-server -p app` as the interim sampling command.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 205-01-01 | 01 | 1 | AMCP-03 (content fix) | — | N/A | unit | `cargo test -p ferro-mcp-server tools_call_result_parses` | ❌ W0 | ⬜ pending |
| 205-01-02 | 01 | 1 | AMCP-03 (structuredContent) | — | N/A | unit | `cargo test -p ferro-mcp-server tools_call_result_parses` | ❌ W0 | ⬜ pending |
| 205-02-01 | 02 | 2 | AMCP-10 (tenant scoping preserved) | — | tenant A token returns only tenant 1 rows after envelope change | integration | `cargo test -p app tenant_a_isolation` | ✅ (update required) | ⬜ pending |
| 205-02-02 | 02 | 2 | AMCP-10 (cross-tenant) | — | tenant B token returns only tenant 2 rows | integration | `cargo test -p app tenant_b_isolation` | ✅ (update required) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-mcp-server/src/jsonrpc.rs` — add `#[cfg(test)] mod tests` covering `tools_call_result_parses_as_valid_mcp_content` (AMCP-03 interop, D-04): deserialize the emitted `result` into `rmcp::model::CallToolResult` (custom Deserialize impl exists) and assert each content block parses + `structuredContent` round-trips to `{rows,total,limit,offset}`.
- [ ] `app/src/tests/mcp_tenant_isolation.rs` — update `tenant_a_isolation` / `tenant_b_isolation` to navigate `result["result"]["structuredContent"]["rows"]` instead of `result["result"]["content"]` (behavior unchanged; assertion path only).
- [ ] No new test infrastructure files needed.

*Existing infrastructure covers all phase requirements aside from the new interop test above.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Claude Code MCP client parses `list_order` result without Zod errors; alice@acme.test sees 2/4 orders | D-06 | Requires live browser-OAuth flow + a real MCP client SDK; cannot be driven from `cargo test` | Run `cd app && ../target/debug/app` (port 8090); MCP server `ferro-sample-app` → http://127.0.0.1:8090/mcp; drive OAuth via chrome-devtools-3 (clear `/tmp/chrome-mcp-3/Singleton{Lock,Socket,Cookie}` first); grab `/auth/verify?token=…` from DOM; approve consent; call `list_order`; confirm no parse error + exactly 2 Acme orders. Plan marks this task `autonomous: false`. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
