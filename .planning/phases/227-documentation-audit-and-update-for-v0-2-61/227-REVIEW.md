---
phase: 227-documentation-audit-and-update-for-v0-2-61
reviewed: 2026-06-14T22:41:11Z
depth: quick
files_reviewed: 5
files_reviewed_list:
  - docs/src/cli/frontend-types.md
  - docs/src/getting-started/working-with-agents.md
  - docs/src/introduction.md
  - docs/src/reference/cli.md
  - docs/src/upgrading/migration-guide.md
findings:
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 227: Code Review Report

**Reviewed:** 2026-06-14T22:41:11Z
**Depth:** quick (with oracle source verification)
**Files Reviewed:** 5
**Status:** issues_found

## Summary

Five documentation files were reviewed for factual accuracy following the v0.2.59→v0.2.61 release. The TLS stack is clean (no stale `native-tls`/OpenSSL references remain). No hard version pins of the form `0.2.NN` appear in any file. The `<pinned>` placeholder convention is correctly used in `frontend-types.md`.

Three findings require fixes: a phantom CLI command (`ferro make:handler`, which has no corresponding `make_handler.rs` in `ferro-cli/src/commands/`) appears in `working-with-agents.md`; and two broken intra-doc links in `reference/cli.md` point to `../authentication.md` and `../api-resources.md` — paths that resolve to non-existent `docs/src/authentication.md` and `docs/src/api-resources.md` (the real files live under `docs/src/features/`).

Two info-level items cover the `migration-guide.md` title ("cancer to ferro") being exposed as a public page with the original internal codename in the heading, and a `$schema` value in `cli.md` that uses a literal `"ferro-json-ui/v2"` string rather than the versioned URL form documented elsewhere.

## Warnings

### WR-01: Phantom command `ferro make:handler` in working-with-agents.md

**File:** `docs/src/getting-started/working-with-agents.md:123`
**Issue:** The "Other generation hints" bullet references `ferro make:handler` as a scaffolding command. There is no `make_handler.rs` in `ferro-cli/src/commands/` and no `make:handler` entry in the CLI summary table. A user or agent copy-pasting this command would get "unknown command" at runtime. The correct command for scaffolding a controller with handlers is `ferro make:controller`.
**Fix:**
```markdown
- `code_templates` → `ferro make:controller` — scaffold a request handler
```

### WR-02: Broken intra-doc link — `../authentication.md` in cli.md

**File:** `docs/src/reference/cli.md:328`
**Issue:** The `make:auth` section links to `[Authentication guide](../authentication.md)`. Relative to `docs/src/reference/cli.md`, this resolves to `docs/src/authentication.md`, which does not exist. The actual file is `docs/src/features/authentication.md`.
**Fix:**
```markdown
**See also:** [Authentication guide](../features/authentication.md) for the complete auth setup walkthrough.
```

### WR-03: Broken intra-doc link — `../api-resources.md` in cli.md

**File:** `docs/src/reference/cli.md:707`
**Issue:** The `make:resource` section links to `[API Resources guide](../api-resources.md)`. Relative to `docs/src/reference/cli.md`, this resolves to `docs/src/api-resources.md`, which does not exist. The actual file is `docs/src/features/api-resources.md`.
**Fix:**
```markdown
**See also:** [API Resources guide](../features/api-resources.md) for the complete resource system documentation.
```

## Info

### IN-01: Migration guide title exposes internal codename "cancer" in public docs

**File:** `docs/src/upgrading/migration-guide.md:1`
**Issue:** The page title is "Migration Guide: cancer to ferro". The name "cancer" was the internal pre-publication codename (the git log shows `docs(22-02): add migration guide for cancer to ferro upgrade`). This is now a public page (the repo and docs are public). The heading reads awkwardly for new users who have never used the prior codename and may find the term surprising or confusing.
**Fix:** Retitle to a neutral framing, for example:
```markdown
# Migration Guide: v1.x to v2.0
```
or remove the old-name reference from the `h1` entirely, since the content already explains what changed.

### IN-02: `$schema` value `"ferro-json-ui/v2"` in generated JSON sample in cli.md

**File:** `docs/src/reference/cli.md:538`
**Issue:** The `make:json-view` generated file sample includes `"$schema": "ferro-json-ui/v2"`. The `feedback_json_ui_naming.md` memory note records that public docs must not use the "v2" label — it should be described as the current JSON-UI schema with no version comparison. This schema string in a code sample could reinforce the "v2" framing.
**Fix:** Replace with the canonical schema identifier used in the real generated output, e.g. `"ferro-json-ui/v1"` if that is the published schema value, or confirm the actual value emitted by `make:json-view` and align the sample. If the schema string itself is intentional, add a note clarifying it refers to the current schema format, not a version comparison against a legacy "v1".

---

_Reviewed: 2026-06-14T22:41:11Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: quick (with oracle source verification against ferro-cli/src/commands/)_
