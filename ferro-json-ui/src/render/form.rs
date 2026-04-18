//! Phase 116 stubs for form-control renderers. Bodies arrive in Plan 116-05.

use crate::spec::{Element, Spec};
use serde_json::Value;

macro_rules! stub_renderer {
    ($name:ident) => {
        pub(crate) fn $name(_el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
            String::new()
        }
    };
}

stub_renderer!(render_form);
stub_renderer!(render_input);
stub_renderer!(render_select);
stub_renderer!(render_checkbox);
stub_renderer!(render_switch);
