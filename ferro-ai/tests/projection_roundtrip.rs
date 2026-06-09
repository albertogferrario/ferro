//! AICLI-06 — projection-roundtrip proof (v12.1 capstone).
//!
//! Offline, deterministic, no network, no LLM key. A constructed ServiceDef
//! fixture is rendered through the ServiceDef-aware deterministic path
//! (`Spec::from_service_def`) and validated against the catalog. The currency
//! assertion pins the FieldMeaning::Money -> ColumnFormat::Currency dispatch;
//! it cannot pass via a generic schema-normalization fallback (SC5).

use ferro_json_ui::{global_catalog, VisualContext};
use ferro_projections::{derive_intents, DataType, FieldMeaning, Intent, ServiceDef};

fn invoice_fixture() -> ServiceDef {
    ServiceDef::new("invoice")
        .display_name("Invoice")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("total", DataType::Float, FieldMeaning::Money)
        .field("recipient", DataType::String, FieldMeaning::EntityName)
}

#[test]
fn servicedef_browse_projection_validates_and_pins_servicedef_path() {
    let service = invoice_fixture();
    let intents = derive_intents(&service);
    assert!(
        !intents.is_empty(),
        "invoice fixture must derive at least one intent"
    );

    let browse_idx = intents
        .iter()
        .position(|i| matches!(i.intent, Intent::Browse))
        .unwrap_or(0);
    let ctx = VisualContext {
        intent_index: browse_idx,
        ..VisualContext::default()
    };

    // SC3 + SC5: deterministic render via the ServiceDef-aware path.
    let spec = ferro_json_ui::Spec::from_service_def(&service, &intents, &ctx)
        .expect("invoice fixture must project successfully");

    // SC2: catalog write-gate.
    assert!(
        global_catalog().validate(&spec).is_ok(),
        "projected spec must pass catalog validation"
    );
    assert_eq!(spec.schema, "ferro-json-ui/v2");

    let root = spec
        .elements
        .get(&spec.root)
        .expect("root element must exist");
    assert_eq!(
        root.type_name, "DataTable",
        "Browse intent must produce a DataTable root"
    );

    let cols = root
        .props
        .get("columns")
        .and_then(|c| c.as_array())
        .expect("DataTable must have a columns prop");
    let has_currency = cols
        .iter()
        .any(|c| c.get("format").and_then(|f| f.as_str()) == Some("currency"));
    assert!(
        has_currency,
        "Money field must produce a currency-formatted column — \
         proves the ServiceDef-aware dispatch (FieldMeaning::Money -> ColumnFormat::Currency)"
    );
}
