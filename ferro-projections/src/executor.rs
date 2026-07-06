//! Pure, serializable derivation of write plans.
//!
//! `derive_transition_plan` reads a declared [`StateMachine`](crate::StateMachine)
//! plus an [`ActionDef`](crate::ActionDef) and produces a [`TransitionPlan`] — the
//! transition facts (`from_states` → `event` → `to_state`, guard, effects) the
//! consumer runtime needs to execute a write.
//!
//! `derive_crud_plan` reads a [`ServiceDef`](crate::ServiceDef) and a [`CrudVerb`]
//! and produces a [`CrudPlan`] — the CRUD write facts (table, column set, soft-delete
//! predicate) the `framework::write` kernel needs to execute a generic INSERT/UPDATE/
//! soft-delete. Both plans are **data, not behavior**: this crate touches no database
//! and runs no I/O.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::field::FieldMeaning;
use crate::state::Transition;

/// A serializable description of a state-transition write, derived from the
/// declared `StateMachine` + `ActionDef`.
///
/// Carries no behavior — the consumer interprets it against a concrete entity.
/// `to_state` is sourced only from `Transition.to`, eliminating the duplicated
/// hand-written transition target on the write path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TransitionPlan {
    /// `ActionDef.name` that produced this plan.
    pub action: String,
    /// = `ActionDef.transition_trigger`; the `StateMachine` event name.
    pub event: String,
    /// Every `Transition.from` carrying this event (multi-source aware).
    pub from_states: Vec<String>,
    /// `Transition.to` — the single target. Replaces the hand-written `match`.
    pub to_state: String,
    /// `Transition.guard`, if any — re-checked LIVE at execution (EXEC-02).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    /// `Transition.actions` ∪ `ActionDef.effects` — override-hook inputs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<String>,
}

/// Derives the [`TransitionPlan`] for `action_name` from the service's declared
/// state machine.
///
/// Pure: no database access, no async, no closures. Returns a typed `Err` for
/// every failure mode rather than panicking.
///
/// # Errors
/// - [`Error::Validation`](crate::Error::Validation) — no action named `action_name`.
/// - [`Error::NoTransitionTrigger`](crate::Error::NoTransitionTrigger) — the action
///   declares no `transition_trigger`.
/// - [`Error::NoStateMachine`](crate::Error::NoStateMachine) — the service has no
///   state machine.
/// - [`Error::UndeclaredTransition`](crate::Error::UndeclaredTransition) — the trigger
///   names an event no transition carries (the EXEC-04 drift condition).
/// - [`Error::AmbiguousTransition`](crate::Error::AmbiguousTransition) — the event fans
///   out to more than one target state.
pub fn derive_transition_plan(
    svc: &crate::ServiceDef,
    action_name: &str,
) -> Result<TransitionPlan, crate::Error> {
    let action = svc
        .actions
        .iter()
        .find(|a| a.name == action_name)
        .ok_or_else(|| crate::Error::Validation(format!("no action '{action_name}'")))?;

    let event = action
        .transition_trigger
        .as_deref()
        .ok_or_else(|| crate::Error::NoTransitionTrigger(action_name.to_string()))?;

    let sm = svc
        .state_machine
        .as_ref()
        .ok_or_else(|| crate::Error::NoStateMachine(svc.name.clone()))?;

    let matches: Vec<&Transition> = sm.states_for_event(event);
    if matches.is_empty() {
        return Err(crate::Error::UndeclaredTransition {
            action: action_name.to_string(),
            event: event.to_string(),
        });
    }

    let to_state = matches[0].to.clone();
    if matches.iter().any(|t| t.to != to_state) {
        return Err(crate::Error::AmbiguousTransition {
            event: event.to_string(),
        });
    }

    let from_states: Vec<String> = matches.iter().map(|t| t.from.clone()).collect();

    // Guard is per-transition; all matches share one event/target. Take the
    // first transition's guard. Dedup against action.preconditions happens in
    // the consumer runtime (Plan 02), not here.
    let guard = matches[0].guard.clone();

    // effects: union of the transition's actions and the action's effects,
    // order-preserving, deduped by string equality.
    let mut effects: Vec<String> = Vec::new();
    for e in matches[0].actions.iter().chain(action.effects.iter()) {
        if !effects.iter().any(|existing| existing == e) {
            effects.push(e.clone());
        }
    }

    Ok(TransitionPlan {
        action: action_name.to_string(),
        event: event.to_string(),
        from_states,
        to_state,
        guard,
        effects,
    })
}

/// The three generic CRUD write verbs derived from a projection's opt-in flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CrudVerb {
    Create,
    Update,
    Delete,
}

/// Tenant-scope predicate slot. Phase 241 leaves this `None` on every plan;
/// Phase 242 fills it (column name only — the runtime `tenant_id` is injected by
/// the executor from the `dispatch_write` `tenant_id` parameter, never stored here).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TenantColumn {
    pub column: String,
}

/// A pure, serializable CRUD write plan — the data-only analog of [`TransitionPlan`].
///
/// Carries no behavior: `framework::write::execute_crud_plan` interprets it into
/// parameterized SQL. Values are [`serde_json::Value`] because this crate has no
/// sea-orm dependency (the schema-only boundary); the framework executor coerces
/// `serde_json::Value` to the SQL binding type at execution time.
///
/// # created_at contract
///
/// `created_at` is **not** included in `CrudPlan::Create.columns`. The framework
/// executor injects it as a server-side `datetime('now')` / `NOW()` literal.
/// This keeps the plan free of magic sentinels and matches the DB DEFAULT behavior.
///
/// # tenant_column contract (D-09)
///
/// Every variant carries `tenant_column: Option<TenantColumn>`, always `None` in
/// Phase 241. Phase 242 sets `Some(TenantColumn { column })` so the executor can
/// inject the runtime `tenant_id` without reworking the plan struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum CrudPlan {
    /// Generic INSERT: user-supplied writable fields plus server-set initial
    /// Status (when a StateMachine exists). `created_at` is server-injected by
    /// the executor (not present in `columns`).
    Create {
        table: String,
        /// Ordered (column, value) pairs: user-supplied writable fields first,
        /// then server-set Status = initial state (if StateMachine declared).
        columns: Vec<(String, serde_json::Value)>,
        /// None in Phase 241; Phase 242 sets `Some(TenantColumn { .. })`.
        #[serde(skip_serializing_if = "Option::is_none")]
        tenant_column: Option<TenantColumn>,
    },
    /// Generic UPDATE (patch semantics): only supplied writable fields change.
    /// The executor always emits `AND <soft_delete_column> IS NULL` (CRUD-03 / SC#2).
    Update {
        table: String,
        id_column: String,
        id_value: serde_json::Value,
        /// Only the supplied writable fields (patch semantics).
        patch: Vec<(String, serde_json::Value)>,
        /// Always present — executor emits `AND <col> IS NULL` so a soft-deleted
        /// row is unaddressable (CRUD-03 / SC#2).
        soft_delete_column: String,
        /// None in Phase 241; Phase 242 sets `Some(TenantColumn { .. })`.
        #[serde(skip_serializing_if = "Option::is_none")]
        tenant_column: Option<TenantColumn>,
    },
    /// Generic soft-DELETE: sets `soft_delete_column = now` (never a physical DELETE).
    Delete {
        table: String,
        id_column: String,
        id_value: serde_json::Value,
        /// Set to `datetime('now')` / `NOW()` by the executor (soft-delete).
        soft_delete_column: String,
        /// None in Phase 241; Phase 242 sets `Some(TenantColumn { .. })`.
        #[serde(skip_serializing_if = "Option::is_none")]
        tenant_column: Option<TenantColumn>,
    },
}

/// Derives a pure [`CrudPlan`] for `verb` from the service declaration and agent inputs.
///
/// Mirrors `derive_transition_plan`: pure, side-effect-free, no I/O. Reuses the
/// Phase 239/240 resolver accessors so the column sets are single-sourced with the
/// schema builders.
///
/// Phase 241 leaves `tenant_column = None` on every variant (D-09); Phase 242 fills it.
///
/// # Errors
/// - [`Error::VerbNotEnabled`](crate::Error::VerbNotEnabled) — the service has not
///   opted into the requested verb (`.creatable(false)`, etc.).
pub fn derive_crud_plan(
    svc: &crate::ServiceDef,
    verb: CrudVerb,
    inputs: &serde_json::Value,
) -> Result<CrudPlan, crate::Error> {
    let table = svc.resolved_table();
    let has_sm = svc.state_machine.is_some();

    match verb {
        CrudVerb::Create => {
            if !svc.creatable {
                return Err(crate::Error::VerbNotEnabled(format!("{}.create", svc.name)));
            }
            // Collect user-supplied writable fields (excludes Identifier, CreatedAt,
            // UpdatedAt, Sensitive, list fields, and Status when SM exists).
            let mut columns: Vec<(String, serde_json::Value)> = svc
                .fields
                .iter()
                .filter(|f| !svc.is_write_excluded_field(f, has_sm))
                .filter_map(|f| inputs.get(&f.name).map(|v| (f.name.clone(), v.clone())))
                .collect();
            // Server-set: Status = initial state when a StateMachine is declared.
            if let Some(sm) = svc.state_machine.as_ref() {
                if let Some(status_field) = svc
                    .fields
                    .iter()
                    .find(|f| matches!(f.meaning, FieldMeaning::Status))
                {
                    columns.push((
                        status_field.name.clone(),
                        serde_json::json!(sm.initial_state),
                    ));
                }
            }
            // Note: created_at is NOT added here — the executor injects it server-side.
            Ok(CrudPlan::Create {
                table,
                columns,
                tenant_column: svc.tenant_column.as_ref().map(|col| TenantColumn {
                    column: col.clone(),
                }),
            })
        }
        CrudVerb::Update => {
            if !svc.updatable {
                return Err(crate::Error::VerbNotEnabled(format!("{}.update", svc.name)));
            }
            let id_value = inputs
                .get("id")
                .filter(|v| !v.is_null())
                .cloned()
                .ok_or_else(|| crate::Error::Validation("update requires an 'id' field".into()))?;
            // Patch: only the supplied writable fields (Status excluded when SM exists).
            let patch = svc
                .fields
                .iter()
                .filter(|f| !svc.is_write_excluded_field(f, has_sm))
                .filter_map(|f| inputs.get(&f.name).map(|v| (f.name.clone(), v.clone())))
                .collect();
            Ok(CrudPlan::Update {
                table,
                id_column: "id".into(),
                id_value,
                patch,
                soft_delete_column: svc.resolved_soft_delete_column().to_string(),
                tenant_column: svc.tenant_column.as_ref().map(|col| TenantColumn {
                    column: col.clone(),
                }),
            })
        }
        CrudVerb::Delete => {
            if !svc.deletable {
                return Err(crate::Error::VerbNotEnabled(format!("{}.delete", svc.name)));
            }
            let id_value = inputs
                .get("id")
                .filter(|v| !v.is_null())
                .cloned()
                .ok_or_else(|| crate::Error::Validation("delete requires an 'id' field".into()))?;
            Ok(CrudPlan::Delete {
                table,
                id_column: "id".into(),
                id_value,
                soft_delete_column: svc.resolved_soft_delete_column().to_string(),
                tenant_column: svc.tenant_column.as_ref().map(|col| TenantColumn {
                    column: col.clone(),
                }),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::ActionDef;
    use crate::state::{StateMachine, Transition};
    use crate::ServiceDef;

    /// Synthetic order service mirroring the app's EXEC-01 reference fixture.
    /// `cancel` is multi-source (draft + submitted → cancelled) to exercise
    /// the multi-source path. Reproduced inline — no dependency on the `app` crate.
    fn order_service() -> ServiceDef {
        let machine = StateMachine::new("order_lifecycle")
            .initial("draft")
            .transition(Transition::new("draft", "submit", "submitted").actions(vec!["log_submit"]))
            .transition(Transition::new("submitted", "approve", "approved").guard("is_manager"))
            .transition(Transition::new("submitted", "reject", "cancelled"))
            .transition(Transition::new("approved", "ship", "shipped"))
            .transition(Transition::new("shipped", "deliver", "delivered"))
            .transition(Transition::new("draft", "cancel", "cancelled"))
            .transition(Transition::new("submitted", "cancel", "cancelled"));

        ServiceDef::new("order")
            .state_machine(machine)
            .action(
                ActionDef::new("submit")
                    .transition_trigger("submit")
                    .effect("notify"),
            )
            .action(
                ActionDef::new("approve")
                    .transition_trigger("approve")
                    .precondition("is_manager"),
            )
            .action(ActionDef::new("ship").transition_trigger("ship"))
            .action(ActionDef::new("cancel").transition_trigger("cancel"))
    }

    #[test]
    fn derive_transition_plan() {
        let svc = order_service();
        let plan = super::derive_transition_plan(&svc, "submit").unwrap();
        assert_eq!(plan.action, "submit");
        assert_eq!(plan.event, "submit");
        assert_eq!(plan.to_state, "submitted");
        assert_eq!(plan.from_states, vec!["draft".to_string()]);
        assert_eq!(plan.guard, None);
        // transition.actions ∪ action.effects, order-preserving, deduped.
        assert_eq!(
            plan.effects,
            vec!["log_submit".to_string(), "notify".to_string()]
        );
    }

    #[test]
    fn derive_approve_carries_transition_guard() {
        let svc = order_service();
        let plan = super::derive_transition_plan(&svc, "approve").unwrap();
        assert_eq!(plan.to_state, "approved");
        assert_eq!(plan.guard, Some("is_manager".to_string()));
    }

    #[test]
    fn derive_multi_source_event() {
        let svc = order_service();
        let plan = super::derive_transition_plan(&svc, "cancel").unwrap();
        assert_eq!(plan.to_state, "cancelled");
        assert!(plan.from_states.contains(&"draft".to_string()));
        assert!(plan.from_states.contains(&"submitted".to_string()));
        assert_eq!(plan.from_states.len(), 2);
    }

    #[test]
    fn derive_no_trigger() {
        let svc = ServiceDef::new("order")
            .state_machine(StateMachine::new("sm").initial("draft"))
            .action(ActionDef::new("noop"));
        let err = super::derive_transition_plan(&svc, "noop").unwrap_err();
        assert!(matches!(err, crate::Error::NoTransitionTrigger(a) if a == "noop"));
    }

    #[test]
    fn derive_undeclared_trigger_errors() {
        let svc = ServiceDef::new("order")
            .state_machine(
                StateMachine::new("sm")
                    .initial("draft")
                    .transition(Transition::new("draft", "submit", "submitted")),
            )
            .action(ActionDef::new("bogus").transition_trigger("does_not_exist"));
        let err = super::derive_transition_plan(&svc, "bogus").unwrap_err();
        assert!(matches!(
            err,
            crate::Error::UndeclaredTransition { ref action, ref event }
                if action == "bogus" && event == "does_not_exist"
        ));
    }

    #[test]
    fn derive_no_state_machine() {
        let svc =
            ServiceDef::new("order").action(ActionDef::new("submit").transition_trigger("submit"));
        let err = super::derive_transition_plan(&svc, "submit").unwrap_err();
        assert!(matches!(err, crate::Error::NoStateMachine(name) if name == "order"));
    }

    #[test]
    fn derive_no_action() {
        let svc = order_service();
        let err = super::derive_transition_plan(&svc, "ghost").unwrap_err();
        assert!(matches!(err, crate::Error::Validation(_)));
    }

    #[test]
    fn derive_ambiguous_fan_out() {
        // Same event `split` fans out to two different targets per source.
        let svc = ServiceDef::new("order")
            .state_machine(
                StateMachine::new("sm")
                    .initial("a")
                    .transition(Transition::new("a", "split", "b"))
                    .transition(Transition::new("c", "split", "d")),
            )
            .action(ActionDef::new("split").transition_trigger("split"));
        let err = super::derive_transition_plan(&svc, "split").unwrap_err();
        assert!(matches!(err, crate::Error::AmbiguousTransition { event } if event == "split"));
    }

    #[test]
    fn transition_plan_serde_round_trip() {
        let svc = order_service();
        let plan = super::derive_transition_plan(&svc, "submit").unwrap();
        let json = serde_json::to_string(&plan).unwrap();
        let back: TransitionPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, back);
    }

    // ── CRUD plan derivation tests ──────────────────────────────────────────

    use crate::field::{DataType, FieldMeaning};

    /// CRUD-capable order service with an SM (status excluded from create/update)
    /// and two writable data fields: `amount` (Money) and `note` (FreeText).
    fn crud_order_service() -> ServiceDef {
        let machine = StateMachine::new("order_lifecycle")
            .initial("draft")
            .transition(Transition::new("draft", "submit", "submitted"));

        ServiceDef::new("order")
            .mcp_write_ability("manage-orders")
            .creatable(true)
            .updatable(true)
            .deletable(true)
            .state_machine(machine)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("amount", DataType::String, FieldMeaning::Money)
            .field("note", DataType::String, FieldMeaning::FreeText)
            .field("created_at", DataType::String, FieldMeaning::CreatedAt)
    }

    /// Like crud_order_service but without a StateMachine (status is writable).
    fn crud_order_service_no_sm() -> ServiceDef {
        ServiceDef::new("order")
            .mcp_write_ability("manage-orders")
            .creatable(true)
            .updatable(true)
            .deletable(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("amount", DataType::String, FieldMeaning::Money)
            .field("note", DataType::String, FieldMeaning::FreeText)
            .field("created_at", DataType::String, FieldMeaning::CreatedAt)
    }

    #[test]
    fn derive_crud_plan_create() {
        let svc = crud_order_service();
        let inputs = serde_json::json!({ "amount": "100.00", "note": "rush order" });
        let plan = super::derive_crud_plan(&svc, CrudVerb::Create, &inputs).unwrap();

        let CrudPlan::Create {
            ref table,
            ref columns,
            ref tenant_column,
        } = plan
        else {
            panic!("expected CrudPlan::Create, got {plan:?}");
        };

        assert_eq!(table, "orders"); // resolved_table(): "order" + "s"

        // User-supplied writable fields present
        assert!(
            columns
                .iter()
                .any(|(col, val)| col == "amount" && val == &serde_json::json!("100.00")),
            "expected amount in columns"
        );
        assert!(
            columns
                .iter()
                .any(|(col, val)| col == "note" && val == &serde_json::json!("rush order")),
            "expected note in columns"
        );

        // Server-set initial status (SM declared → status = "draft")
        assert!(
            columns
                .iter()
                .any(|(col, val)| col == "status" && val == &serde_json::json!("draft")),
            "expected status=draft in columns when SM exists"
        );

        // Server-injected fields NOT present (id, created_at excluded)
        assert!(
            !columns.iter().any(|(col, _)| col == "id"),
            "id must not be in columns"
        );
        assert!(
            !columns.iter().any(|(col, _)| col == "created_at"),
            "created_at must not be in columns (executor injects it)"
        );

        // D-09: tenant_column is None
        assert_eq!(tenant_column, &None);
    }

    #[test]
    fn derive_crud_plan_create_no_sm_status_included() {
        // Without a StateMachine, Status IS a writable input field.
        let svc = crud_order_service_no_sm();
        let inputs = serde_json::json!({ "amount": "50.00", "status": "active", "note": "test" });
        let plan = super::derive_crud_plan(&svc, CrudVerb::Create, &inputs).unwrap();

        let CrudPlan::Create { ref columns, .. } = plan else {
            panic!("expected CrudPlan::Create");
        };

        // Status present because no SM (exclude_sm_status = false)
        assert!(
            columns
                .iter()
                .any(|(col, val)| col == "status" && val == &serde_json::json!("active")),
            "status must be in columns when no SM"
        );
    }

    #[test]
    fn derive_crud_plan_update() {
        let svc = crud_order_service();
        let inputs = serde_json::json!({ "id": 42, "amount": "200.00" });
        let plan = super::derive_crud_plan(&svc, CrudVerb::Update, &inputs).unwrap();

        let CrudPlan::Update {
            ref table,
            ref id_column,
            ref id_value,
            ref patch,
            ref soft_delete_column,
            ref tenant_column,
        } = plan
        else {
            panic!("expected CrudPlan::Update, got {plan:?}");
        };

        assert_eq!(table, "orders");
        assert_eq!(id_column, "id");
        assert_eq!(id_value, &serde_json::json!(42));

        // Only supplied field in patch
        assert!(
            patch
                .iter()
                .any(|(col, val)| col == "amount" && val == &serde_json::json!("200.00")),
            "expected amount in patch"
        );
        // Unsupplied field not in patch (patch semantics)
        assert!(
            !patch.iter().any(|(col, _)| col == "note"),
            "note must not be in patch (not supplied)"
        );
        // Status excluded because SM exists
        assert!(
            !patch.iter().any(|(col, _)| col == "status"),
            "status must not be in patch when SM exists"
        );
        // id not in patch (server-injected / Identifier)
        assert!(
            !patch.iter().any(|(col, _)| col == "id"),
            "id must not be in patch"
        );

        // soft_delete_column from resolved_soft_delete_column() = "deleted_at"
        assert_eq!(soft_delete_column, "deleted_at");

        // D-09: tenant_column is None
        assert_eq!(tenant_column, &None);
    }

    #[test]
    fn derive_crud_plan_delete() {
        let svc = crud_order_service();
        let inputs = serde_json::json!({ "id": 7 });
        let plan = super::derive_crud_plan(&svc, CrudVerb::Delete, &inputs).unwrap();

        let CrudPlan::Delete {
            ref table,
            ref id_column,
            ref id_value,
            ref soft_delete_column,
            ref tenant_column,
        } = plan
        else {
            panic!("expected CrudPlan::Delete, got {plan:?}");
        };

        assert_eq!(table, "orders");
        assert_eq!(id_column, "id");
        assert_eq!(id_value, &serde_json::json!(7));
        assert_eq!(soft_delete_column, "deleted_at");
        assert_eq!(tenant_column, &None);
    }

    #[test]
    fn derive_crud_plan_verb_not_enabled() {
        // Service with creatable/updatable/deletable all false (defaults)
        let svc = ServiceDef::new("product")
            .mcp_write_ability("manage-products")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName);

        let inputs = serde_json::json!({ "name": "Widget" });

        let err_create = super::derive_crud_plan(&svc, CrudVerb::Create, &inputs).unwrap_err();
        assert!(
            matches!(err_create, crate::Error::VerbNotEnabled(ref s) if s.contains("product.create")),
            "expected VerbNotEnabled for create, got {err_create:?}"
        );

        let err_update = super::derive_crud_plan(&svc, CrudVerb::Update, &inputs).unwrap_err();
        assert!(
            matches!(err_update, crate::Error::VerbNotEnabled(ref s) if s.contains("product.update")),
            "expected VerbNotEnabled for update, got {err_update:?}"
        );

        let err_delete = super::derive_crud_plan(&svc, CrudVerb::Delete, &inputs).unwrap_err();
        assert!(
            matches!(err_delete, crate::Error::VerbNotEnabled(ref s) if s.contains("product.delete")),
            "expected VerbNotEnabled for delete, got {err_delete:?}"
        );
    }

    #[test]
    fn crud_plan_serde_round_trip() {
        let svc = crud_order_service();
        let inputs = serde_json::json!({ "amount": "50.00" });

        // Create round-trip
        let create_plan = super::derive_crud_plan(&svc, CrudVerb::Create, &inputs).unwrap();
        let json = serde_json::to_string(&create_plan).unwrap();
        let back: CrudPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(create_plan, back);

        // Update round-trip
        let update_inputs = serde_json::json!({ "id": 1, "amount": "75.00" });
        let update_plan = super::derive_crud_plan(&svc, CrudVerb::Update, &update_inputs).unwrap();
        let json = serde_json::to_string(&update_plan).unwrap();
        let back: CrudPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(update_plan, back);

        // Delete round-trip
        let delete_inputs = serde_json::json!({ "id": 3 });
        let delete_plan = super::derive_crud_plan(&svc, CrudVerb::Delete, &delete_inputs).unwrap();
        let json = serde_json::to_string(&delete_plan).unwrap();
        let back: CrudPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(delete_plan, back);
    }

    // WR-02: missing or null id is rejected at derivation time, not deferred to SQL.
    #[test]
    fn derive_crud_plan_update_missing_id_is_validation_error() {
        let svc = crud_order_service();
        // id absent
        let err = super::derive_crud_plan(
            &svc,
            CrudVerb::Update,
            &serde_json::json!({ "amount": "50.00" }),
        )
        .unwrap_err();
        assert!(
            matches!(err, crate::Error::Validation(_)),
            "expected Validation error for missing id on update, got {err:?}"
        );
        // id explicitly null
        let err_null = super::derive_crud_plan(
            &svc,
            CrudVerb::Update,
            &serde_json::json!({ "id": null, "amount": "50.00" }),
        )
        .unwrap_err();
        assert!(
            matches!(err_null, crate::Error::Validation(_)),
            "expected Validation error for null id on update, got {err_null:?}"
        );
    }

    #[test]
    fn derive_crud_plan_delete_missing_id_is_validation_error() {
        let svc = crud_order_service();
        // id absent
        let err =
            super::derive_crud_plan(&svc, CrudVerb::Delete, &serde_json::json!({})).unwrap_err();
        assert!(
            matches!(err, crate::Error::Validation(_)),
            "expected Validation error for missing id on delete, got {err:?}"
        );
        // id explicitly null
        let err_null =
            super::derive_crud_plan(&svc, CrudVerb::Delete, &serde_json::json!({ "id": null }))
                .unwrap_err();
        assert!(
            matches!(err_null, crate::Error::Validation(_)),
            "expected Validation error for null id on delete, got {err_null:?}"
        );
    }

    // ── Task 1: tenant_column derivation tests (RED → GREEN in Task 1 impl) ──

    /// Build a CRUD-capable service WITH a tenant column declared.
    fn crud_order_service_with_tenant() -> ServiceDef {
        let machine = StateMachine::new("order_lifecycle")
            .initial("draft")
            .transition(Transition::new("draft", "submit", "submitted"));

        ServiceDef::new("order")
            .mcp_write_ability("manage-orders")
            .creatable(true)
            .updatable(true)
            .deletable(true)
            .tenant_column("tenant_id")
            .state_machine(machine)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("amount", DataType::String, FieldMeaning::Money)
            .field("note", DataType::String, FieldMeaning::FreeText)
            .field("created_at", DataType::String, FieldMeaning::CreatedAt)
    }

    #[test]
    fn derive_crud_plan_create_tenant_column() {
        // WITH tenant column → Some(TenantColumn { column: "tenant_id" })
        let svc = crud_order_service_with_tenant();
        let inputs = serde_json::json!({ "amount": "100.00" });
        let plan = super::derive_crud_plan(&svc, CrudVerb::Create, &inputs).unwrap();

        let CrudPlan::Create {
            ref tenant_column, ..
        } = plan
        else {
            panic!("expected CrudPlan::Create, got {plan:?}");
        };
        assert_eq!(
            tenant_column,
            &Some(TenantColumn {
                column: "tenant_id".into()
            }),
            "create with tenant_column set must yield Some(TenantColumn)"
        );

        // WITHOUT tenant column → None (non-tenant projection stays unscoped)
        let svc_no_tenant = crud_order_service();
        let plan_no = super::derive_crud_plan(&svc_no_tenant, CrudVerb::Create, &inputs).unwrap();
        let CrudPlan::Create {
            ref tenant_column, ..
        } = plan_no
        else {
            panic!("expected CrudPlan::Create");
        };
        assert_eq!(
            tenant_column, &None,
            "create without tenant_column must yield None"
        );
    }

    #[test]
    fn derive_crud_plan_update_tenant_column() {
        // WITH tenant column → Some(TenantColumn { column: "tenant_id" })
        let svc = crud_order_service_with_tenant();
        let inputs = serde_json::json!({ "id": 1, "amount": "200.00" });
        let plan = super::derive_crud_plan(&svc, CrudVerb::Update, &inputs).unwrap();

        let CrudPlan::Update {
            ref tenant_column, ..
        } = plan
        else {
            panic!("expected CrudPlan::Update, got {plan:?}");
        };
        assert_eq!(
            tenant_column,
            &Some(TenantColumn {
                column: "tenant_id".into()
            }),
            "update with tenant_column set must yield Some(TenantColumn)"
        );

        // WITHOUT tenant column → None
        let svc_no_tenant = crud_order_service();
        let plan_no = super::derive_crud_plan(&svc_no_tenant, CrudVerb::Update, &inputs).unwrap();
        let CrudPlan::Update {
            ref tenant_column, ..
        } = plan_no
        else {
            panic!("expected CrudPlan::Update");
        };
        assert_eq!(
            tenant_column, &None,
            "update without tenant_column must yield None"
        );
    }

    #[test]
    fn derive_crud_plan_delete_tenant_column() {
        // WITH tenant column → Some(TenantColumn { column: "tenant_id" })
        let svc = crud_order_service_with_tenant();
        let inputs = serde_json::json!({ "id": 7 });
        let plan = super::derive_crud_plan(&svc, CrudVerb::Delete, &inputs).unwrap();

        let CrudPlan::Delete {
            ref tenant_column, ..
        } = plan
        else {
            panic!("expected CrudPlan::Delete, got {plan:?}");
        };
        assert_eq!(
            tenant_column,
            &Some(TenantColumn {
                column: "tenant_id".into()
            }),
            "delete with tenant_column set must yield Some(TenantColumn)"
        );

        // WITHOUT tenant column → None
        let svc_no_tenant = crud_order_service();
        let plan_no = super::derive_crud_plan(&svc_no_tenant, CrudVerb::Delete, &inputs).unwrap();
        let CrudPlan::Delete {
            ref tenant_column, ..
        } = plan_no
        else {
            panic!("expected CrudPlan::Delete");
        };
        assert_eq!(
            tenant_column, &None,
            "delete without tenant_column must yield None"
        );
    }
}
