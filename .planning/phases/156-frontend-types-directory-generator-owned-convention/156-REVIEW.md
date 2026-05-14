---
phase: 156-frontend-types-directory-generator-owned-convention
reviewed: 2026-05-14T00:00:00Z
depth: standard
files_reviewed: 17
files_reviewed_list:
  - ferro-cli/src/templates/files/root/gitignore.tpl
  - ferro-cli/src/commands/generate_types.rs
  - ferro-cli/src/doctor/checks/frontend_types_convention.rs
  - ferro-cli/src/doctor/checks/mod.rs
  - ferro-cli/src/doctor/registry.rs
  - ferro-cli/src/doctor/check.rs
  - ferro-cli/src/templates/docker.rs
  - ferro-cli/src/commands/docker_init.rs
  - ferro-cli/src/doctor/checks/docker_template_drift.rs
  - ferro-cli/tests/gestiscilo_fixture.rs
  - docs/src/cli/frontend-types.md
  - docs/src/SUMMARY.md
  - docs/src/cli/doctor.md
  - docs/src/reference/cli.md
  - ferro-cli/src/templates/files/root/README.md.tpl
  - Cargo.toml
  - CHANGELOG.md
findings:
  critical: 0
  warning: 3
  info: 3
  total: 6
status: issues_found
---

# Phase 156: Code Review Report

**Reviewed:** 2026-05-14
**Depth:** standard
**Files Reviewed:** 17
**Status:** issues_found

## Summary

Phase 156 introduces the generator-owned convention for `frontend/src/types/`: the `.gitignore` template now excludes the directory, the new `FrontendTypesConventionCheck` warns on hand-written files in it, the Dockerfile renderer gains a `types-gen` Rust stage to regenerate those files inside Docker builds, and a `ferro_version` resolution function ties them all together. The implementation is coherent and tests are thorough.

Three warnings and three info items were found. None are correctness-critical; two warnings are real logic gaps that could produce incorrect behavior in edge cases.

## Warnings

### WR-01: `topological_sort` silently drops structs with cycles

**File:** `ferro-cli/src/commands/generate_types.rs:546-594`

**Issue:** Kahn's algorithm in `topological_sort` processes only nodes with in-degree zero. If circular type references exist (e.g., struct `A` has a field of type `B` and `B` has a field of type `A`), neither node ever reaches in-degree zero, so both are silently dropped from the `result` vec. The generated TypeScript file would then be missing those interfaces entirely, with no warning to the user.

**Fix:** After the Kahn loop, check whether `result.len() < structs.len()` and emit a warning or append the remaining (cycle-forming) structs in an unordered fallback pass:

```rust
// after the while loop:
if result.len() < structs.len() {
    // cycle detected — append unreachable structs in original order
    eprintln!("Warning: circular type references detected; ordering may be incorrect.");
    for s in structs {
        if !result.iter().any(|r| r.name == s.name) {
            result.push(s);
        }
    }
}
```

---

### WR-02: `build_name_map` loses one of two same-named structs from different modules

**File:** `ferro-cli/src/commands/generate_types.rs:739-768`

**Issue:** `build_name_map` is keyed by the original struct name (`s.name`), not the namespaced name. When two structs from different modules both have the same Rust name (e.g., `ShowProps` from `shelter::applications` and `ShowProps` from `adopter::applications`), both produce different namespaced names (`ShelterApplicationsShowProps` vs `AdopterApplicationsShowProps`), but the map is inserted as:

```rust
name_map.insert(s.name.clone(), namespaced.clone());
```

The second insertion silently overwrites the first. `generate_typescript` then calls `name_map.get(&s.name)` for each struct. The first `ShowProps` in iteration order will get the second one's namespaced name. The test `test_build_name_map_no_collisions` at line 1706 acknowledges this in a comment: *"Since both have the same name "ShowProps", the last one wins in the map."* The collision detector at line 752 fires on the reverse map but the forward map is still wrong: one struct gets the wrong TypeScript name.

The test accepts this silently; the production code will emit one correctly-named interface and one with the wrong name — a compile error in the TypeScript consumer.

**Fix:** Key the forward map by `(module_path, name)` rather than just `name`, and resolve the mapping from the same composite key during generation:

```rust
// Key: (module_path, original_name) -> namespaced_name
let mut name_map: HashMap<(String, String), String> = HashMap::new();
// ...
name_map.insert((s.module_path.clone(), s.name.clone()), namespaced.clone());
```

Then pass module path alongside struct name when resolving: look up `(s.module_path, field_type_name)` at render time. This requires `module_path` to be preserved through `collect_custom_types`, which it currently is not — so the simplest short-term fix is to namespace all Custom types by scanning the structs list for a match first, or to emit the collision as an error (not just a warning to stderr) so users know generation is broken.

---

### WR-03: `FrontendTypesConventionCheck` flags subdirectories as hand-written files

**File:** `ferro-cli/src/doctor/checks/frontend_types_convention.rs:40-53`

**Issue:** `std::fs::read_dir` returns both files and directories. If a user has a subdirectory inside `frontend/src/types/` (not uncommon if someone accidentally creates one), `e.file_name()` returns the directory's name and it is compared against the `GENERATED_ALLOWLIST`. Any subdirectory name not in the allowlist will be flagged as a hand-written file. The message `"move to frontend/src/lib/types/: <dirname>"` would then be misleading for a directory entry.

**Fix:** Filter entries to files only before checking against the allowlist:

```rust
.filter_map(|e| {
    // skip subdirectories
    if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
        return None;
    }
    let name = e.file_name().to_string_lossy().into_owned();
    if GENERATED_ALLOWLIST.contains(&name.as_str()) {
        None
    } else {
        Some(name)
    }
})
```

## Info

### IN-01: `#[allow(dead_code)]` annotation on `module_path` is left in place

**File:** `ferro-cli/src/commands/generate_types.rs:93-94`

**Issue:** The `module_path` field on `InertiaPropsStruct` carries an `#[allow(dead_code)]` annotation with the comment "Will be used in namespaced interface generation (Task 2)". Namespaced generation is now implemented (`build_name_map`, `generate_namespaced_name`). The annotation and comment are now misleading — the field is actively used.

**Fix:** Remove the `#[allow(dead_code)]` attribute and its comment:

```rust
// before:
#[allow(dead_code)] // Will be used in namespaced interface generation (Task 2)
pub module_path: String,

// after:
pub module_path: String,
```

---

### IN-02: `cli.md` project structure diagram shows the old `frontend/src/types/inertia.d.ts` layout

**File:** `docs/src/reference/cli.md:84`

**Issue:** The generated structure diagram under `ferro new` shows `frontend/src/types/inertia.d.ts` as a committed scaffold file. Phase 156's convention is that `frontend/src/types/` is gitignored and owned by the generator; there should be no scaffolded file there. A developer following this diagram could believe committing files to `frontend/src/types/` is expected.

**Fix:** Remove the `types/` subtree from the diagram, or replace it with a note explaining the directory is generated:

```
frontend/
  src/
    pages/
      └── Home.tsx
    layouts/
      └── Layout.tsx
    # types/ is gitignored — generated by `ferro generate-types` / `cargo run`
    app.tsx
    main.tsx
```

---

### IN-03: `README.md.tpl` troubleshooting section references `cargo run` as the type-generation trigger

**File:** `ferro-cli/src/templates/files/root/README.md.tpl:85`

**Issue:** The troubleshooting entry says *"run `cargo run` once to generate types before running `npm run dev`"*. The dedicated command is `ferro generate-types` (or `ferro serve`). Pointing users to `cargo run` bypasses the CLI entry point and may confuse them if the project has a custom binary or if `cargo run` invokes a setup step that is not equivalent to a plain serve boot.

**Fix:** Align with the actual command surface:

```
- **TypeScript errors about `Cannot find module './types/inertia-props'`** — run
  `ferro generate-types` (or start the server with `ferro serve`) to populate
  `frontend/src/types/` before running `npm run dev`.
```

---

_Reviewed: 2026-05-14_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
