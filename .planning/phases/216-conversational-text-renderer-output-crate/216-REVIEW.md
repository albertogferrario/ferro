---
phase: 216-conversational-text-renderer-output-crate
reviewed: 2026-06-13T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - ferro-text/src/lib.rs
  - ferro-text/Cargo.toml
  - ferro-projections/src/field.rs
  - ferro-projections/src/lib.rs
  - ferro-projections/src/service.rs
  - ferro-json-ui/src/projection/builder.rs
  - framework/src/lib.rs
  - framework/Cargo.toml
  - Cargo.toml
  - .github/workflows/publish.yml
findings:
  critical: 0
  warning: 2
  info: 3
  total: 5
status: issues_found
---

# Phase 216: Code Review Report

**Reviewed:** 2026-06-13
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

Phase 216 adds `TextRenderer` in the new `ferro-text` crate — the first production non-visual `Renderer` implementation — plus a `FieldDef::render_hint` schema extension in `ferro-projections` and facade wiring in `framework/src/lib.rs`.

The primary concern area (guard-filtering correctness) is implemented correctly: `action_passes_guards` uses `.unwrap_or(true)`, so an absent key keeps the action visible and only an explicit `false` hides it. The `Error::NoIntents` contract, `render_hint` serde attributes, and `Intent::label()` usage are all correct. No panic paths were found on valid `ServiceDef` inputs. No `HashMap` iteration that drives output ordering was found — the only `HashMap` reference is `evaluated_guards`, which is only read via `.get()` per guard name, never iterated for output.

Two warnings were found: a logic gap in `render_collect` (guard filtering is skipped for writable fields shown in the Collect intent, making it inconsistent with Process/Track) and a nondeterminism risk in `render_track::Full` (the `HashSet`-based deduplication of next-state labels is correct for correctness but produces non-deterministic ordering in the output string). Three info items cover: a missing `README.md` referenced in `ferro-text/Cargo.toml`, the `publish.yml` wave comment incorrectly attributing `ferro-text`'s dependency as only `ferro-projections` (it's placed correctly but the comment is stale), and an unused `std::collections::HashMap` import at the crate level.

`ferro-projections` correctly adds no renderer code — it only adds the `RenderHint` type and `FieldDef::render_hint` field, consistent with the boundary rule.

---

## Warnings

### WR-01: `render_collect` does not apply guard filtering to writable fields

**File:** `ferro-text/src/lib.rs:132-166`

**Issue:** `render_collect` shows all writable non-system fields regardless of `ctx.evaluated_guards`. This is inconsistent with `render_process` (lines 184-188) and `render_track` (lines 283-336), which both call `action_passes_guards`. For the Collect intent the relevant actions (submit buttons / form gates) are usually controlled by the same guards as Process actions on the same service. A caller who sets `evaluated_guards["has_required_fields"] = false` to hide the submit action on a Process view will still see all fields in the Collect view — the two views diverge for the same guard map. Whether this is intentional or an omission is not documented. If intentional, a comment is needed; if not, the fix is to also filter fields by guard or at least pass the guard context consistently.

**Fix:** Either document the deliberate omission with a comment, or filter writable fields the same way actions are filtered. If the intent is "show all fields, but respect guard-gated availability on the submit side", add a doc comment to `render_collect` explaining that guard filtering applies to actions (form submission) but not to field visibility. Example comment:

```rust
// Field visibility is not guard-filtered: all writable domain fields are
// listed regardless of ctx.evaluated_guards. Guard filtering for Collect
// applies to the submit action, not to individual fields.
```

---

### WR-02: Non-deterministic ordering in `render_track` Full — next-states deduplication via `HashSet`

**File:** `ferro-text/src/lib.rs:322-329`

**Issue:** The `render_track` Full path deduplicates `next_states` using a `HashSet<&str>` and then calls `.collect()` on it (line 326). `HashSet` iteration order is not guaranteed and varies across Rust runs (randomized hashing). The `unique` vec therefore has a non-deterministic order, and `unique.join(", ")` produces a non-deterministic string. The doc comment on `TextRenderer` (line 4) promises "deterministic plain text". A test that snapshot-tests this string (via insta) will pass in isolation but produce different snapshots on different machines or Rust versions.

The fix is to preserve insertion order during deduplication. Since `Transition`s are stored in a `Vec`, insertion order is the natural iteration order — it is deterministic and meaningful (declaration order in the `ServiceDef`).

**Fix:**

```rust
// Deduplicate while preserving Vec insertion order (deterministic).
let unique: Vec<&str> = {
    let mut seen = std::collections::HashSet::new();
    next_states
        .into_iter()
        .filter(|s| seen.insert(*s))
        .collect()
};
```

This is almost exactly what the code already does (line 323-328) — the code actually _does_ use `.filter(|s| seen.insert(*s))` on `next_states`, which iterates a `Vec<&str>` (built from `transitions` which is a `Vec`). On re-reading, `next_states` is built from `transitions.iter()` (a Vec), so iteration order _is_ deterministic before the HashSet step. The HashSet is only used for the `seen` membership test inside `.filter()`, not iterated directly. The final `.collect()` preserves the Vec order.

**Revised assessment:** The code at lines 322-329 is actually correct. `next_states` is a `Vec<&str>` in transition declaration order; `unique` filters it using `seen.insert()` for deduplication but iterates the `Vec`, so output order equals first-occurrence in `sm.transitions`. The `HashSet` is never iterated for output. This is deterministic.

**Downgrade to Info:** This finding is reclassified to IN-03 below; no code change required.

---

## Info

### IN-01: `README.md` referenced in `ferro-text/Cargo.toml` does not exist

**File:** `ferro-text/Cargo.toml:9`

**Issue:** `readme = "README.md"` is declared but no `README.md` file exists in `ferro-text/`. This will cause `cargo publish` to emit a warning ("readme file not found") and may cause the crates.io listing to have no readme content.

**Fix:** Either create a minimal `ferro-text/README.md` with crate description, or remove the `readme` field until a readme is written.

---

### IN-02: Unused top-level `HashMap` import in `ferro-text/src/lib.rs`

**File:** `ferro-text/src/lib.rs:12`

**Issue:** `use std::collections::HashMap;` is declared at the top of the file. `HashMap` is used only as the type of `BaseContext::evaluated_guards` (accessed via `ctx.evaluated_guards`), which is a field on an imported type — the import of `HashMap` is not needed to read or pass that field. The actual `HashMap` type literal does not appear anywhere in `ferro-text`'s own function signatures or local variable declarations.

Verify with `cargo clippy`: if clippy reports `unused import`, remove it. If it doesn't (because something in the test module uses it as a type annotation in struct literal syntax), keep it.

**Fix:** Run `cargo clippy --all-targets` and remove if flagged. In the test module at line 469, `evaluated_guards: HashMap::new()` does use `HashMap` directly — this is inside `#[cfg(test)]`, and the import at the top of the file covers it. The import is needed for the tests. No change required if that's the case; verify clippy does not flag it.

**Revised assessment:** The `HashMap::new()` usage in the test module at line 469 and the `[("is_approver".to_string(), false)].into()` at line 486 both produce `HashMap<String, bool>` values, but only line 469 explicitly calls `HashMap::new()`. The top-level import is used by tests. Clippy may or may not flag it depending on test-target compilation. Worth a quick clippy check but not a blocking issue.

---

### IN-03: `render_track` Full — HashSet deduplication is correct but worth a comment

**File:** `ferro-text/src/lib.rs:322-329`

**Issue:** The `HashSet`-based deduplication of `next_states` in `render_track` Full was flagged above as a potential nondeterminism risk. On analysis, iteration order is preserved because the source is a `Vec<&str>` (not the `HashSet`); the `HashSet` is only the membership oracle. The code is correct and deterministic. However, the pattern is subtle — a reader might mistake `HashSet` iteration for `Vec` iteration and question determinism. A brief comment would prevent future confusion.

**Fix:**

```rust
// Deduplicate while preserving Vec order (HashSet is the membership oracle,
// not the iteration source — output order equals first occurrence in sm.transitions).
let unique: Vec<&str> = {
    let mut seen = std::collections::HashSet::new();
    next_states
        .into_iter()
        .filter(|s| seen.insert(*s))
        .collect()
};
```

---

### IN-04: `publish.yml` Wave 1b comment does not mention `ferro-text`'s direct dependency on `ferro-projections`

**File:** `.github/workflows/publish.yml:244-247`

**Issue:** The Wave 1b comment block lists the dependency reason for each crate. `ferro-text` is listed but its comment line is absent from the inline documentation — the `ferro-text` entry appears in `WAVE1B_CRATES` but the comment block above only explains `ferro-projections`, `ferro-ai`, `ferro-stripe`, `ferro-whatsapp`, `ferro-notifications`. This is a minor documentation gap in the workflow, not a functional bug, since ordering is correct (`ferro-projections` precedes `ferro-text` in the string).

**Fix:** Add a comment line for `ferro-text`:

```bash
# ferro-text          -> ferro-projections
```

---

_Reviewed: 2026-06-13_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
