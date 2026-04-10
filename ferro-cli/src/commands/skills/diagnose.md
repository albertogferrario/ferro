---
name: ferro:diagnose
description: Diagnose errors using ferro-mcp introspection
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
---

<objective>
Diagnose application errors using ferro-mcp tools and intelligent analysis.

This command:
1. Retrieves the last error from ferro-mcp
2. Analyzes error context (related code, configuration)
3. Suggests fixes based on common patterns
4. Offers to apply fixes automatically
</objective>

<arguments>
Optional:
- `[error_message]` - Specific error to diagnose (otherwise uses last_error)
- `--verbose` - Show detailed diagnostic information

Examples:
- `/ferro:diagnose` - Diagnose last error
- `/ferro:diagnose "connection refused"` - Diagnose specific error
- `/ferro:diagnose --verbose` - Detailed diagnostics
</arguments>

<process>

<step name="get_error">

**If no error message provided:**

Use ferro-mcp `last_error` tool to get the most recent error.

**If error message provided:**

Use the provided message as the diagnostic target.

Also check:
```bash
# Check for recent panics in logs
tail -100 storage/logs/*.log 2>/dev/null | grep -i "panic\|error\|failed"
```

</step>

<step name="categorize_error">

Categorize the error type:

| Category | Indicators |
|----------|------------|
| Database | "connection", "query", "migration", "sqlx", "sea_orm" |
| Route | "not found", "404", "handler", "route" |
| Validation | "validation", "required", "invalid" |
| Auth | "unauthorized", "forbidden", "token", "jwt" |
| Config | "env", "configuration", "missing" |
| Dependency | "version", "cargo", "crate" |
| Runtime | "panic", "unwrap", "overflow" |

</step>

<step name="gather_context">

Based on error category, gather relevant context:

**Database errors:**
- Use `database_schema` tool
- Check `.env` for DATABASE_URL
- Check migration status

**Route errors:**
- Use `list_routes` tool
- Use `explain_route` tool for the failing route
- Check middleware configuration

**Validation errors:**
- Use `get_handler` tool
- Check form request definitions
- Review validation rules

**Auth errors:**
- Check auth middleware configuration
- Review token/session setup
- Check user model

</step>

<step name="analyze">

Analyze the error with context:

1. **Identify root cause** - What actually failed?
2. **Find related code** - Which files are involved?
3. **Check configuration** - Are settings correct?
4. **Review recent changes** - Did something change?

</step>

<step name="suggest_fixes">

Present diagnosis:

```
# Error Diagnosis

## Error
{error_message}

## Category
{category}

## Root Cause
{explanation of what went wrong}

## Related Files
- {file1}: {reason}
- {file2}: {reason}

## Suggested Fixes

### Option 1: {fix_title}
{explanation}

```rust
// Change this:
{old_code}

// To this:
{new_code}
```

### Option 2: {fix_title}
{explanation}

## Prevention
{how to prevent this in the future}
```

</step>

<step name="offer_fix">

Ask if user wants to apply the fix:

"Would you like me to apply the suggested fix?"

If yes, make the necessary code changes.

</step>

</process>

<common_errors>

## Database Connection
**Error:** "error communicating with database" / "connection refused"
**Cause:** Database not running or wrong credentials
**Fix:** Check DATABASE_URL in .env, ensure database server is running

## Missing Migration
**Error:** "table does not exist"
**Cause:** Migration not run
**Fix:** Run `ferro db:migrate`

## Route Not Found
**Error:** "no route found for path"
**Cause:** Route not registered or wrong path
**Fix:** Check routes.rs, verify path matches

## Validation Failed
**Error:** "validation failed"
**Cause:** Input doesn't meet validation rules
**Fix:** Check form request rules, review input data

## Unauthorized
**Error:** "unauthorized" / "token expired"
**Cause:** Missing or invalid authentication
**Fix:** Check auth middleware, verify token handling

## Serialization Error
**Error:** "failed to serialize"
**Cause:** Type doesn't implement Serialize or field issue
**Fix:** Add #[derive(Serialize)], check field types

</common_errors>
