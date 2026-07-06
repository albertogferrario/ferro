# Phase 2, Plan 1: CancerModel Derive Macro Integration

## Overview

The `CancerModel` derive macro already exists in `cancer-macros/src/model.rs` and generates most model boilerplate (builder pattern, setters, query/create/update/delete methods). However:

1. CLI templates (`db:sync`) generate all boilerplate manually instead of using the macro
2. The macro doesn't generate `Model`/`ModelMut` trait implementations
3. Sample app models have 150+ lines of handwritten boilerplate each

This plan integrates the existing macro with CLI generation and enhances it to reduce model files from ~180 lines to ~20 lines.

## Goal

A model file that currently looks like this (180+ lines):
```rust
pub use super::entities::users::*;
use cancer::database::{ModelMut, QueryBuilder};
use sea_orm::{entity::prelude::*, Set};

pub type User = Model;

impl ActiveModelBehavior for ActiveModel {}
impl cancer::database::Model for Entity {}
impl cancer::database::ModelMut for Entity {}

impl Model {
    pub fn query() -> QueryBuilder<Entity> { ... }
    pub fn create() -> UserBuilder { ... }
    pub fn set_name(mut self, value: impl Into<String>) -> Self { ... }
    // 20+ more setter methods
    pub async fn update(self) -> Result<Self, cancer::FrameworkError> { ... }
    pub async fn delete(self) -> Result<u64, cancer::FrameworkError> { ... }
    fn to_active_model(&self) -> ActiveModel { ... }
}

#[derive(Default)]
pub struct UserBuilder { ... }
impl UserBuilder {
    // 20+ setter methods
    pub async fn insert(self) -> Result<Model, cancer::FrameworkError> { ... }
}
```

Should become (~20 lines):
```rust
pub use super::entities::users::*;

use cancer::CancerModel;

pub type User = Model;

#[derive(CancerModel)]
impl Model {}

// Custom methods and relations go here
```

## Scope

**Files to modify:**
- `cancer-macros/src/model.rs` - Enhance macro to generate trait impls
- `cancer-macros/src/lib.rs` - Update derive macro export if needed
- `cancer-cli/src/templates/mod.rs` - Update `user_model_template()` to use derive macro
- `app/src/models/users.rs` - Migrate to use derive macro
- `app/src/models/todos.rs` - Migrate to use derive macro
- `app/src/models/password_reset_tokens.rs` - Migrate to use derive macro

**Not in scope:**
- Entity generation (`entities/` directory) - already auto-generated, working fine
- SeaORM internals
- Database schema changes

## Current State Analysis

### CancerModel Macro Generates (cancer-macros/src/model.rs:176-229)
- `query()` - Start query builder
- `create()` - Return builder for inserts
- `set_*()` - Field setters on Model
- `update()` - Save changes to database
- `delete()` - Delete record
- `to_active_model()` - Convert to ActiveModel
- `{Model}Builder` struct with setters and `insert()`

### CancerModel Macro Missing
- `impl ActiveModelBehavior for ActiveModel {}`
- `impl cancer::database::Model for Entity {}`
- `impl cancer::database::ModelMut for Entity {}`

### CLI Template Generates (templates/mod.rs:428-624)
All of the above manually via string interpolation (200+ lines of code).

## Tasks

### Wave 1: Enhance CancerModel Macro

- [x] **1.1** Add Model/ModelMut trait generation to macro
  - Read `framework/src/database/model.rs` for trait definitions
  - Update `cancer_model_impl()` to generate empty trait impls
  - The traits are marker traits, just need `impl cancer::database::Model for Entity {}`

- [x] **1.2** Add ActiveModelBehavior generation
  - Generate `impl ActiveModelBehavior for ActiveModel {}`
  - This is a SeaORM trait, usually empty impl

- [x] **1.3** Test macro changes compile
  - Create minimal test model in `cancer-macros/tests/` or test inline
  - Verify generated code compiles

### Wave 2: Update CLI Templates

- [x] **2.1** Rewrite `user_model_template()` function
  - Replace 200-line string interpolation with ~30 lines
  - Use `#[derive(CancerModel)]` instead of manual impls
  - Keep re-export, type alias, and Authenticatable impl for users
  - Keep doc comments for custom methods section

- [x] **2.2** Verify CLI template output
  - Run `cancer db:sync` on test database
  - Check generated model file uses derive macro
  - Verify compilation succeeds

### Wave 3: Migrate Sample App Models

- [x] **3.1** Migrate `app/src/models/users.rs`
  - Replace manual implementation with `#[derive(CancerModel)]`
  - Keep Authenticatable impl (users-specific)
  - Keep any custom methods
  - Verify compilation

- [x] **3.2** Migrate `app/src/models/todos.rs`
  - Replace manual implementation with `#[derive(CancerModel)]`
  - Keep any custom query scopes
  - Verify compilation

- [x] **3.3** Migrate `app/src/models/password_reset_tokens.rs`
  - N/A: File does not exist in sample app

### Wave 4: Validation

- [x] **4.1** Run full test suite
  - `cargo test --all-features`
  - `cargo clippy`
  - `cargo fmt --check`

- [x] **4.2** Verify sample app functionality
  - Sample app builds successfully
  - All tests pass (excluding pre-existing failures)

## Success Criteria

1. CancerModel macro generates all required boilerplate including trait impls
2. CLI `db:sync` generates models using `#[derive(CancerModel)]`
3. Sample app models reduced from ~180 lines to ~20 lines each
4. All tests pass
5. Sample app functions identically

## Line Count Target

| File | Before | After | Reduction |
|------|--------|-------|-----------|
| users.rs | ~180 | ~25 | 86% |
| todos.rs | ~150 | ~20 | 87% |
| password_reset_tokens.rs | ~120 | ~15 | 88% |
| CLI template function | ~200 | ~40 | 80% |

## Risks

- **Macro attribute syntax might need adjustment**: The current macro uses `#[derive(CancerModel)]` on a struct, but we need it on the Model struct which is in a different file (entities). May need to switch to an attribute macro on `impl Model {}` instead.

- **Clone bound on to_active_model**: Current manual impl uses `.clone()` for some fields. Need to verify the macro handles this correctly.

## Notes

The `CancerModel` derive macro was created but never integrated with CLI generation. This plan completes that integration and validates the macro works for real-world models.
