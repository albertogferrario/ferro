//! Renderer for `.github/workflows/ci.yml` (D-13..D-17, D-21).
//!
//! Loads the canonical GitHub Actions template via `include_str!` and
//! returns it verbatim. The context struct exists to make future
//! placeholder substitution (e.g. package name) trivial without an API
//! break.

pub const CI_WORKFLOW_TEMPLATE: &str = include_str!("files/ci/github-actions-ci.yml.tpl");

/// Context for rendering the CI workflow file.
///
/// Currently a passthrough — the template has no placeholders. Kept as
/// a struct so future fields (e.g. matrix toolchains, extra steps) can
/// be added without changing the function signature.
pub struct CiWorkflowContext<'a> {
    pub package_name: &'a str,
}

/// Render `.github/workflows/ci.yml` from a context.
///
/// Pure function — no IO, deterministic output (D-17 idempotency).
pub fn render_ci_workflow(_ctx: &CiWorkflowContext<'_>) -> String {
    CI_WORKFLOW_TEMPLATE.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render() -> String {
        render_ci_workflow(&CiWorkflowContext {
            package_name: "sample",
        })
    }

    #[test]
    fn contains_all_five_lint_gate_steps() {
        let out = render();
        assert!(out.contains("cargo fmt --all -- --check"));
        assert!(out.contains("cargo clippy --all-targets -- -D warnings"));
        assert!(out.contains("cargo test --all-features"));
        assert!(out.contains("api:check"));
        assert!(out.contains("validate:contracts"));
    }

    #[test]
    fn triggers_on_pull_request_and_push_to_main() {
        let out = render();
        assert!(out.contains("pull_request:"));
        assert!(out.contains("push:"));
        assert!(out.contains("branches: [main]"));
    }

    #[test]
    fn uses_swatinem_rust_cache_v2() {
        let out = render();
        assert!(out.contains("Swatinem/rust-cache@v2"));
    }

    #[test]
    fn uses_dtolnay_rust_toolchain_stable() {
        let out = render();
        assert!(out.contains("dtolnay/rust-toolchain@stable"));
    }

    #[test]
    fn structural_yaml_anchors_present() {
        // No yaml parser is in the workspace; verify top-level keys and
        // indentation anchors instead. (`serde_yaml` intentionally not added.)
        let out = render();
        assert!(out.contains("\nname: CI"));
        assert!(out.contains("\non:"));
        assert!(out.contains("\njobs:"));
        assert!(out.contains("\n  lint-and-test:"));
        assert!(out.contains("\n    runs-on: ubuntu-latest"));
        assert!(out.contains("\n    steps:"));
    }

    #[test]
    fn rendering_is_deterministic() {
        let a = render();
        let b = render();
        assert_eq!(a, b);
    }
}
