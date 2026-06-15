//! The AUTHORED artifact measured by this benchmark.
//!
//! Everything below `service_def()` is the entire code a Ferro developer writes
//! to obtain a data-bound list / detail / create-form / stat / kanban UI for the
//! `product` resource. No controller, no views, no migration — `derive_intents`
//! + `JsonUiRenderer` produce the surfaces from this declaration alone.
//!
//! Fields are deliberately SCALAR so the create form is fully fair: name/price/
//! stock map to text/number Inputs with no enum or foreign-key dropdowns (those
//! render with empty options today — see RESULTS.md caveat (a)). `status` is a
//! scalar string used for Process/Kanban DISPLAY grouping; it is not required to
//! drive the create form.

use ferro_projections::{DataType, FieldMeaning, ServiceDef, StateDef, StateMachine, Transition};

/// Build the Product service projection.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("product")
        .display_name("Product")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .field("price", DataType::Float, FieldMeaning::Money)
        .field("stock", DataType::Integer, FieldMeaning::Quantity)
        .field("status", DataType::String, FieldMeaning::Status)
        .state_machine(
            StateMachine::new("product_lifecycle")
                .initial("draft")
                .state(StateDef::new("draft").display_name("Draft"))
                .state(StateDef::new("active").display_name("Active"))
                .state(StateDef::new("discontinued").display_name("Discontinued").final_state())
                .transition(Transition::new("draft", "publish", "active"))
                .transition(Transition::new("active", "retire", "discontinued")),
        )
}
