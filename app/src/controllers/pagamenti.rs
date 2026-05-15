use ferro::{handler, serde_json, JsonUi, Response};

/// List payments — data-only handler for the pagamenti JSON-UI spec.
///
/// All UI structure is in src/views/pagamenti.json.
/// This handler assembles only the data the spec needs.
#[handler]
pub async fn index() -> Response {
    let data = serde_json::json!({
        "meta": {
            "totale_formattato": "€ 1.245,00"
        },
        "pagamenti": [
            {
                "data": "2026-04-20",
                "descrizione": "Abbonamento mensile",
                "importo": "€ 99,00",
                "stato": "Completato"
            },
            {
                "data": "2026-04-15",
                "descrizione": "Ordine #1042",
                "importo": "€ 246,00",
                "stato": "Completato"
            },
            {
                "data": "2026-04-10",
                "descrizione": "Ordine #1038",
                "importo": "€ 900,00",
                "stato": "In attesa"
            }
        ]
    });
    JsonUi::render_file("views/pagamenti.json", data)
}
