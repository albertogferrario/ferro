use ferro::{handler, serde_json, JsonUi, Response};

/// Product list — data-only handler for the products JSON-UI spec.
///
/// All UI structure is in src/views/products.json.
/// This handler assembles only the data the spec needs.
#[handler]
pub async fn index() -> Response {
    let data = serde_json::json!({
        "products": [
            {
                "id": 1,
                "name": "Logo T-Shirt",
                "category": "Clothing",
                "price": "€ 8.00",
                "status": "Available"
            },
            {
                "id": 2,
                "name": "Premium Hoodie",
                "category": "Clothing",
                "price": "€ 89.00",
                "status": "Available"
            },
            {
                "id": 3,
                "name": "Cap",
                "category": "Accessories",
                "price": "€ 15.00",
                "status": "Out of Stock"
            },
            {
                "id": 4,
                "name": "Limited Edition Sneakers",
                "category": "Footwear",
                "price": "€ 120.00",
                "status": "Available"
            }
        ]
    });
    JsonUi::render_file("src/views/products.json", data)
}

/// GET /products/new — separate create page for a new product.
#[handler]
pub async fn new_form() -> Response {
    JsonUi::render_file("src/views/product_new.json", serde_json::json!({}))
}

/// POST /products — demo store, redirects back to the list.
#[handler]
pub async fn store() -> Response {
    ferro::redirect!("products.index").into()
}

/// POST /products/{id}/delete — demo delete, redirects back to the list.
#[handler]
pub async fn delete(_id: i32) -> Response {
    ferro::redirect!("products.index").into()
}
