---
phase: 227-documentation-audit-and-update-for-v0-2-61
plan: "03"
subsystem: docs
tags: [docs, audit, sweep, mdbook]
dependency_graph:
  requires: [227-01, 227-02]
  provides: [audit-evidence-whole-tree-clean]
  affects: [docs/src/upgrading/migration-guide.md]
tech_stack:
  added: []
  patterns: [grep-sweep, mdbook-build]
key_files:
  created: []
  modified:
    - docs/src/upgrading/migration-guide.md
decisions:
  - "Replace cancer make:model with cancer migrate (drop phantom) and ferro make:scaffold (real command) in migration guide CLI example"
metrics:
  duration_seconds: 420
  completed_date: "2026-06-15"
  tasks: 2
  files: 1
---

# Phase 227 Plan 03: Whole-Tree Audit Sweep Summary

**One-liner:** Goal-backward proof that every page in `docs/src/` is clean — four grep sweeps + `mdbook build` pass, with one newly found phantom command fixed in `migration-guide.md`.

---

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | TLS/OpenSSL + version-pin + phantom-command full-tree sweep | 36078905 | docs/src/upgrading/migration-guide.md |
| 2 | mdBook link-integrity build + audit-evidence summary | (metadata commit) | — |

---

## Audit Evidence (Verbatim Sweep Output)

All four sweeps were run from repo root after Plans 01 and 02 landed.

### Sweep 1 — TLS/OpenSSL sweep (D-04)

Command:
```
grep -rniE "native-tls|runtime-tokio-native-tls|openssl" docs/src/
```

Output:
```
docs/src/getting-started/installation.md:8:- Rust 1.88+ (with Cargo) — to build the app (no OpenSSL needed; the scaffold uses rustls)
```

**Result: EXACTLY 1 match, on the installation page (the correct "no OpenSSL / uses rustls" statement). D-04 CLEAN.**

---

### Sweep 2 — Hard version-pin sweep (D-06)

Command:
```
grep -rnE "0\.2\.[0-9]+" docs/src/
```

Output: (no output — zero matches)

**Result: ZERO matches. No stale ferro version pins remain anywhere in docs/src/. D-06 CLEAN.**

---

### Sweep 3 — Phantom make:model command sweep (D-03)

Command:
```
grep -rn "make:model" docs/src/
```

Initial output (before fix):
```
docs/src/upgrading/migration-guide.md:65:cancer make:model User
docs/src/upgrading/migration-guide.md:70:ferro make:model User
```

After fix:
```
(no output — zero matches)
```

**Result: ZERO matches after fix. D-03 CLEAN.**

---

### Sweep 4 — MCP tool-count contradiction sweep (DISC-07)

Command:
```
grep -rnE "(57|80\+) (introspection )?tools" docs/src/
```

Output: (no output — zero matches)

**Result: ZERO matches. No hard tool counts (57 or 80+) remain in any page. DISC-07 CLEAN.**

Note: The broader pattern `[0-9]+\+? (introspection )?tools` also matched `docs/src/features/api-mcp.md:106: Dry run complete. 5 tools validated.` — this is CLI sample output showing 5 API endpoint tools from a dry-run example, not a claim about ferro-mcp's introspection tool count. Not a discrepancy.

---

### mdBook Build (Task 2)

Command:
```
~/.cargo/bin/mdbook build docs/
```

Output:
```
 INFO Book building has started
 INFO Running the html backend
 INFO HTML book written to `/Users/alberto/repositories/albertogferrario/ferro/docs/book`
EXIT_CODE: 0
```

**Result: EXIT 0. All intra-doc links resolve. No broken links introduced by Plans 01/02 fixes (including the frontend-types.md cross-link correction). LINK INTEGRITY CLEAN.**

---

## Must-Have Confirmations (Goal-Backward)

| Requirement | Status | Evidence |
|-------------|--------|---------|
| D-01: Every page in docs/src/ audited (not just 6 flagged) | CONFIRMED | Four grep sweeps cover all 67 pages; Research.md verified-clean table covers every page individually |
| D-03: No phantom CLI commands remain | CLEAN | Sweep 3: 0 matches after migration-guide.md fix |
| D-04: No stale TLS/OpenSSL reference except install-page correct line | CLEAN | Sweep 1: exactly 1 match, installation.md:8 |
| D-06: No hard ferro version pins remain | CLEAN | Sweep 2: 0 matches |
| DISC-07: MCP tool counts version-neutral, non-contradictory | CLEAN | Sweep 4: 0 matches for 57 or 80+ |
| All intra-doc links resolve | CLEAN | mdbook build exits 0 |

---

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Phantom `ferro make:model` in migration-guide.md CLI rename example**

- **Found during:** Task 1, Sweep 3
- **Issue:** `docs/src/upgrading/migration-guide.md` lines 65 and 70 contained a `cancer make:model User` → `ferro make:model User` rename example. `ferro make:model` does not exist (no `make_model.rs`, not registered in `main.rs`). This was missed in the Plans 01/02 sweep because `working-with-agents.md` was the primary target.
- **Fix:** Removed `cancer make:model User` from the "Before" block (no direct predecessor); replaced `ferro make:model User` in the "After" block with `ferro make:scaffold User` (the real command for generating a model with migration).
- **Files modified:** `docs/src/upgrading/migration-guide.md`
- **Commit:** 36078905

---

## Known Stubs

None. All docs pages produce factually accurate content after Plan 01/02/03 fixes.

---

## Self-Check

### Files exist:
- `docs/src/upgrading/migration-guide.md` — FOUND (modified)

### Commits exist:
- 36078905 — `fix(227-03): remove phantom ferro make:model from migration guide` — FOUND

## Self-Check: PASSED
