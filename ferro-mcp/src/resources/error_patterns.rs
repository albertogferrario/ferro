//! Error patterns catalog - common error patterns and their resolutions

use crate::tools::last_error::ErrorCategory;
use serde::Serialize;

/// A documented error pattern with resolution guidance
#[derive(Debug, Serialize)]
pub struct ErrorPattern {
    /// Pattern identifier for reference
    pub id: &'static str,
    /// Regex or string pattern to match in error messages
    pub pattern: &'static str,
    /// Error category
    pub category: ErrorCategory,
    /// Description of what causes this error
    pub description: &'static str,
    /// How to resolve this error
    pub resolution: &'static str,
    /// Example error message
    pub example: &'static str,
}

/// Collection of error patterns grouped by category
#[derive(Debug, Serialize)]
pub struct ErrorPatternsCatalog {
    /// Total number of patterns
    pub total_patterns: usize,
    /// Patterns grouped by category
    pub categories: Vec<CategoryPatterns>,
}

/// Patterns for a specific category
#[derive(Debug, Serialize)]
pub struct CategoryPatterns {
    /// Category name
    pub category: ErrorCategory,
    /// Category description
    pub description: &'static str,
    /// Patterns in this category
    pub patterns: Vec<ErrorPattern>,
}

/// Get all error patterns organized by category
pub fn get_error_patterns() -> ErrorPatternsCatalog {
    let validation_patterns = vec![
        ErrorPattern {
            id: "validation_field_required",
            pattern: r"(?i)(required|missing).*field|field.*required",
            category: ErrorCategory::Validation,
            description: "A required field was not provided in the request.",
            resolution: "Add the missing field to your request body. Check struct for #[rule(required())] attributes.",
            example: "Validation error for 'email': The email field is required.",
        },
        ErrorPattern {
            id: "validation_email_format",
            pattern: r"(?i)must be.*email|invalid email|email.*format",
            category: ErrorCategory::Validation,
            description: "Email field does not match expected email format.",
            resolution: "Provide a valid email address format (e.g., user@example.com). Check #[rule(email())] on field.",
            example: "Validation error for 'email': The email must be a valid email address.",
        },
        ErrorPattern {
            id: "validation_min_length",
            pattern: r"(?i)(minimum|at least|min).*\d+.*(character|length)|too short",
            category: ErrorCategory::Validation,
            description: "String field is shorter than minimum length requirement.",
            resolution: "Provide a value that meets the minimum length. Check #[rule(min(X.0))] on field.",
            example: "Validation error for 'password': The password must be at least 8 characters.",
        },
        ErrorPattern {
            id: "validation_max_length",
            pattern: r"(?i)(maximum|at most|max).*\d+.*(character|length)|too long",
            category: ErrorCategory::Validation,
            description: "String field exceeds maximum length requirement.",
            resolution: "Reduce the field value to meet the maximum length. Check #[rule(max(X.0))] on field.",
            example: "Validation error for 'name': The name may not be greater than 255 characters.",
        },
    ];

    let database_patterns = vec![
        ErrorPattern {
            id: "db_connection_refused",
            pattern: r"(?i)connection refused|failed to connect|connection.*reset",
            category: ErrorCategory::Database,
            description: "Cannot connect to the database server.",
            resolution: "1. Verify database server is running\n2. Check DATABASE_URL in .env\n3. Verify network/firewall settings",
            example: "Database error: connection refused (tcp:5432)",
        },
        ErrorPattern {
            id: "db_migration_pending",
            pattern: r"(?i)pending migration|migration.*failed|table.*not exist",
            category: ErrorCategory::Database,
            description: "Database schema is out of sync with code. Migrations may be pending.",
            resolution: "Run `ferro migrate` to apply pending migrations. Check list_migrations for status.",
            example: "Database error: relation \"users\" does not exist",
        },
        ErrorPattern {
            id: "db_unique_constraint",
            pattern: r"(?i)unique.*constraint|duplicate.*key|already exists",
            category: ErrorCategory::Database,
            description: "Attempting to insert a duplicate value for a unique column.",
            resolution: "Check if the record already exists before inserting. Use find_or_create patterns or handle the conflict.",
            example: "Database error: duplicate key value violates unique constraint \"users_email_key\"",
        },
        ErrorPattern {
            id: "db_foreign_key",
            pattern: r"(?i)foreign key.*constraint|violates.*reference|referenced.*not exist",
            category: ErrorCategory::Database,
            description: "Foreign key constraint violation - referenced record doesn't exist.",
            resolution: "Ensure the referenced record exists before creating the relationship. Check relation_map for FK structure.",
            example: "Database error: insert violates foreign key constraint \"posts_user_id_fkey\"",
        },
        ErrorPattern {
            id: "db_null_constraint",
            pattern: r"(?i)null.*constraint|cannot be null|not null.*violated",
            category: ErrorCategory::Database,
            description: "Attempting to insert NULL into a non-nullable column.",
            resolution: "Provide a value for the required field or update the migration to allow NULL.",
            example: "Database error: null value in column \"name\" violates not-null constraint",
        },
    ];

    let not_found_patterns = vec![
        ErrorPattern {
            id: "not_found_route",
            pattern: r"(?i)route not found|404.*not found|no matching route",
            category: ErrorCategory::NotFound,
            description: "The requested URL does not match any defined route.",
            resolution: "Use list_routes to verify the route exists. Check for typos in the URL or missing route registration.",
            example: "Not found: GET /api/userz",
        },
        ErrorPattern {
            id: "not_found_model",
            pattern: r"(?i)(\w+) not found|record.*not found|no.*matching.*id",
            category: ErrorCategory::NotFound,
            description: "The requested database record does not exist.",
            resolution: "Verify the ID is correct. Use db_query to check if the record exists in the database.",
            example: "User not found",
        },
        ErrorPattern {
            id: "not_found_file",
            pattern: r"(?i)file not found|no such file|path.*not exist",
            category: ErrorCategory::NotFound,
            description: "The requested file or path does not exist.",
            resolution: "Check the file path is correct. For storage files, verify the file was uploaded successfully.",
            example: "File not found: storage/app/uploads/image.png",
        },
    ];

    let permission_patterns = vec![
        ErrorPattern {
            id: "permission_unauthenticated",
            pattern: r"(?i)401|unauthenticated|not logged in|login required",
            category: ErrorCategory::Permission,
            description: "Request requires authentication but user is not logged in.",
            resolution: "Ensure the user is authenticated. Check session_inspect for valid session. Verify auth middleware is applied.",
            example: "401 Unauthorized: Authentication required",
        },
        ErrorPattern {
            id: "permission_forbidden",
            pattern: r"(?i)403|forbidden|access denied|not authorized",
            category: ErrorCategory::Permission,
            description: "User is authenticated but lacks permission for this action.",
            resolution: "Check user roles/permissions. Review authorization logic in the handler. Verify policy rules.",
            example: "403 Forbidden: You do not have permission to access this resource",
        },
        ErrorPattern {
            id: "permission_token_expired",
            pattern: r"(?i)token.*expired|session.*expired|jwt.*invalid",
            category: ErrorCategory::Permission,
            description: "Authentication token or session has expired.",
            resolution: "Refresh the token or re-authenticate. Check token expiration settings in configuration.",
            example: "Token has expired",
        },
    ];

    let internal_patterns = vec![
        ErrorPattern {
            id: "internal_service_not_found",
            pattern: r"(?i)ServiceNotFound|service.*not registered|no binding",
            category: ErrorCategory::Internal,
            description: "A dependency injection service was not found in the container.",
            resolution: "1. Add #[injectable] to the struct\n2. Add #[service(ConcreteType)] to the trait\n3. Register in application bootstrap",
            example: "Service 'dyn MyService' not registered in container",
        },
        ErrorPattern {
            id: "internal_json_error",
            pattern: r"(?i)json.*error|deserialize.*failed|parse.*json|serde.*error",
            category: ErrorCategory::Internal,
            description: "JSON serialization or deserialization failed.",
            resolution: "Check that request body is valid JSON. Verify struct fields match expected JSON structure.",
            example: "Json deserialize error: missing field `name` at line 1 column 23",
        },
        ErrorPattern {
            id: "internal_param_error",
            pattern: r"(?i)missing.*parameter|invalid parameter|ParamError",
            category: ErrorCategory::Internal,
            description: "A required path parameter is missing or invalid.",
            resolution: "Check route definition matches handler parameters. Verify parameter types are correct.",
            example: "Missing required parameter: id",
        },
        ErrorPattern {
            id: "internal_timeout",
            pattern: r"(?i)timeout|timed out|deadline exceeded",
            category: ErrorCategory::Internal,
            description: "Operation timed out - typically slow database queries or external API calls.",
            resolution: "Optimize slow queries. Add database indexes. Increase timeout for legitimate slow operations.",
            example: "Database query timed out after 30s",
        },
    ];

    let panic_patterns = vec![
        ErrorPattern {
            id: "panic_unwrap_none",
            pattern: r"(?i)called.*unwrap\(\).*on.*None|unwrap.*failed|None.*unwrap",
            category: ErrorCategory::Panic,
            description: "Called .unwrap() on an Option that was None.",
            resolution: "Replace .unwrap() with .ok_or()?, .unwrap_or_default(), or pattern matching.",
            example: "thread 'main' panicked at 'called `Option::unwrap()` on a `None` value'",
        },
        ErrorPattern {
            id: "panic_unwrap_err",
            pattern: r"(?i)called.*unwrap\(\).*on.*Err|unwrap.*error",
            category: ErrorCategory::Panic,
            description: "Called .unwrap() on a Result that was Err.",
            resolution: "Use ? operator for error propagation or handle errors explicitly with match/map_err.",
            example: "thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value'",
        },
        ErrorPattern {
            id: "panic_index_bounds",
            pattern: r"(?i)index out of bounds|length is \d+ but.*index is \d+",
            category: ErrorCategory::Panic,
            description: "Attempted to access an array/vector index that doesn't exist.",
            resolution: "Use .get(index) instead of [index] for safe access. Verify collection length before access.",
            example: "thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 5'",
        },
        ErrorPattern {
            id: "panic_stack_overflow",
            pattern: r"(?i)stack overflow|thread.*overflowed.*stack",
            category: ErrorCategory::Panic,
            description: "Stack overflow, usually caused by infinite recursion.",
            resolution: "Check for recursive function calls without proper base cases. Consider using iteration instead.",
            example: "thread 'main' has overflowed its stack",
        },
    ];

    let categories = vec![
        CategoryPatterns {
            category: ErrorCategory::Validation,
            description: "Input validation errors - field format, length, required fields",
            patterns: validation_patterns,
        },
        CategoryPatterns {
            category: ErrorCategory::Database,
            description: "Database errors - connection, queries, constraints, migrations",
            patterns: database_patterns,
        },
        CategoryPatterns {
            category: ErrorCategory::NotFound,
            description: "Not found errors - missing routes, records, files",
            patterns: not_found_patterns,
        },
        CategoryPatterns {
            category: ErrorCategory::Permission,
            description: "Permission errors - authentication, authorization, tokens",
            patterns: permission_patterns,
        },
        CategoryPatterns {
            category: ErrorCategory::Internal,
            description: "Internal errors - services, JSON, parameters, timeouts",
            patterns: internal_patterns,
        },
        CategoryPatterns {
            category: ErrorCategory::Panic,
            description: "Panic errors - unwrap failures, index bounds, stack overflow",
            patterns: panic_patterns,
        },
    ];

    let total_patterns: usize = categories.iter().map(|c| c.patterns.len()).sum();

    ErrorPatternsCatalog {
        total_patterns,
        categories,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_error_patterns() {
        let catalog = get_error_patterns();
        assert!(
            catalog.total_patterns >= 20,
            "Should have at least 20 patterns"
        );
        assert_eq!(catalog.categories.len(), 6, "Should have 6 categories");
    }

    #[test]
    fn test_all_patterns_have_required_fields() {
        let catalog = get_error_patterns();
        for category in catalog.categories {
            for pattern in category.patterns {
                assert!(!pattern.id.is_empty(), "Pattern should have id");
                assert!(!pattern.pattern.is_empty(), "Pattern should have pattern");
                assert!(
                    !pattern.description.is_empty(),
                    "Pattern should have description"
                );
                assert!(
                    !pattern.resolution.is_empty(),
                    "Pattern should have resolution"
                );
                assert!(!pattern.example.is_empty(), "Pattern should have example");
            }
        }
    }

    #[test]
    fn test_validation_patterns_exist() {
        let catalog = get_error_patterns();
        let validation = catalog
            .categories
            .iter()
            .find(|c| c.category == ErrorCategory::Validation);
        assert!(validation.is_some(), "Should have validation category");
        assert!(
            validation.unwrap().patterns.len() >= 4,
            "Should have at least 4 validation patterns"
        );
    }

    #[test]
    fn test_panic_patterns_exist() {
        let catalog = get_error_patterns();
        let panic = catalog
            .categories
            .iter()
            .find(|c| c.category == ErrorCategory::Panic);
        assert!(panic.is_some(), "Should have panic category");
        assert!(
            panic.unwrap().patterns.len() >= 4,
            "Should have at least 4 panic patterns"
        );
    }
}
