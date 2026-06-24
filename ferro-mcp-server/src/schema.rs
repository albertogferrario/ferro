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

/// Returns `true` if this field should receive `__gt/__gte/__lt/__lte` range params.
///
/// Gate order:
/// 1. Must be readable.
/// 2. Must not be a list.
/// 3. Must not carry `Sensitive` meaning.
/// 4. DataType must not be `Json` or `Binary`.
/// 5. DataType must be ordered/comparable: Integer, Float, DateTime, or Date.
///
/// Gate 5 is DataType-based (Integer/Float/DateTime/Date), NOT meaning-based, so
/// Money/Quantity/Percentage fields — excluded by `is_filter_field`'s meaning gate —
/// still get range params.
pub fn is_range_filter_field(field: &FieldDef) -> bool {
    if !field.readable {
        return false;
    } // gate 1
    if field.is_list {
        return false;
    } // gate 2
    if matches!(field.meaning, FieldMeaning::Sensitive) {
        return false;
    } // gate 3
    if matches!(field.data_type, DataType::Json | DataType::Binary) {
        return false;
    } // gate 4
    matches!(
        field.data_type,
        DataType::Integer | DataType::Float | DataType::DateTime | DataType::Date
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

    // __ne and __in for every is_filter_field field (D-09)
    for field in service.fields.iter().filter(|f| is_filter_field(f)) {
        let scalar = data_type_to_json_schema(field.data_type);
        // __ne: same scalar type, not-equal filter
        let mut ne_prop = scalar.clone();
        if let serde_json::Value::Object(ref mut m) = ne_prop {
            m.insert(
                "description".into(),
                serde_json::Value::String(format!("Filter by {} (not equal)", field.name)),
            );
        }
        properties.insert(format!("{}__{}", field.name, "ne"), ne_prop);
        // __in: array of the same scalar type
        properties.insert(
            format!("{}__{}", field.name, "in"),
            serde_json::json!({
                "type": "array",
                "items": scalar,
                "description": format!("Filter by {} (any of)", field.name),
            }),
        );
    }

    // __gt/__gte/__lt/__lte for ordered (numeric + date/time) fields (D-10)
    for field in service.fields.iter().filter(|f| is_range_filter_field(f)) {
        let scalar = data_type_to_json_schema(field.data_type);
        for op in &["gt", "gte", "lt", "lte"] {
            let mut prop = scalar.clone();
            if let serde_json::Value::Object(ref mut m) = prop {
                m.insert(
                    "description".into(),
                    serde_json::Value::String(format!("Filter by {} ({})", field.name, op)),
                );
            }
            properties.insert(format!("{}__{}", field.name, op), prop);
        }
    }

    // sort param (D-11): prefix with '-' for descending
    properties.insert(
        "sort".into(),
        serde_json::json!({
            "type": "string",
            "description": "Sort field. Prefix with '-' for descending (e.g. 'created_at', '-total')",
        }),
    );

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

/// Builds the MCP tool `inputSchema` for a `create_<svc>` tool.
///
/// Iterates `service.fields`, excluding server-injected, UpdatedAt, Sensitive, list, and
/// (when a StateMachine is present) Status fields via [`ServiceDef::is_write_excluded_field`].
/// Fields marked `required` on the `FieldDef` populate the `required[]` array.
/// The Identifier field is excluded — a new record has no id yet (D-03).
pub fn build_create_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();
    let mut required_fields: Vec<String> = Vec::new();
    let exclude_sm_status = service.state_machine.is_some();

    for field in &service.fields {
        if service.is_write_excluded_field(field, exclude_sm_status) {
            continue;
        }
        let mut prop = data_type_to_json_schema(field.data_type);
        if let serde_json::Value::Object(ref mut m) = prop {
            m.insert(
                "description".into(),
                serde_json::Value::String(format!("Value for the {} field", field.name)),
            );
        }
        properties.insert(field.name.clone(), prop);
        if field.required {
            required_fields.push(field.name.clone());
        }
    }

    Ok(serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required_fields,
    }))
}

/// Builds the MCP tool `inputSchema` for an `update_<svc>` tool (patch semantics).
///
/// Injects the service Identifier as the sole required parameter (the record to patch),
/// then adds every non-excluded data field as optional — patch semantics mean the caller
/// supplies only the fields they want to change (D-06). Exclusions are identical to
/// `build_create_input_schema` via [`ServiceDef::is_write_excluded_field`].
pub fn build_update_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();
    let mut required_fields: Vec<String> = Vec::new();

    // Inject the identifier (required) — the record to patch (T-240-05 / Pitfall 7).
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
                    "ID of the {} record to update",
                    service.display_name.as_deref().unwrap_or(&service.name)
                )),
            );
        }
        properties.insert(id_field.name.clone(), prop);
        required_fields.push(id_field.name.clone());
    }

    let exclude_sm_status = service.state_machine.is_some();
    // Data fields: same exclusion predicate as create; all optional (patch semantics D-06).
    for field in &service.fields {
        // Explicitly skip the identifier — it was already injected above as the required
        // target. is_write_excluded_field also drops it (Gate A), but skipping here makes
        // the patch loop robust to any future relaxation of that gate (no duplicate id prop).
        if matches!(field.meaning, FieldMeaning::Identifier) {
            continue;
        }
        if service.is_write_excluded_field(field, exclude_sm_status) {
            continue;
        }
        let mut prop = data_type_to_json_schema(field.data_type);
        if let serde_json::Value::Object(ref mut m) = prop {
            m.insert(
                "description".into(),
                serde_json::Value::String(format!("New value for the {} field", field.name)),
            );
        }
        properties.insert(field.name.clone(), prop);
        // NOT added to required_fields — patch semantics.
    }

    Ok(serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required_fields,
    }))
}

/// Builds the MCP tool `inputSchema` for a `delete_<svc>` tool.
///
/// Requires the service Identifier (the record to delete). Adds an optional
/// `confirmation_token` field whose enforcement is Phase 241/242 — the schema
/// only advertises the parameter so the agent knows to request one first (D-08).
pub fn build_delete_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();
    let mut required_fields: Vec<String> = Vec::new();

    // Identifier — required (the record to delete).
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
                    "ID of the {} record to delete",
                    service.display_name.as_deref().unwrap_or(&service.name)
                )),
            );
        }
        properties.insert(id_field.name.clone(), prop);
        required_fields.push(id_field.name.clone());
    }

    // Confirmation token — optional; execution/enforcement is Phase 241/242 (D-08).
    properties.insert(
        "confirmation_token".to_string(),
        serde_json::json!({
            "type": "string",
            "description": "Confirmation token from request_confirm_delete_<svc> (Phase 241)",
        }),
    );

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

    // -------------------------------------------------------------------------
    // Phase 240 Plan 02 Task 1: RED tests for is_range_filter_field and extended
    // build_input_schema (range/ne/in/sort params).
    // -------------------------------------------------------------------------

    /// Service with numeric and datetime fields for range/ne/in/sort tests.
    fn range_service() -> ServiceDef {
        ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .field("status", DataType::String, FieldMeaning::Status)
    }

    /// Range params: numeric (Float) and DateTime fields must produce __gt/__gte/__lt/__lte;
    /// a String/Status field must NOT get range params.
    #[test]
    fn test_range_params_in_schema() {
        let service = range_service();
        let schema = build_input_schema(&service).expect("schema ok");
        let props = schema["properties"].as_object().expect("properties object");

        // Float field (total) gets range params
        assert!(
            props.contains_key("total__gt"),
            "total__gt must be present for Float field"
        );
        assert!(
            props.contains_key("total__gte"),
            "total__gte must be present for Float field"
        );
        assert!(
            props.contains_key("total__lt"),
            "total__lt must be present for Float field"
        );
        assert!(
            props.contains_key("total__lte"),
            "total__lte must be present for Float field"
        );

        // DateTime field (created_at) gets range params — Money/Quantity pass even though
        // is_filter_field excludes them by meaning; DataType gate is the criterion
        assert!(
            props.contains_key("created_at__gt"),
            "created_at__gt must be present for DateTime field"
        );

        // String/Status field must NOT get range params (DataType gate)
        assert!(
            !props.contains_key("status__gt"),
            "status__gt must NOT be present for String/Status field"
        );
    }

    /// ne/in params: every is_filter_field field (id/Identifier, status/Status) must get
    /// <field>__ne and <field>__in params; __in must be an array type.
    #[test]
    fn test_ne_in_params_in_schema() {
        let service = range_service();
        let schema = build_input_schema(&service).expect("schema ok");
        let props = schema["properties"].as_object().expect("properties object");

        // Identifier field (id) passes is_filter_field → gets __ne and __in
        assert!(
            props.contains_key("id__ne"),
            "id__ne must be present (Identifier passes is_filter_field)"
        );
        assert!(
            props.contains_key("id__in"),
            "id__in must be present (Identifier passes is_filter_field)"
        );
        assert_eq!(
            props["id__in"]["type"].as_str(),
            Some("array"),
            "id__in must have type: array"
        );

        // Status field passes is_filter_field → gets __ne and __in
        assert!(
            props.contains_key("status__ne"),
            "status__ne must be present (Status passes is_filter_field)"
        );
        assert!(
            props.contains_key("status__in"),
            "status__in must be present (Status passes is_filter_field)"
        );
    }

    /// Sort param: inputSchema must contain a `sort` key of type string.
    #[test]
    fn test_sort_param_in_schema() {
        let service = range_service();
        let schema = build_input_schema(&service).expect("schema ok");
        let props = schema["properties"].as_object().expect("properties object");
        assert!(
            props.contains_key("sort"),
            "sort param must be present in inputSchema"
        );
        assert_eq!(
            props["sort"]["type"].as_str(),
            Some("string"),
            "sort param must have type: string"
        );
    }

    /// Back-compat: existing equality params and limit/offset must remain unchanged.
    #[test]
    fn test_existing_params_backcompat() {
        let service = range_service();
        let schema = build_input_schema(&service).expect("schema ok");
        let props = schema["properties"].as_object().expect("properties object");

        // Pagination params unchanged
        assert!(props.contains_key("limit"), "limit must still be present");
        assert!(props.contains_key("offset"), "offset must still be present");
        assert_eq!(
            props["limit"]["default"], 25,
            "limit default must remain 25"
        );

        // Equality params still present (status is a filter field)
        assert!(
            props.contains_key("status"),
            "bare equality param 'status' must still be present"
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

    // -------------------------------------------------------------------------
    // Phase 240 Plan 02 Task 2: RED tests for build_create/update/delete_input_schema.
    // -------------------------------------------------------------------------

    use ferro_projections::StateMachine;

    /// Full-field service fixture covering all exclusion categories.
    fn write_service_no_sm() -> ServiceDef {
        ServiceDef::new("order")
            .tenant_column("org_id")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)
            .field("org_id", DataType::Integer, FieldMeaning::ForeignKey)
            .field("password", DataType::String, FieldMeaning::Sensitive)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("notes", DataType::String, FieldMeaning::FreeText)
    }

    fn write_service_with_sm() -> ServiceDef {
        write_service_no_sm().state_machine(StateMachine::new("order_lifecycle").initial("pending"))
    }

    /// T-240-04: build_create_input_schema must exclude Identifier, CreatedAt,
    /// tenant column, UpdatedAt, Sensitive, and list fields; FreeText (notes) stays.
    #[test]
    fn test_create_schema_exclusions() {
        let service = write_service_no_sm();
        let schema = build_create_input_schema(&service).expect("schema ok");
        let props = schema["properties"].as_object().expect("properties object");

        // Excluded fields must be absent
        assert!(
            !props.contains_key("id"),
            "Identifier 'id' must be excluded"
        );
        assert!(
            !props.contains_key("created_at"),
            "CreatedAt must be excluded"
        );
        assert!(
            !props.contains_key("updated_at"),
            "UpdatedAt must be excluded"
        );
        assert!(
            !props.contains_key("org_id"),
            "tenant column 'org_id' must be excluded"
        );
        assert!(
            !props.contains_key("password"),
            "Sensitive 'password' must be excluded"
        );

        // Writable field must be present
        assert!(
            props.contains_key("notes"),
            "FreeText 'notes' must be present"
        );
    }

    /// T-240-04: Status absent when SM present; Status present when no SM.
    #[test]
    fn test_create_schema_status_sm() {
        let svc_with_sm = write_service_with_sm();
        let schema_sm = build_create_input_schema(&svc_with_sm).expect("schema ok");
        let props_sm = schema_sm["properties"].as_object().expect("object");
        assert!(
            !props_sm.contains_key("status"),
            "Status must be absent when SM present"
        );

        let svc_no_sm = write_service_no_sm();
        let schema_no_sm = build_create_input_schema(&svc_no_sm).expect("schema ok");
        let props_no_sm = schema_no_sm["properties"].as_object().expect("object");
        assert!(
            props_no_sm.contains_key("status"),
            "Status must be present when no SM"
        );
    }

    /// T-240-05: build_update_input_schema — required[] is exactly ["id"];
    /// data fields (notes, status-when-no-SM) appear in properties but NOT required[].
    #[test]
    fn test_update_schema_patch_semantics() {
        let service = write_service_no_sm();
        let schema = build_update_input_schema(&service).expect("schema ok");
        let props = schema["properties"].as_object().expect("properties object");
        let required = schema["required"].as_array().expect("required array");

        // Identifier must be required
        assert!(
            required.iter().any(|v| v.as_str() == Some("id")),
            "'id' must be in required[]"
        );
        assert_eq!(required.len(), 1, "required[] must be exactly [\"id\"]");

        // Data fields in properties
        assert!(props.contains_key("notes"), "notes must be in properties");
        assert!(
            props.contains_key("status"),
            "status must be in properties (no SM)"
        );

        // Data fields NOT in required
        assert!(
            !required.iter().any(|v| v.as_str() == Some("notes")),
            "notes must NOT be in required[] (patch semantics)"
        );
        assert!(
            !required.iter().any(|v| v.as_str() == Some("status")),
            "status must NOT be in required[] (patch semantics)"
        );
    }

    /// Status absent from update properties when SM present.
    #[test]
    fn test_update_schema_status_sm() {
        let service = write_service_with_sm();
        let schema = build_update_input_schema(&service).expect("schema ok");
        let props = schema["properties"].as_object().expect("properties object");
        assert!(
            !props.contains_key("status"),
            "Status must be absent from update when SM present"
        );
    }

    /// Phase 243.1 Gate F: a read-only (writable: false) field must be absent
    /// from both create and update input schemas. Regression guard for the
    /// `is_write_excluded_field` Gate F added in ferro-projections.
    #[test]
    fn read_only_field_absent_from_write_schemas() {
        let svc = ServiceDef::new("order")
            .field("customer_name", DataType::String, FieldMeaning::EntityName)
            .read_only_field("total", DataType::Float, FieldMeaning::Money);

        let create = build_create_input_schema(&svc).expect("create schema");
        let create_props = create["properties"].as_object().expect("create properties");
        assert!(create_props.contains_key("customer_name"));
        assert!(
            !create_props.contains_key("total"),
            "read-only `total` must not appear in create_order input"
        );

        let update = build_update_input_schema(&svc).expect("update schema");
        let update_props = update["properties"].as_object().expect("update properties");
        assert!(
            !update_props.contains_key("total"),
            "read-only `total` must not appear in update_order input"
        );
    }

    /// build_delete_input_schema — required is ["id"], properties contains id and
    /// confirmation_token (type string); confirmation_token NOT in required.
    #[test]
    fn test_delete_schema() {
        let service = write_service_no_sm();
        let schema = build_delete_input_schema(&service).expect("schema ok");
        let props = schema["properties"].as_object().expect("properties object");
        let required = schema["required"].as_array().expect("required array");

        // id required
        assert!(
            required.iter().any(|v| v.as_str() == Some("id")),
            "'id' must be in required[]"
        );
        assert_eq!(required.len(), 1, "required[] must be exactly [\"id\"]");

        // confirmation_token present in properties, not in required
        assert!(
            props.contains_key("confirmation_token"),
            "confirmation_token must be in properties"
        );
        assert_eq!(
            props["confirmation_token"]["type"].as_str(),
            Some("string"),
            "confirmation_token must have type: string"
        );
        assert!(
            !required
                .iter()
                .any(|v| v.as_str() == Some("confirmation_token")),
            "confirmation_token must NOT be in required[]"
        );
    }
}
