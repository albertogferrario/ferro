//! JSON Schema normalization for structured-output APIs.
//!
//! This module will provide `for_structured_output()` — a normalizer that
//! makes schemars 1.x output compatible with Anthropic and OpenAI structured-output
//! APIs by:
//!
//! 1. Closing projection enums (`FieldMeaning`, `Intent`) in `$defs` first (ServiceDef-aware path).
//! 2. Resolving all `$ref`/`$defs` inline recursively with a cycle guard.
//! 3. Removing `$defs`/`definitions` from the root schema.
//! 4. Stripping Anthropic-rejected keywords (`$schema`, `$id`, `title`, `examples`,
//!    numeric bounds, string bounds).
//! 5. Adding `additionalProperties: false` to every object schema that has `properties`.
//!
//! The ServiceDef-aware path fulfils SC#3: the LLM-facing schema locks `FieldMeaning`
//! and `Intent` to their known snake_case variants, preventing invalid projection values.
//! The Rust types retain the `Custom(String)` deserialization escape hatch unchanged.
//!
//! Implementation is added in Plan 02; this shell is the Wave 0 foundation.
//!
//! ## Verified schema shapes (Wave 0 probe — resolves A1/A2)
//!
//! `FieldMeaning` emits `anyOf` with two branches:
//! - Branch 0: `{"type":"string","enum":["identifier",...18 values...]}` — closed enum (no per-variant docs)
//! - Branch 1: `{"type":"string"}` — open string (the `Custom` escape hatch)
//!
//! `Intent` emits `anyOf` with 8 branches (7 known + 1 open):
//! - Branches 0–6: `{"const":"browse","description":"...","type":"string"}` — per-variant `const`
//!   (schemars emits individual `const` branches when variants carry doc comments)
//! - Branch 7: `{"description":"Escape hatch...","type":"string"}` — open string (no `const`, no `enum`)
//!
//! The closing algorithm in Plan 02 handles both shapes:
//! - `FieldMeaning`: extract `anyOf[0].enum` → emit closed `{"type":"string","enum":[...]}`
//! - `Intent`: collect `anyOf[*].const` values from branches that have `const` → emit closed
//!   `{"type":"string","enum":["browse","focus","collect","process","summarize","analyze","track"]}`

#[cfg(test)]
mod tests {
    use schemars::schema_for;
    use serde_json::Value;

    /// Wave 0 probe: verifies the actual schemars 1.x schema shape for `FieldMeaning`.
    ///
    /// `FieldMeaning` has no per-variant doc comments, so schemars collapses all known
    /// unit variants into a single `anyOf` branch with an `enum` array.
    /// The `Custom(String)` escape hatch becomes a second open-string branch.
    #[test]
    fn schema_probe_field_meaning_any_of_shape() {
        use ferro_projections::FieldMeaning;
        let schema: Value = schema_for!(FieldMeaning).to_value();
        let any_of = schema["anyOf"]
            .as_array()
            .expect("FieldMeaning must emit anyOf");
        assert!(
            any_of.len() >= 2,
            "expected >=2 anyOf branches, got {}",
            any_of.len()
        );
        assert_eq!(any_of[0]["type"], "string");
        let variants = any_of[0]["enum"]
            .as_array()
            .expect("first branch must be closed enum");
        assert!(
            variants.iter().any(|v| v == "money"),
            "known variants must include money"
        );
        assert_eq!(any_of[1]["type"], "string");
        assert!(
            any_of[1].get("enum").is_none(),
            "second branch must be open string (the Custom escape hatch)"
        );
    }

    /// Wave 0 probe: verifies the actual schemars 1.x schema shape for `Intent`.
    ///
    /// `Intent` has per-variant doc comments, so schemars emits individual `const` branches
    /// (one per known variant) rather than a single `enum` array. The `Custom(String)` escape
    /// hatch becomes a final branch with no `const` and no `enum`.
    ///
    /// The closing algorithm in Plan 02 must collect `const` values from branches that have
    /// one, then emit a closed `{"type":"string","enum":[...collected...]}`.
    #[test]
    fn schema_probe_intent_any_of_shape() {
        use ferro_projections::Intent;
        let schema: Value = schema_for!(Intent).to_value();
        let any_of = schema["anyOf"].as_array().expect("Intent must emit anyOf");
        // 7 known variants + 1 open Custom branch = at least 8 branches
        assert!(
            any_of.len() >= 8,
            "expected >=8 anyOf branches (7 const + 1 open), got {}",
            any_of.len()
        );

        // All branches except the last must carry a `const` with a known variant name.
        let expected_variants = [
            "browse",
            "focus",
            "collect",
            "process",
            "summarize",
            "analyze",
            "track",
        ];
        for expected in expected_variants {
            let found = any_of
                .iter()
                .any(|branch| branch.get("const").and_then(|c| c.as_str()) == Some(expected));
            assert!(found, "Intent anyOf missing const branch for '{expected}'");
        }

        // The last branch must be the open-string Custom escape hatch: no `const`, no `enum`.
        let last = any_of.last().expect("anyOf must not be empty");
        assert_eq!(last["type"], "string", "last branch must be type string");
        assert!(
            last.get("const").is_none(),
            "last branch must not have const (it is the Custom escape hatch)"
        );
        assert!(
            last.get("enum").is_none(),
            "last branch must not have enum (it is the Custom escape hatch)"
        );
    }
}
