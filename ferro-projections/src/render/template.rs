//! Template renderer producing structured JSON context from service definitions.
//!
//! Implements the `Renderer` trait to translate `ServiceDef` into a nested
//! JSON object with semantic groups: `fields`, `actions`, and `state_machine`.
//! Consumed by template engines (e.g., MiniJinja) in downstream projects.

use serde_json::{json, Map, Value};

use crate::error::Error;
use crate::intent::IntentScore;
use crate::service::ServiceDef;

use super::{is_system_field, RenderContext, Renderer};

/// Template context renderer producing structured JSON from service definitions.
///
/// Translates a `ServiceDef` into a flat, semantic JSON object that template
/// engines can consume directly. Intent scores and render context are ignored —
/// the output is intent-agnostic per design (D-01 through D-08).
///
/// # Output shape
///
/// ```json
/// {
///   "service": "Order",
///   "fields": {
///     "total": { "name": "total", "data_type": "float", "meaning": "money", "required": true }
///   },
///   "actions": [
///     { "name": "submit", "display_name": "Submit", "inputs": [] }
///   ],
///   "state_machine": {
///     "initial_state": "draft",
///     "states": [{ "name": "draft", "display_name": "Draft", "is_final": false }],
///     "transitions": [{ "from": "draft", "event": "submit", "to": "pending" }]
///   }
/// }
/// ```
///
/// `state_machine` is `null` when the service has no state machine.
///
/// # Example
///
/// ```
/// use ferro_projections::{
///     ServiceDef, DataType, FieldMeaning, derive_intents, TemplateRenderer, Renderer, RenderContext,
/// };
///
/// let svc = ServiceDef::new("order")
///     .display_name("Order")
///     .field("id", DataType::Integer, FieldMeaning::Identifier)
///     .field("total", DataType::Float, FieldMeaning::Money);
///
/// let intents = derive_intents(&svc);
/// let renderer = TemplateRenderer;
/// let result = renderer.render(&svc, &intents, &RenderContext::default());
/// assert!(result.is_ok());
///
/// let json = result.unwrap();
/// assert_eq!(json["service"], "Order");
/// assert!(json["fields"]["total"].is_object());
/// assert!(!json["fields"].as_object().unwrap().contains_key("id"));
/// ```
pub struct TemplateRenderer;

impl Renderer for TemplateRenderer {
    fn render(
        &self,
        service: &ServiceDef,
        _intents: &[IntentScore],
        _ctx: &RenderContext,
    ) -> Result<Value, Error> {
        // Build fields map: keyed by name, excluding system fields.
        let mut fields = Map::new();
        for f in &service.fields {
            if !is_system_field(&f.meaning) {
                fields.insert(
                    f.name.clone(),
                    json!({
                        "name": f.name,
                        "data_type": f.data_type,
                        "meaning": f.meaning,
                        "required": f.required,
                    }),
                );
            }
        }

        // Build actions array: rich objects with display_name and inputs.
        let actions: Vec<Value> = service
            .actions
            .iter()
            .map(|a| {
                let inputs: Vec<Value> = a
                    .inputs
                    .iter()
                    .map(|i| {
                        json!({
                            "name": i.name,
                            "data_type": i.data_type,
                            "required": i.required,
                        })
                    })
                    .collect();
                json!({
                    "name": a.name,
                    "display_name": a.display_name.as_deref().unwrap_or(&a.name),
                    "inputs": inputs,
                })
            })
            .collect();

        // Build state_machine: null if absent.
        let state_machine: Option<Value> = service.state_machine.as_ref().map(|sm| {
            let states: Vec<Value> = sm
                .states
                .iter()
                .map(|s| {
                    json!({
                        "name": s.name,
                        "display_name": s.display_name.as_deref().unwrap_or(&s.name),
                        "is_final": s.is_final,
                    })
                })
                .collect();
            let transitions: Vec<Value> = sm
                .transitions
                .iter()
                .map(|t| {
                    json!({
                        "from": t.from,
                        "event": t.event,
                        "to": t.to,
                    })
                })
                .collect();
            json!({
                "initial_state": sm.initial_state,
                "states": states,
                "transitions": transitions,
            })
        });

        Ok(json!({
            "service": service.display_name.as_deref().unwrap_or(&service.name),
            "fields": Value::Object(fields),
            "actions": actions,
            "state_machine": state_machine,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{ActionDef, InputDef};
    use crate::derive::derive_intents;
    use crate::field::{DataType, FieldMeaning};
    use crate::service::ServiceDef;
    use crate::state::{StateDef, StateMachine, Transition};

    fn render(svc: &ServiceDef) -> Value {
        let intents = derive_intents(svc);
        let renderer = TemplateRenderer;
        renderer
            .render(svc, &intents, &RenderContext::default())
            .expect("render must succeed")
    }

    #[test]
    fn fields_use_original_names() {
        let svc = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status);

        let result = render(&svc);
        let fields = result["fields"].as_object().unwrap();

        // System field excluded
        assert!(!fields.contains_key("id"), "id (Identifier) must be excluded");
        // Domain fields included
        assert!(fields.contains_key("total"), "total must be present");
        assert!(fields.contains_key("status"), "status must be present");
    }

    #[test]
    fn field_values_include_metadata() {
        let svc = ServiceDef::new("order")
            .field("total", DataType::Float, FieldMeaning::Money);

        let result = render(&svc);
        let total = &result["fields"]["total"];

        assert_eq!(total["name"], "total");
        assert_eq!(total["meaning"], "money");
        assert_eq!(total["required"], true);
    }

    #[test]
    fn actions_include_display_name_and_inputs() {
        let svc = ServiceDef::new("cart")
            .field("item", DataType::String, FieldMeaning::EntityName)
            .action(
                ActionDef::new("add_to_cart")
                    .display_name("Add to Cart")
                    .input(InputDef::new(
                        "quantity",
                        DataType::Integer,
                        FieldMeaning::Quantity,
                    )),
            );

        let result = render(&svc);
        let actions = result["actions"].as_array().unwrap();

        assert_eq!(actions.len(), 1);
        let action = &actions[0];
        assert_eq!(action["name"], "add_to_cart");
        assert_eq!(action["display_name"], "Add to Cart");

        let inputs = action["inputs"].as_array().unwrap();
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0]["name"], "quantity");
    }

    #[test]
    fn state_machine_states_and_transitions() {
        let sm = StateMachine::new("lifecycle")
            .initial("pending")
            .state(StateDef::new("pending").display_name("Pending"))
            .state(StateDef::new("done").display_name("Done").final_state())
            .transition(Transition::new("pending", "complete", "done"));

        let svc = ServiceDef::new("task")
            .field("name", DataType::String, FieldMeaning::EntityName)
            .state_machine(sm);

        let result = render(&svc);
        let sm_val = &result["state_machine"];

        assert!(!sm_val.is_null(), "state_machine must not be null");
        let states = sm_val["states"].as_array().unwrap();
        assert_eq!(states.len(), 2, "should have 2 states");

        let transitions = sm_val["transitions"].as_array().unwrap();
        assert_eq!(transitions.len(), 1, "should have 1 transition");
        assert_eq!(transitions[0]["from"], "pending");
        assert_eq!(transitions[0]["event"], "complete");
        assert_eq!(transitions[0]["to"], "done");
    }

    #[test]
    fn no_state_machine_produces_null() {
        let svc = ServiceDef::new("product")
            .field("name", DataType::String, FieldMeaning::EntityName);

        let result = render(&svc);
        assert!(result["state_machine"].is_null());
    }

    #[test]
    fn service_display_name_present() {
        let svc = ServiceDef::new("order")
            .display_name("Order Management");

        let result = render(&svc);
        assert_eq!(result["service"], "Order Management");
    }

    #[test]
    fn service_name_fallback_when_no_display_name() {
        let svc = ServiceDef::new("order");
        let result = render(&svc);
        assert_eq!(result["service"], "order");
    }

    #[test]
    fn empty_actions_produces_empty_array() {
        let svc = ServiceDef::new("product")
            .field("name", DataType::String, FieldMeaning::EntityName);

        let result = render(&svc);
        let actions = result["actions"].as_array().unwrap();
        assert!(actions.is_empty());
    }
}
