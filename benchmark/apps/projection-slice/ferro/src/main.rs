//! Render-evidence harness for the projection-compression benchmark slice.
//!
//! Proves the `service::service_def()` declaration yields REAL data-bound UI for
//! every working intent — not vaporware. For each surface it:
//!   1. ranks intents via `derive_intents`,
//!   2. renders the JSON-UI Spec via `JsonUiRenderer` (the projection pipeline),
//!   3. binds a fixed sample dataset and renders to HTML,
//! then writes both artifacts to `rendered/`.
//!
//! The harness itself is NOT part of the measured authoring surface — only
//! `service::service_def()` is (see RESULTS.md). This file exists solely to
//! capture committable evidence.

mod service;

use std::fs;
use std::path::Path;

use ferro_json_ui::{
    render_spec_to_html_with_plugins, JsonUiRenderer, RenderMode, Spec, VisualContext,
};
use ferro_projections::{
    derive_intents, BaseContext, Intent, IntentScore, Renderer, ServiceDef,
};

/// Fixed sample dataset the rendered Specs bind against. Proves data binding:
/// the rendered HTML contains these literal values, not placeholders.
fn sample_data() -> serde_json::Value {
    serde_json::json!({
        "data": {
            "product": [
                { "id": 1, "name": "Aeron Chair",   "price": 1395.0, "stock": 12, "status": "active" },
                { "id": 2, "name": "Standing Desk",  "price": 649.0,  "stock": 4,  "status": "draft" },
                { "id": 3, "name": "Monitor Arm",    "price": 129.0,  "stock": 27, "status": "discontinued" }
            ]
        }
    })
}

/// Build the `VisualContext` selecting `intent` at the right index in `intents`,
/// in the given render `mode`. Falls back to index 0 if the intent is absent.
/// A single-element intent slice naming the surface to render at index 0.
///
/// `JsonUiRenderer.render` projects whatever intent sits at
/// `ctx.base.intent_index`; pointing it at a one-element slice makes the surface
/// deterministic and independent of derivation ranking. `derive_intents` still
/// runs in `main` and its ranking is printed as honest context — this slice only
/// SELECTS which template to render, it does not fabricate confidence.
fn intent_slice(intent: Intent) -> Vec<IntentScore> {
    vec![IntentScore {
        intent,
        confidence: 1.0,
        matching_signals: Vec::new(),
    }]
}

/// Render one surface: write `<name>.json` (the Spec) and `<name>.html`
/// (the data-bound HTML). Returns a one-line status for the console log.
fn capture(
    out: &Path,
    name: &str,
    service: &ServiceDef,
    intent: Intent,
    mode: RenderMode,
    data: &serde_json::Value,
) -> Result<String, Box<dyn std::error::Error>> {
    let label = intent.label().to_string();
    let intents = intent_slice(intent);
    let ctx = VisualContext {
        base: BaseContext {
            intent_index: 0,
            ..Default::default()
        },
        mode,
        templates: None,
    };
    let spec: Spec = JsonUiRenderer.render(service, &intents, &ctx)?;

    // JSON-UI Spec — the projection pipeline's structural output.
    let spec_json = serde_json::to_string_pretty(&spec)?;
    fs::write(out.join(format!("{name}.json")), &spec_json)?;

    // Data-bound HTML — merge the sample dataset, then render. Proves the Spec
    // binds real values (resolve_expressions wires $data paths to the dataset).
    let bound = spec.clone().merge_data(data.clone());
    let html = render_spec_to_html_with_plugins(&bound, data).html;
    fs::write(out.join(format!("{name}.html")), &html)?;

    let root = bound
        .elements
        .get(&bound.root)
        .map(|e| e.type_name.as_str())
        .unwrap_or("?");
    Ok(format!(
        "{name:<10} intent={label:<9} root={root:<13} spec={}B html={}B",
        spec_json.len(),
        html.len()
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = service::service_def();
    let intents = derive_intents(&service);

    // Report the derived intent ranking — context for the render evidence.
    println!("Derived intents for `{}`:", service.name);
    for s in &intents {
        println!("  {:<9} confidence={:.2}", s.intent.label(), s.confidence);
    }
    println!();

    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("rendered");
    fs::create_dir_all(&out)?;

    let data = sample_data();

    // The five surfaces that render data-bound today (per the verified ground
    // truth). Collect uses Input mode (the create form); the rest are Display.
    let surfaces = [
        ("browse", Intent::Browse, RenderMode::Display),
        ("focus", Intent::Focus, RenderMode::Display),
        ("summarize", Intent::Summarize, RenderMode::Display),
        ("process", Intent::Process, RenderMode::Display),
        ("collect", Intent::Collect, RenderMode::Input),
    ];

    for (name, intent, mode) in surfaces {
        let line = capture(&out, name, &service, intent, mode, &data)?;
        println!("wrote {line}");
    }

    println!("\nRender evidence written to {}", out.display());
    Ok(())
}
