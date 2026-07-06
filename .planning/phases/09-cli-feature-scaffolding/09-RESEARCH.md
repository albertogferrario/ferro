# Phase 9 Research: CLI Feature Scaffolding

## Research Question

How should cancer-cli enhance feature scaffolding to generate complete, integrated features (not just individual files)?

## Current State Analysis

### Existing Scaffold Implementation

Cancer-cli already has a comprehensive `make:scaffold` command in `cancer-cli/src/commands/make_scaffold.rs` (1274 lines).

**Current `make:scaffold <Name> <fields...>` generates:**
1. Migration file with table creation
2. Model (SeaORM entity) with field mappings
3. Controller with full CRUD (index, show, create, store, edit, update, destroy)
4. 4 Inertia pages (Index.tsx, Show.tsx, Create.tsx, Edit.tsx)
5. Route registration instructions (printed, not auto-added)

**Field type system:**
```rust
enum FieldType {
    String, Text, Integer, BigInteger, Float, Boolean, DateTime, Date, Uuid
}
```

Each field type maps to:
- Rust type (`to_rust_type`)
- SeaORM column method (`to_sea_orm_method`)
- TypeScript type (`to_typescript_type`)
- HTML form input type (`to_form_input_type`)
- Validation attribute (`to_validation_attr`)

### Other `make:*` Commands

| Command | Generates | Notes |
|---------|-----------|-------|
| `make:controller` | Single controller file | Thin template |
| `make:migration` | Migration file | Updates migrator mod.rs |
| `make:middleware` | Middleware file | Updates mod.rs |
| `make:model` | N/A | Not found - db:sync handles models |
| `make:event` | Event struct | cancer-events integration |
| `make:listener` | Event listener | cancer-events integration |
| `make:job` | Background job | cancer-queue integration |
| `make:notification` | Multi-channel notification | cancer-notifications integration |
| `make:task` | Scheduled task | Scheduler integration |
| `make:seeder` | Database seeder | Updates mod.rs |
| `make:factory` | Test factory | For testing |
| `make:policy` | Authorization policy | Access control |
| `make:action` | Action class | Business logic |
| `make:error` | Domain error | Error types |
| `make:inertia` | Inertia page | React component |

### Templates Module

Templates are in `cancer-cli/src/templates/mod.rs` using:
- `include_str!()` for file templates (in `templates/files/`)
- String formatting functions for dynamic templates

## Standard Stack Analysis

### Rails Scaffolding (Reference)

Rails `scaffold` generates:
1. Migration
2. Model
3. Controller with CRUD
4. Views (index, show, new, edit, _form partial)
5. **Tests** (model, controller, system)
6. **Helper** (view helpers)
7. Routes entry (auto-added to routes.rb)
8. **Stylesheet** (optional)

Key Rails scaffold options:
- `--api` - API-only mode (no views, JSON responses)
- `--skip-routes` - Don't add routes
- `--skip-test` - Skip test generation
- `--parent=Model` - Inherit from another model

### Laravel Scaffolding Patterns

Laravel's approach via packages like `crudify`:
- Action/Pipeline pattern for multi-step generation
- Schema-driven model relationships
- Auto-detected belongsTo from foreign key naming (`user_id`)
- Resource controllers with standard method names

## Gap Analysis

### What Cancer Scaffold HAS

1. Complete CRUD generation
2. Migration + Model + Controller + Views
3. Type-safe field mapping
4. Proper TypeScript types
5. Validation attributes
6. Tailwind CSS styling

### What Cancer Scaffold LACKS

| Gap | Rails | Laravel | Priority |
|-----|-------|---------|----------|
| Test generation | Yes | Via packages | **High** |
| Factory generation | Yes | Yes | **High** |
| Auto route registration | Yes | Via artisan | Medium |
| API-only mode | Yes | Yes | Medium |
| Relationship scaffolding | Via associations | Via `--belongs-to` | Medium |
| Seeder generation | Via `db:seed` | Yes | Low |
| Policy generation | N/A | Via `--policy` | Low |

## Architecture Patterns

### Multi-File Generation Coordination

Current scaffold uses sequential generation:
```rust
pub fn run(name: String, fields: Vec<String>) {
    generate_migration(...);
    generate_model(...);
    generate_controller(...);
    generate_inertia_pages(...);
    print_route_instructions(...);
}
```

Better approach - **Pipeline pattern**:
```rust
pub struct ScaffoldPipeline {
    steps: Vec<Box<dyn GenerationStep>>,
    context: ScaffoldContext,
}

trait GenerationStep {
    fn execute(&self, ctx: &mut ScaffoldContext) -> Result<(), Error>;
    fn rollback(&self, ctx: &ScaffoldContext) -> Result<(), Error>;
}
```

Benefits:
- Each step isolated and testable
- Rollback on failure
- Easy to add/skip steps via flags
- Progress reporting

### Template Engines

Cancer currently uses simple string formatting. No need to change.

**Don't adopt:**
- Askama (compile-time) - Overkill for simple templates
- Tera (runtime) - Adds dependency, no real benefit

Current approach with `format!()` and `include_str!()` is sufficient.

## Common Pitfalls

### 1. Scaffold Bloat
**Problem:** Generating too many files that users immediately delete
**Solution:** Make test/factory generation opt-in via flags

### 2. Broken Dependencies
**Problem:** Generated code references files that don't exist
**Solution:** Validate target directories exist before generation

### 3. No Rollback
**Problem:** Partial failure leaves codebase in broken state
**Solution:** Collect files to create, write atomically, rollback on error

### 4. Stale Route Instructions
**Problem:** Printed route instructions are easy to miss or ignore
**Solution:** Offer to auto-add routes (with confirmation)

### 5. Missing Integration Points
**Problem:** Generated files not registered in mod.rs
**Solution:** Auto-update mod.rs files (already done for some commands)

## Recommendations

### Phase 9 Scope (Recommended)

**High Priority - Include in Phase 9:**

1. **Add `--with-tests` flag to scaffold**
   - Generate controller tests (HTTP tests)
   - Generate model tests (query tests)
   - Follow existing cancer test patterns

2. **Add `--with-factory` flag to scaffold**
   - Generate matching factory for model
   - Pre-populate fields from scaffold definition

3. **Auto-register routes (with confirmation)**
   - Parse routes.rs, find appropriate insertion point
   - Ask user before modifying

**Medium Priority - Consider for Phase 9:**

4. **Add `--api` flag for API-only scaffold**
   - Skip Inertia pages
   - Controller returns JSON only
   - Generate OpenAPI-compatible responses

5. **Relationship scaffolding**
   - `--belongs-to User` adds user_id field and relation
   - Auto-generate FK in migration

**Low Priority - Defer:**

6. Policy generation integrated with scaffold
7. Seeder generation integrated with scaffold
8. Job/Event generation for domain operations

### Don't Hand-Roll

1. **Don't build a template engine** - String formatting is sufficient
2. **Don't parse Rust AST** for route insertion - Regex/string matching works
3. **Don't auto-generate all possible files** - Keep scaffold focused
4. **Don't add CLI wizard/interactive mode** - Flags are clearer

### Implementation Notes

1. Keep existing scaffold structure - it's well organized
2. Add flags incrementally, each with tests
3. Use existing `templates` module pattern
4. Consider extracting shared utilities from `make_scaffold.rs`

## Code Examples

### Test Template for Controller

```rust
pub fn controller_test_template(name: &str, plural_name: &str) -> String {
    format!(r#"
use cancer::testing::{{TestApp, TestResponse}};

#[tokio::test]
async fn test_{plural_name}_index() {{
    let app = TestApp::new().await;
    let response = app.get("/{plural_name}").await;
    response.assert_status(200);
}}

#[tokio::test]
async fn test_{plural_name}_show() {{
    let app = TestApp::new().await;
    // Create test record
    let response = app.get("/{plural_name}/1").await;
    response.assert_status(200);
}}

#[tokio::test]
async fn test_{plural_name}_create() {{
    let app = TestApp::new().await;
    let response = app
        .post("/{plural_name}")
        .json(&serde_json::json!({{
            // TODO: Add required fields
        }}))
        .await;
    response.assert_status(201);
}}
"#, plural_name = plural_name)
}
```

### Factory Integration

```rust
// In make_scaffold.rs
if with_factory {
    generate_factory(&name, &snake_name, &parsed_fields);
}

fn generate_factory(name: &str, snake_name: &str, fields: &[Field]) {
    // Generate factory with fields pre-populated
    let factory_content = generate_factory_content(name, fields);
    // Write to src/factories/{snake_name}_factory.rs
    // Update mod.rs
}
```

### Route Auto-Registration

```rust
fn register_routes(name: &str, plural_name: &str) -> Result<(), Error> {
    let routes_path = "src/routes.rs";
    let content = fs::read_to_string(routes_path)?;

    // Find insertion point (before final closing brace or after last resource!)
    let insertion = find_route_insertion_point(&content)?;

    let new_route = format!(
        r#"    resource!("{}", {}_controller);"#,
        plural_name, name.to_lowercase()
    );

    // Insert and write
    let new_content = insert_at(&content, insertion, &new_route);
    fs::write(routes_path, new_content)?;

    Ok(())
}
```

## Sources

1. **Rails Scaffolding**: https://guides.rubyonrails.org/command_line.html#bin-rails-generate
2. **Laravel CRUD Packages**: Various packages (crudify, adminlte-generator)
3. **Existing cancer-cli**: `cancer-cli/src/commands/make_scaffold.rs`
4. **Cancer templates**: `cancer-cli/src/templates/mod.rs`

## Next Steps

1. Create Phase 9 plan with prioritized tasks
2. Focus on `--with-tests` and `--with-factory` flags first
3. Consider API-only mode if time permits
4. Relationship scaffolding can be Phase 10 (CLI Smart Defaults)
