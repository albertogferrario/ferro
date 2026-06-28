//! End-to-end integration tests for $each / $if directive lifecycle.
//!
//! Exercises the full pipeline: Spec::from_json -> expand_directives -> render_spec_to_html.
//! These complement the per-module unit tests in Plans 01-04 by catching
//! cross-layer regressions (render layer vs post-expansion element shape).

use ferro_json_ui::{expand_directives, render_spec_to_html, Spec};
use serde_json::json;

/// Build a Spec from a JSON value (panics on parse error).
fn build_spec(value: serde_json::Value) -> Spec {
    Spec::from_json(&value.to_string()).expect("spec parses")
}

/// Expand directives and render against the spec's embedded data.
fn render(mut spec: Spec) -> String {
    let data = spec.data.clone();
    expand_directives(&mut spec);
    render_spec_to_html(&spec, &data)
}

/// Test 1: full kanban fixture with $each over /orders produces one rendered
/// output per row. Mirrors the cassa orders-kanban friction site.
#[test]
fn e2e_orders_kanban_each_produces_n_cards() {
    let spec = build_spec(json!({
        "$schema": "ferro-json-ui/v2",
        "title": "Ordini",
        "root": "kanban_board",
        "elements": {
            "kanban_board": {
                "type": "Card",
                "props": {"title": "Ordini"},
                "children": ["order_card"]
            },
            "order_card": {
                "type": "Card",
                "$each": {"path": "/orders", "as": "order"},
                "props": {
                    "title": {"$data": "/order/order_number"},
                    "description": {"$data": "/order/customer_name"}
                }
            }
        },
        "data": {
            "orders": [
                {"order_number": "ORD-1", "customer_name": "Alice"},
                {"order_number": "ORD-2", "customer_name": "Bob"},
                {"order_number": "ORD-3", "customer_name": "Carol"}
            ]
        }
    }));

    let html = render(spec);

    assert!(html.contains("ORD-1"), "ORD-1 missing; got: {html}");
    assert!(html.contains("ORD-2"), "ORD-2 missing; got: {html}");
    assert!(html.contains("ORD-3"), "ORD-3 missing; got: {html}");
    assert!(html.contains("Alice"), "Alice missing; got: {html}");
    assert!(html.contains("Bob"), "Bob missing; got: {html}");
    assert!(html.contains("Carol"), "Carol missing; got: {html}");
}

/// Test 2: $if directive gates element emission by predicate.
/// Truthy data → element present; falsy data → element absent.
#[test]
fn e2e_conditional_action_button_if_truthy_renders() {
    let base = json!({
        "$schema": "ferro-json-ui/v2",
        "root": "container",
        "elements": {
            "container": {
                "type": "Card",
                "props": {"title": "Container"},
                "children": ["btn_advance"]
            },
            "btn_advance": {
                "type": "Button",
                "$if": {"path": "/can_advance", "operator": "eq", "value": true},
                "props": {"label": "ADVANCE_LABEL_PRESENT"}
            }
        },
        "data": {"can_advance": true}
    });

    // Truthy path: button label must appear.
    let spec_truthy = build_spec(base.clone());
    let html_truthy = render(spec_truthy);
    assert!(
        html_truthy.contains("ADVANCE_LABEL_PRESENT"),
        "truthy path must render the button label; got: {html_truthy}"
    );

    // Falsy path: button label must be absent.
    let mut falsy = base.clone();
    falsy["data"]["can_advance"] = json!(false);
    let spec_falsy = build_spec(falsy);
    let html_falsy = render(spec_falsy);
    assert!(
        !html_falsy.contains("ADVANCE_LABEL_PRESENT"),
        "falsy path must remove the button; got: {html_falsy}"
    );
}

/// Test 3: correlated $each children — sibling templates with the same
/// {path, as} pair produce per-row groupings.
/// After expansion: card-0 contains badge-0, card-1 contains badge-1.
#[test]
fn e2e_correlated_each_children_groups_per_row() {
    let spec = build_spec(json!({
        "$schema": "ferro-json-ui/v2",
        "root": "list_root",
        "elements": {
            "list_root": {
                "type": "Card",
                "props": {"title": "List"},
                "children": ["card"]
            },
            "card": {
                "type": "Card",
                "$each": {"path": "/items", "as": "item"},
                "props": {"title": {"$data": "/item/name"}},
                "children": ["badge"]
            },
            "badge": {
                "type": "Badge",
                "$each": {"path": "/items", "as": "item"},
                "props": {"label": {"$data": "/item/badge_label"}}
            }
        },
        "data": {
            "items": [
                {"name": "ITEM_ONE", "badge_label": "BADGE_ONE"},
                {"name": "ITEM_TWO", "badge_label": "BADGE_TWO"}
            ]
        }
    }));

    let html = render(spec);

    // All four values appear.
    for needle in &["ITEM_ONE", "ITEM_TWO", "BADGE_ONE", "BADGE_TWO"] {
        assert!(html.contains(needle), "{needle} missing; got: {html}");
    }

    // Render order follows children list: ITEM_ONE precedes BADGE_ONE
    // (same card group), and ITEM_TWO precedes BADGE_TWO (second group).
    let one_pos = html.find("ITEM_ONE").unwrap();
    let badge_one_pos = html.find("BADGE_ONE").unwrap();
    let two_pos = html.find("ITEM_TWO").unwrap();
    let badge_two_pos = html.find("BADGE_TWO").unwrap();
    assert!(one_pos < badge_one_pos, "ITEM_ONE must precede BADGE_ONE");
    assert!(
        badge_one_pos < two_pos,
        "BADGE_ONE must precede ITEM_TWO (group boundary)"
    );
    assert!(two_pos < badge_two_pos, "ITEM_TWO must precede BADGE_TWO");
}

/// Test 4: static spec (no directives) is unchanged by expand_directives.
/// Confirms idempotency at the spec-value level for no-directive inputs.
#[test]
fn e2e_static_spec_unchanged_by_expand_directives() {
    let json_text = r#"{
        "$schema": "ferro-json-ui/v2",
        "root": "root",
        "elements": {
            "root": {"type": "Card", "props": {"title": "T"}}
        }
    }"#;
    let mut spec = Spec::from_json(json_text).expect("parses");
    let before = serde_json::to_value(&spec).unwrap();
    expand_directives(&mut spec);
    let after = serde_json::to_value(&spec).unwrap();
    assert_eq!(
        before, after,
        "no-directive spec must be unchanged by expand_directives"
    );
}

/// Test 5: calendar-day empty-day scenario.
///
/// When `bookings` is empty, the `$each`-templated booking card produces no
/// clones. The visibility-gated EmptyState (`has_bookings eq false`) must
/// still render its label text. This is the regression that `label` alias +
/// visibility interaction must satisfy end-to-end.
#[test]
fn e2e_empty_day_shows_empty_state_hides_booking_cards() {
    let spec = build_spec(json!({
        "$schema": "ferro-json-ui/v2",
        "root": "root",
        "elements": {
            "root": {
                "type": "Grid",
                "props": {"columns": 1, "gap": "md"},
                "children": ["booking_list_empty", "booking_card"]
            },
            "booking_list_empty": {
                "type": "EmptyState",
                "props": {"label": "Nessuna prenotazione"},
                "visible": {"path": "/has_bookings", "operator": "eq", "value": false}
            },
            "booking_card": {
                "type": "Card",
                "$each": {"path": "/bookings", "as": "b"},
                "props": {"title": {"$data": "/b/guest_name"}}
            }
        },
        "data": {
            "has_bookings": false,
            "bookings": []
        }
    }));

    let html = render(spec);

    assert!(
        html.contains("Nessuna prenotazione"),
        "EmptyState label must render on an empty day; got: {html}"
    );
    assert!(
        !html.contains("<!-- ferro-json-ui:"),
        "no diagnostic comments expected; got: {html}"
    );
}
