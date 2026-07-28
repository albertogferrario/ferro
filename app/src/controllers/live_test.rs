use ferro::serde_json;
use ferro::{handler, Event, HttpResponse, JsonUi, Request, Response, Spec};

use crate::projections::live_test::LiveTestEvent;

/// GET /live-test — JSON-UI page with a LiveFragment targeting the live.test projection.
#[handler]
pub async fn index(_req: Request) -> Response {
    let spec_value = serde_json::json!({
        "$schema": "ferro-json-ui/v2",
        "title": "LiveFragment UAT",
        "root": "root",
        "elements": {
            "root": {
                "type": "Grid",
                "props": { "columns": 1 },
                "children": ["heading", "fragment"]
            },
            "heading": {
                "type": "Text",
                "props": { "content": "Counter (updates live via WebSocket)" }
            },
            "fragment": {
                "type": "LiveFragment",
                "props": {
                    "projection": "live.test",
                    "key": "default",
                    "template": {
                        "$schema": "ferro-json-ui/v2",
                        "root": "counter",
                        "elements": {
                            "counter": {
                                "type": "Text",
                                "props": { "content": "Count: {{count}}" }
                            }
                        }
                    }
                }
            }
        }
    });
    let spec: Spec = serde_json::from_value(spec_value)
        .map_err(|e| HttpResponse::text(format!("spec parse error: {e}")).status(500))?;
    JsonUi::render(&spec, &serde_json::json!({ "count": 0 }))
}

/// POST /live-test/trigger — dispatch a LiveTestEvent to update the projection.
#[handler]
pub async fn trigger(_req: Request) -> Response {
    LiveTestEvent { increment: 1 }
        .dispatch()
        .await
        .map_err(|_| HttpResponse::new().status(500))?;
    Ok(HttpResponse::new().status(200))
}
