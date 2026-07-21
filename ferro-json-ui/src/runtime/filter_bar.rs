pub(super) const SOURCE: &str = r#"
    // ── Filter bar ─────────────────────────────────────────────────────────
    //
    // List-page toolbar row: active-filter chips + add-filter affordance +
    // saved-view switcher. Chip removal is a GET link — intercepted by the
    // 248 nav runtime for free (no JS needed). This module handles only the
    // "modificata" canonical-query detection and the saved-view switcher.
    // Idempotent via data-filter-bar-init guard.

    function canonicalizeQueryString(qs) {
        // Sort param keys + sort multi-value params for a stable comparison string.
        try {
            var raw = qs.replace(/^\?/, '');
            if (!raw) return '';
            var params = new URLSearchParams(raw);
            var keys = [];
            params.forEach(function(_, k) {
                if (keys.indexOf(k) === -1) keys.push(k);
            });
            keys.sort();
            var parts = [];
            for (var i = 0; i < keys.length; i++) {
                var vals = params.getAll(keys[i]).slice().sort();
                for (var j = 0; j < vals.length; j++) {
                    parts.push(encodeURIComponent(keys[i]) + '=' + encodeURIComponent(vals[j]));
                }
            }
            return parts.join('&');
        } catch (_) { return qs; }
    }

    function checkModificata(bar) {
        try {
            var switcher = bar.querySelector('[data-saved-state]');
            if (!switcher) return;
            var saved = switcher.getAttribute('data-saved-state') || '';
            var current = canonicalizeQueryString(window.location.search);
            var modified = canonicalizeQueryString(saved) !== current;
            var indicator = bar.querySelector('[data-modificata-indicator]');
            if (indicator) indicator.style.display = modified ? '' : 'none';
            // Update the save button: show when modified.
            var saveBtn = bar.querySelector('.fjui-filter-bar__save-btn');
            if (saveBtn) saveBtn.style.display = modified ? '' : 'none';
        } catch (_) {}
    }

    function attachFilterBarBehavior(bar) {
        checkModificata(bar);
        // The view switcher navigates via window.location.assign which the nav
        // runtime intercepts automatically. Wire keyboard activation.
        var switcher = bar.querySelector('[data-saved-state]');
        if (switcher) {
            switcher.addEventListener('keydown', function(e) {
                if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    var href = switcher.getAttribute('href') || switcher.getAttribute('data-saved-view-url');
                    if (href) {
                        try { window.location.assign(href); } catch (_) {}
                    }
                }
            });
        }
    }

    function setupFilterBar() {
        try {
            var bars = document.querySelectorAll('[data-filter-bar]');
            for (var i = 0; i < bars.length; i++) {
                if (bars[i].dataset.filterBarInit) continue;
                bars[i].dataset.filterBarInit = '1';
                attachFilterBarBehavior(bars[i]);
            }
        } catch (_) {}
        document.addEventListener('fjui:navigated', function() {
            try {
                var bars = document.querySelectorAll('[data-filter-bar]');
                for (var i = 0; i < bars.length; i++) {
                    if (bars[i].dataset.filterBarInit) continue;
                    bars[i].dataset.filterBarInit = '1';
                    attachFilterBarBehavior(bars[i]);
                }
                // Re-check modificata state after every nav (URL has changed).
                var allBars = document.querySelectorAll('[data-filter-bar]');
                for (var j = 0; j < allBars.length; j++) {
                    checkModificata(allBars[j]);
                }
            } catch (_) {}
        });
    }
"#;
