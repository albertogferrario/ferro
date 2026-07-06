# Issue: generate-types missing nested/referenced types

## Summary

The `ferro generate-types` command generates TypeScript interfaces for page props (structs marked with `#[derive(InertiaProps)]`) but does **not** generate the nested types that these props reference.

## Reproduction

1. In mkmenu-ferro project, run: `ferro generate-types`
2. Check `frontend/src/types/inertia-props.ts`

## Expected Behavior

The generated file should include all types referenced by the page props, for example:
- `UserInfo`
- `TenantInfo`
- `MenuSummary`, `MenuDetail`, `MenuOption`
- `CategorySummary`, `CategoryDetail`, `CategoryWithItems`
- `ItemSummary`, `ItemDetail`
- `QRCodeSummary`
- `PublicTenant`, `PublicMenu`, `PublicCategory`, `PublicItem`
- `Value` (for generic JSON values)

## Actual Behavior

The generated file only contains the page prop interfaces like `DashboardProps`, `MenuListProps`, etc. but references undefined types:

```typescript
export interface MenuListProps {
  menus: MenuSummary[];  // <- MenuSummary is NOT defined
  user: UserInfo;        // <- UserInfo is NOT defined
  tenant: TenantInfo;    // <- TenantInfo is NOT defined
}
```

This causes TypeScript compilation errors:
```
error TS2304: Cannot find name 'MenuSummary'.
error TS2304: Cannot find name 'UserInfo'.
error TS2304: Cannot find name 'TenantInfo'.
```

## Root Cause

The type generator in `ferro-cli/src/commands/generate_types.rs` (or similar) parses `InertiaProps` structs but doesn't recursively collect and generate the types of their fields.

## Proposed Solution

1. When parsing InertiaProps structs, collect all referenced type names
2. Search the codebase for those type definitions (likely in `src/models/` or `src/types/`)
3. Generate TypeScript interfaces for all referenced types
4. Handle nested types recursively (e.g., `CategoryWithItems` contains `ItemSummary[]`)
5. Add a `Value` type alias: `export type Value = Record<string, unknown>;`

## Workaround

Manually define all referenced types in a separate file or at the top of `inertia-props.ts` before the generated content.

## Files to Investigate

- `ferro-cli/src/commands/generate_types.rs` (or wherever the generator lives)
- Look for how InertiaProps structs are parsed and how field types are handled

## Priority

High - This makes the type generator unusable for any project with nested types.
