//! JSON Schema normalization for structured-output APIs.
//!
//! This module provides `for_structured_output()` — a normalizer that
//! makes schemars 1.x output compatible with Anthropic and OpenAI structured-output
//! APIs by:
//!
//! 1. Resolving all `$ref`/`$defs` inline recursively with a cycle guard.
//! 2. Removing `$defs`/`definitions` from the root schema.
//! 3. Stripping Anthropic-rejected keywords (`$schema`, `$id`, `title`, `examples`,
//!    numeric bounds, string bounds, `pattern`).
//! 4. Adding `additionalProperties: false` to every object schema that has BOTH
//!    `"type": "object"` AND a `"properties"` key.
//!
//! ## ServiceDef-aware path (Plan 03)
//!
//! The ServiceDef-aware enum-closing pass (Plan 03) runs on `$defs` BEFORE calling
//! this function. That pass closes `FieldMeaning` and `Intent` to their known
//! snake_case variants and drops the `Custom(String)` untagged branch from the
//! LLM-facing schema. This generic normalizer then inlines the already-closed
//! `$defs` entries, preserving `enum` constraints exactly (SC#2 / D-04).
//!
//! **Order is mandatory (Pitfall 2):** close projection enums in `$defs` FIRST,
//! THEN call `for_structured_output`. Inlining before closing would scatter the
//! open `anyOf` shape throughout the schema, requiring a full-tree walk to fix.
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
//! The closing algorithm in Plan 03 handles both shapes:
//! - `FieldMeaning`: extract `anyOf[0].enum` → emit closed `{"type":"string","enum":[...]}`
//! - `Intent`: collect `anyOf[*].const` values from branches that have `const` → emit closed
//!   `{"type":"string","enum":["browse","focus","collect","process","summarize","analyze","track"]}`

use std::collections::HashSet;

use serde_json::{Map, Value};

/// Keywords to strip from normalized schemas (explicit allowlist — only these are removed).
///
/// `enum` is intentionally absent: it is the locking mechanism for the ServiceDef-aware
/// closing pass (D-04, SC#2). `format` is handled separately (strip only non-string formats).
const STRIP_KEYWORDS: &[&str] = &[
    "$schema",
    "$id",
    "title",
    "examples",
    "minimum",
    "maximum",
    "multipleOf",
    "minLength",
    "maxLength",
    "pattern",
];

/// String format values that Anthropic structured-output APIs accept (preserve these).
const ALLOWED_FORMATS: &[&str] = &[
    "date-time",
    "date",
    "time",
    "duration",
    "email",
    "hostname",
    "uri",
    "ipv4",
    "ipv6",
    "uuid",
];

/// Normalize a schemars 1.x schema for Anthropic / OpenAI structured-output APIs.
///
/// This is the **generic** normalizer. The ServiceDef-aware projection-enum closing
/// (Plan 03) must run on `$defs` BEFORE this function is called — see module docs
/// for mandatory ordering.
///
/// Steps (order is fixed):
/// 1. Clone `$defs` / `definitions` into a lookup map.
/// 2. Resolve all `{"$ref": "#/$defs/Name"}` inline recursively (cycle-guarded).
/// 3. Remove `$defs` / `definitions` from the root — the result has no `$ref` anywhere.
/// 4. Strip Anthropic-rejected keywords via `STRIP_KEYWORDS` allowlist; strip
///    non-string `format` values; `enum` is never stripped.
/// 5. Add `additionalProperties: false` to every schema node that has BOTH
///    `"type": "object"` AND a `"properties"` key (Pitfall 6: skip composition nodes).
///
/// The function is idempotent — running it twice on already-normalized input is a no-op.
pub fn for_structured_output(schema: Value) -> Value {
    // Extract $defs (and legacy definitions) into a flat lookup map.
    let mut defs: Map<String, Value> = Map::new();
    if let Some(obj) = schema.as_object() {
        if let Some(Value::Object(d)) = obj.get("$defs") {
            for (k, v) in d {
                defs.insert(k.clone(), v.clone());
            }
        }
        if let Some(Value::Object(d)) = obj.get("definitions") {
            for (k, v) in d {
                defs.insert(k.clone(), v.clone());
            }
        }
    }

    // Inline all $ref occurrences (cycle-guarded).
    let mut visited: HashSet<String> = HashSet::new();
    let mut root = resolve_refs(schema, &defs, &mut visited);

    // Remove $defs / definitions — all refs are now inlined.
    if let Some(obj) = root.as_object_mut() {
        obj.remove("$defs");
        obj.remove("definitions");
    }

    // Strip rejected keywords and add additionalProperties: false.
    normalize_node(root)
}

/// Resolve all `$ref` occurrences in `node` by inlining from `defs`.
///
/// Handles `#/$defs/Name` (Draft 2020-12) and `#/definitions/Name` (Draft 7).
/// Cycle guard: if `name` is already in `visited`, returns `{"type":"object"}`.
fn resolve_refs(node: Value, defs: &Map<String, Value>, visited: &mut HashSet<String>) -> Value {
    match node {
        Value::Object(ref obj) if obj.len() == 1 && obj.contains_key("$ref") => {
            // Pure $ref node — inline it.
            if let Some(ref_str) = obj.get("$ref").and_then(|v| v.as_str()) {
                if let Some(name) = parse_ref_name(ref_str) {
                    if visited.contains(name) {
                        // Cycle detected — return placeholder.
                        return serde_json::json!({"type": "object"});
                    }
                    if let Some(def) = defs.get(name) {
                        visited.insert(name.to_string());
                        let resolved = resolve_refs(def.clone(), defs, visited);
                        visited.remove(name);
                        return resolved;
                    }
                }
            }
            // Unknown $ref — return unchanged.
            node
        }
        Value::Object(obj) => {
            // Rebuild the object, recursing into each value.
            let mut new_obj = Map::with_capacity(obj.len());
            for (k, v) in obj {
                new_obj.insert(k, resolve_refs(v, defs, visited));
            }
            Value::Object(new_obj)
        }
        Value::Array(arr) => {
            // Rebuild the array, recursing into each element.
            Value::Array(
                arr.into_iter()
                    .map(|elem| resolve_refs(elem, defs, visited))
                    .collect(),
            )
        }
        other => other,
    }
}

/// Parse the definition name from a `$ref` string.
///
/// Handles `#/$defs/Name` and `#/definitions/Name`. Returns `None` for other forms.
fn parse_ref_name(ref_str: &str) -> Option<&str> {
    if let Some(name) = ref_str.strip_prefix("#/$defs/") {
        return Some(name);
    }
    if let Some(name) = ref_str.strip_prefix("#/definitions/") {
        return Some(name);
    }
    None
}

/// Strip Anthropic-rejected keywords and add `additionalProperties: false`.
///
/// Recursive rebuild (Pitfall 3: never mutate while iterating). Constructs a new
/// `Value` for each node rather than modifying in place.
fn normalize_node(node: Value) -> Value {
    match node {
        Value::Object(obj) => {
            let mut new_obj = Map::with_capacity(obj.len());

            for (k, v) in obj {
                if STRIP_KEYWORDS.contains(&k.as_str()) {
                    // Strip this key unconditionally.
                    continue;
                }
                if k == "format" {
                    // Keep only string formats Anthropic supports; drop numeric ones.
                    if let Some(fmt) = v.as_str() {
                        if ALLOWED_FORMATS.contains(&fmt) {
                            new_obj.insert(k, v);
                        }
                        // Non-string formats (int32, float, etc.) are silently dropped.
                    }
                    continue;
                }
                // Recurse into the value.
                new_obj.insert(k, normalize_node(v));
            }

            // Add additionalProperties: false ONLY when:
            // - type == "object"  (Pitfall 6: skip anyOf/oneOf/allOf composition nodes)
            // - AND "properties" key is present
            let is_object_type = new_obj
                .get("type")
                .and_then(|t| t.as_str())
                .map(|t| t == "object")
                .unwrap_or(false);
            let has_properties = new_obj.contains_key("properties");

            if is_object_type && has_properties {
                new_obj
                    .entry("additionalProperties")
                    .or_insert(Value::Bool(false));
            }

            Value::Object(new_obj)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(normalize_node).collect()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use schemars::schema_for;
    use serde_json::{json, Value};

    use super::*;

    // ── Wave 0 probe tests (Plan 01) ────────────────────────────────────────────

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
    /// The closing algorithm in Plan 03 must collect `const` values from branches that have
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

    // ── SC#2 normalizer tests (Plan 02) ────────────────────────────────────────

    /// Verifies that rejected Anthropic keywords are stripped, `minLength` on a property
    /// is removed, `additionalProperties: false` is added to objects with properties,
    /// and `required` (a preserved keyword) survives.
    #[test]
    fn schema_normalizer_strips_rejected_keywords() {
        let input = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Foo",
            "type": "object",
            "properties": {
                "x": {
                    "type": "string",
                    "minLength": 1,
                    "enum": ["a", "b"]
                }
            },
            "required": ["x"]
        });
        let out = for_structured_output(input);
        // Top-level stripped keywords gone.
        assert!(out.get("$schema").is_none(), "$schema must be stripped");
        assert!(out.get("title").is_none(), "title must be stripped");
        // Preserved keywords survive.
        assert_eq!(out["required"], json!(["x"]));
        // Property-level stripped keyword gone.
        assert!(
            out["properties"]["x"].get("minLength").is_none(),
            "minLength must be stripped from property"
        );
        // enum survives.
        assert_eq!(out["properties"]["x"]["enum"], json!(["a", "b"]));
        // additionalProperties: false added.
        assert_eq!(out["additionalProperties"], json!(false));
    }

    /// Verifies that `$ref` references are resolved inline and `$defs` is removed.
    #[test]
    fn schema_normalizer_resolves_refs() {
        let input = json!({
            "type": "object",
            "properties": {
                "sub": { "$ref": "#/$defs/Sub" }
            },
            "required": ["sub"],
            "$defs": {
                "Sub": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" }
                    },
                    "required": ["name"]
                }
            }
        });
        let out = for_structured_output(input);
        // $defs removed.
        assert!(out.get("$defs").is_none(), "$defs must be removed");
        // $ref resolved — the property now has the Sub schema inlined.
        assert!(
            out["properties"]["sub"].get("$ref").is_none(),
            "$ref must not remain after normalization"
        );
        assert_eq!(out["properties"]["sub"]["type"], json!("object"));
        // Inlined sub-object also gets additionalProperties: false.
        assert_eq!(
            out["properties"]["sub"]["additionalProperties"],
            json!(false)
        );
    }

    /// Regression guard for Pitfall 1: `enum` must survive the strip pass.
    ///
    /// The `enum` keyword is the locking mechanism for the ServiceDef-aware closing
    /// pass (Plan 03). If it were accidentally stripped, SC#3's structural guarantee
    /// would collapse silently.
    #[test]
    fn schema_normalizer_preserves_enum() {
        let input = json!({
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "enum": ["active", "inactive", "pending"]
                }
            },
            "required": ["status"]
        });
        let out = for_structured_output(input);
        let enum_val = out["properties"]["status"]["enum"]
            .as_array()
            .expect("enum must survive normalization");
        assert_eq!(enum_val.len(), 3, "all enum variants must be preserved");
        assert!(enum_val.iter().any(|v| v == "active"));
        assert!(enum_val.iter().any(|v| v == "inactive"));
        assert!(enum_val.iter().any(|v| v == "pending"));
    }

    /// Regression guard for Pitfall 6: composition nodes without `properties`
    /// must NOT get `additionalProperties: false`.
    ///
    /// An `anyOf` node with no `properties` key should be left unchanged —
    /// injecting `additionalProperties: false` on such a node rejects all properties
    /// from sub-schemas, breaking the schema.
    #[test]
    fn schema_normalizer_skips_additional_properties_on_anyof() {
        let input = json!({
            "type": "object",
            "properties": {
                "val": {
                    "anyOf": [
                        { "type": "string" },
                        { "type": "integer" }
                    ]
                }
            },
            "required": ["val"]
        });
        let out = for_structured_output(input);
        // The `anyOf` node itself has no `properties`, so no additionalProperties.
        let val_schema = &out["properties"]["val"];
        assert!(
            val_schema.get("additionalProperties").is_none(),
            "anyOf node without properties must not get additionalProperties:false"
        );
        // The root has properties, so it should get additionalProperties: false.
        assert_eq!(out["additionalProperties"], json!(false));
    }
}
