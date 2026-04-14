//! Maps FieldMeaning to JSON-UI component JSON for display, input, and column modes.
//!
//! Three public functions provide exhaustive mapping of all 18 FieldMeaning variants
//! (plus Custom fallback) to JSON-UI component specifications.

use serde_json::{json, Value};

use ferro_projections::{FieldDef, FieldMeaning};

use ferro_projections::render::field_display_name;

/// Maps a field to its display (read-only) component JSON.
///
/// Returns `Value::Null` for fields that should not appear in display mode
/// (ForeignKey, Sensitive).
pub fn field_to_display(field: &FieldDef) -> Value {
    let name = &field.name;
    let label = field_display_name(name);
    let data_path = format!("/data/{name}");

    match &field.meaning {
        FieldMeaning::Identifier => json!({
            "type": "Text",
            "content": "",
            "element": "span",
            "key": format!("{name}-display"),
            "label": label,
            "data_path": data_path,
        }),
        FieldMeaning::ForeignKey => Value::Null,
        FieldMeaning::EntityName => json!({
            "type": "Text",
            "content": "",
            "element": "span",
            "key": format!("{name}-display"),
            "label": label,
            "data_path": data_path,
        }),
        FieldMeaning::Email
        | FieldMeaning::Phone
        | FieldMeaning::Url
        | FieldMeaning::Money
        | FieldMeaning::Quantity
        | FieldMeaning::FreeText
        | FieldMeaning::CreatedAt
        | FieldMeaning::UpdatedAt
        | FieldMeaning::DateTime => json!({
            "type": "Text",
            "content": "",
            "element": "span",
            "key": format!("{name}-display"),
            "label": label,
            "data_path": data_path,
        }),
        FieldMeaning::ImageUrl => json!({
            "type": "Avatar",
            "src": "",
            "alt": "",
            "key": format!("{name}-avatar"),
            "label": label,
            "data_path": data_path,
        }),
        FieldMeaning::Percentage => json!({
            "type": "Progress",
            "value": 0,
            "key": format!("{name}-progress"),
            "label": label,
            "data_path": data_path,
        }),
        FieldMeaning::Status => json!({
            "type": "Badge",
            "text": "",
            "variant": "default",
            "key": format!("{name}-badge"),
            "label": label,
            "data_path": data_path,
        }),
        FieldMeaning::Category => json!({
            "type": "Badge",
            "text": "",
            "variant": "secondary",
            "key": format!("{name}-badge"),
            "label": label,
            "data_path": data_path,
        }),
        FieldMeaning::Boolean => json!({
            "type": "Badge",
            "text": "",
            "variant": "outline",
            "key": format!("{name}-badge"),
            "label": label,
            "data_path": data_path,
        }),
        FieldMeaning::Sensitive => Value::Null,
        FieldMeaning::Custom(_) => json!({
            "type": "Text",
            "content": "",
            "element": "span",
            "key": format!("{name}-display"),
            "label": label,
            "data_path": data_path,
        }),
    }
}

/// Maps a field to its input (form) component JSON.
///
/// Every input includes key, name, label, required, and data_path (except Sensitive).
pub fn field_to_input(field: &FieldDef) -> Value {
    let name = &field.name;
    let label = field_display_name(name);
    let data_path = format!("/data/{name}");
    let required = field.required;

    match &field.meaning {
        FieldMeaning::Identifier => json!({
            "type": "Input",
            "input_type": "hidden",
            "key": format!("{name}-input"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": required,
        }),
        FieldMeaning::ForeignKey => json!({
            "type": "Select",
            "key": format!("{name}-select"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": required,
            "options": [],
        }),
        FieldMeaning::EntityName => json!({
            "type": "Input",
            "input_type": "text",
            "key": format!("{name}-input"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": true,
        }),
        FieldMeaning::Email => json!({
            "type": "Input",
            "input_type": "email",
            "key": format!("{name}-input"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": required,
        }),
        FieldMeaning::Phone => json!({
            "type": "Input",
            "input_type": "tel",
            "key": format!("{name}-input"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": required,
        }),
        FieldMeaning::Url | FieldMeaning::ImageUrl => json!({
            "type": "Input",
            "input_type": "url",
            "key": format!("{name}-input"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": required,
        }),
        FieldMeaning::Money => json!({
            "type": "Input",
            "input_type": "number",
            "step": "0.01",
            "key": format!("{name}-input"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": required,
        }),
        FieldMeaning::Percentage => json!({
            "type": "Input",
            "input_type": "number",
            "min": "0",
            "max": "100",
            "key": format!("{name}-input"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": required,
        }),
        FieldMeaning::Quantity => json!({
            "type": "Input",
            "input_type": "number",
            "key": format!("{name}-input"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": required,
        }),
        FieldMeaning::Status | FieldMeaning::Category => json!({
            "type": "Select",
            "key": format!("{name}-select"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": required,
            "options": [],
        }),
        FieldMeaning::Boolean => json!({
            "type": "Switch",
            "key": format!("{name}-switch"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": required,
        }),
        FieldMeaning::FreeText => json!({
            "type": "Input",
            "input_type": "textarea",
            "key": format!("{name}-input"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": required,
        }),
        FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt => json!({
            "type": "Input",
            "input_type": "text",
            "disabled": true,
            "key": format!("{name}-input"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": required,
        }),
        FieldMeaning::DateTime => json!({
            "type": "Input",
            "input_type": "text",
            "key": format!("{name}-input"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": required,
        }),
        FieldMeaning::Sensitive => json!({
            "type": "Input",
            "input_type": "password",
            "key": format!("{name}-input"),
            "name": name,
            "label": label,
            "required": required,
        }),
        FieldMeaning::Custom(_) => json!({
            "type": "Input",
            "input_type": "text",
            "key": format!("{name}-input"),
            "name": name,
            "label": label,
            "data_path": data_path,
            "required": required,
        }),
    }
}

/// Maps a field to a table column definition JSON.
///
/// Includes format hints for columns that need special formatting
/// (currency, datetime, boolean).
pub fn field_to_column(field: &FieldDef) -> Value {
    let name = &field.name;
    let label = field_display_name(name);

    let format = match &field.meaning {
        FieldMeaning::Money => Some("currency"),
        FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt | FieldMeaning::DateTime => {
            Some("datetime")
        }
        FieldMeaning::Boolean => Some("boolean"),
        _ => None,
    };

    match format {
        Some(fmt) => json!({
            "key": name,
            "label": label,
            "format": fmt,
        }),
        None => json!({
            "key": name,
            "label": label,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::DataType;

    fn make_field(name: &str, meaning: FieldMeaning) -> FieldDef {
        FieldDef {
            name: name.to_string(),
            data_type: DataType::String,
            meaning,
            required: true,
            is_list: false,
            readable: true,
            writable: true,
        }
    }

    // -- field_to_display tests --

    #[test]
    fn display_entity_name_produces_text() {
        let field = make_field("title", FieldMeaning::EntityName);
        let result = field_to_display(&field);
        assert_eq!(result["type"], "Text");
        assert_eq!(result["data_path"], "/data/title");
        assert_eq!(result["label"], "Title");
        assert_eq!(result["key"], "title-display");
    }

    #[test]
    fn display_money_produces_text() {
        let field = make_field("total", FieldMeaning::Money);
        let result = field_to_display(&field);
        assert_eq!(result["type"], "Text");
        assert_eq!(result["data_path"], "/data/total");
    }

    #[test]
    fn display_status_produces_badge_default() {
        let field = make_field("status", FieldMeaning::Status);
        let result = field_to_display(&field);
        assert_eq!(result["type"], "Badge");
        assert_eq!(result["variant"], "default");
        assert_eq!(result["key"], "status-badge");
    }

    #[test]
    fn display_category_produces_badge_secondary() {
        let field = make_field("category", FieldMeaning::Category);
        let result = field_to_display(&field);
        assert_eq!(result["type"], "Badge");
        assert_eq!(result["variant"], "secondary");
    }

    #[test]
    fn display_image_url_produces_avatar() {
        let field = make_field("avatar", FieldMeaning::ImageUrl);
        let result = field_to_display(&field);
        assert_eq!(result["type"], "Avatar");
        assert_eq!(result["key"], "avatar-avatar");
        assert_eq!(result["data_path"], "/data/avatar");
    }

    #[test]
    fn display_percentage_produces_progress() {
        let field = make_field("completion", FieldMeaning::Percentage);
        let result = field_to_display(&field);
        assert_eq!(result["type"], "Progress");
        assert_eq!(result["data_path"], "/data/completion");
    }

    #[test]
    fn display_foreign_key_returns_null() {
        let field = make_field("user_id", FieldMeaning::ForeignKey);
        assert!(field_to_display(&field).is_null());
    }

    #[test]
    fn display_sensitive_returns_null() {
        let field = make_field("password", FieldMeaning::Sensitive);
        assert!(field_to_display(&field).is_null());
    }

    #[test]
    fn display_boolean_produces_badge_outline() {
        let field = make_field("is_active", FieldMeaning::Boolean);
        let result = field_to_display(&field);
        assert_eq!(result["type"], "Badge");
        assert_eq!(result["variant"], "outline");
    }

    #[test]
    fn display_identifier_produces_text() {
        let field = make_field("id", FieldMeaning::Identifier);
        let result = field_to_display(&field);
        assert_eq!(result["type"], "Text");
        assert_eq!(result["element"], "span");
    }

    #[test]
    fn display_custom_produces_text() {
        let field = make_field("rating", FieldMeaning::Custom("rating".into()));
        let result = field_to_display(&field);
        assert_eq!(result["type"], "Text");
        assert_eq!(result["data_path"], "/data/rating");
    }

    // -- field_to_input tests --

    #[test]
    fn input_email_produces_email_type() {
        let field = make_field("email", FieldMeaning::Email);
        let result = field_to_input(&field);
        assert_eq!(result["type"], "Input");
        assert_eq!(result["input_type"], "email");
        assert_eq!(result["name"], "email");
        assert_eq!(result["data_path"], "/data/email");
    }

    #[test]
    fn input_boolean_produces_switch() {
        let field = make_field("is_active", FieldMeaning::Boolean);
        let result = field_to_input(&field);
        assert_eq!(result["type"], "Switch");
        assert_eq!(result["name"], "is_active");
        assert_eq!(result["label"], "Is Active");
    }

    #[test]
    fn input_sensitive_produces_password_no_data_path() {
        let field = make_field("password", FieldMeaning::Sensitive);
        let result = field_to_input(&field);
        assert_eq!(result["type"], "Input");
        assert_eq!(result["input_type"], "password");
        assert!(result.get("data_path").is_none());
    }

    #[test]
    fn input_foreign_key_produces_select() {
        let field = make_field("user_id", FieldMeaning::ForeignKey);
        let result = field_to_input(&field);
        assert_eq!(result["type"], "Select");
        assert!(result["options"].is_array());
    }

    #[test]
    fn input_money_produces_number_with_step() {
        let field = make_field("total", FieldMeaning::Money);
        let result = field_to_input(&field);
        assert_eq!(result["type"], "Input");
        assert_eq!(result["input_type"], "number");
        assert_eq!(result["step"], "0.01");
    }

    #[test]
    fn input_percentage_produces_number_with_min_max() {
        let field = make_field("completion", FieldMeaning::Percentage);
        let result = field_to_input(&field);
        assert_eq!(result["type"], "Input");
        assert_eq!(result["input_type"], "number");
        assert_eq!(result["min"], "0");
        assert_eq!(result["max"], "100");
    }

    #[test]
    fn input_free_text_produces_textarea() {
        let field = make_field("description", FieldMeaning::FreeText);
        let result = field_to_input(&field);
        assert_eq!(result["type"], "Input");
        assert_eq!(result["input_type"], "textarea");
    }

    #[test]
    fn input_identifier_produces_hidden() {
        let field = make_field("id", FieldMeaning::Identifier);
        let result = field_to_input(&field);
        assert_eq!(result["type"], "Input");
        assert_eq!(result["input_type"], "hidden");
    }

    #[test]
    fn input_created_at_produces_disabled_text() {
        let field = make_field("created_at", FieldMeaning::CreatedAt);
        let result = field_to_input(&field);
        assert_eq!(result["type"], "Input");
        assert_eq!(result["input_type"], "text");
        assert_eq!(result["disabled"], true);
    }

    #[test]
    fn input_status_produces_select() {
        let field = make_field("status", FieldMeaning::Status);
        let result = field_to_input(&field);
        assert_eq!(result["type"], "Select");
        assert!(result["options"].is_array());
    }

    #[test]
    fn input_required_field_has_required_true() {
        let field = make_field("email", FieldMeaning::Email);
        let result = field_to_input(&field);
        assert_eq!(result["required"], true);
    }

    #[test]
    fn input_optional_field_has_required_false() {
        let mut field = make_field("notes", FieldMeaning::FreeText);
        field.required = false;
        let result = field_to_input(&field);
        assert_eq!(result["required"], false);
    }

    // -- field_to_column tests --

    #[test]
    fn column_money_has_currency_format() {
        let field = make_field("total", FieldMeaning::Money);
        let result = field_to_column(&field);
        assert_eq!(result["key"], "total");
        assert_eq!(result["label"], "Total");
        assert_eq!(result["format"], "currency");
    }

    #[test]
    fn column_datetime_has_datetime_format() {
        let field = make_field("expires_at", FieldMeaning::DateTime);
        let result = field_to_column(&field);
        assert_eq!(result["format"], "datetime");
    }

    #[test]
    fn column_created_at_has_datetime_format() {
        let field = make_field("created_at", FieldMeaning::CreatedAt);
        let result = field_to_column(&field);
        assert_eq!(result["format"], "datetime");
    }

    #[test]
    fn column_boolean_has_boolean_format() {
        let field = make_field("is_active", FieldMeaning::Boolean);
        let result = field_to_column(&field);
        assert_eq!(result["format"], "boolean");
    }

    #[test]
    fn column_entity_name_has_no_format() {
        let field = make_field("title", FieldMeaning::EntityName);
        let result = field_to_column(&field);
        assert_eq!(result["key"], "title");
        assert_eq!(result["label"], "Title");
        assert!(result.get("format").is_none());
    }

    #[test]
    fn column_status_has_no_format() {
        let field = make_field("status", FieldMeaning::Status);
        let result = field_to_column(&field);
        assert!(result.get("format").is_none());
    }
}
