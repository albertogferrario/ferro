//! MCP introspection tools for AI classification and confirmation primitives.
//!
//! Provides two tools:
//! - `test_classifier` — sends a test classification request to the Anthropic provider
//! - `list_pending_confirmations` — scans source for `request_confirmation` call sites

use regex::Regex;
use serde::Serialize;
use std::path::Path;

// ---------------------------------------------------------------------------
// test_classifier
// ---------------------------------------------------------------------------

/// Result of a test classification request.
#[derive(Debug, Serialize)]
pub struct TestClassifierResult {
    /// True when the API call succeeded and returned JSON.
    pub success: bool,
    /// The raw classified JSON output (or null on error).
    pub result: Option<serde_json::Value>,
    /// Model used for the request.
    pub model: String,
    /// Error message (None on success).
    pub error: Option<String>,
}

/// Parameters for a test classification request.
pub struct TestClassifierParams {
    pub system_prompt: String,
    pub user_prompt: String,
    /// JSON Schema string describing the expected output shape.
    pub schema_json: String,
    /// Optional model override (defaults to ClassifierConfig::default().model).
    pub model: Option<String>,
}

/// Send a test classification request to the Anthropic API.
///
/// Reads `ANTHROPIC_API_KEY` from the environment. Returns a structured result
/// with `success: false` and an `error` field if the key is absent or the
/// request fails — it does not panic.
///
/// **Note:** Makes a real API call. Costs tokens.
pub async fn test_classifier(
    project_root: &Path,
    params: TestClassifierParams,
) -> TestClassifierResult {
    use ferro_ai::{AnthropicProvider, ClassificationProvider, ClassifierConfig};

    // Load .env so the key is available without a running server.
    let env_path = project_root.join(".env");
    if env_path.exists() {
        let _ = dotenvy::from_path(&env_path);
    }

    let model = params
        .model
        .unwrap_or_else(|| ClassifierConfig::default().model);

    // Parse the schema JSON.
    let schema: serde_json::Value = match serde_json::from_str(&params.schema_json) {
        Ok(v) => v,
        Err(e) => {
            return TestClassifierResult {
                success: false,
                result: None,
                model,
                error: Some(format!("invalid schema_json: {e}")),
            };
        }
    };

    // Build provider — from_env() returns Err when ANTHROPIC_API_KEY is absent.
    let provider = match AnthropicProvider::from_env() {
        Ok(p) => p,
        Err(e) => {
            return TestClassifierResult {
                success: false,
                result: None,
                model,
                error: Some(format!(
                    "ANTHROPIC_API_KEY is not set. Set it in .env or the environment. ({e})"
                )),
            };
        }
    };

    let config = ClassifierConfig {
        model: model.clone(),
        ..ClassifierConfig::default()
    };

    match provider
        .classify_raw(&params.system_prompt, &params.user_prompt, &schema, &config)
        .await
    {
        Ok(raw_json) => TestClassifierResult {
            success: true,
            result: Some(raw_json),
            model,
            error: None,
        },
        Err(e) => TestClassifierResult {
            success: false,
            result: None,
            model,
            error: Some(e.to_string()),
        },
    }
}

// ---------------------------------------------------------------------------
// list_pending_confirmations
// ---------------------------------------------------------------------------

/// A discovered `request_confirmation` call site in the project source.
#[derive(Debug, Serialize)]
pub struct ConfirmationCallSite {
    /// Relative file path.
    pub file: String,
    /// Line number (1-indexed).
    pub line: usize,
    /// Source line context (trimmed).
    pub context: String,
}

/// Result of scanning source for confirmation usage.
#[derive(Debug, Serialize)]
pub struct PendingConfirmationsResult {
    /// All discovered `request_confirmation` call sites.
    pub call_sites: Vec<ConfirmationCallSite>,
    /// Total number of sites found.
    pub total: usize,
    /// Human-readable note about confirmation state.
    pub note: String,
}

/// Scan `src/` for `request_confirmation(` calls and report file:line:context.
///
/// Confirmation state is in-memory and not inspectable via source scanning at
/// runtime. This tool reports where confirmations are *used* in the codebase,
/// which is useful for auditing flows and understanding which handlers gate
/// actions behind confirmation.
pub fn list_pending_confirmations(project_root: &Path) -> PendingConfirmationsResult {
    let src_dir = project_root.join("src");
    if !src_dir.is_dir() {
        return PendingConfirmationsResult {
            call_sites: Vec::new(),
            total: 0,
            note: "No src/ directory found in project root.".to_string(),
        };
    }

    let re = Regex::new(r"request_confirmation\s*\(").unwrap();
    let mut call_sites = Vec::new();

    scan_dir(&src_dir, project_root, &re, &mut call_sites);

    let total = call_sites.len();
    let note = if total == 0 {
        "No request_confirmation calls found. Add them in handlers to gate destructive actions behind user confirmation.".to_string()
    } else {
        "Confirmation state is in-memory (InMemoryConfirmationStore) and not inspectable at runtime via MCP. Use the call sites above to trace confirmation flows.".to_string()
    };

    PendingConfirmationsResult {
        call_sites,
        total,
        note,
    }
}

/// Recursively scan a directory for `.rs` files containing the pattern.
fn scan_dir(
    dir: &Path,
    project_root: &Path,
    re: &Regex,
    call_sites: &mut Vec<ConfirmationCallSite>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, project_root, re, call_sites);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            scan_file(&path, project_root, re, call_sites);
        }
    }
}

/// Scan a single `.rs` file for pattern matches.
fn scan_file(
    file: &Path,
    project_root: &Path,
    re: &Regex,
    call_sites: &mut Vec<ConfirmationCallSite>,
) {
    let content = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(_) => return,
    };

    let relative = file
        .strip_prefix(project_root)
        .unwrap_or(file)
        .to_string_lossy()
        .to_string();

    for (idx, line) in content.lines().enumerate() {
        if re.is_match(line) {
            call_sites.push(ConfirmationCallSite {
                file: relative.clone(),
                line: idx + 1,
                context: line.trim().to_string(),
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // --- list_pending_confirmations tests ---

    #[test]
    fn test_list_confirmations_no_src_dir() {
        let tmp = TempDir::new().unwrap();
        let result = list_pending_confirmations(tmp.path());
        assert_eq!(result.total, 0);
        assert!(result.call_sites.is_empty());
        assert!(result.note.contains("No src/"));
    }

    #[test]
    fn test_list_confirmations_empty_src() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        let result = list_pending_confirmations(tmp.path());
        assert_eq!(result.total, 0);
        assert!(result.call_sites.is_empty());
        assert!(result.note.contains("No request_confirmation calls found"));
    }

    #[test]
    fn test_list_confirmations_finds_call_sites() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();

        let handler_content = r#"
use ferro_ai::ConfirmationStore;

pub async fn delete_expense(req: Request) -> Response {
    let key = format!("expense:{}:{}", tenant_id, expense_id);
    store.request_confirmation(&key, payload, Duration::from_secs(300)).await?;
    Ok(HttpResponse::json(json!({"status": "pending_confirmation"})))
}
"#;
        fs::write(src.join("handler.rs"), handler_content).unwrap();

        let result = list_pending_confirmations(tmp.path());
        assert_eq!(result.total, 1);
        assert_eq!(result.call_sites[0].file, "src/handler.rs");
        assert_eq!(result.call_sites[0].line, 6);
        assert!(result.call_sites[0]
            .context
            .contains("request_confirmation"));
    }

    #[test]
    fn test_list_confirmations_multiple_files() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();

        fs::write(
            src.join("expenses.rs"),
            "store.request_confirmation(\"key\", payload, ttl).await?;\n",
        )
        .unwrap();
        fs::write(
            src.join("subscriptions.rs"),
            "let _ = store.request_confirmation(\"sub:cancel\", data, ttl).await;\n",
        )
        .unwrap();

        let result = list_pending_confirmations(tmp.path());
        assert_eq!(result.total, 2);

        let files: Vec<&str> = result.call_sites.iter().map(|s| s.file.as_str()).collect();
        assert!(files.contains(&"src/expenses.rs"));
        assert!(files.contains(&"src/subscriptions.rs"));
    }

    #[test]
    fn test_list_confirmations_scans_subdirectories() {
        let tmp = TempDir::new().unwrap();
        let handler_dir = tmp.path().join("src").join("handlers");
        fs::create_dir_all(&handler_dir).unwrap();

        fs::write(
            handler_dir.join("invoice.rs"),
            "store.request_confirmation(\"invoice:delete:1\", payload, ttl).await?;\n",
        )
        .unwrap();

        let result = list_pending_confirmations(tmp.path());
        assert_eq!(result.total, 1);
        assert!(result.call_sites[0].file.contains("invoice.rs"));
    }

    #[test]
    fn test_list_confirmations_serializes() {
        let result = PendingConfirmationsResult {
            call_sites: vec![ConfirmationCallSite {
                file: "src/handler.rs".to_string(),
                line: 42,
                context: "store.request_confirmation(key, payload, ttl).await?;".to_string(),
            }],
            total: 1,
            note: "See note".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("src/handler.rs"));
        assert!(json.contains("42"));
        assert!(json.contains("request_confirmation"));
    }
}
