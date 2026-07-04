//! Renderer output: messages plus the data contract the host must fill.

use crate::message::A2uiMessage;
use ferro_projections::DataType;
use serde::{Deserialize, Serialize};

/// A rendered surface: the message skeleton and its data contract.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceRendering {
    /// Messages to send (a `createSurface` carrying the component skeleton).
    pub messages: Vec<A2uiMessage>,
    /// Catalog tier actually emitted.
    pub catalog_id: String,
    /// JSON Pointer paths the skeleton binds; the host supplies the data model.
    pub data_contract: DataContract,
}

/// The set of data-model paths a surface skeleton binds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DataContract {
    /// All bindings, in emission order.
    pub bindings: Vec<DataBinding>,
}

/// One bound path. List-item bindings use `*` for the index segment
/// (e.g. `/items/*/total`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataBinding {
    /// Absolute JSON Pointer (with `*` wildcards for template scopes).
    pub path: String,
    /// Expected value type, when derived from a field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,
    /// The `ServiceDef` field this binding projects, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_field: Option<String>,
}

impl DataContract {
    /// Records a binding.
    pub fn bind(
        &mut self,
        path: impl Into<String>,
        data_type: Option<DataType>,
        source_field: Option<&str>,
    ) {
        self.bindings.push(DataBinding {
            path: path.into(),
            data_type,
            source_field: source_field.map(str::to_string),
        });
    }

    /// All bound paths, in order.
    pub fn paths(&self) -> Vec<&str> {
        self.bindings.iter().map(|b| b.path.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::DataType;

    #[test]
    fn contract_bind_and_paths() {
        let mut c = DataContract::default();
        c.bind("/items", None, None);
        c.bind("/items/*/total", Some(DataType::Float), Some("total"));
        assert_eq!(c.paths(), vec!["/items", "/items/*/total"]);
        assert_eq!(c.bindings[1].source_field.as_deref(), Some("total"));
    }
}
