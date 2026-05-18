pub(super) const SOURCE: &str = r#"
    // ── Scroll preservation across form actions ───────────────────────────
    //
    // Many dashboard interactions (kanban "Avanza", "Pagato in contanti",
    // delete, etc.) POST and redirect back to the same page. Without help
    // the browser drops the user back at scrollTop=0 and any kanban column
    // scroll is reset. We snapshot scrollY + every `.ferro-kanban-scroll`
    // column's scrollTop on form submit, keyed by current pathname, and
    // restore on the next DOMContentLoaded if the destination pathname
    // matches. Columns are matched by index since DOM is regenerated.
    //
    // Stale entries are read-once: removed from sessionStorage as soon as
    // they're applied, so they don't leak into unrelated future visits.

    function setupScrollPreserve() {
        var KEY = 'ferro:scroll:' + location.pathname;

        // Restore first — synchronously before any other setup mutates layout.
        try {
            var raw = sessionStorage.getItem(KEY);
            if (raw) {
                sessionStorage.removeItem(KEY);
                var state = JSON.parse(raw);
                // Defer to next frame so the browser has finished layout —
                // scrollTop on an `overflow:auto` element only sticks once the
                // content height is known.
                requestAnimationFrame(function() {
                    if (typeof state.window === 'number') {
                        window.scrollTo(0, state.window);
                    }
                    var scrollers = document.querySelectorAll('.ferro-kanban-scroll');
                    if (state.scrollers && state.scrollers.length === scrollers.length) {
                        for (var i = 0; i < scrollers.length; i++) {
                            scrollers[i].scrollTop = state.scrollers[i] || 0;
                        }
                    }
                });
            }
        } catch (e) { /* sessionStorage may be unavailable */ }

        // Capture on submit. Use capture phase so we run before any
        // preventDefault handlers might cancel the navigation.
        document.addEventListener('submit', function() {
            try {
                var state = { window: window.scrollY, scrollers: [] };
                var scrollers = document.querySelectorAll('.ferro-kanban-scroll');
                for (var i = 0; i < scrollers.length; i++) {
                    state.scrollers.push(scrollers[i].scrollTop);
                }
                sessionStorage.setItem(KEY, JSON.stringify(state));
            } catch (e) { /* ignore quota / privacy errors */ }
        }, true);
    }
"#;
