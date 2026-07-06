use ferro::{handler, serde_json, JsonUi, Response};

/// Orders kanban — data-only handler for the ordini JSON-UI spec.
///
/// All UI structure is in src/views/ordini.json.
/// This handler assembles only the data the spec needs.
#[handler]
pub async fn index() -> Response {
    let data = serde_json::json!({
        "ordini": [
            {
                "id": 1,
                "titolo": "#1 — € 16,00",
                "descrizione": "Maglietta logo × 2",
                "stato": "in_attesa"
            },
            {
                "id": 2,
                "titolo": "#2 — € 89,00",
                "descrizione": "Felpa premium × 1, spedizione espressa",
                "stato": "in_attesa"
            },
            {
                "id": 3,
                "titolo": "#3 — € 45,50",
                "descrizione": "Cappellino × 3",
                "stato": "in_lavorazione"
            },
            {
                "id": 4,
                "titolo": "#4 — € 120,00",
                "descrizione": "Sneakers edizione limitata × 1",
                "stato": "spedito"
            },
            {
                "id": 5,
                "titolo": "#5 — € 32,00",
                "descrizione": "Calze sportive × 4",
                "stato": "spedito"
            }
        ]
    });
    JsonUi::render_file("src/views/ordini.json", data)
}

/// POST /ordini/{id}/elimina — demo delete, redirects back to the board.
#[handler]
pub async fn elimina(_id: i32) -> Response {
    ferro::redirect!("ordini.index").into()
}
