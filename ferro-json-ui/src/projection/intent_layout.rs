//! Intent → layout dispatch for the schema-driven projection pipeline.
//!
//! Provides the built-in `IntentModeTemplates` per `Intent` variant (D-05)
//! and a helper to pick theme-supplied overrides from `ThemeTemplates`.
//!
//! The slot vocabulary is the 8-slot vocabulary defined by `ferro-theme`
//! (`title`, `body`, `fields`, `actions`, `relationships`, `pagination`,
//! `metadata`, `stats`). `Spec::from_service_def` consumes one slot list
//! per projection, dispatching each slot to a concrete emit function in
//! `builder.rs`.
//!
//! Input-mode templates are always `IntentSlotTemplate::default()` — per D-11
//! `RenderMode::Input` collapses every intent to a Form at the builder level
//! and never consults these templates.

use ferro_projections::Intent;
use ferro_theme::{IntentModeTemplates, IntentSlotTemplate, ThemeTemplates};

/// Look up a theme-supplied `IntentModeTemplates` override for the given
/// intent, or `None` if the theme does not define one for this intent variant.
///
/// `Intent::Custom(_)` always returns `None` (theme overrides are only
/// defined for the seven structural intents).
pub fn pick_intent_template<'a>(
    templates: &'a ThemeTemplates,
    intent: &Intent,
) -> Option<&'a IntentModeTemplates> {
    match intent {
        Intent::Browse => templates.browse.as_ref(),
        Intent::Focus => templates.focus.as_ref(),
        Intent::Collect => templates.collect.as_ref(),
        Intent::Process => templates.process.as_ref(),
        Intent::Summarize => templates.summarize.as_ref(),
        Intent::Analyze => templates.analyze.as_ref(),
        Intent::Track => templates.track.as_ref(),
        Intent::Custom(_) => None,
    }
}

/// Ready-made [`ThemeTemplates`] overriding the **Collect** intent to render
/// with the Register layout. Supply via `VisualContext.templates` to project a
/// `ServiceDef` into a register composition (fill-viewport Grid + Form +
/// TileGrid + SelectionPanel).
///
/// The `slots` list is informational: `emit_register_root` derives the full
/// element tree from the `ServiceDef` directly and ignores slot granularity
/// (same convention as `emit_datatable_root`). The built-in
/// `default_template(Collect)` (Form layout) is unaffected — existing Collect
/// projections keep rendering as forms.
pub fn register_template() -> ThemeTemplates {
    // Stub for TDD RED phase — returns incorrect value so the test can fail.
    // Replaced with correct implementation in the GREEN commit.
    ThemeTemplates {
        collect: None,
        browse: None,
        focus: None,
        process: None,
        summarize: None,
        analyze: None,
        track: None,
    }
}

/// Built-in `IntentModeTemplates` per structural intent.
///
/// The Display-mode slot baselines match D-05:
///
/// | Intent     | Layout         | Slots                                             |
/// |------------|----------------|---------------------------------------------------|
/// | Browse     | `DataTable`    | title, fields, pagination                         |
/// | Focus      | `Card`         | title, fields, relationships, actions             |
/// | Collect    | `Form`         | title, fields, actions                            |
/// | Process    | `KanbanBoard`  | title, body, actions                              |
/// | Summarize  | `StatCard`     | title, stats, metadata                            |
/// | Analyze    | `Card`         | title, body, metadata                             |
/// | Track      | `DataTable`    | title, fields, metadata                           |
/// | Custom     | `Card`         | title, fields                                     |
///
/// Input-mode is left empty; D-11 collapses Input to a Form in `builder.rs`.
pub fn default_template(intent: &Intent) -> IntentModeTemplates {
    match intent {
        Intent::Browse => IntentModeTemplates {
            display: IntentSlotTemplate {
                slots: vec!["title".into(), "fields".into(), "pagination".into()],
                layout: Some("DataTable".into()),
            },
            input: IntentSlotTemplate::default(),
        },
        Intent::Focus => IntentModeTemplates {
            display: IntentSlotTemplate {
                slots: vec![
                    "title".into(),
                    "fields".into(),
                    "relationships".into(),
                    "actions".into(),
                ],
                layout: Some("Card".into()),
            },
            input: IntentSlotTemplate::default(),
        },
        Intent::Collect => IntentModeTemplates {
            display: IntentSlotTemplate {
                slots: vec!["title".into(), "fields".into(), "actions".into()],
                layout: Some("Form".into()),
            },
            input: IntentSlotTemplate::default(),
        },
        Intent::Process => IntentModeTemplates {
            display: IntentSlotTemplate {
                slots: vec!["title".into(), "body".into(), "actions".into()],
                layout: Some("KanbanBoard".into()),
            },
            input: IntentSlotTemplate::default(),
        },
        Intent::Summarize => IntentModeTemplates {
            display: IntentSlotTemplate {
                slots: vec!["title".into(), "stats".into(), "metadata".into()],
                layout: Some("StatCard".into()),
            },
            input: IntentSlotTemplate::default(),
        },
        Intent::Analyze => IntentModeTemplates {
            display: IntentSlotTemplate {
                slots: vec!["title".into(), "body".into(), "metadata".into()],
                layout: Some("Card".into()),
            },
            input: IntentSlotTemplate::default(),
        },
        Intent::Track => IntentModeTemplates {
            display: IntentSlotTemplate {
                slots: vec!["title".into(), "fields".into(), "metadata".into()],
                layout: Some("DataTable".into()),
            },
            input: IntentSlotTemplate::default(),
        },
        Intent::Custom(_) => IntentModeTemplates {
            display: IntentSlotTemplate {
                slots: vec!["title".into(), "fields".into()],
                layout: Some("Card".into()),
            },
            input: IntentSlotTemplate::default(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_template_browse_uses_data_table() {
        let t = default_template(&Intent::Browse);
        assert_eq!(t.display.layout.as_deref(), Some("DataTable"));
        assert_eq!(
            t.display.slots,
            vec![
                "title".to_string(),
                "fields".to_string(),
                "pagination".to_string()
            ]
        );
        assert!(t.input.slots.is_empty());
    }

    #[test]
    fn default_template_focus_has_relationships_slot() {
        let t = default_template(&Intent::Focus);
        assert_eq!(t.display.layout.as_deref(), Some("Card"));
        assert!(t.display.slots.contains(&"relationships".to_string()));
    }

    #[test]
    fn default_template_all_intents_have_non_empty_display_slots() {
        for intent in [
            Intent::Browse,
            Intent::Focus,
            Intent::Collect,
            Intent::Process,
            Intent::Summarize,
            Intent::Analyze,
            Intent::Track,
            Intent::Custom("anything".into()),
        ] {
            let t = default_template(&intent);
            assert!(
                !t.display.slots.is_empty(),
                "intent {intent:?} has empty display slots"
            );
            assert!(
                t.display.layout.is_some(),
                "intent {intent:?} has no outer container layout"
            );
        }
    }

    #[test]
    fn pick_intent_template_returns_override_when_present() {
        let templates = ThemeTemplates {
            browse: Some(IntentModeTemplates {
                display: IntentSlotTemplate {
                    slots: vec!["title".into()],
                    layout: Some("CustomTable".into()),
                },
                input: IntentSlotTemplate::default(),
            }),
            focus: None,
            collect: None,
            process: None,
            summarize: None,
            analyze: None,
            track: None,
        };
        let picked = pick_intent_template(&templates, &Intent::Browse);
        assert!(picked.is_some());
        assert_eq!(
            picked.unwrap().display.layout.as_deref(),
            Some("CustomTable")
        );

        let not_picked = pick_intent_template(&templates, &Intent::Focus);
        assert!(not_picked.is_none());
    }

    #[test]
    fn register_template_overrides_collect() {
        let t = register_template();
        let collect = t.collect.expect("register_template must set collect to Some");
        assert_eq!(
            collect.display.layout.as_deref(),
            Some("Register"),
            "register_template must set Collect display layout to Register"
        );
        // Every non-Collect intent must be None.
        assert!(t.browse.is_none(), "browse must be None");
        assert!(t.focus.is_none(), "focus must be None");
        assert!(t.process.is_none(), "process must be None");
        assert!(t.summarize.is_none(), "summarize must be None");
        assert!(t.analyze.is_none(), "analyze must be None");
        assert!(t.track.is_none(), "track must be None");
        // Regression: the built-in Collect default must remain Form.
        assert_eq!(
            default_template(&Intent::Collect).display.layout.as_deref(),
            Some("Form"),
            "default_template(Collect) must remain Form after register_template exists"
        );
    }

    #[test]
    fn pick_intent_template_custom_intent_always_none() {
        let templates = ThemeTemplates {
            browse: Some(IntentModeTemplates::default()),
            focus: None,
            collect: None,
            process: None,
            summarize: None,
            analyze: None,
            track: None,
        };
        assert!(pick_intent_template(&templates, &Intent::Custom("x".into())).is_none());
    }
}
