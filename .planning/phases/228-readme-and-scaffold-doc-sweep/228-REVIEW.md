---
phase: 228-readme-and-scaffold-doc-sweep
reviewed: 2026-06-15T00:00:00Z
depth: quick
files_reviewed: 4
files_reviewed_list:
  - README.md
  - ferro-cli/src/templates/files/root/README.md.tpl
  - scripts/install.sh
  - scripts/create-app.sh
findings:
  critical: 0
  warning: 1
  info: 2
  total: 3
status: issues_found
---

# Phase 228: Code Review Report

**Reviewed:** 2026-06-15
**Depth:** quick (with targeted oracle reads per review focus note)
**Files Reviewed:** 4
**Status:** issues_found

## Summary

This review covers the Phase 228 README and scaffold doc sweep for Ferro v0.2.61: factual accuracy of CLI command references, install guidance consistency, MSRV/Node/SQLite defaults, template placeholder integrity, and shell safety.

All four template placeholders (`{project_title}`, `{description}`, `{project_name}`, and the three substitutions in `project.rs`) are intact and consistent with the substitution code. Shell scripts are structurally sound: `set -e` is present, `$TMP_DIR` is quoted in all unsafe positions, and the `trap "rm -rf $TMP_DIR" EXIT` pattern is correct (double-quoted so the path is baked in at definition time, not deferred — correct behavior here).

One phantom CLI command in the scaffold template is a user-visible correctness bug. Two info-level items cover minor documentation inconsistencies.

---

## Warnings

### WR-01: Phantom `ferro routes` command in scaffold README template

**File:** `ferro-cli/src/templates/files/root/README.md.tpl:44`
**Issue:** The generated project README lists `ferro routes` as "List all registered HTTP routes." This command does not exist in the CLI (`ferro-cli/src/main.rs` defines no `Routes` or `#[command(name = "routes")]` variant). The closest real command is `ferro generate-routes` (which emits TypeScript route helpers, not a human-readable route listing). Every project scaffolded with `ferro new` will contain this phantom command in its README, leading users to run a command that fails.
**Fix:** Replace the phantom with the real command. Options:
1. Remove the row entirely if there is no equivalent yet.
2. Replace with `ferro generate-routes` and update the description to "Generate TypeScript route helpers (`routes.ts`)."

```diff
-| `ferro routes`       | List all registered HTTP routes           |
+| `ferro generate-routes` | Generate TypeScript route helpers (`frontend/src/types/routes.ts`) |
```

---

## Info

### IN-01: `README.md` Quick Start omits `db:migrate` step

**File:** `README.md:17-26`
**Issue:** The project-level Quick Start block goes directly from `ferro serve` with no migration step. The scaffold template (`.tpl`) correctly shows `ferro db:migrate` before `ferro serve`. A first-time user following the repo README without running migrations will hit a database error on first request.
**Fix:** Add the migration step to match the scaffold template's flow:

```bash
ferro new myapp
cd myapp
ferro db:migrate
ferro serve
```

### IN-02: `scripts/create-app.sh` "Next steps" suggests `ferro db:migrate` without noting that the CLI must be on `$PATH`

**File:** `scripts/create-app.sh:138`
**Issue:** The "Next steps" block at line 138 instructs the user to run `ferro db:migrate`, but `create-app.sh` is a one-shot script that downloads the binary to a temp directory (cleaned up on exit). After the script finishes, `ferro` is not on `$PATH` unless the user has it installed permanently. The script does provide the Homebrew / cargo install hint (lines 141-143), but the ordering ("Next steps" first, install hint after) may leave users confused when `ferro db:migrate` fails with "command not found."
**Fix:** Reorder or annotate — either move the install hint before the "Next steps" block, or add a short note inline:

```diff
 echo "Next steps:"
+echo ""
+echo "  (Install the CLI first if you haven't already — see below)"
+echo ""
 printf "  ${CYAN}cd %s${NC}\n" "$PROJECT_NAME"
```

---

_Reviewed: 2026-06-15_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: quick (with targeted oracle reads)_
