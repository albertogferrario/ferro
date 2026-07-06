//! Diagnose error tool - analyze errors and suggest fixes

use crate::tools::last_error::{categorize_error, ErrorCategory};
use serde::Serialize;

/// Parameters for the diagnose_error tool
#[derive(Debug)]
pub struct DiagnoseErrorParams {
    /// Error message to analyze
    pub error_message: Option<String>,
    /// Pre-classified error category
    pub category: Option<ErrorCategory>,
}

/// A fix suggestion with action, details, and priority
#[derive(Debug, Serialize)]
pub struct FixSuggestion {
    /// Action to take (imperative form)
    pub action: String,
    /// Details on how to perform the action
    pub details: String,
    /// Priority (1 = highest, 3 = lowest)
    pub priority: u8,
}

/// Complete error diagnosis
#[derive(Debug, Serialize)]
pub struct ErrorDiagnosis {
    /// Classified error category
    pub category: ErrorCategory,
    /// Likely cause of the error
    pub likely_cause: String,
    /// Ordered list of fix suggestions
    pub fix_suggestions: Vec<FixSuggestion>,
    /// Code example if applicable
    pub code_example: Option<String>,
    /// Related tools to investigate further
    pub related_tools: Vec<String>,
}

/// Diagnose an error and return fix suggestions
pub fn execute(params: DiagnoseErrorParams) -> ErrorDiagnosis {
    // Determine category from params or message
    let category = params.category.unwrap_or_else(|| {
        params
            .error_message
            .as_ref()
            .map(|m| categorize_error(m, "ERROR"))
            .unwrap_or(ErrorCategory::Internal)
    });

    // Generate diagnosis based on category
    match category {
        ErrorCategory::Validation => diagnose_validation(&params.error_message),
        ErrorCategory::Database => diagnose_database(&params.error_message),
        ErrorCategory::NotFound => diagnose_not_found(&params.error_message),
        ErrorCategory::Permission => diagnose_permission(&params.error_message),
        ErrorCategory::Panic => diagnose_panic(&params.error_message),
        ErrorCategory::Internal => diagnose_internal(&params.error_message),
    }
}

fn diagnose_validation(message: &Option<String>) -> ErrorDiagnosis {
    let msg = message.as_deref().unwrap_or("");
    let msg_lower = msg.to_lowercase();

    // Extract field name if present
    let field_context = if msg_lower.contains("field") {
        "Check the specific field mentioned in the error"
    } else {
        "Review all form fields"
    };

    ErrorDiagnosis {
        category: ErrorCategory::Validation,
        likely_cause: "Input data failed validation rules. This typically means a required field is missing, a value is in the wrong format, or a constraint was violated.".to_string(),
        fix_suggestions: vec![
            FixSuggestion {
                action: "Check validation rules on the struct".to_string(),
                details: format!(
                    "{field_context}. Look for #[rule(...)] attributes on the input struct fields."
                ),
                priority: 1,
            },
            FixSuggestion {
                action: "Verify the ValidateRules derive macro is applied".to_string(),
                details: "Ensure the struct has #[derive(ValidateRules)] attribute.".to_string(),
                priority: 2,
            },
            FixSuggestion {
                action: "Check input data format".to_string(),
                details: "Verify the request body matches expected field types and formats.".to_string(),
                priority: 2,
            },
            FixSuggestion {
                action: "Add explicit error messages to rules".to_string(),
                details: r#"Use message parameter: #[rule(required().message("Custom error"))]"#.to_string(),
                priority: 3,
            },
        ],
        code_example: Some(r#"#[derive(ValidateRules, Deserialize)]
pub struct CreateUserInput {
    #[rule(required())]
    pub name: String,
    #[rule(required(), email())]
    pub email: String,
    #[rule(required(), min(8.0))]
    pub password: String,
}"#.to_string()),
        related_tools: vec![
            "list_props".to_string(),
            "inspect_props".to_string(),
            "get_handler".to_string(),
        ],
    }
}

fn diagnose_database(message: &Option<String>) -> ErrorDiagnosis {
    let msg = message.as_deref().unwrap_or("");
    let msg_lower = msg.to_lowercase();

    let likely_cause = if msg_lower.contains("connection") {
        "Database connection failed. Check if the database server is running and credentials are correct."
    } else if msg_lower.contains("migration") {
        "Database migration issue. There may be pending migrations or a migration conflict."
    } else if msg_lower.contains("constraint") || msg_lower.contains("unique") {
        "Database constraint violation. Likely a unique constraint or foreign key constraint was violated."
    } else if msg_lower.contains("query") || msg_lower.contains("syntax") {
        "SQL query error. The query syntax may be incorrect or reference non-existent columns/tables."
    } else {
        "Database operation failed. Check the database configuration and query structure."
    };

    ErrorDiagnosis {
        category: ErrorCategory::Database,
        likely_cause: likely_cause.to_string(),
        fix_suggestions: vec![
            FixSuggestion {
                action: "Check DATABASE_URL in .env".to_string(),
                details: "Verify the database connection string is correct and the database exists.".to_string(),
                priority: 1,
            },
            FixSuggestion {
                action: "Run pending migrations".to_string(),
                details: "Execute `ferro db:migrate` to apply any pending database migrations.".to_string(),
                priority: 1,
            },
            FixSuggestion {
                action: "Verify table and column existence".to_string(),
                details: "Use db_schema tool to check if the referenced tables and columns exist.".to_string(),
                priority: 2,
            },
            FixSuggestion {
                action: "Check for constraint violations".to_string(),
                details: "If inserting/updating, ensure unique fields are unique and foreign keys reference existing records.".to_string(),
                priority: 2,
            },
        ],
        code_example: Some(r#"// Check database connection
let db = DB::get()?;

// Verify entity exists before update
let entity = Entity::find_by_id(id)
    .one(&*db)
    .await?
    .ok_or(FrameworkError::model_not_found("Entity"))?;"#.to_string()),
        related_tools: vec![
            "db_schema".to_string(),
            "list_migrations".to_string(),
            "db_query".to_string(),
            "get_config".to_string(),
        ],
    }
}

fn diagnose_not_found(message: &Option<String>) -> ErrorDiagnosis {
    let msg = message.as_deref().unwrap_or("");
    let msg_lower = msg.to_lowercase();

    let likely_cause = if msg_lower.contains("route") || msg_lower.contains("path") {
        "The requested route does not exist. The URL may be incorrect or the route is not registered."
    } else if msg_lower.contains("model") || msg_lower.contains("record") {
        "The requested database record does not exist. The ID may be invalid or the record was deleted."
    } else if msg_lower.contains("file") {
        "The requested file does not exist at the specified path."
    } else {
        "The requested resource was not found. Check if the resource exists and the identifier is correct."
    };

    ErrorDiagnosis {
        category: ErrorCategory::NotFound,
        likely_cause: likely_cause.to_string(),
        fix_suggestions: vec![
            FixSuggestion {
                action: "Verify route exists in list_routes output".to_string(),
                details: "Use the list_routes tool to see all registered routes and verify the path.".to_string(),
                priority: 1,
            },
            FixSuggestion {
                action: "Check if the record exists in database".to_string(),
                details: "Use db_query to verify the record with that ID exists: SELECT * FROM table WHERE id = X".to_string(),
                priority: 1,
            },
            FixSuggestion {
                action: "Verify URL parameters are correct".to_string(),
                details: "Check that route parameters match the expected format (e.g., /users/{id} expects numeric ID).".to_string(),
                priority: 2,
            },
            FixSuggestion {
                action: "Add proper 404 handling in handler".to_string(),
                details: "Use FrameworkError::model_not_found() for clearer error messages.".to_string(),
                priority: 3,
            },
        ],
        code_example: Some(r#"// Proper 404 handling in handler
let user = User::find_by_id(id)
    .one(&*db)
    .await?
    .ok_or(FrameworkError::model_not_found("User"))?;

// Or with custom error
.ok_or(AppError::not_found(format!("User {} not found", id)))?;"#.to_string()),
        related_tools: vec![
            "list_routes".to_string(),
            "db_query".to_string(),
            "get_handler".to_string(),
        ],
    }
}

fn diagnose_permission(message: &Option<String>) -> ErrorDiagnosis {
    let msg = message.as_deref().unwrap_or("");
    let msg_lower = msg.to_lowercase();

    let likely_cause = if msg_lower.contains("401") || msg_lower.contains("unauthenticated") {
        "User is not authenticated. The request requires a valid session or token."
    } else if msg_lower.contains("403") || msg_lower.contains("forbidden") {
        "User is authenticated but lacks permission for this action."
    } else if msg_lower.contains("token") || msg_lower.contains("jwt") {
        "Authentication token is invalid, expired, or missing."
    } else {
        "Authorization failed. User may not be logged in or may lack required permissions."
    };

    ErrorDiagnosis {
        category: ErrorCategory::Permission,
        likely_cause: likely_cause.to_string(),
        fix_suggestions: vec![
            FixSuggestion {
                action: "Check auth middleware is applied to route".to_string(),
                details: "Verify the route group has auth middleware: Route::group(...).middleware(auth::middleware())".to_string(),
                priority: 1,
            },
            FixSuggestion {
                action: "Verify user session is valid".to_string(),
                details: "Use session_inspect tool to check if the user has an active session.".to_string(),
                priority: 1,
            },
            FixSuggestion {
                action: "Check permission logic in handler".to_string(),
                details: "If using role-based access, verify the user has required roles/permissions.".to_string(),
                priority: 2,
            },
            FixSuggestion {
                action: "Verify CORS configuration for API routes".to_string(),
                details: "For cross-origin requests, ensure CORS middleware allows the request origin.".to_string(),
                priority: 3,
            },
        ],
        code_example: Some(r#"// Protected route group
Route::group()
    .prefix("/dashboard")
    .middleware(auth::middleware())
    .routes(|route| {
        route.get("/", dashboard::index);
    });

// Handler with user extraction
#[handler]
pub async fn index(req: Request, user: User) -> Response {
    // user is automatically extracted from session
    // Returns 401 if not authenticated
}"#.to_string()),
        related_tools: vec![
            "list_middleware".to_string(),
            "session_inspect".to_string(),
            "get_handler".to_string(),
            "list_routes".to_string(),
        ],
    }
}

fn diagnose_panic(message: &Option<String>) -> ErrorDiagnosis {
    let msg = message.as_deref().unwrap_or("");
    let msg_lower = msg.to_lowercase();

    let likely_cause = if msg_lower.contains("unwrap") {
        "Called .unwrap() on a None or Err value. This is a common cause of panics."
    } else if msg_lower.contains("index out of bounds") {
        "Array or vector index out of bounds. Accessing an element that doesn't exist."
    } else if msg_lower.contains("overflow") {
        "Integer overflow or underflow occurred during arithmetic operation."
    } else if msg_lower.contains("stack overflow") {
        "Stack overflow, likely due to infinite recursion."
    } else {
        "Application panic occurred. Review the stack trace to find the cause."
    };

    ErrorDiagnosis {
        category: ErrorCategory::Panic,
        likely_cause: likely_cause.to_string(),
        fix_suggestions: vec![
            FixSuggestion {
                action: "Replace .unwrap() with proper error handling".to_string(),
                details: "Use .ok_or()?, .map_err()?, or pattern matching instead of .unwrap()."
                    .to_string(),
                priority: 1,
            },
            FixSuggestion {
                action: "Add bounds checking before array access".to_string(),
                details: "Use .get(index) instead of [index] for safe array access.".to_string(),
                priority: 1,
            },
            FixSuggestion {
                action: "Review the stack trace".to_string(),
                details: "Use last_error to get the full stack trace and identify the exact line."
                    .to_string(),
                priority: 2,
            },
            FixSuggestion {
                action: "Add panic handler for graceful degradation".to_string(),
                details: "Consider using std::panic::catch_unwind for critical paths.".to_string(),
                priority: 3,
            },
        ],
        code_example: Some(
            r#"// Instead of unwrap:
let value = some_option.unwrap();  // PANIC if None!

// Use error propagation:
let value = some_option.ok_or(FrameworkError::internal("Value missing"))?;

// Or pattern matching:
let value = match some_option {
    Some(v) => v,
    None => return Err(AppError::not_found("Value missing").into()),
};

// Safe array access:
let item = items.get(index).ok_or(AppError::bad_request("Invalid index"))?;"#
                .to_string(),
        ),
        related_tools: vec![
            "last_error".to_string(),
            "read_logs".to_string(),
            "get_handler".to_string(),
        ],
    }
}

fn diagnose_internal(message: &Option<String>) -> ErrorDiagnosis {
    let msg = message.as_deref().unwrap_or("");
    let msg_lower = msg.to_lowercase();

    let likely_cause = if msg_lower.contains("service") {
        "A service dependency was not found in the container. Check if it's registered."
    } else if msg_lower.contains("timeout") {
        "Operation timed out. This could be a slow database query, external API, or resource exhaustion."
    } else if msg_lower.contains("json") || msg_lower.contains("deserialize") {
        "JSON serialization/deserialization failed. Request or response format may be incorrect."
    } else {
        "An internal server error occurred. Review logs for more context."
    };

    ErrorDiagnosis {
        category: ErrorCategory::Internal,
        likely_cause: likely_cause.to_string(),
        fix_suggestions: vec![
            FixSuggestion {
                action: "Check the full error logs".to_string(),
                details: "Use read_logs tool with level=error to see detailed error context.".to_string(),
                priority: 1,
            },
            FixSuggestion {
                action: "Verify service registration".to_string(),
                details: "If ServiceNotFound, ensure the service has #[injectable] and is registered in bootstrap.".to_string(),
                priority: 2,
            },
            FixSuggestion {
                action: "Check request/response serialization".to_string(),
                details: "Verify JSON structure matches expected Serialize/Deserialize types.".to_string(),
                priority: 2,
            },
            FixSuggestion {
                action: "Add structured error handling".to_string(),
                details: "Use FrameworkError variants for typed errors instead of generic strings.".to_string(),
                priority: 3,
            },
        ],
        code_example: Some(r#"// Service registration
#[service(MyServiceImpl)]
pub trait MyService: Send + Sync { ... }

#[injectable]
pub struct MyServiceImpl;

// Proper error handling
pub async fn handler(req: Request) -> Response {
    let service = App::resolve::<dyn MyService>()?;

    let data = req.input::<MyInput>().await
        .map_err(|e| FrameworkError::internal(format!("Invalid input: {}", e)))?;

    Ok(json!({"status": "ok"}))
}"#.to_string()),
        related_tools: vec![
            "read_logs".to_string(),
            "last_error".to_string(),
            "list_services".to_string(),
            "get_handler".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnose_validation_error() {
        let params = DiagnoseErrorParams {
            error_message: Some("Validation failed for field 'email'".to_string()),
            category: None,
        };
        let diagnosis = execute(params);
        assert_eq!(diagnosis.category, ErrorCategory::Validation);
        assert!(!diagnosis.fix_suggestions.is_empty());
        assert!(diagnosis.code_example.is_some());
    }

    #[test]
    fn test_diagnose_database_error() {
        let params = DiagnoseErrorParams {
            error_message: Some("Database connection refused".to_string()),
            category: None,
        };
        let diagnosis = execute(params);
        assert_eq!(diagnosis.category, ErrorCategory::Database);
        assert!(diagnosis
            .likely_cause
            .contains("Database connection failed"));
    }

    #[test]
    fn test_diagnose_not_found_error() {
        let params = DiagnoseErrorParams {
            error_message: Some("User not found".to_string()),
            category: None,
        };
        let diagnosis = execute(params);
        assert_eq!(diagnosis.category, ErrorCategory::NotFound);
    }

    #[test]
    fn test_diagnose_permission_error() {
        let params = DiagnoseErrorParams {
            error_message: Some("401 Unauthorized".to_string()),
            category: None,
        };
        let diagnosis = execute(params);
        assert_eq!(diagnosis.category, ErrorCategory::Permission);
    }

    #[test]
    fn test_diagnose_panic_error() {
        let params = DiagnoseErrorParams {
            error_message: Some("thread 'main' panicked at 'called unwrap() on None'".to_string()),
            category: None,
        };
        let diagnosis = execute(params);
        assert_eq!(diagnosis.category, ErrorCategory::Panic);
        assert!(diagnosis.likely_cause.contains("unwrap"));
    }

    #[test]
    fn test_diagnose_with_explicit_category() {
        let params = DiagnoseErrorParams {
            error_message: None,
            category: Some(ErrorCategory::Internal),
        };
        let diagnosis = execute(params);
        assert_eq!(diagnosis.category, ErrorCategory::Internal);
    }
}
