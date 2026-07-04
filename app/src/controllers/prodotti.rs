use ferro::{handler, serde_json, JsonUi, Response};

/// Product list — data-only handler for the prodotti JSON-UI spec.
///
/// All UI structure is in src/views/prodotti.json.
/// This handler assembles only the data the spec needs.
#[handler]
pub async fn index() -> Response {
    let data = serde_json::json!({
        "prodotti": [
            {
                "id": 1,
                "nome": "Maglietta logo",
                "categoria": "Abbigliamento",
                "prezzo": "€ 8,00",
                "stato": "Disponibile"
            },
            {
                "id": 2,
                "nome": "Felpa premium",
                "categoria": "Abbigliamento",
                "prezzo": "€ 89,00",
                "stato": "Disponibile"
            },
            {
                "id": 3,
                "nome": "Cappellino",
                "categoria": "Accessori",
                "prezzo": "€ 15,00",
                "stato": "Esaurito"
            },
            {
                "id": 4,
                "nome": "Sneakers edizione limitata",
                "categoria": "Calzature",
                "prezzo": "€ 120,00",
                "stato": "Disponibile"
            }
        ]
    });
    JsonUi::render_file("src/views/prodotti.json", data)
}

/// GET /prodotti/nuovo — separate create page for a new product.
#[handler]
pub async fn nuovo() -> Response {
    JsonUi::render_file("src/views/prodotto_nuovo.json", serde_json::json!({}))
}

/// POST /prodotti — demo store, redirects back to the list.
#[handler]
pub async fn store() -> Response {
    ferro::redirect!("prodotti.index").into()
}

/// POST /prodotti/{id}/elimina — demo delete, redirects back to the list.
#[handler]
pub async fn elimina(_id: i32) -> Response {
    ferro::redirect!("prodotti.index").into()
}
