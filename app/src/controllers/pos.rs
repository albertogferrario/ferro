use ferro::{
    derive_intents, handler, register_template, serde_json, ActionDef, DataType, FieldMeaning,
    Intent, IntentHint, JsonUi, JsonUiRenderer, Renderer, Response, ServiceDef, VisualContext,
};

/// The POS register ServiceDef. Named "pos" so the derived confirm-action path
/// (`/{service}/{action}`) resolves to the `pos.confirm` route.
pub fn pos_service_def() -> ServiceDef {
    ServiceDef::new("pos")
        .display_name("POS Register")
        .field("id", DataType::String, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .field("price", DataType::String, FieldMeaning::Money)
        .action(ActionDef::new("confirm").display_name("Confirm order"))
        .intent_hint(IntentHint::Primary(Intent::Collect))
}

/// Demo product rows. Each row carries the entity fields PLUS the register
/// data contract keys `price_cents` (integer cents) and `field` (hidden-input
/// name). `price` (display) and `price_cents` derive from ONE source (cents).
pub fn pos_products() -> Vec<serde_json::Value> {
    // (name, price_cents) — display string derived from cents keeps the two in sync.
    let items: [(&str, u64); 24] = [
        ("Espresso", 120),
        ("Cappuccino", 180),
        ("Croissant", 150),
        ("Orange Juice", 350),
        ("Club Sandwich", 300),
        ("Toast", 400),
        ("Water 0.5L", 100),
        ("Cola", 250),
        ("Draft Beer", 500),
        ("Aperol Spritz", 600),
        ("Prosciutto Roll", 550),
        ("Garden Salad", 800),
        ("Mini Pizza", 280),
        ("Ice Cream Cup", 350),
        ("Iced Tea", 220),
        ("ACE Juice", 280),
        ("Muffin", 250),
        ("Doughnut", 180),
        ("Prosecco Glass", 450),
        ("Amaro", 350),
        ("Chinotto", 250),
        ("Focaccia", 320),
        ("Fruit Cup", 400),
        ("Ginseng Coffee", 150),
    ];
    items
        .iter()
        .enumerate()
        .map(|(i, (name, cents))| {
            let id = (i + 1).to_string();
            serde_json::json!({
                "id": id,
                "name": name,
                "price": format!("€ {}.{:02}", cents / 100, cents % 100),
                "price_cents": cents,
                "field": format!("qty_{}", i + 1),
            })
        })
        .collect()
}

/// GET /pos — projection-derived POS register screen.
///
/// Derives the Spec from `pos_service_def()` via `JsonUiRenderer` with
/// `register_template()` overriding the Collect intent to the Register layout.
/// The rendered page carries the `ferro-fill` class chain (fill_viewport=true).
#[handler]
pub async fn index() -> Response {
    let service = pos_service_def();
    let intents = derive_intents(&service);
    let ctx = VisualContext {
        templates: Some(register_template()),
        ..Default::default()
    };
    let spec = JsonUiRenderer
        .render(&service, &intents, &ctx)
        .map_err(|e| ferro::error_response!(500, format!("pos projection failed: {e}")))?;
    // Projection convention: `$each`/`data_path` point at `/data/{service}`,
    // so the handler payload nests rows under a top-level `data` key.
    let data = serde_json::json!({ "data": { "pos": pos_products() } });
    JsonUi::render(&spec, &data)
}

/// POST /pos/confirm — demo confirm, redirects back to the register.
#[handler]
pub async fn confirm() -> Response {
    ferro::redirect!("pos.index").into()
}
