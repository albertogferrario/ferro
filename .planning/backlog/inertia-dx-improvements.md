# Inertia DX Improvements

Discovered during mkmenu development. To be addressed post-v2.0 rebrand.

## Priority Order

| # | Issue | Severity | Effort |
|---|-------|----------|--------|
| 1 | Shared Props Mechanism | Medium | High |
| 2 | SavedInertiaContext Documentation | Low | Low |
| 3 | Auto Type Generation | Low | Medium |
| 4 | JSON Accept Header Handling | Low | Low |
| 5 | Type Generation Gaps | Medium | Medium |

---

## Issue 1: No Built-in Shared Props Mechanism

**Problem:** Every Inertia page requires manually including common props (user, tenant) in every handler.

**Current Pattern:**
```rust
#[derive(InertiaProps)]
struct PageProps {
    user: UserInfo,      // Repeated in every struct
    tenant: TenantInfo,  // Repeated in every struct
    // ... actual page props
}
```

**Desired Pattern:**
```rust
// In middleware - done once
Inertia::share("user", |req| async { get_current_user(req).await });
Inertia::share("tenant", |req| async { get_current_tenant(req).await });

// Props automatically merged at render time
```

**Implementation Notes:**
- Store shared prop closures in app state
- Merge shared props with page props in `Inertia::render()`
- Consider lazy evaluation (only call closures if prop not already provided)
- TypeScript: shared props would need a base type that all pages extend

---

## Issue 2: SavedInertiaContext Documentation

**Problem:** Request body consumption before Inertia render is confusing.

**Current Workaround:**
```rust
let ctx = SavedInertiaContext::from(&req);
let form: CreateMenuRequest = req.input().await?;  // Consumes req
Inertia::render_ctx(&ctx, "Form", props)  // Use saved ctx
```

**Action:** Add prominent documentation section in:
- `docs/src/inertia/forms.md`
- `framework/src/inertia/mod.rs` doc comments
- CLI scaffold templates (already have this pattern)

**Nice-to-have API:**
```rust
// Extracts input and preserves Inertia context in one call
let (form, ctx): (CreateMenuRequest, InertiaContext) = req.inertia_input().await?;
```

---

## Issue 3: Auto Type Generation

**Problem:** Forgetting to run `ferro generate-types` after changing props causes runtime undefined errors.

**Options:**
1. **File watcher in dev mode** - ON by default, `--no-watch-types` to disable
2. **Build script integration** - Generate types as part of `cargo build`
3. **Runtime validation** - Dev-mode warning when props don't match TS types

**Decision:** Option 1 (file watcher) - ON by default for best DX. Use `--no-watch-types` to disable if needed.

---

## Issue 4: JSON Accept Header Handling

**Problem:** `Accept: application/json` without `X-Inertia` returns HTML.

**Current Behavior:** Inertia protocol requires `X-Inertia: true` header for JSON.

**Suggested Enhancement:**
```rust
// In Inertia middleware
if req.accepts("application/json") && !req.has_header("X-Inertia") {
    // Return raw props as JSON for API clients/testing
    return Response::json(props);
}
```

**Consideration:** This changes Inertia semantics. May want this as opt-in behavior:
```rust
Inertia::render(&req, "Page", props).with_json_fallback()
```

---

## Issue 5: Type Generation Gaps

**Discovered:** Phase 12 (mkmenu theme configuration)

**Problem:** `ferro generate-types` fails to generate several commonly-needed types:

### 5a. JsonValue not generated

When using `sea_orm::entity::prelude::Json` in models:
```rust
pub struct Model {
    pub theme_config: Option<Json>,  // sea_orm Json type
}
```

The generated TypeScript uses `JsonValue` but never defines it:
```typescript
export interface MenuDetail {
  theme_config: unknown | null;  // Should be JsonValue, but ok as unknown
}
```

**Fix:** Add to generate_types.rs type mappings:
```rust
"sea_orm::entity::prelude::Json" => "JsonValue",
// And emit: export type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };
```

### 5b. ValidationErrors not exported

ValidationErrors is mapped inline but not exported as a standalone type:
```typescript
// Current: only appears inline
export interface LoginProps {
  errors: { [key: string]: string[] } | null;
}

// Need: also export the type for reuse
export type ValidationErrors = { [key: string]: string[] };
```

**Current workaround in mkmenu:**
```typescript
const e = errors as Record<string, string[]>
```

### 5c. Custom Serialize types not discovered

Types like `ThemeConfig` that are used in props but don't have `#[derive(InertiaProps)]`:
```rust
// In controller
let props = MenuFormProps {
    menu: MenuDetail {
        theme_config: menu.theme_config.clone(), // Json → ThemeConfig shape
    },
    // ...
};
```

The generator only scans for `#[derive(InertiaProps)]` structs, missing nested types.

**Options:**
1. Scan for all `Serialize` types used in InertiaProps fields
2. Add `#[inertia_type]` attribute for explicit inclusion
3. Accept manual type files (current workaround - created `theme-config.ts`)

**Current workaround in mkmenu:**
```typescript
// frontend/src/types/theme-config.ts - manually created
export interface ThemeConfig {
  bottomNav?: BottomNavConfig;
  quickGrid?: QuickGridConfig;
  header?: HeaderConfig;
  filters?: FiltersConfig;
}
```

**Recommended approach:** Option 3 is pragmatic. Document that:
- Auto-generated types cover props structs
- Custom/nested types need manual TypeScript files
- Consider adding an `#[inertia_include]` attribute for v2.1

---

## Timeline

Address after v2.0 Rebrand complete (Phase 22). These are DX improvements, not blockers.
