# Phase 228: README and Scaffold Doc Sweep - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.

**Date:** 2026-06-15
**Phase:** 228-readme-and-scaffold-doc-sweep
**Mode:** `--auto` (recommended defaults auto-selected)
**Areas discussed:** Audit scope/threshold, Install-method consistency, Toolchain-free distinction, Tap-repo README (cross-repo), Stale version refs

---

## Audit scope & fix threshold
**Choice:** Factual accuracy + install-method consistency only; no rewrites (D-01). Mirrors Phase 227.

## Install-method consistency
**Choice:** Brew leads everywhere; cargo/source as alternates (D-02). Concrete fixes: scaffold README.tpl:10 & :82, create-app.sh:142; verify install.sh.

## Toolchain-free distinction
**Choice:** State CLI-is-toolchain-free (brew) vs Rust-1.88-needed-to-build-app consistently, reconciled to installation.md (D-03).

## Stale version / milestone refs
**Choice:** Fix root README:185 (`v0.2.0` / `v12.0 spec-driven`) to neutral low-churn phrasing; sweep rest of README (D-04).

## Tap-repo README (cross-repo)
| Option | Selected |
|--------|----------|
| Draft content in ferro repo, do NOT auto-push to separate tap repo | ✓ |
| Edit/commit the tap repo directly from this session | (rejected — cross-repo boundary) |
| Skip entirely | |
**Choice:** Draft ready-to-paste content in the ferro repo; tap-repo commit is a separate user/`gh` action (D-05). Rationale: `albertogferrario/homebrew-ferro` is not checked out locally; editing another repo's tree from a ferro session violates the cross-repo split rule.

## Claude's Discretion
- Exact wording of README/script edits; whether install.sh needs changes (audit during execution).

## Deferred Ideas
- Tap-repo README commit (cross-repo, outside this session).
- CHANGELOG (carried from Phase 227).
