//! Phase 116 stubs for atom (leaf) renderers. Bodies arrive in Plan 116-03.
//!
//! Every function in this module returns `String::new()` until Plan 03 ports
//! the v1 HTML emission verbatim from `git show 40385f32^:ferro-json-ui/src/render.rs`.

use crate::spec::{Element, Spec};
use serde_json::Value;

macro_rules! stub_renderer {
    ($name:ident) => {
        pub(crate) fn $name(_el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
            String::new()
        }
    };
}

stub_renderer!(render_text);
stub_renderer!(render_button);
stub_renderer!(render_badge);
stub_renderer!(render_alert);
stub_renderer!(render_separator);
stub_renderer!(render_progress);
stub_renderer!(render_avatar);
stub_renderer!(render_image);
stub_renderer!(render_skeleton);
stub_renderer!(render_breadcrumb);
stub_renderer!(render_pagination);
stub_renderer!(render_description_list);
stub_renderer!(render_empty_state);
stub_renderer!(render_stat_card);
stub_renderer!(render_checklist);
stub_renderer!(render_toast);
stub_renderer!(render_notification_dropdown);
stub_renderer!(render_sidebar);
stub_renderer!(render_header);
stub_renderer!(render_dropdown_menu);
stub_renderer!(render_calendar_cell);
stub_renderer!(render_action_card);
stub_renderer!(render_product_tile);
