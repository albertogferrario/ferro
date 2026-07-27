use ferro::{handler, serde_json, JsonUi, Response};

/// Payments list — data-only handler for the payments JSON-UI spec.
///
/// All UI structure is in src/views/payments.json.
/// This handler assembles only the data the spec needs.
#[handler]
pub async fn index() -> Response {
    let data = serde_json::json!({
        "meta": {
            "total_formatted": "€ 1,245.00"
        },
        "payments": [
            {
                "date": "2026-04-20",
                "description": "Monthly subscription",
                "amount": "€ 99.00",
                "status": "Completed"
            },
            {
                "date": "2026-04-15",
                "description": "Order #1042",
                "amount": "€ 246.00",
                "status": "Completed"
            },
            {
                "date": "2026-04-10",
                "description": "Order #1038",
                "amount": "€ 900.00",
                "status": "Pending"
            }
        ]
    });
    JsonUi::render_file("src/views/payments.json", data)
}
