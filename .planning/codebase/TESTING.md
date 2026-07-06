# Testing Patterns

**Analysis Date:** 2026-01-15

## Test Framework

**Runner:**
- Rust built-in test framework
- Config: Default cargo test configuration

**Assertion Library:**
- Standard `assert!()`, `assert_eq!()`, `assert_ne!()`
- pretty_assertions 1.4 for enhanced diff output (`framework/Cargo.toml`)

**Run Commands:**
```bash
cargo test                              # Run all tests
cargo test --all-features               # Run with all features enabled
cargo test -- --nocapture               # Show println output
cargo test -p framework                 # Test specific crate
cargo test test_name                    # Run specific test
```

## Test File Organization

**Location:**
- Tests co-located with source code (inline modules)
- `#[cfg(test)]` modules at end of implementation files
- No separate `tests/` directory for unit tests

**Naming:**
- Test functions: `test_<functionality>()` (snake_case)
- Test modules: `mod tests` within source files

**Structure:**
```
framework/src/
  validation/
    validator.rs       # Implementation + tests at bottom
    error.rs          # Implementation + tests at bottom
  database/
    model.rs          # Implementation + tests at bottom
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // arrange
        let input = create_test_input();

        // act
        let result = function_under_test(input);

        // assert
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_operation() {
        // async test code
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

**Patterns:**
- Arrange-Act-Assert pattern
- One assertion focus per test (multiple asserts OK if related)
- Test both success and failure cases
- Use descriptive test names

## Mocking

**Framework:**
- No dedicated mocking framework
- Use trait abstractions for dependency injection
- Test doubles via manual implementations

**Patterns:**
```rust
// Use Arc for shared state in async tests
let counter = Arc::new(AtomicU32::new(0));
let counter_clone = Arc::clone(&counter);

// Pass clone to closure
dispatcher.on::<TestEvent, _, _>(move |event| {
    let counter = Arc::clone(&counter_clone);
    async move {
        counter.fetch_add(event.value, Ordering::SeqCst);
        Ok(())
    }
});
```

**What to Mock:**
- External services (database, Redis, HTTP)
- Time-dependent operations
- File system operations

**What NOT to Mock:**
- Pure functions
- Simple utility functions
- Internal business logic

## Fixtures and Factories

**Test Data:**
```rust
// Inline test data creation
fn create_test_config() -> Config {
    Config {
        host: "localhost".to_string(),
        port: 8080,
    }
}

// Test events
struct TestEvent {
    value: u32,
}
```

**Location:**
- Factory functions defined within test modules
- No separate fixtures directory
- Test utilities: `framework/src/testing/factory.rs`

**Testing Utilities:**
- `framework/src/testing/mod.rs` - Test module root
- `framework/src/testing/factory.rs` - Data factories (1,274 lines)
- `framework/src/testing/http.rs` - HTTP test client (739 lines)
- `framework/src/testing/expect.rs` - Assertion helpers

## Coverage

**Requirements:**
- No enforced coverage target
- Focus on critical paths (handlers, validation, auth)
- Coverage for awareness, not enforcement

**Configuration:**
- No dedicated coverage tool configured
- Can use `cargo-tarpaulin` or `cargo-llvm-cov` externally

**View Coverage:**
```bash
cargo install cargo-tarpaulin
cargo tarpaulin
```

## Test Types

**Unit Tests:**
- Scope: Single function/method in isolation
- Location: Inline `#[cfg(test)]` modules
- Speed: Fast (<100ms per test)
- Examples: `ferro-events/src/dispatcher.rs`, `framework/src/validation/error.rs`

**Integration Tests:**
- Scope: Multiple modules together
- Location: `framework/src/testing/` utilities
- Speed: Moderate
- Examples: Service integration via test utilities

**E2E Tests:**
- Not currently implemented
- `framework/src/testing/http.rs` provides HTTP client foundation
- Note: HTTP client has TODO for actual router integration (line 52)

## Common Patterns

**Async Testing:**
```rust
#[tokio::test]
async fn test_async_dispatch() {
    let dispatcher = EventDispatcher::new();

    dispatcher.on::<TestEvent, _, _>(|_| async { Ok(()) });

    let result = dispatcher.dispatch(TestEvent { value: 5 }).await;
    assert!(result.is_ok());
}
```

**Error Testing:**
```rust
#[test]
fn test_validation_error() {
    let mut errors = ValidationError::new();
    errors.add("email", "required");

    assert!(!errors.is_empty());
    assert!(errors.has("email"));
    assert_eq!(errors.count(), 1);
}
```

**Concurrent State Testing:**
```rust
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn test_concurrent_updates() {
    let counter = Arc::new(AtomicU32::new(0));

    // Clone for closure
    let c = Arc::clone(&counter);

    // Use in async context
    c.fetch_add(1, Ordering::SeqCst);

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}
```

**Serial Test Execution:**
```rust
// For tests that can't run in parallel
use serial_test::serial;

#[test]
#[serial]
fn test_global_state() {
    // Test that modifies global state
}
```

## Development Workflow

**Watch Mode:**
```bash
bacon test  # Run tests on file changes
```

**Bacon Configuration (`bacon.toml`):**
```toml
[jobs.test]
command = ["cargo", "test", "--color", "always"]
need_stdout = true
```

**Pre-commit:**
```bash
cargo test --all-features
cargo fmt --check
cargo clippy
```

## Snapshot Testing

**Usage:** Not used in this codebase
**Recommendation:** Prefer explicit assertions for clarity

---

*Testing analysis: 2026-01-15*
*Update when test patterns change*
