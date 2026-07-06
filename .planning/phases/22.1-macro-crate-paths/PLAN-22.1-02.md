# Plan 22.1-02: Update All Macros to Use Dynamic Crate Resolution

```yaml
phase: 22.1
plan: 02
type: implementation
wave: 2
depends_on: [22.1-01]
files_modified:
  - ferro-macros/src/cancer_test.rs
  - ferro-macros/src/domain_error.rs
  - ferro-macros/src/inertia.rs
  - ferro-macros/src/injectable.rs
  - ferro-macros/src/redirect.rs
  - ferro-macros/src/service.rs
  - ferro-macros/src/test_macro.rs
autonomous: true
```

## Objective

Replace all hardcoded `::ferro_rs::` paths with dynamic crate resolution using the `ferro_crate()` helper from Plan 22.1-01.

## Context

There are 7 files with ~47 total occurrences of hardcoded paths:
- `cancer_test.rs`: 6 occurrences
- `domain_error.rs`: 3 occurrences
- `inertia.rs`: 11 occurrences
- `injectable.rs`: 10 occurrences
- `redirect.rs`: 2 occurrences
- `service.rs`: 6 occurrences
- `test_macro.rs`: 9 occurrences

## Pattern

**Before:**
```rust
quote! {
    ::ferro_rs::testing::set_current_test_name(...)
}
```

**After:**
```rust
use crate::ferro_crate;

// At start of function:
let ferro = ferro_crate();

quote! {
    #ferro::testing::set_current_test_name(...)
}
```

## Tasks

### 1. Update cancer_test.rs (6 occurrences)

**File:** `ferro-macros/src/cancer_test.rs`

Add import and update all `::ferro_rs::` to use `#ferro`:
```rust
use crate::ferro_crate;

pub fn cancer_test_impl(...) {
    let ferro = ferro_crate();

    // Replace all ::ferro_rs:: with #ferro
    quote! {
        #ferro::tokio::test
        #ferro::testing::TestDatabase
        // etc.
    }
}
```

### 2. Update domain_error.rs (3 occurrences)

**File:** `ferro-macros/src/domain_error.rs`

```rust
use crate::ferro_crate;

pub fn derive_domain_error_impl(...) {
    let ferro = ferro_crate();
    // Replace ::ferro_rs:: with #ferro
}
```

### 3. Update inertia.rs (11 occurrences)

**File:** `ferro-macros/src/inertia.rs`

This file has the most occurrences. Update both:
- `derive_inertia_props_impl`
- `inertia_response_impl`

```rust
use crate::ferro_crate;

pub fn derive_inertia_props_impl(...) {
    let ferro = ferro_crate();

    quote! {
        impl #impl_generics #ferro::serde::Serialize for #name ...
        // etc.
    }
}

pub fn inertia_response_impl(...) {
    let ferro = ferro_crate();

    quote! {
        #ferro::serde_json::to_value(...)
        #ferro::InertiaContext::current_path()
        #ferro::InertiaResponse::new(...)
        // etc.
    }
}
```

### 4. Update injectable.rs (10 occurrences)

**File:** `ferro-macros/src/injectable.rs`

```rust
use crate::ferro_crate;

pub fn injectable_impl(...) {
    let ferro = ferro_crate();
    // Replace all ::ferro_rs:: with #ferro
}
```

### 5. Update redirect.rs (2 occurrences)

**File:** `ferro-macros/src/redirect.rs`

```rust
use crate::ferro_crate;

pub fn redirect_impl(...) {
    let ferro = ferro_crate();
    // Replace ::ferro_rs:: with #ferro
}
```

### 6. Update service.rs (6 occurrences)

**File:** `ferro-macros/src/service.rs`

```rust
use crate::ferro_crate;

pub fn service_impl(...) {
    let ferro = ferro_crate();
    // Replace ::ferro_rs:: with #ferro
}
```

### 7. Update test_macro.rs (9 occurrences)

**File:** `ferro-macros/src/test_macro.rs`

```rust
use crate::ferro_crate;

pub fn test_impl(...) {
    let ferro = ferro_crate();

    quote! {
        #[#ferro::cancer_test]
        async fn #fn_name(#db_param_name: #ferro::testing::TestDatabase) {
            #ferro::testing::set_current_test_name(...)
            // etc.
        }
    }
}
```

## Verification

```bash
cd /Users/alberto/repositories/albertogferrario/ferro

# Verify macro crate compiles
cargo build -p ferro-macros

# Verify the entire workspace still builds
cargo build --workspace

# Run the test suite
cargo test --workspace

# Check for any remaining hardcoded paths
grep -r "::ferro_rs::" ferro-macros/src/ && echo "FAIL: Hardcoded paths remain" || echo "OK: No hardcoded paths"
```

## Success Criteria

- [ ] All 7 files updated to use `ferro_crate()` helper
- [ ] No `::ferro_rs::` strings remain in ferro-macros/src/
- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes
- [ ] Existing apps using `ferro_rs = ...` still work (backwards compatible)
