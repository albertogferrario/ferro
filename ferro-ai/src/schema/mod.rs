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

// ── ServiceDef-aware projection enum closing (Plan 03) ─────────────────────

/// Names of ferro-projections types that trigger the ServiceDef-aware closing path (D-07).
///
/// If any of these names appear as a key in the schema's `$defs`, the normalizer
/// activates the projection-enum closing pass before resolving refs.
const PROJECTION_DEF_NAMES: &[&str] = &[
    "FieldMeaning",
    "Intent",
    "ServiceDef",
    "Cardinality",
    "ActionDef",
    "GuardDef",
    "StateDef",
];

/// Returns `true` if `schema.$defs` contains any ferro-projections type name (D-07).
fn has_projection_defs(schema: &Value) -> bool {
    schema
        .get("$defs")
        .and_then(|d| d.as_object())
        .map(|defs| PROJECTION_DEF_NAMES.iter().any(|n| defs.contains_key(*n)))
        .unwrap_or(false)
}

/// Close a projection enum in `$defs` by replacing its open `anyOf` with a closed `enum`.
///
/// Handles two schemars output shapes (D-08: vocabulary derived from schema, never hardcoded):
///
/// - **`FieldMeaning`** shape (no per-variant doc comments): anyOf with a branch that has
///   `{"type":"string","enum":[...known...]}` — extract that branch directly.
/// - **`Intent`** shape (per-variant doc comments): anyOf with individual `{"const":"browse",...}`
///   branches per known variant, plus an open `{"type":"string"}` Custom branch — collect all
///   `const` values, emit `{"type":"string","enum":[...collected...]}`.
///
/// If the entry has no `anyOf`, it is left unchanged (e.g., `Cardinality` is already a
/// closed enum — schemars emits it as `{"type":"string","enum":[...]}` directly).
///
/// The outer `description` (if any) is preserved on the resulting closed schema.
fn close_projection_enum(defs: &mut Map<String, Value>, name: &str) {
    if let Some(entry) = defs.get_mut(name) {
        // Preserve any outer description.
        let desc = entry.get("description").cloned();

        if let Some(branches) = entry
            .get("anyOf")
            .and_then(|a| a.as_array())
            .map(|a| a.to_vec())
        {
            // Shape A: one branch carries a closed `enum` array.
            // Use `.find` (not `[0]`) to be robust against branch reordering (defends A2).
            let closed = if let Some(enum_branch) =
                branches.iter().find(|b| b.get("enum").is_some()).cloned()
            {
                // Shape A: FieldMeaning-style.
                let mut closed = enum_branch;
                if let (Some(d), Some(obj)) = (desc, closed.as_object_mut()) {
                    obj.entry("description").or_insert(d);
                }
                closed
            } else {
                // Shape B: Intent-style — collect all `const` values from branches that have one.
                // Branches without `const` (the open Custom escape hatch) are dropped.
                let consts: Vec<Value> = branches
                    .iter()
                    .filter_map(|b| b.get("const").cloned())
                    .collect();

                if consts.is_empty() {
                    // Nothing to close — leave the entry unchanged.
                    return;
                }

                let mut obj = serde_json::Map::new();
                obj.insert("type".into(), Value::String("string".into()));
                obj.insert("enum".into(), Value::Array(consts));
                if let Some(d) = desc {
                    obj.insert("description".into(), d);
                }
                Value::Object(obj)
            };

            *entry = closed;
        }
        // If there is no `anyOf`, the entry is already a closed-enum schema — no action.
    }
}

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
/// Steps (order is mandatory — see Pitfall 2 in module docs):
/// 1. **Close projection enums in `$defs` FIRST** (ServiceDef-aware path, D-06/D-07).
///    If any ferro-projections type name appears in `$defs`, close `FieldMeaning` and
///    `Intent` by replacing their open `anyOf` with a closed `enum` constraint. This
///    must happen before ref inlining so every inlined occurrence resolves to the
///    closed form.
/// 2. Clone `$defs` / `definitions` into a lookup map.
/// 3. Resolve all `{"$ref": "#/$defs/Name"}` inline recursively (cycle-guarded).
/// 4. Remove `$defs` / `definitions` from the root — the result has no `$ref` anywhere.
/// 5. Strip Anthropic-rejected keywords via `STRIP_KEYWORDS` allowlist; strip
///    non-string `format` values; `enum` is never stripped.
/// 6. Add `additionalProperties: false` to every schema node that has BOTH
///    `"type": "object"` AND a `"properties"` key (Pitfall 6: skip composition nodes).
///
/// The function is idempotent — running it twice on already-normalized input is a no-op.
pub fn for_structured_output(schema: Value) -> Value {
    let mut root = schema;

    // Step 1 (MANDATORY FIRST): Close projection enums in $defs before ref inlining.
    // Pitfall 2: if we inline $ref before closing, the open anyOf propagates everywhere.
    if has_projection_defs(&root) {
        if let Some(defs_mut) = root.get_mut("$defs").and_then(|d| d.as_object_mut()) {
            close_projection_enum(defs_mut, "FieldMeaning");
            close_projection_enum(defs_mut, "Intent");
        }
    }

    // Step 2: Extract $defs (and legacy definitions) into a flat lookup map.
    let mut defs: Map<String, Value> = Map::new();
    if let Some(obj) = root.as_object() {
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

    // Step 3: Inline all $ref occurrences (cycle-guarded).
    let mut visited: HashSet<String> = HashSet::new();
    root = resolve_refs(root, &defs, &mut visited);

    // Step 4: Remove $defs / definitions — all refs are now inlined.
    if let Some(obj) = root.as_object_mut() {
        obj.remove("$defs");
        obj.remove("definitions");
    }

    // Steps 5-6: Strip rejected keywords and add additionalProperties: false.
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

    // ── SC#3 projection enum closing tests (Plan 03) ───────────────────────

    /// Verifies that `close_projection_enum` replaces the FieldMeaning `anyOf` with the
    /// closed enum branch, so the resulting normalized schema has no open string branch.
    ///
    /// Uses an object schema with a `meaning` property that $ref's FieldMeaning, which is
    /// the real pattern schemars emits for types containing a FieldMeaning field.
    #[test]
    fn closes_field_meaning_enum() {
        // Schema shaped like schemars output for a struct with a FieldMeaning field.
        let input = json!({
            "type": "object",
            "properties": {
                "meaning": { "$ref": "#/$defs/FieldMeaning" }
            },
            "required": ["meaning"],
            "$defs": {
                "FieldMeaning": {
                    "description": "Semantic field meaning.",
                    "anyOf": [
                        { "type": "string", "enum": ["money", "status"] },
                        { "type": "string" }
                    ]
                }
            }
        });
        let out = for_structured_output(input);
        // $defs removed after inlining.
        assert!(out.get("$defs").is_none(), "$defs must be removed");
        // The meaning property must be the closed enum — no anyOf remaining.
        let meaning = &out["properties"]["meaning"];
        assert!(
            meaning.get("anyOf").is_none(),
            "anyOf must be gone after closing; got meaning: {meaning:?}"
        );
        assert_eq!(meaning["type"], json!("string"));
        let enum_val = meaning["enum"]
            .as_array()
            .expect("closed enum must have enum key");
        assert!(enum_val.iter().any(|v| v == "money"));
        assert!(enum_val.iter().any(|v| v == "status"));
        // The open string branch must NOT be represented (only 2 values, not a free string).
        assert_eq!(enum_val.len(), 2);
    }

    /// Verifies that non-projection `anyOf` schemas are NOT closed by the projection pass.
    ///
    /// A schema whose `$defs` contains only a non-projection type should leave its
    /// `anyOf` intact (aside from generic normalization like ref inlining).
    #[test]
    fn non_projection_schema_not_closed() {
        // $defs only has a non-projection type name — closing pass must not activate.
        let input = json!({
            "type": "object",
            "properties": {
                "status": { "$ref": "#/$defs/MyStatus" }
            },
            "required": ["status"],
            "$defs": {
                "MyStatus": {
                    "anyOf": [
                        { "type": "string", "enum": ["active", "inactive"] },
                        { "type": "string" }
                    ]
                }
            }
        });
        let out = for_structured_output(input);
        // Non-projection: anyOf must survive after $ref inlining (closing pass skipped).
        let status_schema = &out["properties"]["status"];
        assert!(
            status_schema.get("anyOf").is_some(),
            "non-projection anyOf must survive; got status schema: {status_schema:?}"
        );
    }

    /// Verifies Intent closing (Shape B: const-per-variant).
    ///
    /// The Intent anyOf has individual `const` branches per known variant plus an
    /// open `{"type":"string"}` Custom branch. The closing pass must collect the
    /// `const` values and emit a closed enum.
    #[test]
    fn closes_intent_enum_const_branch_style() {
        let input = json!({
            "type": "object",
            "properties": {
                "intent": { "$ref": "#/$defs/Intent" }
            },
            "required": ["intent"],
            "$defs": {
                "Intent": {
                    "description": "Structural intent.",
                    "anyOf": [
                        { "type": "string", "const": "browse", "description": "Browse intent." },
                        { "type": "string", "const": "focus",  "description": "Focus intent."  },
                        { "type": "string" }
                    ]
                }
            }
        });
        let out = for_structured_output(input);
        let intent = &out["properties"]["intent"];
        assert!(
            intent.get("anyOf").is_none(),
            "anyOf must be gone after closing; got: {intent:?}"
        );
        assert_eq!(intent["type"], json!("string"));
        let enum_val = intent["enum"]
            .as_array()
            .expect("closed enum must have enum key");
        assert!(enum_val.iter().any(|v| v == "browse"));
        assert!(enum_val.iter().any(|v| v == "focus"));
        // Open-string Custom branch must NOT appear as a value.
        assert_eq!(enum_val.len(), 2, "only const values; open branch dropped");
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
