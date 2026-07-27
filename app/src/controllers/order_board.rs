use ferro::{handler, serde_json, JsonUi, Response};

/// Order board (kanban) — data-only handler for the order_board JSON-UI spec.
///
/// All UI structure is in src/views/order_board.json.
/// This handler assembles only the data the spec needs.
#[handler]
pub async fn index() -> Response {
    let data = serde_json::json!({
        "orders": [
            {
                "id": 1,
                "title": "#1 — € 16.00",
                "description": "Logo T-Shirt × 2",
                "status": "pending"
            },
            {
                "id": 2,
                "title": "#2 — € 89.00",
                "description": "Premium Hoodie × 1, express shipping",
                "status": "pending"
            },
            {
                "id": 3,
                "title": "#3 — € 45.50",
                "description": "Cap × 3",
                "status": "processing"
            },
            {
                "id": 4,
                "title": "#4 — € 120.00",
                "description": "Limited Edition Sneakers × 1",
                "status": "shipped"
            },
            {
                "id": 5,
                "title": "#5 — € 32.00",
                "description": "Sports Socks × 4",
                "status": "shipped"
            }
        ]
    });
    JsonUi::render_file("src/views/order_board.json", data)
}

/// POST /order-board/{id}/delete — demo delete, redirects back to the board.
#[handler]
pub async fn delete(_id: i32) -> Response {
    ferro::redirect!("order_board.index").into()
}
