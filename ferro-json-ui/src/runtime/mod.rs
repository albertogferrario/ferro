//! Built-in JavaScript runtime for ferro-json-ui (split per concern).
//!
//! Each submodule contributes one `setup*` function. The assembled bundle
//! wraps them in a single IIFE with a `ferroRuntime()` dispatcher invoked
//! on DOMContentLoaded. The emitted output is a single string — no extra
//! HTTP requests.

mod dismissibles;
mod dropdowns;
mod filters;
mod form_guards;
mod hero_lazy;
mod kanban;
mod modals;
mod notifications;
mod numpad;
mod scroll_preserve;
mod sidebar;
mod sse;
mod tabs;
mod tiles;
mod toasts;

use std::sync::LazyLock;

/// Assembled JS runtime bundle. Lazily concatenated from per-concern
/// submodules on first access; the resulting string is stable for the
/// process lifetime.
pub static FERRO_RUNTIME_JS: LazyLock<String> = LazyLock::new(|| {
    let mut s = String::with_capacity(8 * 1024);
    s.push_str("(function() {\n    'use strict';\n");
    s.push_str(sse::SOURCE);
    s.push_str(tabs::SOURCE);
    s.push_str(toasts::SOURCE);
    s.push_str(dismissibles::SOURCE);
    s.push_str(notifications::SOURCE);
    s.push_str(dropdowns::SOURCE);
    s.push_str(modals::SOURCE);
    s.push_str(sidebar::SOURCE);
    s.push_str(form_guards::SOURCE);
    s.push_str(tiles::SOURCE);
    s.push_str(numpad::SOURCE);
    s.push_str(filters::SOURCE);
    s.push_str(kanban::SOURCE);
    s.push_str(scroll_preserve::SOURCE);
    s.push_str(hero_lazy::SOURCE);
    s.push_str(
        "\n    function ferroRuntime() {\n\
         \x20       // Run each setup in isolation: a throw in one concern (e.g. an\n\
         \x20       // invalid selector built from a malformed attribute) must not\n\
         \x20       // abort the remaining setups.\n\
         \x20       var setups = [\n\
         \x20           setupScrollPreserve,\n\
         \x20           setupSSE,\n\
         \x20           setupTabs,\n\
         \x20           setupDismissibles,\n\
         \x20           setupNotifications,\n\
         \x20           setupDropdowns,\n\
         \x20           setupKanban,\n\
         \x20           setupSidebar,\n\
         \x20           setupFormGuards,\n\
         \x20           setupTiles,\n\
         \x20           setupNumpad,\n\
         \x20           setupFilters,\n\
         \x20           setupModals,\n\
         \x20           setupToasts,\n\
         \x20           setupLazyHeroes\n\
         \x20       ];\n\
         \x20       for (var i = 0; i < setups.length; i++) {\n\
         \x20           try { setups[i](); } catch (err) { /* isolated per concern */ }\n\
         \x20       }\n\
         \x20   }\n\
         \x20   document.addEventListener('DOMContentLoaded', ferroRuntime);\n\
         })();\n",
    );
    s
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_classes_use_semantic_tokens() {
        assert!(FERRO_RUNTIME_JS.contains("bg-primary"));
        assert!(FERRO_RUNTIME_JS.contains("bg-success"));
        assert!(FERRO_RUNTIME_JS.contains("bg-warning"));
        assert!(FERRO_RUNTIME_JS.contains("bg-destructive"));
        assert!(!FERRO_RUNTIME_JS.contains("bg-blue-500"));
        assert!(!FERRO_RUNTIME_JS.contains("bg-green-500"));
        assert!(!FERRO_RUNTIME_JS.contains("bg-yellow-500"));
        assert!(!FERRO_RUNTIME_JS.contains("bg-red-500"));
        // Retired toast vocabulary must not resurface: the SSR emitter and
        // this runtime were renamed to `data-toast-tone` in lockstep.
        assert!(!FERRO_RUNTIME_JS.contains("data-toast-variant"));
        // Retired motion vocabulary: the toast fade uses the duration-base
        // token and dismissal is transitionend-driven (500ms fallback bound).
        assert!(!FERRO_RUNTIME_JS.contains("duration-300"));
        assert!(!FERRO_RUNTIME_JS.contains("duration-150"));
        assert!(FERRO_RUNTIME_JS.contains("duration-base"));
        assert!(FERRO_RUNTIME_JS.contains("transitionend"));
    }

    #[test]
    fn tab_switcher_uses_semantic_tokens() {
        assert!(FERRO_RUNTIME_JS.contains("border-primary"));
        assert!(FERRO_RUNTIME_JS.contains("text-primary"));
        assert!(FERRO_RUNTIME_JS.contains("text-text-muted"));
        assert!(!FERRO_RUNTIME_JS.contains("border-blue-600"));
        assert!(!FERRO_RUNTIME_JS.contains("text-blue-600"));
        assert!(!FERRO_RUNTIME_JS.contains("text-gray-500"));
    }

    #[test]
    fn toast_uses_semantic_text_color() {
        assert!(FERRO_RUNTIME_JS.contains("text-primary-foreground"));
        assert!(!FERRO_RUNTIME_JS.contains("text-white"));
    }

    /// JS↔SSR lockstep: the runtime's VARIANT_CLASSES must carry the exact
    /// tone-class strings the SSR `render_toast` composes from
    /// `render::classes::TOAST_TONE_*`, plus the shared backdrop blur, so
    /// server-rendered and JS-created toasts look identical.
    #[test]
    fn toast_tone_classes_match_ssr() {
        use crate::render::classes::{
            TOAST_TONE_DESTRUCTIVE, TOAST_TONE_NEUTRAL, TOAST_TONE_SUCCESS, TOAST_TONE_WARNING,
        };
        for tone_classes in [
            TOAST_TONE_NEUTRAL,
            TOAST_TONE_SUCCESS,
            TOAST_TONE_WARNING,
            TOAST_TONE_DESTRUCTIVE,
        ] {
            assert!(
                FERRO_RUNTIME_JS.contains(tone_classes),
                "runtime VARIANT_CLASSES drifted from SSR toast tone classes: missing `{tone_classes}`"
            );
        }
        assert!(
            FERRO_RUNTIME_JS.contains("backdrop-blur-md"),
            "runtime toast shell drifted from SSR: missing `backdrop-blur-md`"
        );
    }

    /// Popover-based dropdown wiring: the panel uses the HTML `popover`
    /// attribute so the browser lifts it into the top layer (escaping any
    /// `overflow:hidden` ancestor and the surrounding z-index stack). We
    /// supply only the anchor positioning and the close-on-scroll behavior.
    #[test]
    fn test_runtime_contains_popover_dropdown_wiring() {
        assert!(FERRO_RUNTIME_JS.contains("data-popover-menu"));
        assert!(FERRO_RUNTIME_JS.contains(":popover-open"));
        assert!(FERRO_RUNTIME_JS.contains("hidePopover"));
        assert!(FERRO_RUNTIME_JS.contains("positionUnderTrigger"));
        assert!(FERRO_RUNTIME_JS.contains("getBoundingClientRect"));
    }

    #[test]
    fn test_runtime_contains_modal_wiring() {
        assert!(FERRO_RUNTIME_JS.contains("setupModals"));
        assert!(FERRO_RUNTIME_JS.contains("data-modal-open"));
        assert!(FERRO_RUNTIME_JS.contains("showModal"));
        assert!(FERRO_RUNTIME_JS.contains("data-modal-close"));
    }

    #[test]
    fn test_runtime_contains_toast_from_url() {
        assert!(FERRO_RUNTIME_JS.contains("initToastFromUrl"));
        assert!(FERRO_RUNTIME_JS.contains("URLSearchParams"));
        assert!(FERRO_RUNTIME_JS.contains("history.replaceState"));
    }

    #[test]
    fn runtime_contains_init_tab_from_url() {
        assert!(
            FERRO_RUNTIME_JS.contains("initTabFromUrl"),
            "FERRO_RUNTIME_JS must include initTabFromUrl for F3 — URL-driven tab init"
        );
        assert!(
            FERRO_RUNTIME_JS.contains("URLSearchParams"),
            "FERRO_RUNTIME_JS must use URLSearchParams to parse ?tab= for initTabFromUrl"
        );
    }

    #[test]
    fn bundle_contains_dispatcher() {
        assert!(FERRO_RUNTIME_JS.contains("function ferroRuntime()"));
        assert!(FERRO_RUNTIME_JS.contains("DOMContentLoaded"));
        assert!(FERRO_RUNTIME_JS.contains("ferroRuntime"));
    }

    #[test]
    fn bundle_contains_all_setup_functions() {
        for fn_name in [
            "setupSSE",
            "setupTabs",
            "setupToasts",
            "setupSidebar",
            "setupDropdowns",
            "setupModals",
            "setupDismissibles",
            "setupNotifications",
            "setupFormGuards",
            "setupTiles",
            "setupNumpad",
            "setupFilters",
            "setupKanban",
            "setupScrollPreserve",
            "setupLazyHeroes",
        ] {
            assert!(
                FERRO_RUNTIME_JS.contains(fn_name),
                "bundle missing {fn_name}"
            );
        }
    }

    #[test]
    fn bundle_is_single_iife() {
        assert!(FERRO_RUNTIME_JS.starts_with("(function() {"));
        assert!(FERRO_RUNTIME_JS.trim_end().ends_with("})();"));
    }

    /// Every setup must be registered in the dispatcher's `setups` array and
    /// invoked through the per-concern try/catch loop so one throwing setup
    /// cannot abort the rest.
    #[test]
    fn dispatcher_invokes_every_setup() {
        let js: &str = FERRO_RUNTIME_JS.as_str();
        let dispatcher_start = js.find("function ferroRuntime()").unwrap();
        let dispatcher = &js[dispatcher_start..];
        for name in [
            "setupSSE",
            "setupTabs",
            "setupToasts",
            "setupSidebar",
            "setupDropdowns",
            "setupModals",
            "setupDismissibles",
            "setupNotifications",
            "setupFormGuards",
            "setupTiles",
            "setupNumpad",
            "setupFilters",
            "setupKanban",
            "setupScrollPreserve",
            "setupLazyHeroes",
        ] {
            assert!(
                dispatcher.contains(name),
                "dispatcher setups array missing {name}"
            );
        }
        assert!(
            dispatcher.contains("try { setups[i](); } catch"),
            "dispatcher must invoke setups through the per-concern try/catch"
        );
    }

    #[test]
    fn runtime_contains_lazy_hero_setup() {
        assert!(FERRO_RUNTIME_JS.contains("setupLazyHeroes"));
        assert!(FERRO_RUNTIME_JS.contains("data-lazy-hero"));
        assert!(FERRO_RUNTIME_JS.contains("data-lazy-hero-margin"));
        assert!(FERRO_RUNTIME_JS.contains("data-lazy-hero-promoted"));
        assert!(FERRO_RUNTIME_JS.contains("IntersectionObserver"));
        assert!(FERRO_RUNTIME_JS.contains("preload"));
        // JS source uses single quotes (`setAttribute('preload', 'auto')`),
        // matching sibling-runtime convention; assert the single-quoted literal.
        assert!(FERRO_RUNTIME_JS.contains("'auto'"));
        assert!(FERRO_RUNTIME_JS.contains("unobserve"));
    }

    /// SC-4: inline-source assertion confirming the double-submit guard contract
    /// is present in the assembled bundle (setupFormGuards extension, D-13/D-14/D-15).
    #[test]
    fn runtime_wires_disable_on_submit() {
        assert!(
            FERRO_RUNTIME_JS.contains("data-disable-on-submit"),
            "bundle must contain data-disable-on-submit for the double-submit guard (SC-4)"
        );
    }

    /// SC-3: inline-source assertions confirming the numpad and filter runtime
    /// contracts are present in the assembled bundle.
    #[test]
    fn runtime_exposes_numpad_and_filter_contract() {
        // Numpad attribute contract
        assert!(
            FERRO_RUNTIME_JS.contains("data-numpad-key"),
            "bundle must contain data-numpad-key attribute selector"
        );
        assert!(
            FERRO_RUNTIME_JS.contains("data-numpad-target"),
            "bundle must contain data-numpad-target attribute"
        );
        assert!(
            FERRO_RUNTIME_JS.contains("data-numpad-input"),
            "bundle must contain data-numpad-input attribute"
        );
        assert!(
            FERRO_RUNTIME_JS.contains("data-numpad-display"),
            "bundle must contain data-numpad-display attribute"
        );
        // D-04: every key tap dispatches a bubbling input event
        assert!(
            FERRO_RUNTIME_JS.contains("bubbles: true"),
            "bundle must dispatch bubbling input event on numpad key tap (D-04)"
        );
        // Filter attribute contract
        assert!(
            FERRO_RUNTIME_JS.contains("data-filter-tokens"),
            "bundle must contain data-filter-tokens for token matching"
        );
        assert!(
            FERRO_RUNTIME_JS.contains("data-filter-search"),
            "bundle must contain data-filter-search for text filtering"
        );
        assert!(
            FERRO_RUNTIME_JS.contains("data-filter-text"),
            "bundle must contain data-filter-text as the universal tile marker"
        );
    }
}
