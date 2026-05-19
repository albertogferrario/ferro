pub(super) const SOURCE: &str = r#"
    // ── Scroll preservation across form actions ───────────────────────────
    //
    // Many dashboard interactions (kanban "Avanza", "Pagato in contanti",
    // delete, edit) POST and redirect back to the same page. Without help
    // the browser drops the user back at the top and any inner-scrolled
    // panel (kanban column, <main> in the DashboardLayout) is reset.
    //
    // Three scroll surfaces are captured on submit, keyed by current
    // pathname, and restored on the next DOMContentLoaded if the
    // destination pathname matches:
    //
    //   1. `window` (window.scrollY) — pages where the document scrolls.
    //   2. `main` (<main>.scrollTop) — DashboardLayout pins html/body to
    //      100vh and lets only <main> overflow; window.scrollY is always 0
    //      on those pages. Captured by tag name so the same key restores
    //      across the DOM regeneration that a 302 redirect triggers.
    //   3. `.ferro-kanban-scroll` columns — matched by index since the
    //      DOM is regenerated on the destination page.
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
                    if (typeof state.main === 'number') {
                        var main = document.querySelector('main');
                        if (main) main.scrollTop = state.main;
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
                var state = { window: window.scrollY, main: 0, scrollers: [] };
                var main = document.querySelector('main');
                if (main) state.main = main.scrollTop;
                var scrollers = document.querySelectorAll('.ferro-kanban-scroll');
                for (var i = 0; i < scrollers.length; i++) {
                    state.scrollers.push(scrollers[i].scrollTop);
                }
                sessionStorage.setItem(KEY, JSON.stringify(state));
            } catch (e) { /* ignore quota / privacy errors */ }
        }, true);
    }
"#;
