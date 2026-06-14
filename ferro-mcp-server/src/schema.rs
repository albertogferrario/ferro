use ferro_projections::{ActionDef, DataType, FieldDef, FieldMeaning, ServiceDef};

/// Returns `true` if this field should appear as an equality filter in the
/// MCP tool `inputSchema`.
///
/// Gate order (load-bearing — do not reorder):
/// 1. Must be readable — excludes write-only (e.g., passwords) regardless of meaning.
/// 2. Must not be a list — equality filters on list columns are not useful.
/// 3. Must not carry `Sensitive` meaning — guards fields that ARE readable but still private.
/// 4. `DataType` must not be `Json` or `Binary` — equality filters are not useful there.
/// 5. Meaning must be in the conservative allowlist: Identifier, ForeignKey, Status,
///    Category, Boolean, Custom(_). All other meanings (EntityName, Email, Money, …) are
///    intentionally excluded.
pub fn is_filter_field(field: &FieldDef) -> bool {
    if !field.readable {
        return false;
    } // gate 1
    if field.is_list {
        return false;
    } // gate 2
    if matches!(field.meaning, FieldMeaning::Sensitive) {
        return false;
    } // gate 3
      // gate 4: equality filter on JSON/Binary columns is not useful
    if matches!(field.data_type, DataType::Json | DataType::Binary) {
        return false;
    }
    // gate 5: conservative meaning allowlist
    matches!(
        field.meaning,
        FieldMeaning::Identifier
            | FieldMeaning::ForeignKey
            | FieldMeaning::Status
            | FieldMeaning::Category
            | FieldMeaning::Boolean
            | FieldMeaning::Custom(_)
    )
}

/// Maps a `DataType` to its JSON Schema type fragment.
///
/// Returns a JSON object with at minimum a `"type"` key; date/time/uuid types
/// also emit a `"format"` key.
pub(crate) fn data_type_to_json_schema(dt: DataType) -> serde_json::Value {
    match dt {
        DataType::Integer => serde_json::json!({ "type": "integer" }),
        DataType::Float => serde_json::json!({ "type": "number" }),
        DataType::Boolean => serde_json::json!({ "type": "boolean" }),
        DataType::DateTime => serde_json::json!({ "type": "string", "format": "date-time" }),
        DataType::Date => serde_json::json!({ "type": "string", "format": "date" }),
        DataType::Uuid => serde_json::json!({ "type": "string", "format": "uuid" }),
        // String, Enum, Json, Binary (Json/Binary already filtered out by is_filter_field)
        _ => serde_json::json!({ "type": "string" }),
    }
}

/// Builds the MCP tool `inputSchema` as a JSON Schema object derived from
/// the projection's fields plus pagination parameters.
///
/// The schema always contains `limit` and `offset` pagination parameters.
/// Equality filter properties are added for each field that passes
/// [`is_filter_field`].
///
/// There is no separately declared schema — the property set is derived
/// entirely from `service.fields`. Adding or removing a filterable field
/// changes the schema (AMCP-02 single-source-of-truth guarantee).
pub fn build_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();

    properties.insert(
        "limit".into(),
        serde_json::json!({
            "type": "integer",
            "description": "Maximum number of records to return",
            "default": 25,
            "maximum": 100,
            "minimum": 1
        }),
    );
    properties.insert(
        "offset".into(),
        serde_json::json!({
            "type": "integer",
            "description": "Number of records to skip",
            "default": 0,
            "minimum": 0
        }),
    );

    for field in service.fields.iter().filter(|f| is_filter_field(f)) {
        let mut prop = data_type_to_json_schema(field.data_type);
        if let serde_json::Value::Object(ref mut m) = prop {
            m.insert(
                "description".into(),
                serde_json::Value::String(format!("Filter by {}", field.name)),
            );
        }
        properties.insert(field.name.clone(), prop);
    }

    Ok(serde_json::json!({ "type": "object", "properties": properties }))
}

/// Builds the MCP tool `inputSchema` for a write tool derived from `action`.
///
/// Injects the parent service's first `FieldMeaning::Identifier` field as a required
/// parameter (the record to act on), then maps each `ActionDef.inputs` entry via
/// the `data_type_to_json_schema` mapping. `FieldMeaning::Sensitive` inputs are excluded
/// (mirrors the `is_filter_field` gate 3 — PITFALLS §3). `action.preconditions` and
/// `action.effects` are NOT rendered — preconditions drive the list-time guard filter only.
pub fn build_action_input_schema(
    action: &ActionDef,
    service: &ServiceDef,
) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();
    let mut required_fields: Vec<String> = Vec::new();

    // Inject the identifier field (the record to act on) — always required.
    if let Some(id_field) = service
        .fields
        .iter()
        .find(|f| matches!(f.meaning, FieldMeaning::Identifier))
    {
        let mut prop = data_type_to_json_schema(id_field.data_type);
        if let serde_json::Value::Object(ref mut m) = prop {
            m.insert(
                "description".into(),
                serde_json::Value::String(format!(
                    "ID of the {} record to act on",
                    service.display_name.as_deref().unwrap_or(&service.name)
                )),
            );
        }
        properties.insert(id_field.name.clone(), prop);
        required_fields.push(id_field.name.clone());
    }

    // Map each InputDef; exclude Sensitive meanings (T-218-01 / PITFALLS §3).
    for input in &action.inputs {
        if matches!(input.meaning, FieldMeaning::Sensitive) {
            continue;
        }
        let mut prop = data_type_to_json_schema(input.data_type);
        if let serde_json::Value::Object(ref mut m) = prop {
            if let Some(ref desc) = input.description {
                m.insert(
                    "description".into(),
                    serde_json::Value::String(desc.clone()),
                );
            }
        }
        properties.insert(input.name.clone(), prop);
        if input.required {
            required_fields.push(input.name.clone());
        }
    }

    Ok(serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required_fields,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::InputDef;

    /// A service with a representative mix of field meanings.
    ///
    /// Filter-eligible: id (Identifier/Integer), status (Status/String),
    ///                  customer_id (ForeignKey/Integer)
    /// Excluded by allowlist: name (EntityName/String)
    /// Excluded by gate 3 (Sensitive meaning): password (Sensitive/String)
    fn sample_service() -> ServiceDef {
        ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("password", DataType::String, FieldMeaning::Sensitive)
    }

    #[test]
    fn test_pagination_params_in_schema() {
        let service = sample_service();
        let schema = build_input_schema(&service).expect("schema ok");
        let props = schema["properties"]
            .as_object()
            .expect("properties is object");
        assert!(props.contains_key("limit"), "limit missing");
        assert!(props.contains_key("offset"), "offset missing");
    }

    #[test]
    fn test_input_schema_derivation() {
        let service = sample_service();
        let schema_before = build_input_schema(&service).expect("schema ok");
        let count_before = schema_before["properties"]
            .as_object()
            .expect("object")
            .len();

        // status field (Status meaning) must be present
        assert!(
            schema_before["properties"]["status"].is_object(),
            "status not in properties"
        );

        // adding a ForeignKey field increases property count
        let service_with_extra =
            service.field("supplier_id", DataType::Integer, FieldMeaning::ForeignKey);
        let schema_after = build_input_schema(&service_with_extra).expect("schema ok");
        let count_after = schema_after["properties"]
            .as_object()
            .expect("object")
            .len();

        assert!(
            count_after > count_before,
            "adding a filter field should increase property count: before={count_before} after={count_after}"
        );
    }

    #[test]
    fn test_sensitive_field_excluded() {
        let service = sample_service();
        let schema = build_input_schema(&service).expect("schema ok");
        let props = schema["properties"].as_object().expect("object");
        assert!(
            !props.contains_key("password"),
            "Sensitive field 'password' must not appear in inputSchema"
        );
    }

    #[test]
    fn test_write_only_excluded() {
        let service = ServiceDef::new("user").write_only_field(
            "secret_key",
            DataType::String,
            FieldMeaning::Custom("api_key".into()),
        );
        let schema = build_input_schema(&service).expect("schema ok");
        let props = schema["properties"].as_object().expect("object");
        assert!(
            !props.contains_key("secret_key"),
            "write-only field must not appear in inputSchema"
        );
    }

    #[test]
    fn test_entity_name_excluded_by_allowlist() {
        let service = sample_service();
        let schema = build_input_schema(&service).expect("schema ok");
        let props = schema["properties"].as_object().expect("object");
        assert!(
            !props.contains_key("name"),
            "EntityName field 'name' must be excluded by the meaning allowlist"
        );
    }

    // -------------------------------------------------------------------------
    // Phase 218 RED tests for build_action_input_schema (SC#2, T-218-01).
    //
    // These tests reference `build_action_input_schema` which does NOT exist yet
    // — this is the intentional compile-error RED state for Wave 0.
    // The function is implemented in Plan 01; these tests turn GREEN there.
    // -------------------------------------------------------------------------

    /// Service fixture for action schema tests: has an Identifier field and a
    /// Status field.
    fn order_service_for_actions() -> ServiceDef {
        ServiceDef::new("order")
            .display_name("Order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
    }

    /// SC#2: The parent service's Identifier field must be injected as a
    /// required integer parameter in the action's inputSchema.
    #[test]
    fn test_action_schema_injects_identifier() {
        let service = order_service_for_actions();
        let action = ActionDef::new("submit_order");
        let schema = build_action_input_schema(&action, &service).expect("schema ok");
        assert_eq!(
            schema["properties"]["id"]["type"], "integer",
            "identifier field 'id' must be present as integer property"
        );
        let required = schema["required"]
            .as_array()
            .expect("required must be an array");
        assert!(
            required.iter().any(|v| v.as_str() == Some("id")),
            "'id' must be in required[]"
        );
    }

    /// SC#2: InputDef fields become schema properties with correct type and
    /// description.
    #[test]
    fn test_action_schema_maps_inputs() {
        let service = order_service_for_actions();
        let action = ActionDef::new("submit_order").input(
            InputDef::new("notes", DataType::String, FieldMeaning::FreeText)
                .description("Order notes"),
        );
        let schema = build_action_input_schema(&action, &service).expect("schema ok");
        assert_eq!(
            schema["properties"]["notes"]["type"], "string",
            "notes must be a string property"
        );
        assert_eq!(
            schema["properties"]["notes"]["description"], "Order notes",
            "description must be forwarded from InputDef"
        );
        let required = schema["required"]
            .as_array()
            .expect("required must be an array");
        assert!(
            required.iter().any(|v| v.as_str() == Some("notes")),
            "'notes' (required=true by default) must appear in required[]"
        );
    }

    /// SC#2 / T-218-01 boundary: an optional InputDef must appear in properties
    /// but NOT in required[].
    #[test]
    fn test_action_schema_optional_input_not_required() {
        let service = order_service_for_actions();
        let action = ActionDef::new("submit_order")
            .input(InputDef::new("memo", DataType::String, FieldMeaning::FreeText).required(false));
        let schema = build_action_input_schema(&action, &service).expect("schema ok");
        assert!(
            schema["properties"]["memo"].is_object(),
            "'memo' must appear in properties even when optional"
        );
        let required = schema["required"]
            .as_array()
            .expect("required must be an array");
        assert!(
            !required.iter().any(|v| v.as_str() == Some("memo")),
            "'memo' (required=false) must NOT appear in required[]"
        );
    }

    /// SC#2 / T-218-01 security mitigation: FieldMeaning::Sensitive inputs must
    /// be excluded from the schema (properties AND required).
    #[test]
    fn test_action_schema_excludes_sensitive_input() {
        let service = order_service_for_actions();
        let action = ActionDef::new("submit_order").input(InputDef::new(
            "secret_token",
            DataType::String,
            FieldMeaning::Sensitive,
        ));
        let schema = build_action_input_schema(&action, &service).expect("schema ok");
        let props = schema["properties"].as_object().expect("properties object");
        assert!(
            !props.contains_key("secret_token"),
            "Sensitive input 'secret_token' must not appear in properties"
        );
        let required = schema["required"]
            .as_array()
            .expect("required must be an array");
        assert!(
            !required.iter().any(|v| v.as_str() == Some("secret_token")),
            "Sensitive input 'secret_token' must not appear in required[]"
        );
    }

    /// SC#2: When the service has no Identifier field, identifier injection is
    /// silently skipped. The schema is still valid and contains any declared inputs.
    #[test]
    fn test_action_schema_no_identifier_field_is_silent_noop() {
        let service =
            ServiceDef::new("queue_item").field("status", DataType::String, FieldMeaning::Status);
        let action = ActionDef::new("process_item").input(InputDef::new(
            "priority",
            DataType::Integer,
            FieldMeaning::Custom("priority".into()),
        ));
        let schema = build_action_input_schema(&action, &service).expect("schema ok");
        assert_eq!(
            schema["type"], "object",
            "schema must still be a JSON object when no identifier exists"
        );
        assert!(
            schema["properties"]["priority"].is_object(),
            "'priority' input must appear in properties"
        );
    }
}
