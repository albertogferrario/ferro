use serde::{Deserialize, Serialize};

/// A slot list and optional layout name for one render mode of one intent.
///
/// Slots are ordered names from the fixed vocabulary:
/// `title`, `body`, `fields`, `actions`, `relationships`, `pagination`, `metadata`, `stats`.
/// The `layout` field names a component to use as the outer container
/// (e.g., `"Table"`, `"Card"`, `"Form"`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntentSlotTemplate {
    /// Ordered slot names to include in this render.
    #[serde(default)]
    pub slots: Vec<String>,

    /// Optional outer container component name.
    #[serde(default)]
    pub layout: Option<String>,
}

/// Display and input mode templates for a single intent.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntentModeTemplates {
    /// Template used when rendering data for reading (list, detail, dashboard views).
    #[serde(default)]
    pub display: IntentSlotTemplate,

    /// Template used when rendering a form for data entry or editing.
    #[serde(default)]
    pub input: IntentSlotTemplate,
}

/// Optional intent template overrides for all 7 structural intents.
///
/// Missing intents default to `None`, meaning the built-in Rust renderer
/// handles layout for those intents unchanged.
///
/// ## Example (theme.json)
///
/// ```json
/// {
///   "browse": {
///     "display": { "slots": ["title", "fields", "pagination"], "layout": "Table" }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeTemplates {
    /// Template for the Browse intent (list/grid of records).
    #[serde(default)]
    pub browse: Option<IntentModeTemplates>,

    /// Template for the Focus intent (single record detail).
    #[serde(default)]
    pub focus: Option<IntentModeTemplates>,

    /// Template for the Collect intent (data entry forms).
    #[serde(default)]
    pub collect: Option<IntentModeTemplates>,

    /// Template for the Process intent (workflow / state-machine views).
    #[serde(default)]
    pub process: Option<IntentModeTemplates>,

    /// Template for the Summarize intent (metrics / dashboards).
    #[serde(default)]
    pub summarize: Option<IntentModeTemplates>,

    /// Template for the Analyze intent (data analysis / reporting).
    #[serde(default)]
    pub analyze: Option<IntentModeTemplates>,

    /// Template for the Track intent (timeline / audit-log views).
    #[serde(default)]
    pub track: Option<IntentModeTemplates>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intent_slot_template_default_has_empty_slots_and_none_layout() {
        let t = IntentSlotTemplate::default();
        assert!(t.slots.is_empty());
        assert!(t.layout.is_none());
    }

    #[test]
    fn intent_mode_templates_default_has_empty_display_and_input() {
        let t = IntentModeTemplates::default();
        assert!(t.display.slots.is_empty());
        assert!(t.display.layout.is_none());
        assert!(t.input.slots.is_empty());
        assert!(t.input.layout.is_none());
    }

    #[test]
    fn theme_templates_deserializes_empty_json_to_all_none() {
        let t: ThemeTemplates = serde_json::from_str("{}").unwrap();
        assert!(t.browse.is_none());
        assert!(t.focus.is_none());
        assert!(t.collect.is_none());
        assert!(t.process.is_none());
        assert!(t.summarize.is_none());
        assert!(t.analyze.is_none());
        assert!(t.track.is_none());
    }

    #[test]
    fn theme_templates_deserializes_partial_json_with_other_fields_none() {
        let json = r#"{"browse": {"display": {"slots": ["title", "fields"]}}}"#;
        let t: ThemeTemplates = serde_json::from_str(json).unwrap();
        assert!(t.browse.is_some());
        let browse = t.browse.unwrap();
        assert_eq!(browse.display.slots, vec!["title", "fields"]);
        assert!(browse.display.layout.is_none());
        assert!(t.focus.is_none());
        assert!(t.collect.is_none());
        assert!(t.process.is_none());
        assert!(t.summarize.is_none());
        assert!(t.analyze.is_none());
        assert!(t.track.is_none());
    }

    #[test]
    fn theme_templates_deserializes_full_json_with_all_7_intents() {
        let json = r#"{
            "browse": {"display": {"slots": ["title"], "layout": "Table"}, "input": {"slots": ["fields"]}},
            "focus": {"display": {"slots": ["title", "body"]}},
            "collect": {"input": {"slots": ["fields", "actions"], "layout": "Form"}},
            "process": {"display": {"slots": ["title", "actions"]}},
            "summarize": {"display": {"slots": ["stats", "metadata"]}},
            "analyze": {"display": {"slots": ["fields", "stats"]}},
            "track": {"display": {"slots": ["title", "metadata", "pagination"]}}
        }"#;
        let t: ThemeTemplates = serde_json::from_str(json).unwrap();
        assert!(t.browse.is_some());
        assert!(t.focus.is_some());
        assert!(t.collect.is_some());
        assert!(t.process.is_some());
        assert!(t.summarize.is_some());
        assert!(t.analyze.is_some());
        assert!(t.track.is_some());
        let browse = t.browse.unwrap();
        assert_eq!(browse.display.layout, Some("Table".to_string()));
        assert_eq!(browse.display.slots, vec!["title"]);
        assert_eq!(browse.input.slots, vec!["fields"]);
    }

    #[test]
    fn theme_templates_serde_round_trip_preserves_all_fields() {
        let original = ThemeTemplates {
            browse: Some(IntentModeTemplates {
                display: IntentSlotTemplate {
                    slots: vec!["title".to_string(), "fields".to_string()],
                    layout: Some("Table".to_string()),
                },
                input: IntentSlotTemplate::default(),
            }),
            focus: Some(IntentModeTemplates::default()),
            collect: None,
            process: None,
            summarize: None,
            analyze: None,
            track: None,
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: ThemeTemplates = serde_json::from_str(&json).unwrap();
        assert!(restored.browse.is_some());
        let browse = restored.browse.unwrap();
        assert_eq!(browse.display.slots, vec!["title", "fields"]);
        assert_eq!(browse.display.layout, Some("Table".to_string()));
        assert!(restored.focus.is_some());
        assert!(restored.collect.is_none());
    }
}
