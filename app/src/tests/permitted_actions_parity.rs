//! Permitted-actions parity test (SUBST-02 / SUBST-05).
//!
//! Asserts that `framework::permitted_actions(service, &guards)` returns the
//! same set as the guard-filtered MCP tool list from `render_exposed_tools`.
//!
//! Two invariants are tested:
//!
//! 1. `permitted_actions_matches_mcp_tools_list` — with `is_manager = false`,
//!    `approve` is hidden on BOTH the Inertia surface (permitted_actions) and
//!    the MCP surface (render_exposed_tools); `submit` is visible on both.
//!    The action-name sets are equal.
//!
//! 2. `state_change_updates_both_surfaces_identically` — flipping the guard
//!    from `false` to absent (allow) changes BOTH surfaces identically.
//!
//! Gated `not(feature = "confirmation")`: the order service has transition actions;
//! the confirmation feature modifies the tool list (adds request_confirm_/confirm_
//! tools), which would widen the MCP set without a corresponding Inertia concept.
//! The structural parity claim (same action names on both surfaces) is identical
//! either way — the test gates on the simpler non-confirmation surface.

#[cfg(all(test, not(feature = "confirmation")))]
mod tests {
    use std::collections::HashMap;

    use ferro::permitted_actions;
    use ferro_mcp_server::{render_exposed_tools, McpContext};
    use ferro_projections::{ActionDef, DataType, FieldMeaning, GuardDef, ServiceDef};

    /// A minimal ServiceDef with one guarded action (`approve`) and one open action (`submit`).
    ///
    /// `mcp_exposed(true)` is required for `render_exposed_tools` to include the service.
    fn order_service_with_guards() -> ServiceDef {
        ServiceDef::new("order")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .guard(GuardDef::new("is_manager"))
            .action(ActionDef::new("approve").precondition("is_manager"))
            .action(ActionDef::new("submit"))
    }

    /// Collect write-tool names from `render_exposed_tools`, excluding the `list_` read tool.
    ///
    /// Returns a sorted `Vec<String>` so callers can compare as sets without depending on
    /// emission order differences between the two surfaces.
    fn mcp_action_names(service: &ServiceDef, guards: HashMap<String, bool>) -> Vec<String> {
        let ctx = McpContext {
            evaluated_guards: guards,
            ..Default::default()
        };
        let tools = render_exposed_tools(std::slice::from_ref(service), &ctx)
            .expect("render_exposed_tools ok");
        let mut names: Vec<String> = tools
            .into_iter()
            .filter(|t| !t.name.starts_with("list_"))
            .map(|t| t.name.to_string())
            .collect();
        names.sort();
        names
    }

    /// Returns `permitted_actions` as a sorted Vec so it can be compared as a set.
    fn inertia_action_names(service: &ServiceDef, guards: &HashMap<String, bool>) -> Vec<String> {
        let mut names = permitted_actions(service, guards);
        names.sort();
        names
    }

    /// SUBST-02: the Inertia permitted_actions set equals the guard-filtered MCP
    /// tools/list action set for the same service and evaluated_guards.
    ///
    /// With `is_manager = false`: `approve` must be hidden on BOTH surfaces;
    /// `submit` must be visible on BOTH surfaces. The sorted sets must be identical.
    #[test]
    fn permitted_actions_matches_mcp_tools_list() {
        let service = order_service_with_guards();
        let guards: HashMap<String, bool> = [("is_manager".to_string(), false)].into();

        // Inertia path
        let inertia_allowed = inertia_action_names(&service, &guards);
        assert!(
            !inertia_allowed.contains(&"approve".to_string()),
            "inertia: approve must be hidden when is_manager=false; got: {inertia_allowed:?}"
        );
        assert!(
            inertia_allowed.contains(&"submit".to_string()),
            "inertia: submit must be visible regardless of is_manager; got: {inertia_allowed:?}"
        );

        // MCP path
        let mcp_allowed = mcp_action_names(&service, guards);
        assert!(
            !mcp_allowed.contains(&"approve".to_string()),
            "mcp: approve must be hidden when is_manager=false; got: {mcp_allowed:?}"
        );
        assert!(
            mcp_allowed.contains(&"submit".to_string()),
            "mcp: submit must be visible regardless of is_manager; got: {mcp_allowed:?}"
        );

        // The two action-name SETS must be equal (membership only, order irrelevant).
        assert_eq!(
            inertia_allowed, mcp_allowed,
            "permitted_actions and render_exposed_tools action sets must be equal; \
             inertia={inertia_allowed:?} mcp={mcp_allowed:?}"
        );
    }

    /// SUBST-05(a): flipping the guard changes BOTH surfaces identically.
    ///
    /// Guard = false  → `approve` hidden on BOTH Inertia and MCP surfaces.
    /// Guard = absent → `approve` visible on BOTH Inertia and MCP surfaces.
    /// The sets flip in lockstep — no surface leads or lags the other.
    #[test]
    fn state_change_updates_both_surfaces_identically() {
        let service = order_service_with_guards();

        // Guard explicit false: approve must be hidden on both.
        let guards_deny: HashMap<String, bool> = [("is_manager".to_string(), false)].into();
        let inertia_deny = inertia_action_names(&service, &guards_deny);
        let mcp_deny = mcp_action_names(&service, guards_deny);

        assert!(
            !inertia_deny.contains(&"approve".to_string()),
            "inertia deny: approve must be hidden; got: {inertia_deny:?}"
        );
        assert!(
            !mcp_deny.contains(&"approve".to_string()),
            "mcp deny: approve must be hidden; got: {mcp_deny:?}"
        );
        // Both surfaces must agree on the deny set.
        assert_eq!(
            inertia_deny, mcp_deny,
            "deny: both surfaces must agree; inertia={inertia_deny:?} mcp={mcp_deny:?}"
        );

        // Guard absent (default-open): approve must be visible on both.
        let guards_allow: HashMap<String, bool> = HashMap::new();
        let inertia_allow = inertia_action_names(&service, &guards_allow);
        let mcp_allow = mcp_action_names(&service, guards_allow);

        assert!(
            inertia_allow.contains(&"approve".to_string()),
            "inertia allow: approve must be visible when guard absent; got: {inertia_allow:?}"
        );
        assert!(
            mcp_allow.contains(&"approve".to_string()),
            "mcp allow: approve must be visible when guard absent; got: {mcp_allow:?}"
        );
        // Both surfaces must agree on the allow set.
        assert_eq!(
            inertia_allow, mcp_allow,
            "allow: both surfaces must agree; inertia={inertia_allow:?} mcp={mcp_allow:?}"
        );

        // The flip is symmetric: deny set must not contain approve, allow set must.
        assert_ne!(
            inertia_deny, inertia_allow,
            "inertia: sets must differ when guard flips"
        );
        assert_ne!(
            mcp_deny, mcp_allow,
            "mcp: sets must differ when guard flips"
        );
    }
}
