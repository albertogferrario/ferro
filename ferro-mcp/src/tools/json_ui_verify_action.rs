//! D-09: json_ui_verify_action MCP tool.
//! Verifies a handler name is registered as a route. On miss, returns the
//! closest-by-Levenshtein candidate from named routes. Reads route names from
//! the existing route registry — there is no second source of truth (D-10
//! rejects `#[handler(name = "...")]`).

use crate::error::{McpError, Result};
use crate::tools::list_routes::{self, RouteInfo};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Input length cap — mitigates DoS from O(n*m) Levenshtein on pathological strings.
const MAX_HANDLER_INPUT_LEN: usize = 256;

#[derive(Debug, Deserialize)]
pub struct VerifyActionInput {
    /// Handler name to look up (e.g. "dashboard.show").
    pub handler: String,
    /// Optional HTTP method filter (case-insensitive). When omitted, all methods match.
    #[serde(default)]
    pub method: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct VerifyActionResult {
    /// True when a matching route was found.
    pub found: bool,
    /// The matching route when found.
    pub route: Option<RouteInfo>,
    /// Closest Levenshtein candidate name when not found; `None` when found or no named routes.
    pub candidate: Option<String>,
    /// Human-readable summary.
    pub message: String,
}

/// Public entry: reads routes from the project, then delegates to `find_handler`.
pub async fn execute(
    project_root: &Path,
    handler: &str,
    method: Option<&str>,
) -> Result<VerifyActionResult> {
    if handler.len() > MAX_HANDLER_INPUT_LEN {
        return Err(McpError::ToolError(format!(
            "handler input exceeds {MAX_HANDLER_INPUT_LEN} chars"
        )));
    }
    let routes_info = list_routes::execute(project_root).await?;
    Ok(find_handler(&routes_info.routes, handler, method))
}

/// Pure lookup helper — testable without project I/O.
pub(crate) fn find_handler(
    routes: &[RouteInfo],
    handler: &str,
    method: Option<&str>,
) -> VerifyActionResult {
    let found = routes.iter().find(|r| {
        r.name.as_deref() == Some(handler)
            && method
                .map(|m| r.method.eq_ignore_ascii_case(m))
                .unwrap_or(true)
    });

    if let Some(route) = found {
        return VerifyActionResult {
            found: true,
            route: Some(route.clone()),
            candidate: None,
            message: format!("Route '{handler}' found"),
        };
    }

    // Not found — pick closest Levenshtein candidate among named routes.
    let candidate = routes
        .iter()
        .filter_map(|r| {
            r.name
                .as_ref()
                .map(|n| (n.clone(), strsim::levenshtein(n, handler)))
        })
        .min_by_key(|(_, dist)| *dist)
        .map(|(name, _)| name);

    VerifyActionResult {
        found: false,
        route: None,
        candidate,
        message: format!("Route '{handler}' not found"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::list_routes::RouteInfo;

    fn mk_route(name: &str, method: &str) -> RouteInfo {
        RouteInfo {
            method: method.to_string(),
            path: format!("/{}", name.replace('.', "/")),
            name: Some(name.to_string()),
            handler: name.to_string(),
            middleware: vec![],
        }
    }

    #[test]
    fn verify_action_found_returns_route_info() {
        let routes = vec![mk_route("dashboard.show", "GET")];
        let result = find_handler(&routes, "dashboard.show", None);
        assert!(result.found);
        assert!(result.route.is_some());
        assert_eq!(result.candidate, None);
    }

    #[test]
    fn verify_action_found_filters_by_method() {
        let routes = vec![mk_route("dashboard.show", "GET")];
        let result = find_handler(&routes, "dashboard.show", Some("POST"));
        assert!(!result.found, "GET-only route should not match a POST query");
    }

    #[test]
    fn verify_action_not_found_returns_closest_levenshtein_candidate() {
        let routes = vec![
            mk_route("dashboard.show", "GET"),
            mk_route("account.edit", "GET"),
            mk_route("billing.show", "GET"),
        ];
        // "dashboar.show" is distance 1 from "dashboard.show"
        let result = find_handler(&routes, "dashboar.show", None);
        assert!(!result.found);
        assert_eq!(result.candidate.as_deref(), Some("dashboard.show"));
    }

    #[test]
    fn verify_action_empty_route_list_returns_no_candidate() {
        let result = find_handler(&[], "anything", None);
        assert!(!result.found);
        assert_eq!(result.candidate, None);
    }

    #[tokio::test]
    async fn verify_action_rejects_oversized_handler_input() {
        let huge = "a".repeat(MAX_HANDLER_INPUT_LEN + 1);
        // The length check fires before any I/O, so any path works.
        let result = execute(std::path::Path::new("."), &huge, None).await;
        assert!(result.is_err());
    }
}
