//! Generate a ServiceDef projection from a SeaORM model.

use std::path::Path;

use ferro_projections::{derive_intents, FieldMetadata, ModelMetadata, ServiceDef};
use serde::Serialize;

use crate::tools::list_models;

/// Intent scoring information.
#[derive(Debug, Serialize)]
pub struct IntentInfo {
    pub intent: String,
    pub confidence: f64,
    pub signals: Vec<String>,
}

/// Result of generating a ServiceDef from a model.
#[derive(Debug, Serialize)]
pub struct GenerateProjectionResult {
    pub model_name: String,
    pub service_def: serde_json::Value,
    pub intents: Vec<IntentInfo>,
    pub inferred_field_count: usize,
    pub manual_enrichment_needed: Vec<String>,
}

/// Generate a ServiceDef projection from a SeaORM model by name.
///
/// Finds the model via `list_models`, converts its fields to `ModelMetadata`,
/// derives a `ServiceDef` via `ServiceDef::from_model()`, then runs intent
/// derivation and returns the full result.
pub fn execute(project_root: &Path, model_name: &str) -> Result<GenerateProjectionResult, String> {
    // 1. Find model via list_models::execute()
    let models = list_models::execute(project_root)
        .map_err(|e| format!("Failed to list models: {e}"))?;

    let model = models
        .iter()
        .find(|m| m.name == model_name)
        .ok_or_else(|| {
            let available: Vec<&str> = models.iter().map(|m| m.name.as_str()).collect();
            format!("Model '{}' not found. Available: {:?}", model_name, available)
        })?;

    // 2. Convert ModelDetails -> ModelMetadata
    let meta = ModelMetadata {
        name: model.name.clone(),
        display_name: None,
        table: model.table.clone(),
        fields: model
            .fields
            .iter()
            .map(|f| FieldMetadata {
                name: f.name.clone(),
                column_type: f.field_type.clone(),
                is_primary_key: f.is_primary_key,
                is_nullable: f.is_nullable,
            })
            .collect(),
    };

    // 3. Derive ServiceDef
    let service_def = ServiceDef::from_model(&meta);

    // 4. Derive intents
    let intents = derive_intents(&service_def);

    // 5. Serialize ServiceDef to JSON value
    let service_json = serde_json::to_value(&service_def)
        .map_err(|e| format!("Failed to serialize ServiceDef: {e}"))?;

    // 6. Map IntentScore -> IntentInfo
    let intent_infos: Vec<IntentInfo> = intents
        .iter()
        .map(|score| IntentInfo {
            intent: format!("{:?}", score.intent),
            confidence: score.confidence,
            signals: score.matching_signals.clone(),
        })
        .collect();

    // 7. Build result
    let inferred_count = service_def.fields.len();
    Ok(GenerateProjectionResult {
        model_name: model_name.to_string(),
        service_def: service_json,
        intents: intent_infos,
        inferred_field_count: inferred_count,
        manual_enrichment_needed: vec![
            "actions".to_string(),
            "state_machine".to_string(),
            "relationships".to_string(),
        ],
    })
}
