//! A2UI stream message envelopes (v1.0 RC wire format).

use crate::component::Component;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MIME type for A2UI payloads (standardized in spec v0.9.1).
pub const A2UI_MIME_TYPE: &str = "application/a2ui+json";

/// One A2UI stream message. Externally tagged — the wire object has exactly
/// one top-level key naming the message type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum A2uiMessage {
    /// Opens a surface, optionally with initial components and data model.
    #[serde(rename = "createSurface")]
    CreateSurface(CreateSurface),
    /// Adds or replaces components (by ID) on an open surface.
    #[serde(rename = "updateComponents")]
    UpdateComponents(UpdateComponents),
    /// Writes (or deletes) a value in the surface's data model.
    #[serde(rename = "updateDataModel")]
    UpdateDataModel(UpdateDataModel),
    /// Closes a surface.
    #[serde(rename = "deleteSurface")]
    DeleteSurface(DeleteSurface),
    /// Server reply to a client action sent with `wantResponse: true` (v1.0).
    #[serde(rename = "actionResponse")]
    ActionResponse(ActionResponse),
    /// Server-initiated call of a catalog-declared client function (v1.0).
    #[serde(rename = "callFunction")]
    CallFunction(CallFunction),
}

/// `createSurface` payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSurface {
    /// Surface identifier, unique within the connection.
    pub surface_id: String,
    /// Catalog the surface's components are drawn from.
    pub catalog_id: String,
    /// Presentation metadata (e.g. `agentDisplayName`, `iconUrl`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surface_properties: Option<Value>,
    /// Enables client→server data-model transmission.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_data_model: Option<bool>,
    /// Initial flat component list; must contain the `root` component.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<Component>,
    /// Initial data model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_model: Option<Value>,
}

/// `updateComponents` payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponents {
    /// Target surface.
    pub surface_id: String,
    /// Components to add or replace by ID.
    pub components: Vec<Component>,
}

/// `updateDataModel` payload. Omitting `value` deletes the key at `path`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDataModel {
    /// Target surface.
    pub surface_id: String,
    /// JSON Pointer to write at; defaults to the root (`/`) when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Value to write; omitted = delete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

/// `deleteSurface` payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSurface {
    /// Surface to close.
    pub surface_id: String,
}

/// `actionResponse` payload (v1.0 client→server RPC reply).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResponse {
    /// Matches the `actionId` the client sent with the action.
    pub action_id: String,
    /// Result carrying `value` XOR `error`.
    pub action_response: ActionResult,
}

/// Action result: exactly one of `value` / `error` is set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    /// Success payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    /// Error payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

/// `callFunction` payload (v1.0 server→client RPC).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallFunction {
    /// Correlates the client's `functionResponse`.
    pub function_call_id: String,
    /// Whether the server expects a `functionResponse`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub want_response: Option<bool>,
    /// The catalog-declared function to invoke.
    pub call_function: FunctionCall,
}

/// A catalog-declared function reference with arguments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCall {
    /// Function name; must exist in the client's active catalog registry.
    pub call: String,
    /// Function arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Component;

    fn create_msg() -> A2uiMessage {
        A2uiMessage::CreateSurface(CreateSurface {
            surface_id: "s1".into(),
            catalog_id: "cat".into(),
            surface_properties: None,
            send_data_model: Some(true),
            components: vec![Component::new("root", "Column")],
            data_model: None,
        })
    }

    #[test]
    fn create_surface_wire_shape_is_camel_case() {
        let v = serde_json::to_value(create_msg()).unwrap();
        assert_eq!(
            v,
            serde_json::json!({"createSurface": {
                "surfaceId": "s1",
                "catalogId": "cat",
                "sendDataModel": true,
                "components": [{"id": "root", "component": "Column"}]
            }})
        );
    }

    #[test]
    fn every_message_has_exactly_one_top_level_key() {
        let msgs = vec![
            create_msg(),
            A2uiMessage::UpdateComponents(UpdateComponents {
                surface_id: "s1".into(),
                components: vec![],
            }),
            A2uiMessage::UpdateDataModel(UpdateDataModel {
                surface_id: "s1".into(),
                path: None,
                value: Some(serde_json::json!(1)),
            }),
            A2uiMessage::DeleteSurface(DeleteSurface {
                surface_id: "s1".into(),
            }),
            A2uiMessage::ActionResponse(ActionResponse {
                action_id: "a1".into(),
                action_response: ActionResult {
                    value: Some(serde_json::json!("ok")),
                    error: None,
                },
            }),
            A2uiMessage::CallFunction(CallFunction {
                function_call_id: "f1".into(),
                want_response: None,
                call_function: FunctionCall {
                    call: "fmt".into(),
                    args: None,
                },
            }),
        ];
        for m in msgs {
            let v = serde_json::to_value(&m).unwrap();
            assert_eq!(
                v.as_object().unwrap().len(),
                1,
                "message must have one top-level key: {v}"
            );
            let back: A2uiMessage = serde_json::from_value(v).unwrap();
            assert_eq!(m, back);
        }
    }

    #[test]
    fn update_data_model_omits_absent_path_and_value() {
        let m = A2uiMessage::UpdateDataModel(UpdateDataModel {
            surface_id: "s1".into(),
            path: None,
            value: None,
        });
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(
            v,
            serde_json::json!({"updateDataModel": {"surfaceId": "s1"}})
        );
    }

    #[test]
    fn mime_type_constant() {
        assert_eq!(A2UI_MIME_TYPE, "application/a2ui+json");
    }
}
