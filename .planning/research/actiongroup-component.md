# ActionGroup — ferro-json-ui Action Primitive (research seed for Phase 237)

Grounding for the planner. ActionGroup formalizes the dashboard kebab/action
pattern as a first-class component and **replaces** `DropdownMenu` as the public
action primitive (decision: replace, not wrap). Source refs below are against the
ferro checkout at workspace version `0.2.71`; verify line numbers before editing.

## Decision: replace DropdownMenu

`ActionGroup` becomes the sole public action component. `DropdownMenu` is removed
from the public surface (props export, `BUILTIN_TYPES`, `BUILTIN_SPECS`, catalog).
Its kebab HTML/popover rendering may remain as an **internal** render helper that
`ActionGroup` calls for its overflow menu — keeping the kebab glyph, popover
anchoring, and destructive styling in one place — but no `DropdownMenu` spec is
authored by consumers anymore. Per project convention "delete old code completely,
no deprecation": once call sites migrate, the public `DropdownMenu` component is gone.

## What ActionGroup encapsulates (structural guarantees, not author discipline)

The dashboard conventions ("actions in PageHeader/DetailPage slot; kebab always
last; destructive always inside the kebab; ≤2–3 inline buttons; primary action
first") are today enforced only by hand. ActionGroup enforces them structurally:

1. One ordered input list; the component partitions inline vs. overflow.
2. Any `destructive: true` item is forced into the overflow kebab and rendered last.
3. `max_inline` (default 2) caps non-destructive inline buttons; remainder overflow.
4. A primary slot keeps the navigational action ("Vedi dettagli") first.
5. Non-GET inline buttons auto-wrap in `<form>` (the v1 Button-in-header limitation,
   today worked around by hand-built `form_toggle_active` siblings).
6. `items` accepts a literal array OR `{"$data":"/path"}` binding, with `{row_key}`
   substitution and `visible_if` row gates — parity with today's DropdownMenu so the
   server-built kanban action arrays flow in unchanged.

## Proposed prop shape (sketch — planner refines)

```rust
pub struct ActionItem {
    pub label: String,
    pub action: Action,
    #[serde(default)] pub destructive: bool,          // -> overflow, rendered last
    #[serde(default, skip_serializing_if = "Option::is_none")] pub variant: Option<ButtonVariant>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub visible_if: Option<String>,
}

pub struct ActionGroupProps {
    pub items: Vec<ActionItem>,                        // or {"$data":"/o/actions"}
    pub menu_id: String,                               // overflow popover id pairing
    #[serde(default, skip_serializing_if = "Option::is_none")] pub max_inline: Option<u8>,       // default 2
    #[serde(default, skip_serializing_if = "Option::is_none")] pub overflow_label: Option<String>, // aria-label, default "Azioni"
    #[serde(default, skip_serializing_if = "Option::is_none")] pub row_key: Option<String>,
}
```

## Existing components to read

- `Action` / `ConfirmDialog` / `ActionHandler` — `ferro-json-ui/src/action.rs:153,46,90`
- `ButtonProps` / `ButtonVariant` — `ferro-json-ui/src/component.rs:262,55`; render `render/atoms.rs:203`
- `DropdownMenuAction` / `DropdownMenuProps` — `component.rs:1058,1074`
- kebab render (reuse) — `render/atoms.rs:1154` (`render_dropdown_menu`), `render_menu_item`,
  kebab SVG `atoms.rs:1166`, destructive styling `atoms.rs:1184-1190`
- inline (DataTable/Kanban) variant + per-row gate — `render/data.rs:520`, `data.rs:445/468`
- `PageHeaderProps.actions: Vec<String>` slot — `component.rs:917`; render `containers.rs:597,609-613,653`
- `DetailPageProps` slot — `component.rs:1040`; render `containers.rs:685`
- `ButtonGroup` (existing gapped flex row) — `component.rs:931`; render `containers.rs:946`

## Registration surface (all must be touched — two drift guards enforce parity)

1. Props structs `ActionGroupProps` (+ `ActionItem`) in `component.rs` — derive
   `Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema`; `#[serde(default, skip_serializing_if=…)]` on optionals.
2. Public export in the `pub use component::{…}` block — `lib.rs:49-65`.
3. `BUILTIN_TYPES` add `"ActionGroup"` (containers section, since it resolves slots) — `render/mod.rs:43` / container area `:70`.
4. Dispatch arm `"ActionGroup" => render_action_group(...)` in `match el.type_name.as_str()` — `render/mod.rs:176-230`.
5. `render_action_group(...)` impl (likely `render/containers.rs`); reuse the kebab helper for overflow.
6. `BUILTIN_SPECS` tuple `("ActionGroup", "<desc>", schema_for!(ActionGroupProps), &[slot_fields])` — `catalog.rs:124`, same ordinal position as `BUILTIN_TYPES`.
7. Drift guards: runtime length check `catalog.rs:576`; count test `builtin_types_count_drift_guard` `catalog.rs:1093` (bump expected count).
8. Schema-nonempty test `assert_schema_nonempty_object::<ActionGroupProps>(...)` (pattern at `component.rs:1510-1515`).

Plus, under "replace": **remove** the `DropdownMenu` public entries from items 2/3/6 and
update the count in item 7; and update projection codegen `emit_actions_placeholder`
(`projection/builder.rs:672`) to emit an `ActionGroup` element instead of a `DropdownMenu`.

## Migration within ferro

- Update `emit_actions_placeholder` (projection codegen) to emit `ActionGroup`.
- Migrate any ferro-internal/example/test specs that author a `DropdownMenu`.
- Update json-ui docs (Phase 121 doc set) to document ActionGroup and drop DropdownMenu.

## Release

Version-bump the ferro workspace `0.2.71 → 0.2.72` and publish `ferro-json-ui` (and
the `ferro-rs` re-export) to crates.io. Gestiscilo's adoption phase depends on the
published `0.2.72`.
