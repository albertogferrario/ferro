//! Collect archetype: form surface from writable fields.

use crate::actions::{action_button, crud_name};
use crate::builder::{emit_title, writable_fields, Emit};
use crate::component::Component;
use crate::context::A2uiContext;
use ferro_projections::render::field_display_name;
use ferro_projections::{DataType, Error, FieldDef, FieldMeaning, ServiceDef};
use ferro_theme::IntentSlotTemplate;

pub(crate) fn emit(
    e: &mut Emit,
    service: &ServiceDef,
    _ctx: &A2uiContext,
    template: &IntentSlotTemplate,
) -> Result<Vec<String>, Error> {
    let mut children = Vec::new();
    for slot in &template.slots {
        match slot.as_str() {
            "title" => children.push(emit_title(e, service)),
            "fields" => {
                for f in writable_fields(service) {
                    children.push(emit_input(e, f));
                    children.push(emit_error_text(e, f));
                }
            }
            "actions" => {
                if let Some(id) = emit_submit(e, service) {
                    children.push(id);
                }
            }
            _ => {}
        }
    }
    Ok(children)
}

fn form_path(f: &FieldDef) -> String {
    format!("/form/{}", f.name)
}

fn emit_input(e: &mut Emit, f: &FieldDef) -> String {
    let id = format!("{}_input", f.name);
    let label = field_display_name(&f.name);
    let path = form_path(f);
    e.contract
        .bind(path.clone(), Some(f.data_type), Some(&f.name));

    let is_choice = matches!(f.meaning, FieldMeaning::Status | FieldMeaning::Category)
        || f.data_type == DataType::Enum;
    let is_bool = f.meaning == FieldMeaning::Boolean || f.data_type == DataType::Boolean;
    let is_datetime = matches!(
        f.meaning,
        FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt | FieldMeaning::DateTime
    ) || matches!(f.data_type, DataType::DateTime | DataType::Date);

    let c = if is_bool {
        Component::new(id.clone(), "CheckBox")
            .prop("label", label)
            .bound_prop("value", path)
    } else if is_choice {
        let options_path = format!("/form/options/{}", f.name);
        e.contract.bind(options_path.clone(), None, Some(&f.name));
        Component::new(id.clone(), "ChoicePicker")
            .prop("label", label)
            .bound_prop("selections", path)
            .bound_prop("options", options_path)
            .prop("maxAllowedSelections", 1)
    } else if is_datetime {
        Component::new(id.clone(), "DateTimeInput")
            .bound_prop("value", path)
            .prop("enableDate", true)
            .prop("enableTime", f.data_type == DataType::DateTime)
    } else {
        Component::new(id.clone(), "TextField")
            .prop("label", label)
            .bound_prop("value", path)
    };
    e.push(c);
    id
}

fn emit_error_text(e: &mut Emit, f: &FieldDef) -> String {
    let id = format!("{}_error", f.name);
    let path = format!("/form/errors/{}", f.name);
    e.contract
        .bind(path.clone(), Some(DataType::String), Some(&f.name));
    e.push(Component::new(id.clone(), "Text").bound_prop("text", path));
    id
}

fn emit_submit(e: &mut Emit, service: &ServiceDef) -> Option<String> {
    let verb = if service.creatable {
        "create"
    } else if service.updatable {
        "update"
    } else {
        return None;
    };
    let event_name = crud_name(verb, service);
    // The submit event references the whole form scope; contract it so the
    // host knows the surface reads `/form` as a unit.
    e.contract.bind("/form", None, None);
    let context = serde_json::json!({"form": {"path": "/form"}});
    Some(action_button(e, "submit", "Save", &event_name, context))
}

#[cfg(test)]
mod tests {
    use crate::context::A2uiContext;
    use crate::message::A2uiMessage;
    use crate::test_support::{order_service, scored};
    use crate::A2uiRenderer;
    use ferro_projections::render::Renderer;
    use ferro_projections::Intent;

    fn render_collect() -> (Vec<crate::component::Component>, crate::DataContract) {
        let out = A2uiRenderer
            .render(
                &order_service(),
                &scored(Intent::Collect),
                &A2uiContext::default(),
            )
            .unwrap();
        let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
            panic!()
        };
        (cs.components.clone(), out.data_contract)
    }

    #[test]
    fn collect_emits_inputs_by_field_meaning() {
        let (components, _) = render_collect();
        let by_id = |id: &str| components.iter().find(|c| c.id == id).unwrap();
        // writable non-system fields: customer_name, total, status, notes
        assert_eq!(by_id("customer_name_input").component, "TextField");
        assert_eq!(
            by_id("customer_name_input").props["value"],
            serde_json::json!({"path": "/form/customer_name"})
        );
        assert_eq!(
            by_id("customer_name_input").props["label"],
            serde_json::json!("Customer Name")
        );
        assert_eq!(by_id("total_input").component, "TextField");
        let status = by_id("status_input");
        assert_eq!(status.component, "ChoicePicker");
        assert_eq!(
            status.props["options"],
            serde_json::json!({"path": "/form/options/status"})
        );
        assert_eq!(status.props["maxAllowedSelections"], serde_json::json!(1));
        assert_eq!(by_id("notes_input").component, "TextField");
        // per-field error text bound to the error path
        assert_eq!(
            by_id("customer_name_error").props["text"],
            serde_json::json!({"path": "/form/errors/customer_name"})
        );
    }

    #[test]
    fn collect_submit_fires_create_with_form_scope() {
        let (components, _) = render_collect();
        let submit = components.iter().find(|c| c.id == "submit").unwrap();
        assert_eq!(
            submit.props["action"]["event"]["name"],
            serde_json::json!("create_order")
        );
        assert_eq!(
            submit.props["action"]["event"]["context"]["form"],
            serde_json::json!({"path": "/form"})
        );
    }

    #[test]
    fn collect_contract_covers_form_paths_and_sets_send_data_model() {
        let out = A2uiRenderer
            .render(
                &order_service(),
                &scored(Intent::Collect),
                &A2uiContext::default(),
            )
            .unwrap();
        let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
            panic!()
        };
        assert_eq!(cs.send_data_model, Some(true));
        let paths = out.data_contract.paths();
        for p in [
            "/form/customer_name",
            "/form/total",
            "/form/status",
            "/form/notes",
            "/form/errors/customer_name",
            "/form/options/status",
        ] {
            assert!(paths.contains(&p), "missing contract path {p}");
        }
    }
}
