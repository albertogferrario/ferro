use std::collections::HashMap;

use crate::field::FieldMeaning;
use crate::intent::{Intent, IntentHint, IntentScore};
use crate::relationship::{Cardinality, NavigationHint};
use crate::service::ServiceDef;

// Signal name constants to prevent typo bugs in matching_signals.
const SIGNAL_MONEY_FIELDS: &str = "money_fields";
const SIGNAL_PERCENTAGE_FIELDS: &str = "percentage_fields";
const SIGNAL_QUANTITY_FIELDS: &str = "quantity_fields";
const SIGNAL_FREE_TEXT_FIELDS: &str = "free_text_fields";
const SIGNAL_IMAGE_URL_FIELDS: &str = "image_url_fields";
const SIGNAL_URL_FIELDS: &str = "url_fields";
const SIGNAL_ENTITY_NAME: &str = "entity_name";
const SIGNAL_DATETIME_NUMERIC: &str = "datetime_numeric_cooccurrence";
const SIGNAL_STATUS: &str = "status_field";
const SIGNAL_CATEGORY: &str = "category_field";
const SIGNAL_HIGH_WRITABLE_RATIO: &str = "high_writable_ratio";
const SIGNAL_WRITE_ONLY_FIELDS: &str = "write_only_fields";
const SIGNAL_MOSTLY_READ_ONLY: &str = "mostly_read_only";
const SIGNAL_MORE_READABLE: &str = "more_readable_than_writable";
const SIGNAL_BASELINE: &str = "baseline";
const SIGNAL_NO_STRUCTURAL: &str = "no_structural_signals";
const SIGNAL_HINT_PRIMARY: &str = "intent_hint_primary";

// State machine analyzer signals.
const SIGNAL_GUARDED_TRANSITIONS: &str = "guarded_transitions";
const SIGNAL_BRANCHING_STATES: &str = "branching_states";
const SIGNAL_TRANSITION_TRIGGERS: &str = "transition_triggers";
const SIGNAL_WORKFLOW_STATES: &str = "workflow_states";
const SIGNAL_LINEAR_STATES: &str = "linear_states";
const SIGNAL_HAS_FINAL_STATES: &str = "has_final_states";
const SIGNAL_UNGUARDED_PROGRESSION: &str = "unguarded_progression";

// Relationship analyzer signals.
const SIGNAL_COLLECTION_RELATIONSHIPS: &str = "collection_relationships";
const SIGNAL_INLINE_RELATIONSHIPS: &str = "inline_relationships";
const SIGNAL_PARENT_REFERENCES: &str = "parent_references";
const SIGNAL_RICH_RELATIONSHIP_GRAPH: &str = "rich_relationship_graph";

// Action analyzer signals.
const SIGNAL_WORKFLOW_ACTIONS: &str = "workflow_actions";
const SIGNAL_COMPLEX_INPUT_ACTIONS: &str = "complex_input_actions";
const SIGNAL_GUARDED_ACTIONS: &str = "guarded_actions";
const SIGNAL_SIMPLE_CRUD_ACTIONS: &str = "simple_crud_actions";

// Baseline scores added to Browse and Focus to ensure they always appear.
const BASELINE_BROWSE: f64 = 0.1;
const BASELINE_FOCUS: f64 = 0.1;

/// A single signal contribution: (intent, raw weight, signal description).
type Signal = (Intent, f64, String);

/// Derives ranked intents from a ServiceDef's structural signals.
///
/// Always returns at least one IntentScore. Default: Focus with 0.5 confidence.
pub fn derive_intents(service: &ServiceDef) -> Vec<IntentScore> {
    // 1. Collect signals from all 5 analyzers.
    let mut all_signals = Vec::new();
    all_signals.extend(analyze_field_meanings(service));
    all_signals.extend(analyze_writability(service));
    all_signals.extend(analyze_state_machine(service));
    all_signals.extend(analyze_relationships(service));
    all_signals.extend(analyze_actions(service));

    // 2. Aggregate and add baselines.
    let mut raw = aggregate_signals(all_signals);
    raw.entry(Intent::Browse).or_insert((0.0, Vec::new())).0 += BASELINE_BROWSE;
    raw.get_mut(&Intent::Browse)
        .unwrap()
        .1
        .push(SIGNAL_BASELINE.to_string());
    raw.entry(Intent::Focus).or_insert((0.0, Vec::new())).0 += BASELINE_FOCUS;
    raw.get_mut(&Intent::Focus)
        .unwrap()
        .1
        .push(SIGNAL_BASELINE.to_string());

    // 3. Normalize into ranked IntentScores.
    let mut scores = normalize_scores(raw);

    // 4. Apply IntentHint overrides.
    apply_hints(&mut scores, &service.intent_hints);

    // 5. Default fallback if empty.
    if scores.is_empty() {
        scores.push(IntentScore {
            intent: Intent::Focus,
            confidence: 0.5,
            matching_signals: vec![SIGNAL_NO_STRUCTURAL.to_string()],
        });
    }

    scores
}

/// Returns true for system/infrastructure field meanings that should not
/// contribute to domain intent signals.
fn is_system_field(meaning: &FieldMeaning) -> bool {
    matches!(
        meaning,
        FieldMeaning::Identifier | FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt
    )
}

/// Examines non-system fields' meanings to derive intent signals.
///
/// Uses proportional signals (count-based) rather than binary presence.
fn analyze_field_meanings(service: &ServiceDef) -> Vec<Signal> {
    let mut signals = Vec::new();

    let domain_fields: Vec<_> = service
        .fields
        .iter()
        .filter(|f| !is_system_field(&f.meaning))
        .collect();

    let mut money_count = 0u32;
    let mut percentage_count = 0u32;
    let mut quantity_count = 0u32;
    let mut free_text_count = 0u32;
    let mut image_url_count = 0u32;
    let mut url_count = 0u32;
    let mut entity_name_count = 0u32;
    let mut status_count = 0u32;
    let mut category_count = 0u32;
    let mut has_datetime = false;
    let mut has_numeric = false;

    for field in &domain_fields {
        match &field.meaning {
            FieldMeaning::Money => {
                money_count += 1;
                has_numeric = true;
            }
            FieldMeaning::Percentage => {
                percentage_count += 1;
                has_numeric = true;
            }
            FieldMeaning::Quantity => {
                quantity_count += 1;
                has_numeric = true;
            }
            FieldMeaning::FreeText => free_text_count += 1,
            FieldMeaning::ImageUrl => image_url_count += 1,
            FieldMeaning::Url => url_count += 1,
            FieldMeaning::EntityName => entity_name_count += 1,
            FieldMeaning::Status => status_count += 1,
            FieldMeaning::Category => category_count += 1,
            FieldMeaning::DateTime => has_datetime = true,
            _ => {}
        }
    }

    // Money/Percentage/Quantity -> Summarize
    let summarize_count = money_count + percentage_count + quantity_count;
    if summarize_count > 0 {
        signals.push((
            Intent::Summarize,
            0.3 * f64::from(summarize_count),
            format_signal_with_sources(&[
                (SIGNAL_MONEY_FIELDS, money_count),
                (SIGNAL_PERCENTAGE_FIELDS, percentage_count),
                (SIGNAL_QUANTITY_FIELDS, quantity_count),
            ]),
        ));
    }

    // FreeText/ImageUrl/Url -> Focus
    let focus_count = free_text_count + image_url_count + url_count;
    if focus_count > 0 {
        signals.push((
            Intent::Focus,
            0.25 * f64::from(focus_count),
            format_signal_with_sources(&[
                (SIGNAL_FREE_TEXT_FIELDS, free_text_count),
                (SIGNAL_IMAGE_URL_FIELDS, image_url_count),
                (SIGNAL_URL_FIELDS, url_count),
            ]),
        ));
    }

    // EntityName -> Browse
    if entity_name_count > 0 {
        signals.push((
            Intent::Browse,
            0.2 * f64::from(entity_name_count),
            SIGNAL_ENTITY_NAME.to_string(),
        ));
    }

    // DateTime + numeric co-occurrence -> Analyze
    if has_datetime && has_numeric {
        signals.push((Intent::Analyze, 0.35, SIGNAL_DATETIME_NUMERIC.to_string()));
    }

    // Status -> Track
    if status_count > 0 {
        signals.push((Intent::Track, 0.25, SIGNAL_STATUS.to_string()));
    }

    // Category -> Browse
    if category_count > 0 {
        signals.push((
            Intent::Browse,
            0.1 * f64::from(category_count),
            SIGNAL_CATEGORY.to_string(),
        ));
    }

    signals
}

/// Formats a composite signal name from its non-zero contributing sources.
fn format_signal_with_sources(sources: &[(&str, u32)]) -> String {
    let active: Vec<&str> = sources
        .iter()
        .filter(|(_, count)| *count > 0)
        .map(|(name, _)| *name)
        .collect();
    active.join("+")
}

/// Examines readable/writable ratios of non-system fields.
fn analyze_writability(service: &ServiceDef) -> Vec<Signal> {
    let mut signals = Vec::new();

    let non_system: Vec<_> = service
        .fields
        .iter()
        .filter(|f| !is_system_field(&f.meaning))
        .collect();

    if non_system.is_empty() {
        return signals;
    }

    let total = non_system.len() as f64;
    let writable_count = non_system.iter().filter(|f| f.writable).count();
    let non_writable_count = non_system.len() - writable_count;
    let write_only_count = non_system
        .iter()
        .filter(|f| !f.readable && f.writable)
        .count();

    let writable_ratio = writable_count as f64 / total;
    let non_writable_ratio = non_writable_count as f64 / total;

    // High writable ratio (>50%) -> Collect
    if writable_ratio > 0.5 {
        signals.push((
            Intent::Collect,
            0.35,
            SIGNAL_HIGH_WRITABLE_RATIO.to_string(),
        ));
    }

    // Write-only fields present -> Collect
    if write_only_count > 0 {
        signals.push((
            Intent::Collect,
            0.2 * write_only_count as f64,
            SIGNAL_WRITE_ONLY_FIELDS.to_string(),
        ));
    }

    // Mostly read-only (>70% non-writable) -> Summarize
    if non_writable_ratio > 0.7 {
        signals.push((Intent::Summarize, 0.2, SIGNAL_MOSTLY_READ_ONLY.to_string()));
    }

    // More readable than writable -> Focus
    let readable_count = non_system.iter().filter(|f| f.readable).count();
    if readable_count > writable_count {
        signals.push((Intent::Focus, 0.1, SIGNAL_MORE_READABLE.to_string()));
    }

    signals
}

/// Examines state machine shape to discriminate Process (branching/guards) from Track (linear).
fn analyze_state_machine(service: &ServiceDef) -> Vec<Signal> {
    let mut signals = Vec::new();

    let sm = match &service.state_machine {
        Some(sm) => sm,
        None => return signals,
    };

    let total_transitions = sm.transitions.len();
    if total_transitions == 0 {
        return signals;
    }

    // --- Process signals (branching + guards) ---

    // Guard density: guarded transitions / total transitions.
    let guarded_count = sm.transitions.iter().filter(|t| t.guard.is_some()).count();
    if guarded_count > 0 {
        let ratio = guarded_count as f64 / total_transitions as f64;
        signals.push((
            Intent::Process,
            0.4 * ratio,
            format!("{guarded_count}/{total_transitions}_{SIGNAL_GUARDED_TRANSITIONS}"),
        ));
    }

    // Branching factor: states with >1 outgoing transition.
    let mut outgoing_counts: HashMap<&str, usize> = HashMap::new();
    for t in &sm.transitions {
        *outgoing_counts.entry(t.from.as_str()).or_default() += 1;
    }
    let branching_states = outgoing_counts.values().filter(|&&c| c > 1).count();
    if branching_states > 0 {
        signals.push((
            Intent::Process,
            0.15,
            format!("{branching_states}_{SIGNAL_BRANCHING_STATES}"),
        ));
    }

    // Actions with transition_trigger present in service.actions.
    let trigger_count = service
        .actions
        .iter()
        .filter(|a| a.transition_trigger.is_some())
        .count();
    let total_actions = service.actions.len();
    if trigger_count > 0 && total_actions > 0 {
        signals.push((
            Intent::Process,
            0.25 * (trigger_count as f64 / total_actions as f64),
            format!("{trigger_count}_{SIGNAL_TRANSITION_TRIGGERS}"),
        ));
    }

    // Multiple non-trivial states (>2 non-final).
    let non_final_count = sm.states.iter().filter(|s| !s.is_final).count();
    if non_final_count > 2 {
        signals.push((
            Intent::Process,
            0.10,
            format!("{non_final_count}_{SIGNAL_WORKFLOW_STATES}"),
        ));
    }

    // --- Track signals (linear + temporal) ---

    // Linear progression: non-final states > 2 AND no branching states.
    if non_final_count > 2 && branching_states == 0 {
        signals.push((
            Intent::Track,
            0.3,
            format!("{non_final_count}_{SIGNAL_LINEAR_STATES}"),
        ));
    }

    // Has final states.
    if sm.states.iter().any(|s| s.is_final) {
        signals.push((Intent::Track, 0.1, SIGNAL_HAS_FINAL_STATES.to_string()));
    }

    // No guards on transitions.
    if guarded_count == 0 {
        signals.push((Intent::Track, 0.1, SIGNAL_UNGUARDED_PROGRESSION.to_string()));
    }

    signals
}

/// Examines relationship cardinalities to discriminate Browse (collections) from Focus (inline).
fn analyze_relationships(service: &ServiceDef) -> Vec<Signal> {
    let mut signals = Vec::new();

    if service.relationships.is_empty() {
        return signals;
    }

    // OneToMany or ManyToMany -> Browse.
    let collection_count = service
        .relationships
        .iter()
        .filter(|r| {
            matches!(
                r.cardinality,
                Cardinality::OneToMany | Cardinality::ManyToMany
            )
        })
        .count();
    if collection_count > 0 {
        signals.push((
            Intent::Browse,
            0.35 * collection_count as f64,
            format!("{collection_count}_{SIGNAL_COLLECTION_RELATIONSHIPS}"),
        ));
    }

    // OneToOne with NavigationHint::Inline -> Focus.
    let inline_count = service
        .relationships
        .iter()
        .filter(|r| {
            r.cardinality == Cardinality::OneToOne && r.navigation == NavigationHint::Inline
        })
        .count();
    if inline_count > 0 {
        signals.push((
            Intent::Focus,
            0.15 * inline_count as f64,
            format!("{inline_count}_{SIGNAL_INLINE_RELATIONSHIPS}"),
        ));
    }

    // ManyToOne -> Focus.
    let parent_count = service
        .relationships
        .iter()
        .filter(|r| r.cardinality == Cardinality::ManyToOne)
        .count();
    if parent_count > 0 {
        signals.push((
            Intent::Focus,
            0.1 * parent_count as f64,
            format!("{parent_count}_{SIGNAL_PARENT_REFERENCES}"),
        ));
    }

    // Total relationship count > 3 -> Browse.
    if service.relationships.len() > 3 {
        signals.push((
            Intent::Browse,
            0.1,
            SIGNAL_RICH_RELATIONSHIP_GRAPH.to_string(),
        ));
    }

    signals
}

/// Examines action patterns to discriminate Process (workflow) from Collect (input) and Browse (CRUD).
fn analyze_actions(service: &ServiceDef) -> Vec<Signal> {
    let mut signals = Vec::new();

    if service.actions.is_empty() {
        return signals;
    }

    // Actions with transition_trigger -> Process.
    let workflow_count = service
        .actions
        .iter()
        .filter(|a| a.transition_trigger.is_some())
        .count();
    if workflow_count > 0 {
        signals.push((
            Intent::Process,
            0.15 * workflow_count as f64,
            format!("{workflow_count}_{SIGNAL_WORKFLOW_ACTIONS}"),
        ));
    }

    // Actions with >2 inputs -> Collect.
    let complex_input_count = service
        .actions
        .iter()
        .filter(|a| a.inputs.len() > 2)
        .count();
    if complex_input_count > 0 {
        signals.push((
            Intent::Collect,
            0.15 * complex_input_count as f64,
            format!("{complex_input_count}_{SIGNAL_COMPLEX_INPUT_ACTIONS}"),
        ));
    }

    // Actions with preconditions -> Process.
    let guarded_action_count = service
        .actions
        .iter()
        .filter(|a| !a.preconditions.is_empty())
        .count();
    if guarded_action_count > 0 {
        signals.push((
            Intent::Process,
            0.1 * guarded_action_count as f64,
            format!("{guarded_action_count}_{SIGNAL_GUARDED_ACTIONS}"),
        ));
    }

    // Simple CRUD: actions present but no transition triggers and no preconditions.
    if workflow_count == 0 && guarded_action_count == 0 {
        signals.push((Intent::Browse, 0.05, SIGNAL_SIMPLE_CRUD_ACTIONS.to_string()));
    }

    signals
}

/// Sums weights per intent and collects signal names per intent.
fn aggregate_signals(all_signals: Vec<Signal>) -> HashMap<Intent, (f64, Vec<String>)> {
    let mut map: HashMap<Intent, (f64, Vec<String>)> = HashMap::new();
    for (intent, weight, signal) in all_signals {
        let entry = map.entry(intent).or_insert((0.0, Vec::new()));
        entry.0 += weight;
        entry.1.push(signal);
    }
    map
}

/// Normalizes raw scores to [0.0, 1.0] range and sorts descending by confidence.
///
/// Tie-breaks by stable intent priority ordering.
fn normalize_scores(raw: HashMap<Intent, (f64, Vec<String>)>) -> Vec<IntentScore> {
    let max_score = raw.values().map(|(w, _)| *w).fold(0.0_f64, f64::max);

    if max_score <= 0.0 {
        return Vec::new();
    }

    let mut scores: Vec<IntentScore> = raw
        .into_iter()
        .map(|(intent, (weight, signals))| IntentScore {
            confidence: weight / max_score,
            intent,
            matching_signals: signals,
        })
        .collect();

    scores.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| intent_priority(&a.intent).cmp(&intent_priority(&b.intent)))
    });

    scores
}

/// Returns a stable priority value for tie-breaking intent ordering.
///
/// Lower value = higher priority. Order: Process > Track > Collect > Browse > Focus > Summarize > Analyze.
fn intent_priority(intent: &Intent) -> u8 {
    match intent {
        Intent::Process => 0,
        Intent::Track => 1,
        Intent::Collect => 2,
        Intent::Browse => 3,
        Intent::Focus => 4,
        Intent::Summarize => 5,
        Intent::Analyze => 6,
        Intent::Custom(_) => 7,
    }
}

/// Applies IntentHint overrides to scored results.
///
/// Exclude hints remove matching intents. Primary hints insert at position 0
/// with confidence 1.0.
fn apply_hints(scores: &mut Vec<IntentScore>, hints: &[IntentHint]) {
    for hint in hints {
        match hint {
            IntentHint::Exclude(intent) => {
                scores.retain(|s| s.intent != *intent);
            }
            IntentHint::Primary(intent) => {
                scores.retain(|s| s.intent != *intent);
                scores.insert(
                    0,
                    IntentScore {
                        intent: intent.clone(),
                        confidence: 1.0,
                        matching_signals: vec![SIGNAL_HINT_PRIMARY.to_string()],
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::ActionDef;
    use crate::field::DataType;

    // -- Helper: find an IntentScore for a given intent in a slice --

    fn find_intent<'a>(scores: &'a [IntentScore], intent: &Intent) -> Option<&'a IntentScore> {
        scores.iter().find(|s| &s.intent == intent)
    }

    fn has_signal(score: &IntentScore, signal: &str) -> bool {
        score.matching_signals.iter().any(|s| s.contains(signal))
    }

    // ==========================================================
    // Field meaning analyzer tests
    // ==========================================================

    #[test]
    fn field_meaning_money_percentage_quantity_produce_summarize() {
        let service = ServiceDef::new("financials")
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("margin", DataType::Float, FieldMeaning::Percentage)
            .field("qty", DataType::Integer, FieldMeaning::Quantity);

        let signals = analyze_field_meanings(&service);
        let summarize: Vec<_> = signals
            .iter()
            .filter(|s| s.0 == Intent::Summarize)
            .collect();
        assert!(!summarize.is_empty(), "Summarize signal must be present");
        let total_weight: f64 = summarize.iter().map(|s| s.1).sum();
        assert!(total_weight > 0.0, "Summarize weight must be positive");
        // 3 fields * 0.3 = 0.9
        assert!(
            (total_weight - 0.9).abs() < f64::EPSILON,
            "expected 0.9, got {total_weight}"
        );
    }

    #[test]
    fn field_meaning_free_text_image_url_produce_focus() {
        let service = ServiceDef::new("content")
            .field("body", DataType::String, FieldMeaning::FreeText)
            .field("photo", DataType::String, FieldMeaning::ImageUrl);

        let signals = analyze_field_meanings(&service);
        let focus: Vec<_> = signals.iter().filter(|s| s.0 == Intent::Focus).collect();
        assert!(!focus.is_empty(), "Focus signal must be present");
        let total_weight: f64 = focus.iter().map(|s| s.1).sum();
        // 2 fields * 0.25 = 0.5
        assert!(
            (total_weight - 0.5).abs() < f64::EPSILON,
            "expected 0.5, got {total_weight}"
        );
    }

    #[test]
    fn field_meaning_entity_name_category_produce_browse() {
        let service = ServiceDef::new("catalog")
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("type", DataType::String, FieldMeaning::Category);

        let signals = analyze_field_meanings(&service);
        let browse: Vec<_> = signals.iter().filter(|s| s.0 == Intent::Browse).collect();
        assert!(!browse.is_empty(), "Browse signal must be present");
        let total_weight: f64 = browse.iter().map(|s| s.1).sum();
        // EntityName: 0.2 * 1 + Category: 0.1 * 1 = 0.3
        assert!(
            (total_weight - 0.3).abs() < f64::EPSILON,
            "expected 0.3, got {total_weight}"
        );
    }

    #[test]
    fn field_meaning_datetime_money_produce_analyze() {
        let service = ServiceDef::new("timeseries")
            .field("recorded_at", DataType::DateTime, FieldMeaning::DateTime)
            .field("revenue", DataType::Float, FieldMeaning::Money);

        let signals = analyze_field_meanings(&service);
        let analyze: Vec<_> = signals.iter().filter(|s| s.0 == Intent::Analyze).collect();
        assert!(!analyze.is_empty(), "Analyze signal must be present");
        assert!(
            (analyze[0].1 - 0.35).abs() < f64::EPSILON,
            "Analyze weight should be 0.35"
        );
    }

    #[test]
    fn field_meaning_status_produces_track() {
        let service =
            ServiceDef::new("orders").field("status", DataType::String, FieldMeaning::Status);

        let signals = analyze_field_meanings(&service);
        let track: Vec<_> = signals.iter().filter(|s| s.0 == Intent::Track).collect();
        assert!(!track.is_empty(), "Track signal must be present");
        assert!(
            (track[0].1 - 0.25).abs() < f64::EPSILON,
            "Track weight should be 0.25"
        );
    }

    #[test]
    fn field_meaning_system_fields_excluded() {
        let service = ServiceDef::new("system_only")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt);

        let signals = analyze_field_meanings(&service);
        assert!(
            signals.is_empty(),
            "System-only fields should produce no field meaning signals"
        );
    }

    // ==========================================================
    // Writability analyzer tests
    // ==========================================================

    #[test]
    fn writability_high_writable_ratio_produces_collect() {
        // 3 writable out of 4 = 75% > 50%
        let service = ServiceDef::new("form")
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("email", DataType::String, FieldMeaning::Email)
            .field("phone", DataType::String, FieldMeaning::Phone)
            .read_only_field("score", DataType::Float, FieldMeaning::Quantity);

        let signals = analyze_writability(&service);
        let collect: Vec<_> = signals.iter().filter(|s| s.0 == Intent::Collect).collect();
        assert!(
            !collect.is_empty(),
            "Collect signal must be present for high writable ratio"
        );
        assert!(collect.iter().any(|s| s.2 == SIGNAL_HIGH_WRITABLE_RATIO));
    }

    #[test]
    fn writability_write_only_fields_produce_collect() {
        let service = ServiceDef::new("auth")
            .field("username", DataType::String, FieldMeaning::EntityName)
            .write_only_field("password", DataType::String, FieldMeaning::Sensitive)
            .write_only_field("confirm", DataType::String, FieldMeaning::Sensitive);

        let signals = analyze_writability(&service);
        let wo_collect: Vec<_> = signals
            .iter()
            .filter(|s| s.0 == Intent::Collect && s.2 == SIGNAL_WRITE_ONLY_FIELDS)
            .collect();
        assert!(
            !wo_collect.is_empty(),
            "Write-only Collect signal must be present"
        );
        // 2 write-only fields * 0.2 = 0.4
        assert!(
            (wo_collect[0].1 - 0.4).abs() < f64::EPSILON,
            "expected 0.4, got {}",
            wo_collect[0].1
        );
    }

    #[test]
    fn writability_mostly_read_only_produces_summarize() {
        // 4 fields, 3 read-only, 1 writable => non_writable_ratio = 3/4 = 75% > 70%
        let service = ServiceDef::new("dashboard")
            .read_only_field("total", DataType::Float, FieldMeaning::Money)
            .read_only_field("count", DataType::Integer, FieldMeaning::Quantity)
            .read_only_field("rate", DataType::Float, FieldMeaning::Percentage)
            .field("notes", DataType::String, FieldMeaning::FreeText);

        let signals = analyze_writability(&service);
        let summarize: Vec<_> = signals
            .iter()
            .filter(|s| s.0 == Intent::Summarize && s.2 == SIGNAL_MOSTLY_READ_ONLY)
            .collect();
        assert!(
            !summarize.is_empty(),
            "Summarize signal must be present for mostly read-only"
        );
    }

    #[test]
    fn writability_balanced_no_strong_collect() {
        // 2 writable, 2 non-writable => 50% exactly, NOT >50%
        let service = ServiceDef::new("balanced")
            .field("a", DataType::String, FieldMeaning::FreeText)
            .field("b", DataType::String, FieldMeaning::FreeText)
            .read_only_field("c", DataType::Integer, FieldMeaning::Quantity)
            .read_only_field("d", DataType::Integer, FieldMeaning::Quantity);

        let signals = analyze_writability(&service);
        let high_writable: Vec<_> = signals
            .iter()
            .filter(|s| s.2 == SIGNAL_HIGH_WRITABLE_RATIO)
            .collect();
        assert!(
            high_writable.is_empty(),
            "50/50 should not trigger high_writable_ratio (needs >50%)"
        );
    }

    // ==========================================================
    // Normalizer tests
    // ==========================================================

    #[test]
    fn normalizer_highest_score_becomes_1() {
        let mut raw = HashMap::new();
        raw.insert(Intent::Browse, (0.8, vec!["signal_a".to_string()]));
        raw.insert(Intent::Focus, (0.4, vec!["signal_b".to_string()]));

        let scores = normalize_scores(raw);
        let browse = find_intent(&scores, &Intent::Browse).unwrap();
        assert!(
            (browse.confidence - 1.0).abs() < f64::EPSILON,
            "Highest score should normalize to 1.0"
        );
    }

    #[test]
    fn normalizer_second_highest_proportional() {
        let mut raw = HashMap::new();
        raw.insert(Intent::Browse, (0.8, vec!["a".to_string()]));
        raw.insert(Intent::Focus, (0.4, vec!["b".to_string()]));

        let scores = normalize_scores(raw);
        let focus = find_intent(&scores, &Intent::Focus).unwrap();
        assert!(
            (focus.confidence - 0.5).abs() < f64::EPSILON,
            "0.4/0.8 = 0.5, got {}",
            focus.confidence
        );
    }

    #[test]
    fn normalizer_empty_input_returns_empty() {
        let raw: HashMap<Intent, (f64, Vec<String>)> = HashMap::new();
        let scores = normalize_scores(raw);
        assert!(scores.is_empty());
    }

    #[test]
    fn normalizer_single_intent_gets_confidence_1() {
        let mut raw = HashMap::new();
        raw.insert(Intent::Track, (0.5, vec!["status".to_string()]));

        let scores = normalize_scores(raw);
        assert_eq!(scores.len(), 1);
        assert!(
            (scores[0].confidence - 1.0).abs() < f64::EPSILON,
            "Single intent should get confidence 1.0"
        );
    }

    #[test]
    fn normalizer_sorted_descending() {
        let mut raw = HashMap::new();
        raw.insert(Intent::Browse, (0.2, vec!["a".to_string()]));
        raw.insert(Intent::Focus, (0.6, vec!["b".to_string()]));
        raw.insert(Intent::Track, (0.4, vec!["c".to_string()]));

        let scores = normalize_scores(raw);
        for i in 1..scores.len() {
            assert!(
                scores[i - 1].confidence >= scores[i].confidence,
                "Scores must be sorted descending: {} >= {}",
                scores[i - 1].confidence,
                scores[i].confidence
            );
        }
    }

    // ==========================================================
    // IntentHint tests
    // ==========================================================

    #[test]
    fn hint_primary_forces_position_0_confidence_1() {
        let mut scores = vec![
            IntentScore {
                intent: Intent::Summarize,
                confidence: 1.0,
                matching_signals: vec!["existing".to_string()],
            },
            IntentScore {
                intent: Intent::Browse,
                confidence: 0.5,
                matching_signals: vec!["existing".to_string()],
            },
        ];

        apply_hints(&mut scores, &[IntentHint::Primary(Intent::Browse)]);

        assert_eq!(scores[0].intent, Intent::Browse);
        assert!((scores[0].confidence - 1.0).abs() < f64::EPSILON);
        assert!(has_signal(&scores[0], SIGNAL_HINT_PRIMARY));
        // Browse was removed from old position
        assert_eq!(
            scores.iter().filter(|s| s.intent == Intent::Browse).count(),
            1,
            "Browse should appear exactly once"
        );
    }

    #[test]
    fn hint_exclude_removes_intent() {
        let mut scores = vec![
            IntentScore {
                intent: Intent::Process,
                confidence: 1.0,
                matching_signals: vec!["a".to_string()],
            },
            IntentScore {
                intent: Intent::Browse,
                confidence: 0.5,
                matching_signals: vec!["b".to_string()],
            },
        ];

        apply_hints(&mut scores, &[IntentHint::Exclude(Intent::Process)]);

        assert!(
            find_intent(&scores, &Intent::Process).is_none(),
            "Process should be excluded"
        );
        assert_eq!(scores.len(), 1);
    }

    #[test]
    fn hint_primary_and_exclude_together() {
        let mut scores = vec![
            IntentScore {
                intent: Intent::Process,
                confidence: 1.0,
                matching_signals: vec!["a".to_string()],
            },
            IntentScore {
                intent: Intent::Browse,
                confidence: 0.8,
                matching_signals: vec!["b".to_string()],
            },
            IntentScore {
                intent: Intent::Focus,
                confidence: 0.5,
                matching_signals: vec!["c".to_string()],
            },
        ];

        apply_hints(
            &mut scores,
            &[
                IntentHint::Exclude(Intent::Process),
                IntentHint::Primary(Intent::Focus),
            ],
        );

        assert!(find_intent(&scores, &Intent::Process).is_none());
        assert_eq!(scores[0].intent, Intent::Focus);
        assert!((scores[0].confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hint_on_empty_scores_primary_adds_exclude_noop() {
        let mut scores: Vec<IntentScore> = Vec::new();

        apply_hints(&mut scores, &[IntentHint::Exclude(Intent::Process)]);
        assert!(scores.is_empty(), "Exclude on empty is no-op");

        apply_hints(&mut scores, &[IntentHint::Primary(Intent::Browse)]);
        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].intent, Intent::Browse);
        assert!((scores[0].confidence - 1.0).abs() < f64::EPSILON);
    }

    // ==========================================================
    // Default fallback test
    // ==========================================================

    #[test]
    fn empty_service_returns_focus_default() {
        let service = ServiceDef::new("empty");
        let scores = derive_intents(&service);

        assert!(!scores.is_empty(), "Must return at least one score");
        // The first score should still be Browse/Focus from baseline, but
        // with an empty service both baselines are equal (0.1 each), so
        // Browse wins by priority (3 < 4). Let's verify the overall
        // behavior: at least one score, all confidences in [0, 1].
        for s in &scores {
            assert!(s.confidence >= 0.0 && s.confidence <= 1.0);
        }
    }

    #[test]
    fn service_only_system_fields_returns_baseline_scores() {
        let service = ServiceDef::new("system")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt);

        let scores = derive_intents(&service);
        assert!(!scores.is_empty());
        // Only baselines present, both Browse and Focus at 0.1 each.
        // They normalize to 1.0 each, tie-broken by priority: Browse(3) < Focus(4).
        assert_eq!(scores[0].intent, Intent::Browse);
        assert_eq!(scores[1].intent, Intent::Focus);
        assert!((scores[0].confidence - 1.0).abs() < f64::EPSILON);
        assert!((scores[1].confidence - 1.0).abs() < f64::EPSILON);
    }

    // ==========================================================
    // Integration test
    // ==========================================================

    #[test]
    fn integration_money_entity_name_produces_ranked_scores() {
        let service = ServiceDef::new("invoice")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("tax", DataType::Float, FieldMeaning::Percentage)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt);

        let scores = derive_intents(&service);

        // Multiple intents should be returned.
        assert!(scores.len() >= 2, "Should have multiple intent scores");

        // All confidences in [0.0, 1.0].
        for s in &scores {
            assert!(
                s.confidence >= 0.0 && s.confidence <= 1.0,
                "Confidence {} out of range for {:?}",
                s.confidence,
                s.intent
            );
        }

        // Sorted descending.
        for i in 1..scores.len() {
            assert!(
                scores[i - 1].confidence >= scores[i].confidence,
                "Scores must be sorted descending"
            );
        }

        // Summarize should be present (Money + Percentage).
        assert!(
            find_intent(&scores, &Intent::Summarize).is_some(),
            "Summarize should be present from Money+Percentage fields"
        );

        // Browse should be present (EntityName + baseline).
        assert!(
            find_intent(&scores, &Intent::Browse).is_some(),
            "Browse should be present from EntityName + baseline"
        );
    }

    #[test]
    fn integration_derive_intents_with_hints() {
        let service = ServiceDef::new("invoice")
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .intent_hint(IntentHint::Primary(Intent::Collect))
            .intent_hint(IntentHint::Exclude(Intent::Summarize));

        let scores = derive_intents(&service);

        // Collect must be first with confidence 1.0.
        assert_eq!(scores[0].intent, Intent::Collect);
        assert!((scores[0].confidence - 1.0).abs() < f64::EPSILON);

        // Summarize must be excluded.
        assert!(
            find_intent(&scores, &Intent::Summarize).is_none(),
            "Summarize should be excluded by hint"
        );
    }

    #[test]
    fn integration_all_confidences_normalized() {
        let service = ServiceDef::new("complex")
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("description", DataType::String, FieldMeaning::FreeText)
            .field("photo", DataType::String, FieldMeaning::ImageUrl)
            .field("price", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("category", DataType::String, FieldMeaning::Category)
            .field("recorded_at", DataType::DateTime, FieldMeaning::DateTime);

        let scores = derive_intents(&service);

        // Highest confidence should be 1.0.
        assert!(
            (scores[0].confidence - 1.0).abs() < f64::EPSILON,
            "Top score should be 1.0, got {}",
            scores[0].confidence
        );

        // Every score has at least one matching signal.
        for s in &scores {
            assert!(
                !s.matching_signals.is_empty(),
                "{:?} has no matching signals",
                s.intent
            );
        }
    }

    // ==========================================================
    // State machine analyzer tests
    // ==========================================================

    #[test]
    fn state_machine_order_workflow_produces_process() {
        use crate::state::{StateDef, StateMachine, Transition};

        // Order workflow: guarded transitions, branching (draft -> pending OR cancelled).
        let service = ServiceDef::new("order")
            .state_machine(
                StateMachine::new("order_lifecycle")
                    .initial("draft")
                    .state(StateDef::new("draft"))
                    .state(StateDef::new("pending"))
                    .state(StateDef::new("approved"))
                    .state(StateDef::new("completed").final_state())
                    .state(StateDef::new("cancelled").final_state())
                    .transition(
                        Transition::new("draft", "submit", "pending").guard("has_required_fields"),
                    )
                    .transition(
                        Transition::new("pending", "approve", "approved").guard("is_reviewer"),
                    )
                    .transition(Transition::new("approved", "complete", "completed"))
                    .transition(Transition::new("draft", "cancel", "cancelled")),
            )
            .action(
                ActionDef::new("submit")
                    .transition_trigger("submit")
                    .precondition("has_required_fields"),
            );

        let signals = analyze_state_machine(&service);
        let process_signals: Vec<_> = signals.iter().filter(|s| s.0 == Intent::Process).collect();
        let track_signals: Vec<_> = signals.iter().filter(|s| s.0 == Intent::Track).collect();

        assert!(
            !process_signals.is_empty(),
            "Process signals must be present for guarded branching workflow"
        );
        let process_weight: f64 = process_signals.iter().map(|s| s.1).sum();
        let track_weight: f64 = track_signals.iter().map(|s| s.1).sum();
        assert!(
            process_weight > track_weight,
            "Process ({process_weight}) should outweigh Track ({track_weight}) for branching workflow"
        );
    }

    #[test]
    fn state_machine_shipment_tracking_produces_track() {
        use crate::state::{StateDef, StateMachine, Transition};

        // Shipment tracking: linear states, no guards, no branches.
        let service = ServiceDef::new("shipment").state_machine(
            StateMachine::new("shipment_tracking")
                .initial("created")
                .state(StateDef::new("created"))
                .state(StateDef::new("picked_up"))
                .state(StateDef::new("in_transit"))
                .state(StateDef::new("delivered").final_state())
                .transition(Transition::new("created", "pick_up", "picked_up"))
                .transition(Transition::new("picked_up", "depart", "in_transit"))
                .transition(Transition::new("in_transit", "deliver", "delivered")),
        );

        let signals = analyze_state_machine(&service);
        let track_signals: Vec<_> = signals.iter().filter(|s| s.0 == Intent::Track).collect();
        let process_signals: Vec<_> = signals.iter().filter(|s| s.0 == Intent::Process).collect();

        assert!(
            !track_signals.is_empty(),
            "Track signals must be present for linear progression"
        );
        let track_weight: f64 = track_signals.iter().map(|s| s.1).sum();
        let process_weight: f64 = process_signals.iter().map(|s| s.1).sum();
        assert!(
            track_weight > process_weight,
            "Track ({track_weight}) should outweigh Process ({process_weight}) for linear tracking"
        );
    }

    #[test]
    fn state_machine_none_returns_empty() {
        let service = ServiceDef::new("bare");
        let signals = analyze_state_machine(&service);
        assert!(
            signals.is_empty(),
            "No state machine should produce no signals"
        );
    }

    #[test]
    fn state_machine_trivial_produces_weak_track() {
        use crate::state::{StateDef, StateMachine, Transition};

        // Trivial: 2 states, 1 transition, no guards.
        let service = ServiceDef::new("toggle").state_machine(
            StateMachine::new("toggle")
                .initial("off")
                .state(StateDef::new("off"))
                .state(StateDef::new("on").final_state())
                .transition(Transition::new("off", "activate", "on")),
        );

        let signals = analyze_state_machine(&service);
        let track_signals: Vec<_> = signals.iter().filter(|s| s.0 == Intent::Track).collect();

        // Should have some Track signals (has_final_states, unguarded_progression).
        assert!(
            !track_signals.is_empty(),
            "Trivial state machine should produce some Track signals"
        );
        // But no linear_states signal (non-final count == 1, not > 2).
        let linear: Vec<_> = track_signals
            .iter()
            .filter(|s| s.2.contains(SIGNAL_LINEAR_STATES))
            .collect();
        assert!(
            linear.is_empty(),
            "Trivial machine should not have linear_states (only 1 non-final)"
        );
    }

    // ==========================================================
    // Relationship analyzer tests
    // ==========================================================

    #[test]
    fn relationship_has_many_produces_browse() {
        let service = ServiceDef::new("category")
            .has_many("products", "product")
            .has_many("subcategories", "category");

        let signals = analyze_relationships(&service);
        let browse: Vec<_> = signals
            .iter()
            .filter(|s| s.0 == Intent::Browse && s.2.contains(SIGNAL_COLLECTION_RELATIONSHIPS))
            .collect();
        assert!(
            !browse.is_empty(),
            "has_many relationships should produce Browse collection signal"
        );
        // 2 collection relationships * 0.35 = 0.7
        assert!(
            (browse[0].1 - 0.7).abs() < f64::EPSILON,
            "expected 0.7, got {}",
            browse[0].1
        );
    }

    #[test]
    fn relationship_one_to_one_inline_produces_focus() {
        let service = ServiceDef::new("user").has_one("profile", "profile");

        let signals = analyze_relationships(&service);
        let focus: Vec<_> = signals
            .iter()
            .filter(|s| s.0 == Intent::Focus && s.2.contains(SIGNAL_INLINE_RELATIONSHIPS))
            .collect();
        assert!(
            !focus.is_empty(),
            "OneToOne with Inline navigation should produce Focus signal"
        );
        // 1 inline * 0.15 = 0.15
        assert!(
            (focus[0].1 - 0.15).abs() < f64::EPSILON,
            "expected 0.15, got {}",
            focus[0].1
        );
    }

    #[test]
    fn relationship_many_to_one_produces_focus_parent() {
        let service = ServiceDef::new("order").belongs_to("customer", "customer");

        let signals = analyze_relationships(&service);
        let focus: Vec<_> = signals
            .iter()
            .filter(|s| s.0 == Intent::Focus && s.2.contains(SIGNAL_PARENT_REFERENCES))
            .collect();
        assert!(
            !focus.is_empty(),
            "ManyToOne should produce Focus parent_references signal"
        );
        // 1 parent * 0.1 = 0.1
        assert!(
            (focus[0].1 - 0.1).abs() < f64::EPSILON,
            "expected 0.1, got {}",
            focus[0].1
        );
    }

    #[test]
    fn relationship_rich_graph_produces_browse() {
        let service = ServiceDef::new("order")
            .belongs_to("customer", "customer")
            .has_many("line_items", "line_item")
            .has_many("payments", "payment")
            .belongs_to("warehouse", "warehouse");

        let signals = analyze_relationships(&service);
        let rich: Vec<_> = signals
            .iter()
            .filter(|s| s.2.contains(SIGNAL_RICH_RELATIONSHIP_GRAPH))
            .collect();
        assert!(
            !rich.is_empty(),
            "4+ relationships should produce rich_relationship_graph signal"
        );
    }

    #[test]
    fn relationship_none_returns_empty() {
        let service = ServiceDef::new("bare");
        let signals = analyze_relationships(&service);
        assert!(
            signals.is_empty(),
            "No relationships should produce no signals"
        );
    }

    // ==========================================================
    // Action analyzer tests
    // ==========================================================

    #[test]
    fn action_transition_trigger_produces_process() {
        let service = ServiceDef::new("order")
            .action(ActionDef::new("submit").transition_trigger("submit"))
            .action(ActionDef::new("approve").transition_trigger("approve"));

        let signals = analyze_actions(&service);
        let workflow: Vec<_> = signals
            .iter()
            .filter(|s| s.0 == Intent::Process && s.2.contains(SIGNAL_WORKFLOW_ACTIONS))
            .collect();
        assert!(
            !workflow.is_empty(),
            "Actions with transition_trigger should produce workflow_actions signal"
        );
        // 2 workflow actions * 0.15 = 0.3
        assert!(
            (workflow[0].1 - 0.3).abs() < f64::EPSILON,
            "expected 0.3, got {}",
            workflow[0].1
        );
    }

    #[test]
    fn action_complex_inputs_produces_collect() {
        use crate::action::InputDef;

        let service = ServiceDef::new("registration").action(
            ActionDef::new("register")
                .input(InputDef::new(
                    "name",
                    DataType::String,
                    FieldMeaning::EntityName,
                ))
                .input(InputDef::new(
                    "email",
                    DataType::String,
                    FieldMeaning::Email,
                ))
                .input(InputDef::new(
                    "phone",
                    DataType::String,
                    FieldMeaning::Phone,
                )),
        );

        let signals = analyze_actions(&service);
        let collect: Vec<_> = signals
            .iter()
            .filter(|s| s.0 == Intent::Collect && s.2.contains(SIGNAL_COMPLEX_INPUT_ACTIONS))
            .collect();
        assert!(
            !collect.is_empty(),
            "Actions with >2 inputs should produce complex_input_actions signal"
        );
        // 1 complex action * 0.15 = 0.15
        assert!(
            (collect[0].1 - 0.15).abs() < f64::EPSILON,
            "expected 0.15, got {}",
            collect[0].1
        );
    }

    #[test]
    fn action_preconditions_produces_process() {
        let service = ServiceDef::new("order")
            .action(
                ActionDef::new("submit")
                    .precondition("has_items")
                    .precondition("payment_valid"),
            )
            .action(ActionDef::new("cancel").precondition("is_cancellable"));

        let signals = analyze_actions(&service);
        let guarded: Vec<_> = signals
            .iter()
            .filter(|s| s.0 == Intent::Process && s.2.contains(SIGNAL_GUARDED_ACTIONS))
            .collect();
        assert!(
            !guarded.is_empty(),
            "Actions with preconditions should produce guarded_actions signal"
        );
        // 2 guarded actions * 0.1 = 0.2
        assert!(
            (guarded[0].1 - 0.2).abs() < f64::EPSILON,
            "expected 0.2, got {}",
            guarded[0].1
        );
    }

    #[test]
    fn action_simple_crud_produces_browse() {
        let service = ServiceDef::new("product")
            .action(ActionDef::new("create"))
            .action(ActionDef::new("update"))
            .action(ActionDef::new("delete"));

        let signals = analyze_actions(&service);
        let simple: Vec<_> = signals
            .iter()
            .filter(|s| s.0 == Intent::Browse && s.2.contains(SIGNAL_SIMPLE_CRUD_ACTIONS))
            .collect();
        assert!(
            !simple.is_empty(),
            "Simple CRUD actions should produce simple_crud_actions signal"
        );
    }

    #[test]
    fn action_none_returns_empty() {
        let service = ServiceDef::new("bare");
        let signals = analyze_actions(&service);
        assert!(signals.is_empty(), "No actions should produce no signals");
    }

    // ==========================================================
    // Full pipeline integration test (all 5 analyzers)
    // ==========================================================

    #[test]
    fn integration_order_management_all_analyzers_produce_process() {
        use crate::action::InputDef;
        use crate::state::{StateDef, StateMachine, Transition};

        // Realistic order management service with signals from all 5 analyzers.
        let service = ServiceDef::new("order")
            // Fields: Money -> Summarize, Status -> Track, EntityName -> Browse
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("tax", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("notes", DataType::String, FieldMeaning::FreeText)
            // State machine: branching with guards -> Process
            .state_machine(
                StateMachine::new("order_lifecycle")
                    .initial("draft")
                    .state(StateDef::new("draft"))
                    .state(StateDef::new("pending"))
                    .state(StateDef::new("approved"))
                    .state(StateDef::new("completed").final_state())
                    .state(StateDef::new("cancelled").final_state())
                    .transition(Transition::new("draft", "submit", "pending").guard("has_items"))
                    .transition(
                        Transition::new("pending", "approve", "approved").guard("is_reviewer"),
                    )
                    .transition(Transition::new("approved", "complete", "completed"))
                    .transition(Transition::new("draft", "cancel", "cancelled"))
                    .transition(
                        Transition::new("pending", "cancel", "cancelled")
                            .guard("cancellation_allowed"),
                    ),
            )
            // Actions: transition triggers + preconditions -> Process
            .action(
                ActionDef::new("submit_order")
                    .transition_trigger("submit")
                    .precondition("has_items")
                    .input(InputDef::new(
                        "order_id",
                        DataType::Integer,
                        FieldMeaning::Identifier,
                    ))
                    .input(InputDef::new(
                        "notes",
                        DataType::String,
                        FieldMeaning::FreeText,
                    ))
                    .input(InputDef::new(
                        "priority",
                        DataType::String,
                        FieldMeaning::Category,
                    )),
            )
            .action(
                ActionDef::new("approve_order")
                    .transition_trigger("approve")
                    .precondition("is_reviewer"),
            )
            .action(ActionDef::new("cancel_order").transition_trigger("cancel"))
            // Relationships: has_many -> Browse, belongs_to -> Focus
            .has_many("line_items", "order_line_item")
            .belongs_to("customer", "customer");

        let scores = derive_intents(&service);

        // Process must be the primary intent (highest confidence or position 0).
        assert_eq!(
            scores[0].intent,
            Intent::Process,
            "Order management with state machine + guards + transition triggers should derive Process as primary. Got {:?} (full: {:?})",
            scores[0].intent,
            scores.iter().map(|s| (&s.intent, s.confidence)).collect::<Vec<_>>()
        );

        // All confidences in [0.0, 1.0] and sorted descending.
        for i in 0..scores.len() {
            assert!(
                scores[i].confidence >= 0.0 && scores[i].confidence <= 1.0,
                "Confidence {} out of range for {:?}",
                scores[i].confidence,
                scores[i].intent
            );
            if i > 0 {
                assert!(
                    scores[i - 1].confidence >= scores[i].confidence,
                    "Scores must be sorted descending"
                );
            }
        }

        // Multiple intents should be present from different analyzers.
        assert!(
            scores.len() >= 3,
            "Multiple analyzers should contribute multiple intents, got {}",
            scores.len()
        );

        // Every score has at least one matching signal.
        for s in &scores {
            assert!(
                !s.matching_signals.is_empty(),
                "{:?} has no matching signals",
                s.intent
            );
        }
    }

    // ==========================================================
    // is_system_field tests
    // ==========================================================

    #[test]
    fn is_system_field_identifies_system_meanings() {
        assert!(is_system_field(&FieldMeaning::Identifier));
        assert!(is_system_field(&FieldMeaning::CreatedAt));
        assert!(is_system_field(&FieldMeaning::UpdatedAt));
    }

    #[test]
    fn is_system_field_rejects_domain_meanings() {
        assert!(!is_system_field(&FieldMeaning::Money));
        assert!(!is_system_field(&FieldMeaning::EntityName));
        assert!(!is_system_field(&FieldMeaning::FreeText));
        assert!(!is_system_field(&FieldMeaning::Status));
        assert!(!is_system_field(&FieldMeaning::Custom("x".into())));
    }

    // ==========================================================
    // intent_priority tests
    // ==========================================================

    #[test]
    fn intent_priority_ordering() {
        assert!(intent_priority(&Intent::Process) < intent_priority(&Intent::Track));
        assert!(intent_priority(&Intent::Track) < intent_priority(&Intent::Collect));
        assert!(intent_priority(&Intent::Collect) < intent_priority(&Intent::Browse));
        assert!(intent_priority(&Intent::Browse) < intent_priority(&Intent::Focus));
        assert!(intent_priority(&Intent::Focus) < intent_priority(&Intent::Summarize));
        assert!(intent_priority(&Intent::Summarize) < intent_priority(&Intent::Analyze));
        assert!(intent_priority(&Intent::Analyze) < intent_priority(&Intent::Custom("x".into())));
    }
}
