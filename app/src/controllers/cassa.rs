use ferro::{
    derive_intents, handler, register_template, serde_json, ActionDef, DataType, FieldMeaning,
    Intent, IntentHint, JsonUi, JsonUiRenderer, Renderer, Response, ServiceDef, VisualContext,
};

/// The register ServiceDef for the /cassa demo. Named "cassa" so the derived
/// confirm-action path (`/{service}/{action}`) resolves to the existing
/// `cassa.conferma` route. Italian display copy lives in app-land (allowed —
/// ferro-* crates stay neutral).
pub fn cassa_service_def() -> ServiceDef {
    ServiceDef::new("cassa")
        .display_name("Cassa")
        .field("id", DataType::String, FieldMeaning::Identifier)
        .field("nome", DataType::String, FieldMeaning::EntityName)
        .field("prezzo", DataType::String, FieldMeaning::Money)
        .action(ActionDef::new("conferma").display_name("Conferma ordine"))
        .intent_hint(IntentHint::Primary(Intent::Collect))
}

/// Demo product rows. Each row carries the entity fields PLUS the register
/// data contract keys `price_cents` (integer cents) and `field` (hidden-input
/// name). `prezzo` (display) and `price_cents` derive from ONE source (cents).
pub fn cassa_products() -> Vec<serde_json::Value> {
    // (name, price_cents) — display string derived from cents keeps the two in sync.
    let items: [(&str, u64); 24] = [
        ("Caffè", 120),
        ("Cappuccino", 180),
        ("Cornetto", 150),
        ("Spremuta", 350),
        ("Tramezzino", 300),
        ("Toast", 400),
        ("Acqua 0,5l", 100),
        ("Coca Cola", 250),
        ("Birra media", 500),
        ("Aperol Spritz", 600),
        ("Panino crudo", 550),
        ("Insalatona", 800),
        ("Pizzetta", 280),
        ("Gelato coppa", 350),
        ("Tè freddo", 220),
        ("Succo ACE", 280),
        ("Muffin", 250),
        ("Krapfen", 180),
        ("Prosecco calice", 450),
        ("Amaro", 350),
        ("Chinotto", 250),
        ("Focaccia", 320),
        ("Macedonia", 400),
        ("Ginseng", 150),
    ];
    items
        .iter()
        .enumerate()
        .map(|(i, (nome, cents))| {
            let id = (i + 1).to_string();
            serde_json::json!({
                "id": id,
                "nome": nome,
                "prezzo": format!("€ {},{:02}", cents / 100, cents % 100),
                "price_cents": cents,
                "field": format!("qty_{}", i + 1),
            })
        })
        .collect()
}

/// GET /cassa — projection-derived POS register screen.
///
/// Derives the Spec from `cassa_service_def()` via `JsonUiRenderer` with
/// `register_template()` overriding the Collect intent to the Register layout.
/// The rendered page carries the `ferro-fill` class chain (fill_viewport=true).
#[handler]
pub async fn index() -> Response {
    let service = cassa_service_def();
    let intents = derive_intents(&service);
    let ctx = VisualContext {
        templates: Some(register_template()),
        ..Default::default()
    };
    let spec = JsonUiRenderer
        .render(&service, &intents, &ctx)
        .map_err(|e| ferro::error_response!(500, format!("cassa projection failed: {e}")))?;
    // Projection convention: `$each`/`data_path` point at `/data/{service}`,
    // so the handler payload nests rows under a top-level `data` key.
    let data = serde_json::json!({ "data": { "cassa": cassa_products() } });
    JsonUi::render(&spec, &data)
}

/// POST /cassa/conferma — demo confirm, redirects back to the register.
#[handler]
pub async fn conferma() -> Response {
    ferro::redirect!("cassa.index").into()
}
