//! Slot-template resolution: theme overrides over built-in defaults.

use ferro_projections::Intent;
use ferro_theme::{IntentModeTemplates, IntentSlotTemplate, ThemeTemplates};

/// Render mode: reading data vs entering data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Read-oriented render.
    Display,
    /// Form render (Collect).
    Input,
}

fn slots(names: &[&str]) -> IntentSlotTemplate {
    IntentSlotTemplate {
        slots: names.iter().map(|s| s.to_string()).collect(),
        layout: None,
    }
}

fn default_template(intent: &Intent, mode: Mode) -> IntentSlotTemplate {
    match (intent, mode) {
        (Intent::Collect, _) | (_, Mode::Input) => slots(&["title", "fields", "actions"]),
        (Intent::Focus, _) => slots(&["title", "fields", "actions"]),
        (Intent::Process, _) => slots(&["title", "body"]),
        (Intent::Summarize, _) => slots(&["title", "stats"]),
        (Intent::Analyze, _) => slots(&["title", "stats", "fields"]),
        (Intent::Track, _) => slots(&["title", "fields"]),
        // Browse and Custom fall back to the browse shape.
        _ => slots(&["title", "fields", "pagination"]),
    }
}

fn override_for<'a>(
    intent: &Intent,
    overrides: &'a ThemeTemplates,
) -> Option<&'a IntentModeTemplates> {
    match intent {
        Intent::Browse => overrides.browse.as_ref(),
        Intent::Focus => overrides.focus.as_ref(),
        Intent::Collect => overrides.collect.as_ref(),
        Intent::Process => overrides.process.as_ref(),
        Intent::Summarize => overrides.summarize.as_ref(),
        Intent::Analyze => overrides.analyze.as_ref(),
        Intent::Track => overrides.track.as_ref(),
        Intent::Custom(_) => None,
    }
}

/// Resolves the slot template for an intent: a non-empty theme override wins,
/// otherwise the built-in default applies.
pub fn resolve_template(
    intent: &Intent,
    mode: Mode,
    overrides: Option<&ThemeTemplates>,
) -> IntentSlotTemplate {
    if let Some(modes) = overrides.and_then(|o| override_for(intent, o)) {
        let t = match mode {
            Mode::Display => &modes.display,
            Mode::Input => &modes.input,
        };
        if !t.slots.is_empty() {
            return t.clone();
        }
    }
    default_template(intent, mode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::Intent;
    use ferro_theme::{IntentModeTemplates, IntentSlotTemplate, ThemeTemplates};

    #[test]
    fn default_templates_per_intent() {
        assert_eq!(
            resolve_template(&Intent::Browse, Mode::Display, None).slots,
            vec!["title", "fields", "pagination"]
        );
        assert_eq!(
            resolve_template(&Intent::Focus, Mode::Display, None).slots,
            vec!["title", "fields", "actions"]
        );
        assert_eq!(
            resolve_template(&Intent::Collect, Mode::Input, None).slots,
            vec!["title", "fields", "actions"]
        );
        assert_eq!(
            resolve_template(&Intent::Process, Mode::Display, None).slots,
            vec!["title", "body"]
        );
        assert_eq!(
            resolve_template(&Intent::Summarize, Mode::Display, None).slots,
            vec!["title", "stats"]
        );
        assert_eq!(
            resolve_template(&Intent::Analyze, Mode::Display, None).slots,
            vec!["title", "stats", "fields"]
        );
        assert_eq!(
            resolve_template(&Intent::Track, Mode::Display, None).slots,
            vec!["title", "fields"]
        );
    }

    #[test]
    fn custom_intent_falls_back_to_browse() {
        let t = resolve_template(&Intent::Custom("inbox".into()), Mode::Display, None);
        assert_eq!(t.slots, vec!["title", "fields", "pagination"]);
    }

    #[test]
    fn theme_override_wins_when_present() {
        let overrides = ThemeTemplates {
            browse: Some(IntentModeTemplates {
                display: IntentSlotTemplate {
                    slots: vec!["title".into(), "fields".into()],
                    layout: None,
                },
                input: IntentSlotTemplate::default(),
            }),
            ..Default::default()
        };
        let t = resolve_template(&Intent::Browse, Mode::Display, Some(&overrides));
        assert_eq!(t.slots, vec!["title", "fields"]);
        // Focus has no override → default applies.
        let f = resolve_template(&Intent::Focus, Mode::Display, Some(&overrides));
        assert_eq!(f.slots, vec!["title", "fields", "actions"]);
    }
}
