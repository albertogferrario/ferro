//! Phase 116 stubs for data-display renderers. Bodies arrive in Plan 116-05.

use crate::spec::{Element, Spec};
use serde_json::Value;

macro_rules! stub_renderer {
    ($name:ident) => {
        pub(crate) fn $name(_el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
            String::new()
        }
    };
}

stub_renderer!(render_table);
stub_renderer!(render_data_table);
