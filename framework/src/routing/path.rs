//! Path combination helper for route groups.
//!
//! Centralizes the (prefix, route_path) -> (canonical, alternate) rule used
//! by both the macro-based `group!` (via `macros.rs::GroupDef::register_with_inherited`)
//! and the builder-based `Router::group(...)` (via `group.rs::GroupBuilder::finalize`).
//!
//! Keeping the rule in a single function prevents the two implementations
//! from drifting apart again.

/// Resolve a group prefix combined with a nested route path into a canonical
/// registration path plus an optional alternate that differs only by a
/// trailing slash.
///
/// # Rules
///
/// - One trailing `/` is stripped from `prefix` before concatenation.
/// - If `route_path` is `"/"` and the stripped prefix is non-empty, the
///   canonical form is the stripped prefix and the alternate form is the
///   stripped prefix with a trailing `/` appended. Both must be registered
///   so `group!("/prefix", { get!("/", h) })` reaches `h` at both
///   `/prefix` and `/prefix/`.
/// - If `route_path` is `"/"` and the stripped prefix is empty, the
///   canonical form is `"/"` and no alternate is emitted (the root-in-root
///   `group!("/", { get!("/", h) })` degenerate case).
/// - Otherwise, the canonical form is `stripped_prefix + route_path` and no
///   alternate is emitted.
///
/// Returns `(canonical, alternate)`; the caller inserts both into the
/// router, bypassing `register_route` for the alternate so the
/// introspection registry stays canonical.
// Consumers arrive in Plans 02 and 03 via `use super::path::combine_group_path;`.
#[allow(dead_code)]
pub(crate) fn combine_group_path(prefix: &str, route_path: &str) -> (String, Option<String>) {
    let stripped = prefix.strip_suffix('/').unwrap_or(prefix);

    if route_path == "/" {
        if stripped.is_empty() {
            return ("/".to_string(), None);
        }
        let canonical = stripped.to_string();
        let alternate = format!("{canonical}/");
        return (canonical, Some(alternate));
    }

    if stripped.is_empty() {
        return (route_path.to_string(), None);
    }

    (format!("{stripped}{route_path}"), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine_group_path_matrix() {
        // (prefix, route_path, expected_canonical, expected_alternate)
        let cases: &[(&str, &str, &str, Option<&str>)] = &[
            ("", "/", "/", None),
            ("/", "/", "/", None),
            ("/", "/x", "/x", None),
            ("/api", "/", "/api", Some("/api/")),
            ("/api", "/x", "/api/x", None),
            ("/api/", "/x", "/api/x", None),
            ("/api/", "/", "/api", Some("/api/")),
            ("/s/{slug}", "/", "/s/{slug}", Some("/s/{slug}/")),
        ];

        for (prefix, route, want_canon, want_alt) in cases {
            let (canon, alt) = combine_group_path(prefix, route);
            assert_eq!(&canon, want_canon, "prefix={prefix:?} route={route:?}");
            assert_eq!(
                alt.as_deref(),
                *want_alt,
                "prefix={prefix:?} route={route:?}"
            );
        }
    }
}
