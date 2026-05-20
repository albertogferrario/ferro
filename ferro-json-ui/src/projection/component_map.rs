//! Schema-driven meaning → component dispatch for the projection pipeline.
//!
//! Replaces the hardcoded `field_to_display` / `field_to_input` / `field_to_column`
//! match arms in `field_map.rs` (deleted in Plan 03) with a single auditable
//! dispatch function (`lookup_meaning`) plus typed `*Props` builders that are
//! cross-validated against `global_catalog().components` at test time (D-13).
//!
//! Relationships are handled in parallel by `RELATIONSHIP_COMPONENT_TABLE` per D-09.
//! `NavigationHint::Tab` is lifted to a Tabs container at intent-layout time;
//! see `builder.rs` `emit_relationships`.
//!
//! Per D-15 plugins are out of scope for auto-projection in Phase 117.1.

use ferro_projections::render::field_display_name;
use ferro_projections::{FieldDef, FieldMeaning, NavigationHint, RelationshipDef};

use crate::component::{
    AvatarProps, BadgeProps, BadgeVariant, ButtonProps, ButtonVariant, Column, ColumnFormat,
    DescriptionItem, InputProps, InputType, ProgressProps, SelectOption, SelectProps, SwitchProps,
    TextElement, TextProps,
};

/// Per-meaning component choices for the three rendering modes.
///
/// Each `ComponentChoice` row names the built-in component that renders the
/// field in Display, Input, and column-cell roles. `None` = omit.
#[derive(Debug, Clone, Copy)]
pub struct ComponentChoice {
    /// Built-in component type name for Display mode. `None` = skip field entirely.
    pub display: Option<&'static str>,
    /// Built-in component type name for Input mode. `None` = no form control emitted.
    pub input: Option<&'static str>,
    /// Whether to include this field as a DataTable column. `None` = exclude.
    /// (Not a component name — the actual `Column` struct is built by
    /// `build_column_for_field`.)
    pub column: Option<()>,
}

const FALLBACK: ComponentChoice = ComponentChoice {
    display: Some("Text"),
    input: Some("Input"),
    column: Some(()),
};

/// Dispatch a `FieldMeaning` to the triple of component choices for
/// Display / Input / column roles. Replaces the three parallel match
/// functions that used to live in `field_map.rs`.
///
/// `FieldMeaning::Custom(_)` falls back to Text / Input / column (both
/// `Text` and `Input` are catalog-verified by the drift-guard test).
pub fn lookup_meaning(meaning: &FieldMeaning) -> ComponentChoice {
    match meaning {
        FieldMeaning::Identifier => ComponentChoice {
            display: Some("Text"),
            input: Some("Input"),
            column: None,
        },
        FieldMeaning::ForeignKey => ComponentChoice {
            display: None,
            input: Some("Select"),
            column: None,
        },
        FieldMeaning::EntityName => ComponentChoice {
            display: Some("Text"),
            input: Some("Input"),
            column: Some(()),
        },
        FieldMeaning::Email => ComponentChoice {
            display: Some("Text"),
            input: Some("Input"),
            column: Some(()),
        },
        FieldMeaning::Phone => ComponentChoice {
            display: Some("Text"),
            input: Some("Input"),
            column: Some(()),
        },
        FieldMeaning::Url => ComponentChoice {
            display: Some("Text"),
            input: Some("Input"),
            column: Some(()),
        },
        FieldMeaning::ImageUrl => ComponentChoice {
            display: Some("Avatar"),
            input: Some("Input"),
            column: None,
        },
        FieldMeaning::Money => ComponentChoice {
            display: Some("Text"),
            input: Some("Input"),
            column: Some(()),
        },
        FieldMeaning::Percentage => ComponentChoice {
            display: Some("Progress"),
            input: Some("Input"),
            column: Some(()),
        },
        FieldMeaning::Quantity => ComponentChoice {
            display: Some("Text"),
            input: Some("Input"),
            column: Some(()),
        },
        FieldMeaning::Status => ComponentChoice {
            display: Some("Badge"),
            input: Some("Select"),
            column: Some(()),
        },
        FieldMeaning::Category => ComponentChoice {
            display: Some("Badge"),
            input: Some("Select"),
            column: Some(()),
        },
        FieldMeaning::Boolean => ComponentChoice {
            display: Some("Badge"),
            input: Some("Switch"),
            column: Some(()),
        },
        FieldMeaning::FreeText => ComponentChoice {
            display: Some("Text"),
            input: Some("Input"),
            column: Some(()),
        },
        FieldMeaning::CreatedAt => ComponentChoice {
            display: Some("Text"),
            input: Some("Input"),
            column: Some(()),
        },
        FieldMeaning::UpdatedAt => ComponentChoice {
            display: Some("Text"),
            input: Some("Input"),
            column: Some(()),
        },
        FieldMeaning::DateTime => ComponentChoice {
            display: Some("Text"),
            input: Some("Input"),
            column: Some(()),
        },
        FieldMeaning::Sensitive => ComponentChoice {
            display: None,
            input: Some("Input"),
            column: None,
        },
        FieldMeaning::Custom(_) => FALLBACK,
    }
}

/// Pick the concrete `InputType` variant that `Input` should use for this
/// field meaning.
fn input_type_for(meaning: &FieldMeaning) -> InputType {
    match meaning {
        FieldMeaning::Email => InputType::Email,
        FieldMeaning::Phone => InputType::Tel,
        FieldMeaning::Url | FieldMeaning::ImageUrl => InputType::Url,
        FieldMeaning::Sensitive => InputType::Password,
        FieldMeaning::Money | FieldMeaning::Percentage | FieldMeaning::Quantity => {
            InputType::Number
        }
        FieldMeaning::FreeText => InputType::Textarea,
        _ => InputType::Text,
    }
}

/// Pick the `BadgeVariant` for meanings that render as Badge in Display mode.
fn badge_variant_for(meaning: &FieldMeaning) -> BadgeVariant {
    match meaning {
        FieldMeaning::Status => BadgeVariant::Default,
        FieldMeaning::Category => BadgeVariant::Secondary,
        FieldMeaning::Boolean => BadgeVariant::Outline,
        _ => BadgeVariant::Default,
    }
}

/// Build `TextProps` for Display mode of a field. Caller decides the element id.
pub fn build_text_props(_field: &FieldDef) -> serde_json::Value {
    serde_json::to_value(TextProps {
        content: String::new(),
        element: TextElement::Span,
    })
    .expect("TextProps serialization cannot fail")
}

/// Build `BadgeProps` for Display mode (Status / Category / Boolean meanings).
pub fn build_badge_props(field: &FieldDef) -> serde_json::Value {
    serde_json::to_value(BadgeProps {
        label: field_display_name(&field.name),
        variant: badge_variant_for(&field.meaning),
    })
    .expect("BadgeProps serialization cannot fail")
}

/// Build `AvatarProps` for Display mode (ImageUrl meaning).
pub fn build_avatar_props(field: &FieldDef) -> serde_json::Value {
    serde_json::to_value(AvatarProps {
        src: None,
        alt: field_display_name(&field.name),
        fallback: None,
        size: None,
    })
    .expect("AvatarProps serialization cannot fail")
}

/// Build `ProgressProps` for Display mode (Percentage meaning).
pub fn build_progress_props(_field: &FieldDef) -> serde_json::Value {
    serde_json::to_value(ProgressProps {
        value: 0,
        max: None,
        label: None,
    })
    .expect("ProgressProps serialization cannot fail")
}

/// Build `InputProps` for Input mode. `input_type` is derived from the field
/// meaning. Sensitive fields omit `data_path` per field_map.rs line 257-262.
pub fn build_input_props(field: &FieldDef) -> serde_json::Value {
    let input_type = input_type_for(&field.meaning);
    let is_sensitive = matches!(field.meaning, FieldMeaning::Sensitive);
    let data_path = if is_sensitive {
        None
    } else {
        Some(format!("/data/{}", field.name))
    };
    serde_json::to_value(InputProps {
        field: field.name.clone(),
        label: field_display_name(&field.name),
        input_type,
        placeholder: None,
        required: Some(field.required),
        disabled: None,
        error: None,
        description: None,
        default_value: None,
        data_path,
        step: None,
        list: None,
        accept: None,
    })
    .expect("InputProps serialization cannot fail")
}

/// Build `SelectProps` for Input mode (ForeignKey / Status / Category).
pub fn build_select_props(field: &FieldDef) -> serde_json::Value {
    serde_json::to_value(SelectProps {
        field: field.name.clone(),
        label: field_display_name(&field.name),
        options: Vec::<SelectOption>::new(),
        placeholder: None,
        required: Some(field.required),
        disabled: None,
        error: None,
        description: None,
        default_value: None,
        data_path: Some(format!("/data/{}", field.name)),
    })
    .expect("SelectProps serialization cannot fail")
}

/// Build `SwitchProps` for Input mode (Boolean meaning).
pub fn build_switch_props(field: &FieldDef) -> serde_json::Value {
    serde_json::to_value(SwitchProps {
        field: field.name.clone(),
        label: field_display_name(&field.name),
        description: None,
        checked: None,
        data_path: Some(format!("/data/{}", field.name)),
        required: Some(field.required),
        disabled: None,
        error: None,
        action: None,
        compact: None,
    })
    .expect("SwitchProps serialization cannot fail")
}

/// Build a typed `Column` (DataTableProps.columns element) for a readable field.
pub fn build_column_for_field(field: &FieldDef) -> Column {
    let format = match &field.meaning {
        FieldMeaning::Money => Some(ColumnFormat::Currency),
        FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt | FieldMeaning::DateTime => {
            Some(ColumnFormat::DateTime)
        }
        FieldMeaning::Boolean => Some(ColumnFormat::Boolean),
        _ => None,
    };
    Column {
        key: field.name.clone(),
        label: field_display_name(&field.name),
        format,
    }
}

/// Build a typed `DescriptionItem` for Focus intent's `fields` slot.
pub fn build_description_item(field: &FieldDef) -> DescriptionItem {
    let format = match &field.meaning {
        FieldMeaning::Money => Some(ColumnFormat::Currency),
        FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt | FieldMeaning::DateTime => {
            Some(ColumnFormat::DateTime)
        }
        FieldMeaning::Boolean => Some(ColumnFormat::Boolean),
        _ => None,
    };
    DescriptionItem {
        label: field_display_name(&field.name),
        value: String::new(),
        format,
    }
}

/// NavigationHint → built-in component name for relationship rendering.
/// `Tab` is a sentinel — caller (intent-layout `emit_relationships`) groups
/// all `Tab`-hint relationships into a single `Tabs` container.
/// `Hidden` is `None` — skip entirely.
pub static RELATIONSHIP_COMPONENT_TABLE: &[(NavigationHint, Option<&'static str>)] = &[
    (NavigationHint::Inline, Some("Text")),
    (NavigationHint::Link, Some("Button")),
    (NavigationHint::Tab, Some("Tabs")),
    (NavigationHint::Nested, Some("Table")),
    (NavigationHint::Hidden, None),
];

/// Look up the built-in component name for a relationship `NavigationHint`.
/// Returns `None` when the hint is `Hidden` (skip emission).
pub fn lookup_relationship(hint: NavigationHint) -> Option<&'static str> {
    RELATIONSHIP_COMPONENT_TABLE
        .iter()
        .find(|(h, _)| *h == hint)
        .and_then(|(_, name)| *name)
}

/// Build `TextProps` for Inline relationship rendering.
pub fn build_relationship_text_props(_rel: &RelationshipDef) -> serde_json::Value {
    serde_json::to_value(TextProps {
        content: String::new(),
        element: TextElement::Span,
    })
    .expect("TextProps serialization cannot fail")
}

/// Build `ButtonProps` for Link relationship rendering. The `variant` is
/// `Link` to match the visual convention from relationship_map.rs line 29.
pub fn build_relationship_button_props(rel: &RelationshipDef) -> serde_json::Value {
    serde_json::to_value(ButtonProps {
        label: format!("{} \u{2192}", field_display_name(&rel.target)),
        variant: ButtonVariant::Link,
        size: crate::component::Size::default(),
        disabled: None,
        icon: None,
        icon_position: None,
        button_type: None,
        form: None,
    })
    .expect("ButtonProps serialization cannot fail")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;

    /// Every component name referenced by `lookup_meaning` (or
    /// `RELATIONSHIP_COMPONENT_TABLE`) MUST exist as a key in the
    /// built-in catalog. This test fails CI loudly if anyone renames a
    /// built-in component without updating the table.
    ///
    /// Uses `Catalog::build_builtins_only()` rather than `global_catalog()`
    /// so that plugin-registration test pollution (see `BadPlugin_117` in
    /// `catalog.rs` tests) does not cause false failures when the
    /// projector's drift guard runs after the plugin test.
    #[test]
    fn meaning_table_components_exist_in_catalog() {
        let cat = Catalog::build_builtins_only().expect("build builtins");
        let all_meanings = [
            FieldMeaning::Identifier,
            FieldMeaning::ForeignKey,
            FieldMeaning::EntityName,
            FieldMeaning::Email,
            FieldMeaning::Phone,
            FieldMeaning::Url,
            FieldMeaning::ImageUrl,
            FieldMeaning::Money,
            FieldMeaning::Percentage,
            FieldMeaning::Quantity,
            FieldMeaning::Status,
            FieldMeaning::Category,
            FieldMeaning::Boolean,
            FieldMeaning::FreeText,
            FieldMeaning::CreatedAt,
            FieldMeaning::UpdatedAt,
            FieldMeaning::DateTime,
            FieldMeaning::Sensitive,
        ];
        for meaning in &all_meanings {
            let choice = lookup_meaning(meaning);
            for name in [choice.display, choice.input].into_iter().flatten() {
                assert!(
                    cat.components.contains_key(name),
                    "MEANING_COMPONENT_TABLE references unknown component '{name}' \
                     for meaning {meaning:?}"
                );
            }
        }
        for (hint, name_opt) in RELATIONSHIP_COMPONENT_TABLE {
            if let Some(name) = name_opt {
                assert!(
                    cat.components.contains_key(*name),
                    "RELATIONSHIP_COMPONENT_TABLE references unknown component '{name}' \
                     for hint {hint:?}"
                );
            }
        }
    }

    /// Fallback coverage for `FieldMeaning::Custom(_)` — must use FALLBACK
    /// ("Text" / "Input") components, both of which exist in the catalog.
    #[test]
    fn custom_meaning_fallback_uses_catalog_components() {
        let cat = Catalog::build_builtins_only().expect("build builtins");
        let choice = lookup_meaning(&FieldMeaning::Custom("anything".into()));
        assert_eq!(choice.display, Some("Text"));
        assert_eq!(choice.input, Some("Input"));
        assert!(cat.components.contains_key("Text"));
        assert!(cat.components.contains_key("Input"));
    }
}
