//! Phase 116 stubs for container renderers. Bodies arrive in Plan 116-04.

use crate::spec::{Element, Spec};
use serde_json::Value;

macro_rules! stub_renderer {
    ($name:ident) => {
        pub(crate) fn $name(_el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
            String::new()
        }
    };
}

stub_renderer!(render_card);
stub_renderer!(render_modal);
stub_renderer!(render_tabs);
stub_renderer!(render_kanban_board);
stub_renderer!(render_page_header);
stub_renderer!(render_grid);
stub_renderer!(render_collapsible);
stub_renderer!(render_form_section);
stub_renderer!(render_button_group);
