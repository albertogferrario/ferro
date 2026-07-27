use std::collections::HashMap;

use ferro_projections::ServiceDef;

/// Returns the names of actions in `service` whose preconditions are not
/// explicitly denied by `evaluated_guards`.
///
/// Semantics: absent key = allow (default-open); `Some(false)` = deny.
///
/// This is a list-time VISIBILITY filter evaluated once per request from a
/// pre-computed guard map. It is NOT the live guard evaluator. Per-record
/// enforcement happens at `dispatch_write` time via the live `GuardEvaluatorFn`;
/// this function only controls what a surface (MCP tools/list or the Inertia
/// substrate) SHOWS. It must never be used as an authorization gate.
///
/// Single guard-visibility evaluation site: both `ferro-mcp-server`'s
/// `tools/list` and the Inertia delivery helper call this function.
pub fn permitted_actions(
    service: &ServiceDef,
    evaluated_guards: &HashMap<String, bool>,
) -> Vec<String> {
    service
        .actions
        .iter()
        .filter(|action| {
            !action
                .preconditions
                .iter()
                .any(|p| evaluated_guards.get(p) == Some(&false))
        })
        .map(|a| a.name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ferro_projections::{ActionDef, DataType, FieldMeaning, GuardDef, ServiceDef};

    use super::*;

    #[test]
    fn hides_action_when_guard_is_false() {
        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .guard(GuardDef::new("is_manager"))
            .action(ActionDef::new("approve").precondition("is_manager"))
            .action(ActionDef::new("submit"));
        let guards = [("is_manager".to_string(), false)].into_iter().collect();
        let allowed = permitted_actions(&service, &guards);
        assert!(!allowed.contains(&"approve".to_string()));
        assert!(allowed.contains(&"submit".to_string()));
    }

    #[test]
    fn absent_guard_key_allows_action() {
        let service = ServiceDef::new("order")
            .guard(GuardDef::new("is_manager"))
            .action(ActionDef::new("approve").precondition("is_manager"));
        let guards = HashMap::new(); // absent = allow
        let allowed = permitted_actions(&service, &guards);
        assert!(allowed.contains(&"approve".to_string()));
    }

    #[test]
    fn explicit_true_allows_action() {
        let service = ServiceDef::new("order")
            .guard(GuardDef::new("is_manager"))
            .action(ActionDef::new("approve").precondition("is_manager"));
        let guards = [("is_manager".to_string(), true)].into_iter().collect();
        let allowed = permitted_actions(&service, &guards);
        assert!(allowed.contains(&"approve".to_string()));
    }
}
