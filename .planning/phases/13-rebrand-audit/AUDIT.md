# Rebrand Audit: "cancer" Occurrences

Generated: 2026-01-16
Purpose: Complete inventory of all "cancer" occurrences for cancer -> ferro rebrand

## Summary

| Category | Count | Files Affected |
|----------|-------|----------------|
| Crate Directories | 9 | 9 directories |
| Cargo.toml Package Names | 11 | 11 files |
| Cargo.toml Dependencies | 44 | 7 files |
| Rust Code Imports | 278 | ~50 files |
| CancerModel Derive | 22 | 5 files |
| Documentation (markdown) | 380 | 28+ files |
| CLI Templates | 59 | 24 template files |
| MCP Server Code | 68 | 15 files |
| GitHub/Repository URLs | 60+ | 20+ files |
| Environment Variables | 2 | 2 files |
| **Total Estimated** | **900+** | **150+ files** |

---

## 1. Crate Directories (9 total)

All directories at repository root that need renaming:

```
./cancer-broadcast
./cancer-cache
./cancer-cli
./cancer-events
./cancer-macros
./cancer-mcp
./cancer-notifications
./cancer-queue
./cancer-storage
```

**Note:** The `framework/` directory does NOT need renaming (contains cancer-rs package but directory is generic).

---

## 2. Cargo.toml Package Names (11 declarations)

Package name declarations across all crates:

| File | Current Package Name | New Package Name |
|------|---------------------|------------------|
| `./cancer-broadcast/Cargo.toml` | `cancer-broadcast` | `ferro-broadcast` |
| `./cancer-cache/Cargo.toml` | `cancer-cache` | `ferro-cache` |
| `./cancer-cli/Cargo.toml` | `cancer-cli` | `ferro-cli` |
| `./cancer-cli/Cargo.toml` | `cancer` (binary) | `ferro` (binary) |
| `./cancer-events/Cargo.toml` | `cancer-events` | `ferro-events` |
| `./cancer-macros/Cargo.toml` | `cancer-macros` | `ferro-macros` |
| `./cancer-mcp/Cargo.toml` | `cancer-mcp` | `ferro-mcp` |
| `./cancer-notifications/Cargo.toml` | `cancer-notifications` | `ferro-notifications` |
| `./cancer-queue/Cargo.toml` | `cancer-queue` | `ferro-queue` |
| `./cancer-storage/Cargo.toml` | `cancer-storage` | `ferro-storage` |
| `./framework/Cargo.toml` | `cancer-rs` | `ferro` |

---

## 3. Cargo.toml Dependencies (44 references)

### Workspace Cargo.toml (./Cargo.toml)
Members list:
```toml
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
```

### Framework Dependencies (./framework/Cargo.toml)
```toml
cancer-macros = { path = "../cancer-macros", version = "0.1" }
cancer-events = { path = "../cancer-events", version = "0.1" }
cancer-queue = { path = "../cancer-queue", version = "0.1" }
cancer-notifications = { path = "../cancer-notifications", version = "0.1" }
cancer-broadcast = { path = "../cancer-broadcast", version = "0.1" }
cancer-storage = { path = "../cancer-storage", version = "0.1" }
cancer-cache = { path = "../cancer-cache", version = "0.1" }
```

### CLI Dependencies (./cancer-cli/Cargo.toml)
```toml
cancer-mcp = { path = "../cancer-mcp" }
```

### App Dependencies (./app/Cargo.toml)
```toml
cancer = { path = "../framework", package = "cancer-rs" }
```

### Keywords in Cargo.toml files
- `keywords = ["...", "cancer", "..."]` in 6 files

---

## 4. Rust Code - Imports and Uses (278 occurrences)

### Import Patterns Found

**Primary pattern (app code):**
```rust
use cancer::{handler, json_response, Request, Response};
use cancer::{async_trait, Middleware, Next, Request, Response};
use cancer::CancerModel;
use cancer::Authenticatable;
use cancer::validation::{Validator, rules};
use cancer::database::{Model, ModelMut};
use cancer::inertia::Inertia;
```

**Internal crate imports:**
```rust
use cancer_events::{Event, Listener, Error};
use cancer_notifications::{Notification, Channel, MailMessage};
use cancer_queue::{Job, Queueable, Error};
use cancer_rs::validation::Validatable;
```

### Distribution by Location

| Location | Count | Examples |
|----------|-------|----------|
| `app/src/` | 24 | Controllers, middleware, models |
| `framework/src/` | 60+ | Module docs, tests |
| `cancer-events/src/` | 15 | Doc examples |
| `cancer-notifications/src/` | 8 | Doc examples |
| `cancer-macros/src/` | 8 | Model derive |
| `cancer-cli/src/` | 161 | Templates, scaffolding |
| `cancer-mcp/src/` | 68 | Tool implementations |

---

## 5. CancerModel Derive Macro (22 occurrences)

### Derive Declaration
```rust
// cancer-macros/src/lib.rs:466
#[proc_macro_derive(CancerModel)]
```

### Re-export
```rust
// framework/src/lib.rs:190
pub use cancer_macros::CancerModel;
```

### Usage Sites
```rust
// app/src/models/entities/users.rs
use cancer::CancerModel;
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, CancerModel)]

// app/src/models/entities/todos.rs
use cancer::CancerModel;
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, CancerModel)]
```

### In Templates
```rust
// cancer-cli/src/templates/mod.rs
use cancer::CancerModel;
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, CancerModel)]
```

---

## 6. Documentation Files (380 occurrences)

### README Files (5 files)
| File | Occurrences |
|------|-------------|
| `cancer-events/README.md` | 4 |
| `cancer-notifications/README.md` | 3 |
| `cancer-queue/README.md` | 12 |
| `cancer-macros/README.md` | 8 |
| `framework/README.md` | 5 |

### docs/src/ Directory (273 occurrences)
Primary documentation source files:
- `docs/src/features/storage.md` - 18 occurrences
- `docs/src/features/notifications.md` - 15 occurrences
- `docs/src/features/broadcasting.md` - 12 occurrences
- `docs/src/features/caching.md` - 14 occurrences
- `docs/src/features/validation.md` - 18 occurrences
- `docs/src/features/testing.md` - 22 occurrences
- `docs/src/features/queues.md` - 12 occurrences
- `docs/src/features/events.md` - 10 occurrences
- `docs/src/features/inertia.md` - 8 occurrences
- `docs/src/getting-started/installation.md` - 3 occurrences
- `docs/src/reference/cli.md` - 5 occurrences

### Generated Book Files (docs/book/)
All HTML files in `docs/book/` contain GitHub repository URLs and edit links.
These are auto-generated from source - only source files need manual editing.

---

## 7. CLI Binary Name

### Binary Declaration
```toml
# cancer-cli/Cargo.toml:13-15
[[bin]]
name = "cancer"
path = "src/main.rs"
```

### Command Name in Code
```rust
// cancer-cli/src/main.rs:8
#[command(name = "cancer")]
```

### CLI References in Templates (59 occurrences)
Commands referenced in generated code and documentation:
- `cancer new <name>`
- `cancer serve`
- `cancer make:controller`
- `cancer make:middleware`
- `cancer make:migration`
- `cancer make:event`
- `cancer make:job`
- `cancer make:task`
- `cancer make:notification`
- `cancer migrate`
- `cancer db:sync`
- `cancer generate-types`
- `cancer mcp`

---

## 8. MCP Server (68 occurrences)

### Debug Endpoint URLs
```rust
// cancer-mcp/src/tools/*.rs
"{}/_cancer/routes"
"{}/_cancer/middleware"
"{}/_cancer/services"
"{}/_cancer/metrics"
"{}/_cancer/queue/jobs"
"{}/_cancer/queue/stats"
```

### Tool Descriptions and Help Text
```rust
// cancer-mcp/src/service.rs:283
"List all available CLI commands from the cancer-cli tool."

// cancer-mcp/src/service.rs:1022-1027
"- Background jobs via cancer-queue"
"- Event system via cancer-events"
"- WebSocket broadcasting (cancer-broadcast)"
"- File storage abstraction (cancer-storage)"
"- Caching with tags (cancer-cache)"
```

### Code Templates in MCP
```rust
// cancer-mcp/src/tools/code_templates.rs
"use cancer::prelude::*;"
"use cancer::middleware::{Middleware, Next};"
"use cancer::validation::{Validator, rules};"
```

### MCP Instructions Constant
```rust
// cancer-mcp/src/service.rs
const CANCER_MCP_INSTRUCTIONS: &str = r#"..."#;
```

---

## 9. GitHub/Repository References (60+ occurrences)

### Cargo.toml Repository URLs
```toml
repository = "https://github.com/albertogferrario/cancer"
homepage = "https://github.com/albertogferrario/cancer"
```

Files with these URLs:
- `framework/Cargo.toml`
- `cancer-cli/Cargo.toml`
- `cancer-events/Cargo.toml`
- `cancer-notifications/Cargo.toml`
- `cancer-queue/Cargo.toml`
- `cancer-macros/Cargo.toml`
- `cancer-broadcast/Cargo.toml`

### docs/book.toml
```toml
git-repository-url = "https://github.com/albertogferrario/cancer"
edit-url-template = "https://github.com/albertogferrario/cancer/edit/main/docs/{path}"
```

### Template Files
```toml
# cancer-cli/src/templates/files/backend/Cargo.toml.tpl
cancer = { package = "cancer-rs", git = "https://github.com/albertogferrario/cancer.git" }
```

### Clone Instructions
```markdown
git clone https://github.com/albertogferrario/cancer.git
```

---

## 10. Special Patterns

### Environment Variables (2 occurrences)
```rust
// framework/src/metrics/mod.rs:191
std::env::var("CANCER_COLLECT_METRICS")

// framework/src/debug/mod.rs
std::env::var("CANCER_DEBUG_ENDPOINTS")
```

### Uppercase Constants
```rust
// cancer-mcp/src/service.rs
const CANCER_MCP_INSTRUCTIONS: &str = ...
```

### Queue Prefix Default
```rust
// cancer-queue documentation
| `QUEUE_PREFIX` | Redis key prefix | cancer_queue |
```

### Docker Compose Defaults
```yaml
# cancer-cli/src/templates/files/docker/docker-compose.yml.tpl
POSTGRES_USER: ${DB_USER:-cancer}
POSTGRES_PASSWORD: ${DB_PASSWORD:-cancer_secret}
POSTGRES_DB: ${DB_NAME:-cancer_db}
```

### Cancer.toml Config File Support
```rust
// cancer-mcp/src/tools/get_config.rs:73
let cancer_toml = project_root.join("Cancer.toml");
```

### Generated Type Comments
```typescript
// cancer-cli/src/templates/files/frontend/src/types/inertia-props.ts.tpl
// Run `cancer generate-types` to regenerate.
```

---

## Files NOT Requiring Changes

1. **Target directories** - Build artifacts, not tracked
2. **node_modules** - Third-party dependencies
3. **docs/book/*** - Auto-generated from source
4. **.git/*** - Git internal files (except if repo URL changes)
5. **.planning/*** - Planning documents (can update separately)

---

## Priority Order for Renaming

Based on dependency analysis:

1. **cancer-macros** - No dependencies, others depend on it
2. **cancer-events** - No dependencies on other cancer crates
3. **cancer-queue** - No dependencies on other cancer crates
4. **cancer-notifications** - No dependencies on other cancer crates
5. **cancer-broadcast** - No dependencies on other cancer crates
6. **cancer-storage** - No dependencies on other cancer crates
7. **cancer-cache** - No dependencies on other cancer crates
8. **framework (cancer-rs)** - Depends on all above
9. **cancer-mcp** - Standalone but references framework patterns
10. **cancer-cli** - Depends on cancer-mcp
11. **Documentation** - After code changes
12. **Sample app** - After framework changes
