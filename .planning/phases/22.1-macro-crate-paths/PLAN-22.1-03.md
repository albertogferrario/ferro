# Plan 22.1-03: Test Alternate Crate Names and Document

```yaml
phase: 22.1
plan: 03
type: verification
wave: 3
depends_on: [22.1-02]
files_modified:
  - ferro-macros/src/crate_path.rs
  - docs/src/installation.md
autonomous: true
```

## Objective

Verify the dynamic crate resolution works with different dependency naming patterns and document the supported configurations.

## Context

After implementing dynamic resolution, we need to confirm it works for:
1. `ferro = "x.y.z"` - Standard crates.io name
2. `ferro_rs = "x.y.z"` - Legacy name (backwards compatibility)
3. `ferro = { package = "ferro", ... }` - With package alias
4. `my_framework = { package = "ferro", ... }` - Custom alias

## Tasks

### 1. Add unit tests for crate path resolution

**File:** `ferro-macros/src/crate_path.rs`

Add tests at the end of the file:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ferro_crate_returns_tokenstream() {
        // This will use the default fallback in test context
        // (proc-macro-crate reads from CARGO_MANIFEST_DIR which
        // points to ferro-macros during tests)
        let tokens = ferro_crate();
        let token_str = tokens.to_string();

        // Should produce a valid crate path
        assert!(
            token_str.contains("ferro") || token_str == "crate",
            "Expected ferro crate path, got: {}",
            token_str
        );
    }
}
```

### 2. Test with the sample app

The `app/` directory is the sample application. Verify it still works:

```bash
cd /Users/alberto/repositories/albertogferrario/ferro/app
cargo build
cargo test
```

### 3. Update installation documentation

**File:** `docs/src/installation.md`

Add a section about dependency naming:

```markdown
## Dependency Naming

Ferro supports flexible dependency naming in your `Cargo.toml`:

### Standard (Recommended)

```toml
[dependencies]
ferro = "2.0"
```

### With Alias

```toml
[dependencies]
my_web = { package = "ferro", version = "2.0" }
```

When using an alias, all imports use your chosen name:
```rust
use my_web::prelude::*;
```

### Legacy Name

For backwards compatibility, the legacy name is still supported:
```toml
[dependencies]
ferro_rs = "2.0"
```
```

### 4. Run full workspace verification

```bash
cd /Users/alberto/repositories/albertogferrario/ferro

# Full test suite
cargo test --workspace

# Clippy checks
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --check
```

## Verification

```bash
cd /Users/alberto/repositories/albertogferrario/ferro

# Run macro crate tests specifically
cargo test -p ferro-macros

# Build sample app
cargo build -p app

# Run sample app tests
cargo test -p app

# Full workspace test
cargo test --workspace

# Verify no clippy warnings
cargo clippy --workspace -- -D warnings
```

## Success Criteria

- [ ] Unit tests pass for crate path resolution
- [ ] Sample app (`app/`) builds and tests pass
- [ ] Documentation updated with dependency naming options
- [ ] Full workspace tests pass
- [ ] No clippy warnings
- [ ] Code formatted correctly
