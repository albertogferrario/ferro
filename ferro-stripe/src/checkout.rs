#[cfg(test)]
mod tests {
    use crate::Error;

    // Import the types that will be defined in this module.
    use super::{CheckoutBuilder, LineItem, Mode};

    #[test]
    fn checkout_builder_new_is_empty() {
        let b = CheckoutBuilder::new(Mode::Payment);
        assert_eq!(b.mode, Mode::Payment);
        assert!(b.line_items.is_empty());
        assert!(b.success_url.is_none());
        assert!(b.cancel_url.is_none());
        assert!(b.metadata.is_empty());
        assert!(b.customer_email.is_none());
        assert!(b.destination.is_none());
        assert!(b.idempotency_key.is_none());
    }

    #[test]
    fn line_item_public_fields_constructable() {
        let li = LineItem {
            name: "Widget".to_string(),
            description: Some("A widget".to_string()),
            unit_amount_cents: 1000,
            quantity: 2,
            currency: "usd".to_string(),
        };
        assert_eq!(li.name, "Widget");
        assert_eq!(li.unit_amount_cents, 1000);
        assert_eq!(li.quantity, 2);
    }

    #[tokio::test]
    async fn checkout_create_missing_key_returns_err() {
        // No .idempotency_key() call — must return Err BEFORE any network call.
        // (Stripe::init is not called in the test binary, so if we reach
        // `Stripe::client()`, we would panic — a passing result here proves
        // the guard fires before the network code.)
        let result = CheckoutBuilder::new(Mode::Payment)
            .success_url("https://example.com/ok")
            .cancel_url("https://example.com/cancel")
            .line_item(LineItem {
                name: "Widget".to_string(),
                description: None,
                unit_amount_cents: 100,
                quantity: 1,
                currency: "usd".to_string(),
            })
            .create()
            .await;
        assert!(
            matches!(result, Err(Error::MissingIdempotencyKey)),
            "expected Err(MissingIdempotencyKey), got {result:?}"
        );
    }
}
