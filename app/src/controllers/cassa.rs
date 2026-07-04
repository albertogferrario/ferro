use ferro::{handler, serde_json, JsonUi, Response};

/// POS register — data-only handler for the cassa JSON-UI spec.
///
/// Demonstrates the viewport-fill workspace: `Spec.fill_viewport` + a
/// fill-mode Grid give the cart pane and the product grid independent
/// internal scrollbars; the document itself never scrolls.
#[handler]
pub async fn index() -> Response {
    let nomi = [
        ("Caffè", "€ 1,20"),
        ("Cappuccino", "€ 1,80"),
        ("Cornetto", "€ 1,50"),
        ("Spremuta", "€ 3,50"),
        ("Tramezzino", "€ 3,00"),
        ("Toast", "€ 4,00"),
        ("Acqua 0,5l", "€ 1,00"),
        ("Coca Cola", "€ 2,50"),
        ("Birra media", "€ 5,00"),
        ("Aperol Spritz", "€ 6,00"),
        ("Panino crudo", "€ 5,50"),
        ("Insalatona", "€ 8,00"),
        ("Pizzetta", "€ 2,80"),
        ("Gelato coppa", "€ 3,50"),
        ("Tè freddo", "€ 2,20"),
        ("Succo ACE", "€ 2,80"),
        ("Muffin", "€ 2,50"),
        ("Krapfen", "€ 1,80"),
        ("Prosecco calice", "€ 4,50"),
        ("Amaro", "€ 3,50"),
        ("Chinotto", "€ 2,50"),
        ("Focaccia", "€ 3,20"),
        ("Macedonia", "€ 4,00"),
        ("Ginseng", "€ 1,50"),
    ];
    let prodotti: Vec<serde_json::Value> = nomi
        .iter()
        .enumerate()
        .map(|(i, (nome, prezzo))| {
            serde_json::json!({
                "id": (i + 1).to_string(),
                "nome": nome,
                "prezzo": prezzo,
                "field": format!("qty_{}", i + 1)
            })
        })
        .collect();

    let data = serde_json::json!({
        "badge_articoli": "3 articoli",
        "totale": "€ 8,50",
        "carrello": [
            { "id": 1, "prodotto": "Caffè", "qta": "2", "importo": "€ 2,40" },
            { "id": 3, "prodotto": "Cornetto", "qta": "1", "importo": "€ 1,50" },
            { "id": 10, "prodotto": "Aperol Spritz", "qta": "1", "importo": "€ 6,00" }
        ],
        "prodotti": prodotti
    });
    JsonUi::render_file("src/views/cassa.json", data)
}

/// POST /cassa/conferma — demo confirm, redirects back to the register.
#[handler]
pub async fn conferma() -> Response {
    ferro::redirect!("cassa.index").into()
}

/// POST /cassa/rimuovi/{id} — demo line-item removal, redirects back.
#[handler]
pub async fn rimuovi(_id: i32) -> Response {
    ferro::redirect!("cassa.index").into()
}
