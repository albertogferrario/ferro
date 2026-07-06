# Rename Mapping: cancer -> ferro

Generated: 2026-01-16
Purpose: Transformation rules for complete rebrand

## Quick Reference

| Old | New | Notes |
|-----|-----|-------|
| `cancer-*` directories | `ferro-*` | All 9 crate directories |
| `cancer-rs` | `ferro` | Main framework package |
| `cancer` (CLI binary) | `ferro` | CLI tool name |
| `CancerModel` | `FerroModel` | Derive macro |
| `cancer::*` | `ferro::*` | All imports |
| `cancer_events` | `ferro_events` | Crate imports |
| `cancer_queue` | `ferro_queue` | Crate imports |
| `cancer_notifications` | `ferro_notifications` | Crate imports |
| `CANCER_*` | `FERRO_*` | Environment variables |
| `/_cancer/*` | `/_ferro/*` | Debug endpoints |
| `Cancer.toml` | `Ferro.toml` | Config file |

---

## Directory Renames

### Crate Directories (9 total)

| Old Path | New Path |
|----------|----------|
| `./cancer-broadcast/` | `./ferro-broadcast/` |
| `./cancer-cache/` | `./ferro-cache/` |
| `./cancer-cli/` | `./ferro-cli/` |
| `./cancer-events/` | `./ferro-events/` |
| `./cancer-macros/` | `./ferro-macros/` |
| `./cancer-mcp/` | `./ferro-mcp/` |
| `./cancer-notifications/` | `./ferro-notifications/` |
| `./cancer-queue/` | `./ferro-queue/` |
| `./cancer-storage/` | `./ferro-storage/` |

**Note:** `framework/` directory remains unchanged (only package name changes inside).

---

## Cargo.toml Changes

### Package Names

| File | Old | New |
|------|-----|-----|
| `ferro-broadcast/Cargo.toml` | `name = "cancer-broadcast"` | `name = "ferro-broadcast"` |
| `ferro-cache/Cargo.toml` | `name = "cancer-cache"` | `name = "ferro-cache"` |
| `ferro-cli/Cargo.toml` | `name = "cancer-cli"` | `name = "ferro-cli"` |
| `ferro-cli/Cargo.toml` | `name = "cancer"` (bin) | `name = "ferro"` (bin) |
| `ferro-events/Cargo.toml` | `name = "cancer-events"` | `name = "ferro-events"` |
| `ferro-macros/Cargo.toml` | `name = "cancer-macros"` | `name = "ferro-macros"` |
| `ferro-mcp/Cargo.toml` | `name = "cancer-mcp"` | `name = "ferro-mcp"` |
| `ferro-notifications/Cargo.toml` | `name = "cancer-notifications"` | `name = "ferro-notifications"` |
| `ferro-queue/Cargo.toml` | `name = "cancer-queue"` | `name = "ferro-queue"` |
| `ferro-storage/Cargo.toml` | `name = "cancer-storage"` | `name = "ferro-storage"` |
| `framework/Cargo.toml` | `name = "cancer-rs"` | `name = "ferro"` |

### Workspace Members (Cargo.toml root)

```toml
# OLD
members = [
    "cancer-cli",
    "cancer-macros",
    "cancer-events",
    "cancer-queue",
    "cancer-notifications",
    "cancer-broadcast",
    "cancer-storage",
    "cancer-cache",
    "cancer-mcp",
]

# NEW
members = [
    "ferro-cli",
    "ferro-macros",
    "ferro-events",
    "ferro-queue",
    "ferro-notifications",
    "ferro-broadcast",
    "ferro-storage",
    "ferro-cache",
    "ferro-mcp",
]
```

### Internal Dependencies (framework/Cargo.toml)

```toml
# OLD
cancer-macros = { path = "../cancer-macros", version = "0.1" }
cancer-events = { path = "../cancer-events", version = "0.1" }
cancer-queue = { path = "../cancer-queue", version = "0.1" }
cancer-notifications = { path = "../cancer-notifications", version = "0.1" }
cancer-broadcast = { path = "../cancer-broadcast", version = "0.1" }
cancer-storage = { path = "../cancer-storage", version = "0.1" }
cancer-cache = { path = "../cancer-cache", version = "0.1" }

# NEW
ferro-macros = { path = "../ferro-macros", version = "0.1" }
ferro-events = { path = "../ferro-events", version = "0.1" }
ferro-queue = { path = "../ferro-queue", version = "0.1" }
ferro-notifications = { path = "../ferro-notifications", version = "0.1" }
ferro-broadcast = { path = "../ferro-broadcast", version = "0.1" }
ferro-storage = { path = "../ferro-storage", version = "0.1" }
ferro-cache = { path = "../ferro-cache", version = "0.1" }
```

### CLI Dependencies (ferro-cli/Cargo.toml)

```toml
# OLD
cancer-mcp = { path = "../cancer-mcp" }

# NEW
ferro-mcp = { path = "../ferro-mcp" }
```

### App Dependencies (app/Cargo.toml)

```toml
# OLD
cancer = { path = "../framework", package = "cancer-rs" }

# NEW
ferro = { path = "../framework", package = "ferro" }
```

### Keywords

Replace `"cancer"` with `"ferro"` in all Cargo.toml keywords arrays.

### Repository URLs

```toml
# OLD
repository = "https://github.com/albertogferrario/cancer"
homepage = "https://github.com/albertogferrario/cancer"

# NEW (after repo rename)
repository = "https://github.com/albertogferrario/ferro"
homepage = "https://github.com/albertogferrario/ferro"
```

---

## Rust Code Changes

### Import Statements

| Pattern | Replacement |
|---------|-------------|
| `use cancer::` | `use ferro::` |
| `use cancer_events::` | `use ferro_events::` |
| `use cancer_notifications::` | `use ferro_notifications::` |
| `use cancer_queue::` | `use ferro_queue::` |
| `use cancer_rs::` | `use ferro::` |

### Derive Macros

| Old | New |
|-----|-----|
| `CancerModel` | `FerroModel` |
| `#[derive(..., CancerModel)]` | `#[derive(..., FerroModel)]` |

### Re-exports (framework/src/lib.rs)

```rust
// OLD
pub use cancer_macros::CancerModel;

// NEW
pub use ferro_macros::FerroModel;
```

### Proc Macro Definition (ferro-macros/src/lib.rs)

```rust
// OLD
#[proc_macro_derive(CancerModel)]
pub fn cancer_model(input: TokenStream) -> TokenStream { ... }

// NEW
#[proc_macro_derive(FerroModel)]
pub fn ferro_model(input: TokenStream) -> TokenStream { ... }
```

### Error Messages

```rust
// OLD
"CancerModel only supports named structs"

// NEW
"FerroModel only supports named structs"
```

---

## Environment Variables

| Old | New |
|-----|-----|
| `CANCER_COLLECT_METRICS` | `FERRO_COLLECT_METRICS` |
| `CANCER_DEBUG_ENDPOINTS` | `FERRO_DEBUG_ENDPOINTS` |

---

## Debug Endpoints

| Old | New |
|-----|-----|
| `/_cancer/routes` | `/_ferro/routes` |
| `/_cancer/middleware` | `/_ferro/middleware` |
| `/_cancer/services` | `/_ferro/services` |
| `/_cancer/metrics` | `/_ferro/metrics` |
| `/_cancer/queue/jobs` | `/_ferro/queue/jobs` |
| `/_cancer/queue/stats` | `/_ferro/queue/stats` |

---

## CLI Changes

### Binary Name

```toml
# OLD (ferro-cli/Cargo.toml)
[[bin]]
name = "cancer"

# NEW
[[bin]]
name = "ferro"
```

### Command Registration

```rust
// OLD (ferro-cli/src/main.rs)
#[command(name = "cancer")]

// NEW
#[command(name = "ferro")]
```

### Command References in Templates

Replace all occurrences:
- `cancer new` -> `ferro new`
- `cancer serve` -> `ferro serve`
- `cancer make:*` -> `ferro make:*`
- `cancer migrate` -> `ferro migrate`
- `cancer db:sync` -> `ferro db:sync`
- `cancer generate-types` -> `ferro generate-types`
- `cancer mcp` -> `ferro mcp`

---

## MCP Server Changes

### Constants

```rust
// OLD
const CANCER_MCP_INSTRUCTIONS: &str = ...

// NEW
const FERRO_MCP_INSTRUCTIONS: &str = ...
```

### Tool Descriptions

Update all references to "cancer" in tool descriptions and help text.

### Code Templates

Update all `use cancer::` imports in code templates to `use ferro::`.

---

## Documentation Changes

### docs/book.toml

```toml
# OLD
git-repository-url = "https://github.com/albertogferrario/cancer"
edit-url-template = "https://github.com/albertogferrario/cancer/edit/main/docs/{path}"

# NEW
git-repository-url = "https://github.com/albertogferrario/ferro"
edit-url-template = "https://github.com/albertogferrario/ferro/edit/main/docs/{path}"
```

### Markdown Files

All `docs/src/**/*.md` files need:
1. Import statements: `use cancer::` -> `use ferro::`
2. CLI commands: `cancer` -> `ferro`
3. Package references: `cancer-rs` -> `ferro`
4. Repository URLs: `albertogferrario/cancer` -> `albertogferrario/ferro`

### README Files

Update all README.md files in crate directories.

---

## Template Changes (CLI Scaffolding)

### Files to Update

All files in `ferro-cli/src/templates/`:
1. `mod.rs` - Main template definitions
2. `files/backend/*.tpl` - Backend templates
3. `files/frontend/*.tpl` - Frontend templates
4. `files/docker/*.tpl` - Docker templates
5. `files/root/*.tpl` - Root config templates

### Docker Defaults

```yaml
# OLD
POSTGRES_USER: ${DB_USER:-cancer}
POSTGRES_PASSWORD: ${DB_PASSWORD:-cancer_secret}
POSTGRES_DB: ${DB_NAME:-cancer_db}

# NEW
POSTGRES_USER: ${DB_USER:-ferro}
POSTGRES_PASSWORD: ${DB_PASSWORD:-ferro_secret}
POSTGRES_DB: ${DB_NAME:-ferro_db}
```

### Queue Prefix

```rust
// OLD
| `QUEUE_PREFIX` | Redis key prefix | cancer_queue |

// NEW
| `QUEUE_PREFIX` | Redis key prefix | ferro_queue |
```

---

## Repository/CI Changes

### GitHub Repository Rename

Repository URL changes:
- `https://github.com/albertogferrario/cancer` -> `https://github.com/albertogferrario/ferro`
- `https://github.com/albertogferrario/cancer.git` -> `https://github.com/albertogferrario/ferro.git`

### Clone Instructions

```bash
# OLD
git clone https://github.com/albertogferrario/cancer.git

# NEW
git clone https://github.com/albertogferrario/ferro.git
```

### Template Git URL

```toml
# OLD
cancer = { package = "cancer-rs", git = "https://github.com/albertogferrario/cancer.git" }

# NEW
ferro = { package = "ferro", git = "https://github.com/albertogferrario/ferro.git" }
```

---

## Config File Changes

### Cancer.toml -> Ferro.toml

```rust
// OLD
let cancer_toml = project_root.join("Cancer.toml");

// NEW
let ferro_toml = project_root.join("Ferro.toml");
```

---

## Execution Order

Recommended sequence aligned with ROADMAP phases 14-22:

### Phase 14: Core Framework (cancer-rs -> ferro)
1. Rename `framework/Cargo.toml` package name
2. Update internal dependency declarations
3. Update re-exports in `lib.rs`

### Phase 15: Supporting Crates (cancer-* -> ferro-*)
1. Rename directories (all 9 crates)
2. Update package names in each Cargo.toml
3. Update workspace members in root Cargo.toml
4. Update internal imports

### Phase 16: Derive Macros (CancerModel -> FerroModel)
1. Rename proc macro in ferro-macros
2. Update derive usage in templates
3. Update derive usage in sample app

### Phase 17: CLI Binary and Commands
1. Update binary name declaration
2. Update command name in clap
3. Update all command references in templates

### Phase 18: MCP Server
1. Update debug endpoint paths
2. Update tool descriptions
3. Update code templates
4. Update constants

### Phase 19: Environment Variables
1. Update env var names in framework
2. Update documentation

### Phase 20: Documentation
1. Update all docs/src/**/*.md files
2. Update README files in each crate
3. Rebuild docs/book/

### Phase 21: Sample App
1. Update app/Cargo.toml dependency
2. Update all imports in app code

### Phase 22: Repository/Publishing
1. Rename GitHub repository
2. Update all repository URLs
3. Publish to crates.io

---

## Notes

### Mechanical Changes (Find/Replace)

These changes are straightforward search-and-replace:
- `use cancer::` -> `use ferro::`
- `cancer-*` directories -> `ferro-*`
- `cancer` command -> `ferro` command
- Repository URLs

### Careful Handling Required

These need manual review:
- **Re-exports in lib.rs** - Ensure proper public API
- **Proc macro rename** - Test derive behavior
- **Template interpolation** - Some templates use `{{` syntax
- **Error messages** - Update user-facing text

### Breaking Changes for Existing Users

1. **Import paths** - All `use cancer::` must become `use ferro::`
2. **Derive macro** - `CancerModel` -> `FerroModel`
3. **CLI binary** - `cancer` -> `ferro`
4. **Package names** - When adding as dependency
5. **Environment variables** - Any custom env vars

### Testing After Each Phase

After each phase:
1. `cargo check --all`
2. `cargo test --all`
3. `cargo clippy --all`
4. Test CLI commands if applicable
