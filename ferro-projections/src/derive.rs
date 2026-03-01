use std::collections::HashMap;

use crate::field::FieldMeaning;
use crate::intent::{Intent, IntentHint, IntentScore};
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

// Baseline scores added to Browse and Focus to ensure they always appear.
const BASELINE_BROWSE: f64 = 0.1;
const BASELINE_FOCUS: f64 = 0.1;

/// A single signal contribution: (intent, raw weight, signal description).
type Signal = (Intent, f64, String);

/// Derives ranked intents from a ServiceDef's structural signals.
///
/// Always returns at least one IntentScore. Default: Focus with 0.5 confidence.
pub fn derive_intents(service: &ServiceDef) -> Vec<IntentScore> {
    // 1. Collect signals from all analyzers.
    let mut all_signals = Vec::new();
    all_signals.extend(analyze_field_meanings(service));
    all_signals.extend(analyze_writability(service));

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
