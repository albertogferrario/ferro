# Phase 6 Plan 1 Summary: MCP Error Context Enhancement

## Outcome

**Status:** Complete

All 4 tasks completed successfully with atomic commits. The cancer-mcp server now provides enhanced error diagnostics with route correlation, actionable fix suggestions, and a comprehensive error patterns catalog.

## What Was Built

### Task 1: Enhanced last_error with Route Correlation (8d7bfa3)

Added route correlation and error categorization to the existing `last_error` tool:

- **ErrorCategory enum**: Validation, Database, NotFound, Permission, Internal, Panic
- **RouteContext struct**: Captures handler name, path, and HTTP method from error messages
- **categorize_error()**: Analyzes error message content to classify into categories
- **extract_route_context()**: Uses regex patterns to extract route info from stack traces
- **related_routes**: Suggests related routes that might be affected

Key patterns matched:
- `/path/to/resource` URL patterns
- `handler: handler_name` patterns
- `at controller::action` stack trace patterns

### Task 2: Created diagnose_error Tool (9628c4e)

New tool that analyzes errors and provides Cancer-specific fix suggestions:

- **FixSuggestion struct**: action, details, priority (1-5)
- **ErrorDiagnosis struct**: category, likely_cause, fix_suggestions, code_example, related_tools

Category-specific diagnosis:
- Validation: Check #[rule(...)], review ValidateRules derive
- Database: Check migrations, verify DATABASE_URL, review FK constraints
- NotFound: Verify route exists, check model exists in database
- Permission: Check auth middleware, verify session setup
- Panic: Replace .unwrap() with ?, add error handling
- Internal: Check service registration, review DI container

6 unit tests added to verify diagnosis accuracy.

### Task 3: Added error_patterns Catalog (e3f3f8a)

New resource providing 20+ documented error patterns across 6 categories:

**Validation patterns (4):**
- validation_field_required
- validation_email_format
- validation_min_length
- validation_max_length

**Database patterns (5):**
- db_connection_refused
- db_migration_pending
- db_unique_constraint
- db_foreign_key
- db_null_constraint

**NotFound patterns (3):**
- not_found_route
- not_found_model
- not_found_file

**Permission patterns (3):**
- permission_unauthenticated
- permission_forbidden
- permission_token_expired

**Internal patterns (4):**
- internal_service_not_found
- internal_json_error
- internal_param_error
- internal_timeout

**Panic patterns (4):**
- panic_unwrap_none
- panic_unwrap_err
- panic_index_bounds
- panic_stack_overflow

Each pattern includes: id, regex pattern, category, description, resolution, and example.

### Task 4: Enhanced Tool Descriptions (ce3fca6)

Updated service.rs with error debugging guidance:

- **last_error**: Updated description to mention route correlation, categorization, and link to diagnose_error
- **test_route**: Added "On error" guidance pointing to last_error and diagnose_error
- **validate_contracts**: Added "On error" guidance for validation failures
- **Debugging workflow**: Updated to use error tools first (last_error -> diagnose_error -> error_patterns)
- **Tool categories**: Added error tools to Debugging section
- **When to Use**: Added guidance for last_error, diagnose_error, error_patterns, read_logs

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 8d7bfa3 | feat(cancer-mcp): enhance last_error with route correlation and categorization |
| 2 | 9628c4e | feat(cancer-mcp): add diagnose_error tool with fix suggestions |
| 3 | e3f3f8a | feat(cancer-mcp): add error_patterns catalog resource |
| 4 | ce3fca6 | feat(cancer-mcp): enhance tool descriptions with error debugging guidance |

## Files Modified

- `cancer-mcp/src/tools/last_error.rs` - Enhanced with categorization and route extraction
- `cancer-mcp/src/tools/diagnose_error.rs` - New file with diagnosis logic
- `cancer-mcp/src/tools/mod.rs` - Export diagnose_error module
- `cancer-mcp/src/resources/error_patterns.rs` - New file with error catalog
- `cancer-mcp/src/resources/mod.rs` - Export error_patterns module
- `cancer-mcp/src/service.rs` - Tool registration and descriptions
- `cancer-mcp/Cargo.toml` - Added once_cell dependency

## Dependencies Added

- `once_cell = "1"` - For caching compiled regex patterns

## Test Results

All 42 tests pass including:
- 4 new error_patterns tests
- 6 new diagnose_error tests
- All existing tests unchanged

## Decisions Made

1. **Regex caching with once_cell**: Used `once_cell::sync::Lazy` to cache compiled regex patterns for route extraction, avoiding recompilation overhead.

2. **Category-specific fix prioritization**: Fix suggestions have priority 1-5 where 1 is highest. Most actionable fixes (e.g., "Run migrations") get priority 1.

3. **Static error patterns**: Error patterns are hardcoded rather than dynamic to ensure consistency and avoid runtime overhead. Patterns can be extended in future phases.

## How to Use

**Error debugging workflow:**
1. `last_error` - Get categorized error with route context
2. `diagnose_error` - Get Cancer-specific fix suggestions
3. `error_patterns` - Reference common issues if needed
4. `explain_route` / `explain_model` - Understand affected code
5. `get_handler` - Read the failing code
6. `test_route` - Verify the fix works

**Example:**
```
> last_error
{
  "error_category": "database",
  "route_context": { "path": "/users", "method": "POST" },
  "message": "duplicate key value violates unique constraint"
}

> diagnose_error (with above message)
{
  "category": "database",
  "likely_cause": "Attempting to insert duplicate value",
  "fix_suggestions": [
    { "action": "Check if record exists before inserting", "priority": 1 },
    { "action": "Use find_or_create pattern", "priority": 2 }
  ],
  "related_tools": ["db_query", "list_models", "db_schema"]
}
```

## Next Steps

Phase 6 Plan 1 complete. Ready to proceed with Phase 7 (MCP Developer Experience) or continue with additional error handling features.
