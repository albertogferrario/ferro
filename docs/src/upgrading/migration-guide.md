# Migration Guide: cancer to ferro

This guide documents the upgrade path from `cancer` to `ferro` (v2.0).

## Overview

The framework has been renamed from "cancer" to "ferro" for crates.io publication. The API remains identical - only package names and imports have changed.

## Migration Steps

### 1. Update Cargo.toml Dependencies

Replace all `cancer` dependencies with their `ferro` equivalents:

```toml
# Before
[dependencies]
cancer = "1.0"
cancer-events = "1.0"
cancer-queue = "1.0"
cancer-notifications = "1.0"
cancer-broadcast = "1.0"
cancer-storage = "1.0"
cancer-cache = "1.0"

# After
[dependencies]
ferro = "2.0"
ferro-events = "2.0"
ferro-queue = "2.0"
ferro-notifications = "2.0"
ferro-broadcast = "2.0"
ferro-storage = "2.0"
ferro-cache = "2.0"
```

### 2. Update Rust Imports

Replace `cancer` with `ferro` in all import statements:

```rust
// Before
use cancer::prelude::*;
use cancer_events::Event;
use cancer_queue::Job;

// After
use ferro::prelude::*;
use ferro_events::Event;
use ferro_queue::Job;
```

Use find-and-replace across your project:
- `use cancer::` to `use ferro::`
- `use cancer_` to `use ferro_`
- `cancer::` to `ferro::` (in type paths)

### 3. Update CLI Commands

The CLI binary has been renamed:

```bash
# Before
cancer serve
cancer make:model User
cancer migrate

# After
ferro serve
ferro make:model User
ferro db:migrate
```

Update any scripts, CI configurations, or documentation that reference the CLI.

### 4. Update MCP Server Configuration

If using the MCP server for IDE integration:

```json
// Before
{
  "mcpServers": {
    "cancer-mcp": {
      "command": "cancer-mcp",
      "args": ["serve"]
    }
  }
}

// After
{
  "mcpServers": {
    "ferro-mcp": {
      "command": "ferro-mcp",
      "args": ["serve"]
    }
  }
}
```

### 5. Update Environment Variables (Optional)

If you have custom environment variable prefixes, consider updating them for consistency:

```env
# Optional - these still work but consider updating
CANCER_APP_KEY=... -> FERRO_APP_KEY=...
```

## Breaking Changes

None. The v2.0 release contains only naming changes. All APIs, behaviors, and features remain identical to v1.x.

## Verification

After migration, verify your setup:

```bash
# Check CLI installation
ferro --version

# Run tests
cargo test

# Start development server
ferro serve
```

## Gradual Migration

For large projects, you can migrate gradually using Cargo aliases:

```toml
[dependencies]
# Temporary: use new crate with old import name
cancer = { package = "ferro", version = "2.0" }
```

This allows keeping existing imports while transitioning. Remove the alias once all code is updated.
