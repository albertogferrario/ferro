---
phase: 147
plan: 02
subsystem: ferro-json-ui
tags: [detail-form, component-catalog, serde, edit-mode]
requires:
  - 147-01 (Wave 0 RED scaffold — detail_form_tests module exists with 13 tests, render/resolver/mcp RED tests land via sibling plans)
provides:
  - EditMode enum with View/Edit variants, snake_case serde, from_query() ASCII-case-insensitive constructor
  - DetailField struct (label/value/input) with new() convenience constructor
  - DetailFormProps struct with nine fields, Option skip-serialize discipline
  - Component::DetailForm(DetailFormProps) variant in the Component enum
  - Tagged-serde Serialize + Deserialize arms for "DetailForm"
  - ComponentNode::detail_form(key, props) factory
  - Public re-exports of DetailField, DetailFormProps, EditMode from crate root
  - ### DetailForm block in COMPONENT_CATALOG documenting the Option-A authoring rule
affects:
  - ferro-json-ui public surface (three new exported types)
  - Component enum ABI (new variant — pre-1.0, breaking changes acceptable)
tech-stack:
  added: []
  patterns:
    - form-family struct grouping (EditMode/DetailField/DetailFormProps placed right after FormProps)
    - serde-tagged enum extension (variant + Serialize arm + Deserialize arm + factory, all keyed to the same "DetailForm" tag string)
    - JsonSchema skipped for props containing ComponentNode (matches FormProps / Tab precedent)
    - Option field discipline (#[serde(default, skip_serializing_if = "Option::is_none")]) on every Option<_> field
    - Default via #[serde(default)] for the mode field so missing-in-JSON deserializes to View
key-files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs (+134 lines, -0): three type defs + enum variant + serde arms + factory
    - ferro-json-ui/src/lib.rs (+12 lines, -7): public re-exports + COMPONENT_CATALOG entry
decisions:
  - "EditMode.from_query uses eq_ignore_ascii_case (zero-alloc, locale-independent) per Pitfall 4 / D-02"
  - "DetailField / DetailFormProps skip JsonSchema because Component has custom Serialize/Deserialize (Tab precedent)"
  - "Component::DetailForm positioned between KeyValueEditor and Plugin (family grouping; Plugin last)"
  - "ComponentNode::detail_form factory rustdoc cites UI-SPEC §5 (structural coherence) and §9 (Option-A) for MCP discovery"
  - "COMPONENT_CATALOG ### DetailForm block restates the Option-A empty-label rule in actionable terms so ferro-mcp-driven agents discover it per UI-SPEC §14.8"
metrics:
  duration: 5m 32s
  completed: "2026-04-22"
  tasks: 3
  files: 2
  commits: 3
  lines_added: 146
  lines_removed: 7
---

# Phase 147 Plan 02: DetailForm Rust types + serde plumbing Summary

Add the Rust types (EditMode, DetailField, DetailFormProps), Component enum variant, serde match arms, ComponentNode factory, public re-exports, and COMPONENT_CATALOG entry that make the Wave-0 `mod detail_form_tests` in ferro-json-ui/src/component.rs turn GREEN. No renderer, no resolver arms, no ferro-mcp catalog — those remain with their respective sibling plans.

## What was built

Three structural types in `ferro-json-ui/src/component.rs`:

- `EditMode` enum (lines 211-227) — `Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema` with `#[serde(rename_all = "snake_case")]`; two variants `View` (default) and `Edit`. Impl block lines 229-243 adds `from_query(raw: Option<&str>) -> Self` that returns `Edit` only when `raw.eq_ignore_ascii_case("edit")` succeeds (Pitfall 4: zero-alloc, locale-independent — no `to_lowercase` allocation).
- `DetailField` struct (lines 250-273) — three public fields `label: String`, `value: String`, `input: ComponentNode`; derives `Debug, Clone, PartialEq, Serialize, Deserialize` (no JsonSchema). `DetailField::new(label, value, input)` convenience constructor mirrors `ComponentNode::input(...)` ergonomics.
- `DetailFormProps` struct (lines 284-311) — nine fields exactly per D-04; `mode: EditMode` uses `#[serde(default)]` so missing-in-JSON defaults to View; every `Option<_>` field uses `#[serde(default, skip_serializing_if = "Option::is_none")]`; `action` and `method` reference `crate::action::{Action, HttpMethod}` by crate path.

Component enum + serde wiring in the same file:

- Line 1096 — `DetailForm(DetailFormProps),` variant inserted between `KeyValueEditor(KeyValueEditorProps),` and `Plugin(PluginProps),` (family grouping; Plugin remains last).
- Line 1164 — `Component::DetailForm(p) => serialize_tagged(serializer, "DetailForm", p),` in the `Serialize` impl, positioned right after the `KeyValueEditor` arm and before the `Plugin` arm.
- Lines 1301-1303 — `"DetailForm" => serde_json::from_value::<DetailFormProps>(value).map(Component::DetailForm).map_err(de::Error::custom),` in the `Deserialize` impl, immediately after the `"KeyValueEditor"` arm and immediately before the `_ =>` Plugin fallback.
- Lines 1378-1392 — `pub fn detail_form(key: impl Into<String>, props: DetailFormProps) -> Self` factory placed immediately after `ComponentNode::form`. Rustdoc cites 147-UI-SPEC §5 (structural coherence contract) and §9 (Option-A empty-label authoring rule) per the UI-SPEC §14.7 acceptance requirement.

Public surface in `ferro-json-ui/src/lib.rs`:

- Lines 64-65 — `DetailField`, `DetailFormProps`, and `EditMode` inserted into the `pub use component::{…}` block. rustfmt re-wrapped the block; alphabetical order holds (verified with `awk`-based script: no out-of-order entries).
- Line 120 — `### DetailForm` block added to `COMPONENT_CATALOG` immediately after `### Form`. Description restates the Option-A authoring rule verbatim (`when DetailField.input is an Input/Select/Textarea/Checkbox/Switch component, the caller MUST set its label to "" — the <dt> provides the visible label`) so MCP-driven agents discover the rule without having to read UI-SPEC §9.

## Commits (3)

| Task | Description                                                                                           | Commit     |
| ---- | ----------------------------------------------------------------------------------------------------- | ---------- |
| 1    | `feat(147-02): add EditMode, DetailField, DetailFormProps types`                                      | `aa3dd256` |
| 2    | `feat(147-02): wire Component::DetailForm variant + serde arms + factory`                             | `f3ba25e1` |
| 3    | `feat(147-02): re-export DetailField/DetailFormProps/EditMode + add DetailForm to COMPONENT_CATALOG`  | `728afd66` |

## Rustfmt re-wrap for the re-export block

After inserting the three new names and running `cargo fmt --all`, rustfmt produced (lib.rs:59-72):

```rust
pub use component::{
    ActionCardProps, ActionCardVariant, AlertProps, AlertVariant, AvatarProps, BadgeProps,
    BadgeVariant, BreadcrumbItem, BreadcrumbProps, ButtonGroupProps, ButtonProps, ButtonType,
    ButtonVariant, CardProps, CheckboxProps, ChecklistItem, ChecklistProps, CollapsibleProps,
    Column, ColumnFormat, Component, ComponentNode, DataTableProps, DescriptionItem,
    DescriptionListProps, DetailField, DetailFormProps, DropdownMenuAction, DropdownMenuProps,
    EditMode, EmptyStateProps, FormMaxWidth, FormProps, FormSectionProps, GapSize, GridProps,
    HeaderProps, IconPosition, ImageProps, InputProps, InputType, KanbanBoardProps,
    KanbanColumnProps, KeyValueEditorProps, ModalProps, NotificationDropdownProps,
    NotificationItem, Orientation, PageHeaderProps, PaginationProps, PluginProps, ProductTileProps,
    ProgressProps, SelectOption, SelectProps, SeparatorProps, SidebarGroup, SidebarNavItem,
    SidebarProps, Size, SkeletonProps, SortDirection, StatCardProps, SwitchProps, Tab, TableProps,
    TabsProps, TextElement, TextProps, ToastProps, ToastVariant,
};
```

Rustfmt shifted `EditMode, EmptyStateProps, FormMaxWidth, FormProps` onto their own line together and compacted the downstream lines. The alphabetical-order invariant verification (`awk 'NR>1 { if ($0 < prev) { print "OUT OF ORDER"; exit 1 } }'`) returns clean.

## Targeted test gate — parallel-wave compilation state

**Gate design per 147-02-PLAN.md `<wave_1_green_expectation>`:** Other Wave 1 plans (147-03 render.rs, 147-04 resolve.rs, 147-05 json_ui_catalog.rs) run in parallel worktrees and each is responsible for its own production match arms against the new `Component::DetailForm` variant. Until those worktrees merge back with this one, the `ferro-json-ui` library does not compile standalone — five E0004 non-exhaustive-match errors surface at `render.rs:102`, `render.rs:290`, `resolve.rs:37`, `resolve.rs:210`, `resolve.rs:378` because adding the enum variant made five existing exhaustive matches incomplete.

This is the expected parallel-wave state — plan 02's scope boundary explicitly prohibits touching `render.rs` or `resolve.rs`, and plans 03/04 have explicit negative checks that `Component::DetailForm(_)` MUST NOT appear in any leaf catch-all in those files. The reconciliation happens when all three worktrees merge.

**What is verifiable today (targeted, per this plan's commits):**

- `cargo check -p ferro-json-ui` (lib-only, no tests) — fails with 5 E0004 errors at the four sites listed above. Expected until Plan 03/04 arms land.
- `cargo fmt --all -- --check` — exits 0 clean.
- Structural greps (every acceptance-criterion grep from 147-02-PLAN.md) — all pass:
  - `grep -q 'pub enum EditMode'` ✓
  - `grep -q 'pub fn from_query(raw: Option<&str>) -> Self'` ✓
  - `grep -q 's.eq_ignore_ascii_case("edit")'` ✓
  - `grep -q 'pub struct DetailField'` ✓
  - `grep -q 'pub struct DetailFormProps'` ✓
  - `grep -q '// JsonSchema skipped: contains ComponentNode'` ✓
  - `grep -q '// JsonSchema skipped: contains Vec<DetailField>'` ✓
  - `grep -q 'DetailForm(DetailFormProps),'` ✓
  - `grep -q 'Component::DetailForm(p) => serialize_tagged(serializer, "DetailForm", p)'` ✓
  - `grep -q '"DetailForm" => serde_json::from_value::<DetailFormProps>'` ✓
  - `grep -q 'pub fn detail_form(key: impl Into<String>, props: DetailFormProps) -> Self'` ✓
  - Ordering check (grep -B1/-A1 around each arm) — enum, Serialize, Deserialize all show KeyValueEditor above and Plugin/fallback below ✓
  - `DetailField`, `DetailFormProps`, `EditMode` all in `lib.rs` ✓
  - `### DetailForm` heading in COMPONENT_CATALOG ✓
  - Option-A rule (`Authoring rule (Option A)`, `label to ""`) present in catalog ✓
  - Alphabetical-order verifier script on the `pub use component::{…}` block returns clean ✓

**Thirteen Plan 01 Task 1 tests — GREEN-in-principle:** The 13 tests in `mod detail_form_tests` (component.rs:3704-3921) are fully matched by the types and variant this plan adds. They will run GREEN the instant plans 03/04 land their render.rs/resolve.rs arms (which resolve the library's non-exhaustive-match errors). Breakdown:

- 10 EditMode tests — depend only on types added in Task 1
- `detail_form_props_serde_roundtrip`, `detail_form_props_omits_optional_nones`, `detail_form_props_defaults_mode_to_view` — depend on types + serde arms added across Tasks 1-2
- `component_node_detail_form_factory_shape` — depends on factory added in Task 2

None of the 13 tests depend on any code owned by plans 03/04/05.

## Deviations from Plan

### Scope boundary — compilation-blocking enum additions

- **Found during:** Task 2 (after adding the `DetailForm` variant to the Component enum).
- **Issue:** Adding the enum variant broke five previously exhaustive matches in `render.rs` (lines 102, 290) and `resolve.rs` (lines 37, 210, 378), producing compile-time E0004 errors. The 147-02-PLAN.md explicitly forbids touching those files.
- **Fix:** None applied — honored the explicit scope boundary and the parallel-plan negative-check constraints (plans 03/04 require `Component::DetailForm(_)` NOT appear in any leaf `|`-chain catch-all). The compilation failure is the expected parallel-wave integration state, as documented in 147-02-PLAN.md `<wave_1_green_expectation>`. All acceptance-criterion greps pass; the 13 Plan 01 Task 1 tests will turn GREEN at wave integration time.
- **Impact:** The targeted-test gate (`cargo test -p ferro-json-ui --lib component::detail_form_tests`) cannot run standalone from this worktree. Structural correctness is verified via grep + format checks.
- **Files modified:** None (the deviation was to NOT expand scope).
- **Commits:** n/a.

## Auto-fixed Issues

None beyond the scope-boundary deviation noted above. All grep acceptance criteria for Tasks 1-3 pass on first try. No Rule 1 bugs, no Rule 2 missing-critical-functionality, no Rule 3 unrelated blocking issues.

## Surprises

- **Action struct derivation:** The existing `Action` struct in `ferro-json-ui/src/action.rs` (lines 69-88) is complete and Default-friendly without modification. The Plan 01 test helper `sample_detail_form_props()` (component.rs:3776-3839) constructs `Action { handler, url: Some(...), method, confirm: None, on_success: None, on_error: None, target: None }` directly — no `Default` derivation or builder-based shortcuts needed. No changes to `action.rs` were required.
- **InputProps derivation:** Similarly, the Plan 01 test helper constructs `InputProps` with every field explicit (field, label, input_type, placeholder, required, disabled, error, description, default_value, data_path, step, list), and each field's default is trivial. No `Default` impl was needed.
- **No rustfmt surprises:** the block wrap produced by rustfmt after three insertions moved `EditMode` to group with `EmptyStateProps, FormMaxWidth, FormProps`, which is the natural alphabetical neighborhood — reads cleanly.

## Self-Check: PASSED

File existence (confirmed via filesystem):
- `ferro-json-ui/src/component.rs` — FOUND (modified; 4056 lines after edits, +134 insertions)
- `ferro-json-ui/src/lib.rs` — FOUND (modified; 192 lines after edits)
- `.planning/phases/147-detailform-component-for-inline-edit-ferro-json-ui/147-02-SUMMARY.md` — FOUND (this file)

Commit existence (confirmed via `git log`):
- `aa3dd256` — FOUND: `feat(147-02): add EditMode, DetailField, DetailFormProps types`
- `f3ba25e1` — FOUND: `feat(147-02): wire Component::DetailForm variant + serde arms + factory`
- `728afd66` — FOUND: `feat(147-02): re-export DetailField/DetailFormProps/EditMode + add DetailForm to COMPONENT_CATALOG`
