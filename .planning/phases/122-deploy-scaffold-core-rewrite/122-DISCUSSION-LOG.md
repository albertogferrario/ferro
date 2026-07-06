# Phase 122: Deploy Scaffold Core Rewrite - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md.

**Date:** 2026-04-07
**Phase:** 122-deploy-scaffold-core-rewrite
**Mode:** --auto (no interactive Q&A)
**Areas discussed:** none — SCOPE.md fully specifies the work

---

## Auto-Mode Rationale

SCOPE.md for this phase enumerates 21 concrete decisions across 5 sub-areas (Dockerfile.tpl, path→git rewrite, app.yaml.tpl, command rewrites, dockerignore.tpl) plus an explicit Verification section. There are no meaningful gray areas left for the user to weigh in on — every interface choice, flag name, and behavior is locked. Auto mode forwards SCOPE.md decisions verbatim into CONTEXT.md `<decisions>`.

## Claude's Discretion

- Internal module layout for templates/helpers (planner decides).
- Test approach (golden file tests + unit tests for parsing helpers).
- Exact CLI error message wording (must follow existing ferro-cli conventions).

## Deferred Ideas

All deferrals are explicit in SCOPE.md "Out of scope":
- Phase 123: MCP deploy tools.
- Phase 124: `ferro doctor`, `routes --json`, CI workflow scaffold, `.gitignore`↔`.dockerignore` drift sync.
- Phase 125: `make:module`, json-ui runtime split.
