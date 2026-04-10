---
name: ferro:test
description: Run tests with Ferro-specific setup
allowed-tools:
  - Bash
  - Read
---

<objective>
Run the Ferro application test suite with proper configuration.

Handles:
- Database setup for tests (test database)
- Feature flags
- Coverage reporting
- Filtered test runs
</objective>

<arguments>
Optional:
- `[filter]` - Filter tests by name
- `--coverage` - Generate coverage report
- `--watch` - Watch mode (re-run on changes)
- `--doc` - Run doc tests only
- `--integration` - Run integration tests only

Examples:
- `/ferro:test` - Run all tests
- `/ferro:test user` - Run tests matching "user"
- `/ferro:test --coverage` - Run with coverage
- `/ferro:test --integration` - Integration tests only
</arguments>

<process>

<step name="setup_test_env">

Ensure test environment is ready:

```bash
# Check if test database is configured
if grep -q "DATABASE_URL_TEST" .env 2>/dev/null; then
    export DATABASE_URL=$(grep DATABASE_URL_TEST .env | cut -d= -f2-)
    echo "Using test database"
fi
```

</step>

<step name="run_tests">

**Standard test run:**
```bash
cargo test --all-features {filter}
```

**With coverage (--coverage):**
```bash
# Check if cargo-llvm-cov is installed
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

cargo llvm-cov --all-features --html
echo "Coverage report: target/llvm-cov/html/index.html"
```

**Doc tests only (--doc):**
```bash
cargo test --doc --all-features
```

**Integration tests (--integration):**
```bash
cargo test --test '*' --all-features
```

**Watch mode (--watch):**
```bash
# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch..."
    cargo install cargo-watch
fi

cargo watch -x "test --all-features {filter}"
```

</step>

<step name="report_results">

After tests complete, provide summary:

```
# Test Results

✓ Passed: {passed_count}
✗ Failed: {failed_count}
○ Ignored: {ignored_count}

{if failed}
## Failed Tests

{list of failed tests with brief error}

Run `/ferro:diagnose` for detailed error analysis.
{/if}

{if coverage}
## Coverage

- Lines: {line_coverage}%
- Functions: {fn_coverage}%
- Branches: {branch_coverage}%

Report: target/llvm-cov/html/index.html
{/if}
```

</step>

</process>

<test_patterns>

## Writing Ferro Tests

**Handler test:**
```rust
#[ferro_test]
async fn test_user_index() {
    let response = TestClient::new()
        .get("/api/users")
        .send()
        .await;

    response.assert_ok();
    response.assert_json_contains(json!({"data": []}));
}
```

**Authenticated request:**
```rust
#[ferro_test]
async fn test_protected_route() {
    let user = User::factory().create().await;

    let response = TestClient::new()
        .acting_as(&user)
        .get("/api/profile")
        .send()
        .await;

    response.assert_ok();
}
```

**Database test:**
```rust
#[ferro_test]
async fn test_user_creation() {
    let user = User::create(CreateUser {
        email: "test@example.com".into(),
        name: "Test User".into(),
    }).await.unwrap();

    assert_eq!(user.email, "test@example.com");

    // Database is automatically rolled back after test
}
```

</test_patterns>
