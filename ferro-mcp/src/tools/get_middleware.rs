//! Get middleware source tool - extract middleware logic

use crate::error::{McpError, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct MiddlewareSource {
    pub name: String,
    pub file_path: String,
    pub source_code: String,
    pub handle_method: Option<String>,
    pub dependencies: Vec<String>,
}

pub fn execute(project_root: &Path, middleware_name: &str) -> Result<MiddlewareSource> {
    let middleware_dir = project_root.join("src/middleware");

    if !middleware_dir.exists() {
        return Err(McpError::FileNotFound("src/middleware".to_string()));
    }

    // Search for the middleware file
    for entry in WalkDir::new(&middleware_dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(path) {
            // Check if this file contains the middleware we're looking for
            let normalized_name = normalize_middleware_name(middleware_name);

            if contains_middleware(&content, &normalized_name) {
                let handle_method = extract_handle_method(&content, &normalized_name);
                let dependencies = extract_dependencies(&content);

                let relative_path = path
                    .strip_prefix(project_root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();

                return Ok(MiddlewareSource {
                    name: middleware_name.to_string(),
                    file_path: relative_path,
                    source_code: content,
                    handle_method,
                    dependencies,
                });
            }
        }
    }

    // Also check framework's built-in middleware
    let framework_middleware = check_framework_middleware(middleware_name);
    if let Some(info) = framework_middleware {
        return Ok(info);
    }

    Err(McpError::FileNotFound(format!(
        "Middleware '{middleware_name}' not found"
    )))
}

fn normalize_middleware_name(name: &str) -> String {
    // Convert various formats to struct name format
    // auth -> Auth, AuthMiddleware
    // rate_limit -> RateLimit, RateLimitMiddleware

    let base = name
        .trim_end_matches("Middleware")
        .trim_end_matches("middleware");

    // Convert snake_case to PascalCase
    base.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

fn contains_middleware(content: &str, normalized_name: &str) -> bool {
    // Check for struct definition
    let struct_patterns = vec![
        format!("struct {}", normalized_name),
        format!("struct {}Middleware", normalized_name),
        format!("pub struct {}", normalized_name),
        format!("pub struct {}Middleware", normalized_name),
    ];

    for pattern in struct_patterns {
        if content.contains(&pattern) {
            return true;
        }
    }

    false
}

fn extract_handle_method(content: &str, _normalized_name: &str) -> Option<String> {
    // Find the handle method in Middleware impl
    let lines: Vec<&str> = content.lines().collect();
    let mut in_impl = false;
    let mut in_handle = false;
    let mut brace_count = 0;
    let mut handle_lines = Vec::new();

    for line in lines {
        // Check for Middleware impl block
        if line.contains("impl") && line.contains("Middleware") && line.contains("for") {
            in_impl = true;
            continue;
        }

        if in_impl {
            // Check for handle method start
            if line.contains("fn handle") || line.contains("async fn handle") {
                in_handle = true;
            }

            if in_handle {
                handle_lines.push(line);
                brace_count += line.matches('{').count();
                brace_count -= line.matches('}').count();

                if brace_count == 0 && !handle_lines.is_empty() && handle_lines.len() > 1 {
                    return Some(handle_lines.join("\n"));
                }
            }

            // End of impl block
            if !in_handle && line.trim() == "}" {
                in_impl = false;
            }
        }
    }

    None
}

fn extract_dependencies(content: &str) -> Vec<String> {
    let mut deps = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("use ") {
            // Extract the dependency path
            let dep = trimmed
                .trim_start_matches("use ")
                .trim_end_matches(';')
                .to_string();

            // Filter out standard library and common imports
            if !dep.starts_with("std::")
                && !dep.starts_with("super::")
                && !dep.starts_with("crate::")
            {
                deps.push(dep);
            }
        }
    }

    deps
}

fn check_framework_middleware(name: &str) -> Option<MiddlewareSource> {
    // Provide information about built-in framework middleware
    let builtin = match name.to_lowercase().as_str() {
        "auth" | "authmiddleware" => Some((
            "AuthMiddleware",
            "Built-in authentication middleware that checks for authenticated user session",
            r#"
impl Middleware for AuthMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        if let Some(user) = req.auth().user() {
            next.run(req).await
        } else {
            Redirect::to("/login").into()
        }
    }
}
"#,
        )),
        "guest" | "guestmiddleware" => Some((
            "GuestMiddleware",
            "Built-in middleware that redirects authenticated users away",
            r#"
impl Middleware for GuestMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        if req.auth().user().is_none() {
            next.run(req).await
        } else {
            Redirect::to("/").into()
        }
    }
}
"#,
        )),
        "csrf" | "csrfmiddleware" => Some((
            "CsrfMiddleware",
            "Built-in CSRF protection middleware",
            r#"
impl Middleware for CsrfMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        // Validates CSRF token for non-GET requests
        // Token must be in X-CSRF-TOKEN header or _token field
        if req.method().is_safe() || validate_csrf_token(&req) {
            next.run(req).await
        } else {
            HttpResponse::forbidden("CSRF token mismatch")
        }
    }
}
"#,
        )),
        "throttle" | "ratelimit" | "ratelimitmiddleware" => Some((
            "Throttle",
            "Built-in rate limiting middleware",
            r#"
impl Middleware for Throttle {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let key = self.resolve_request_signature(&req);
        if self.limiter.too_many_attempts(&key, self.max_attempts) {
            HttpResponse::too_many_requests()
                .header("Retry-After", self.limiter.available_in(&key))
        } else {
            self.limiter.hit(&key, self.decay_seconds);
            next.run(req).await
        }
    }
}
"#,
        )),
        _ => None,
    };

    builtin.map(|(name, desc, code)| MiddlewareSource {
        name: name.to_string(),
        file_path: "framework/src/middleware.rs (built-in)".to_string(),
        source_code: format!("// {desc}\n{code}"),
        handle_method: Some(code.to_string()),
        dependencies: vec!["ferro::Middleware".to_string()],
    })
}
