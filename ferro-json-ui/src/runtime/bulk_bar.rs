pub(super) const SOURCE: &str = r#"
    // ── Bulk action bar ────────────────────────────────────────────────────
    //
    // Floating action bar at viewport bottom, visible when ≥1 DataTable rows
    // are checked via bulk-select checkboxes with data-row-key. Tracks
    // selection state; destructive actions require native <dialog> confirm.
    // Idempotent via data-bulk-bar-init guard. Degrades gracefully when
    // #fjui-bulk-bar or #fjui-bulk-confirm are absent from the layout.

    function updateBulkBar(selected, bar, countEl) {
        try {
            var count = Object.keys(selected).length;
            if (!bar) return;
            bar.style.display = count > 0 ? 'flex' : 'none';
            bar.setAttribute('aria-label', 'Azioni su ' + count + ' elementi selezionati');
            if (countEl) countEl.textContent = count + ' selezionati';
        } catch (_) {}
    }

    function executeBulkAction(keys, action, endpoint, selected, bar, countEl) {
        try {
            var body = 'action=' + encodeURIComponent(action);
            for (var i = 0; i < keys.length; i++) {
                body += '&keys[]=' + encodeURIComponent(keys[i]);
            }
            fetch(endpoint, {
                method: 'POST',
                credentials: 'same-origin',
                headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                body: body
            }).then(function(r) {
                if (r.ok) {
                    // Clear selection and hide bar on success.
                    var keysArr = Object.keys(selected);
                    for (var k = 0; k < keysArr.length; k++) {
                        delete selected[keysArr[k]];
                    }
                    // Uncheck all checkboxes in bulk-select tables.
                    var cbs = document.querySelectorAll('[data-bulk-select-table] input[type="checkbox"]');
                    for (var j = 0; j < cbs.length; j++) {
                        cbs[j].checked = false;
                    }
                    updateBulkBar(selected, bar, countEl);
                }
            }).catch(function() {});
        } catch (_) {}
    }

    function confirmBulkAction(keys, action, endpoint, selected, bar, countEl) {
        var dlg = document.getElementById('fjui-bulk-confirm');
        if (!dlg || typeof dlg.showModal !== 'function') {
            // Degrade: execute without confirm if dialog absent.
            executeBulkAction(keys, action, endpoint, selected, bar, countEl);
            return;
        }
        dlg.addEventListener('close', function handler() {
            dlg.removeEventListener('close', handler);
            if (dlg.returnValue === 'confirm') {
                executeBulkAction(keys, action, endpoint, selected, bar, countEl);
            }
        });
        try { dlg.showModal(); } catch (_) {
            executeBulkAction(keys, action, endpoint, selected, bar, countEl);
        }
    }

    function attachBulkBarBehavior(table) {
        var bar = document.getElementById('fjui-bulk-bar');
        var countEl = bar ? bar.querySelector('[data-bulk-count]') : null;
        var selected = {};

        // Delegated change listener on the table container covers checkboxes
        // injected after a nav swap.
        table.addEventListener('change', function(e) {
            try {
                var cb = e.target;
                if (!cb || cb.type !== 'checkbox') return;
                var key = cb.getAttribute('data-row-key');
                if (!key) return;
                if (cb.checked) {
                    selected[key] = true;
                } else {
                    delete selected[key];
                }
                updateBulkBar(selected, bar, countEl);
            } catch (_) {}
        });

        // Select-all header checkbox.
        table.addEventListener('change', function(e) {
            try {
                var cb = e.target;
                if (!cb || cb.type !== 'checkbox') return;
                var isHeaderCb = cb.closest('thead') !== null;
                if (!isHeaderCb) return;
                var cbs = table.querySelectorAll('tbody input[type="checkbox"]');
                for (var i = 0; i < cbs.length; i++) {
                    cbs[i].checked = cb.checked;
                    var key = cbs[i].getAttribute('data-row-key');
                    if (!key) continue;
                    if (cb.checked) {
                        selected[key] = true;
                    } else {
                        delete selected[key];
                    }
                }
                updateBulkBar(selected, bar, countEl);
            } catch (_) {}
        });

        // Action buttons in the bar.
        if (bar) {
            bar.addEventListener('click', function(e) {
                try {
                    var btn = e.target.closest('[data-bulk-action]');
                    if (!btn) return;
                    var action = btn.getAttribute('data-bulk-action');
                    var endpoint = btn.getAttribute('data-bulk-endpoint') || '';
                    var isDestructive = btn.hasAttribute('data-bulk-destructive');
                    var keys = Object.keys(selected);
                    if (keys.length === 0) return;
                    if (isDestructive) {
                        confirmBulkAction(keys, action, endpoint, selected, bar, countEl);
                    } else {
                        executeBulkAction(keys, action, endpoint, selected, bar, countEl);
                    }
                } catch (_) {}
            });

            // Dismiss button clears selection.
            var dismissBtn = bar.querySelector('[data-bulk-dismiss]');
            if (dismissBtn) {
                dismissBtn.addEventListener('click', function() {
                    try {
                        var keysArr = Object.keys(selected);
                        for (var k = 0; k < keysArr.length; k++) {
                            delete selected[keysArr[k]];
                        }
                        var cbs = document.querySelectorAll('[data-bulk-select-table] input[type="checkbox"]');
                        for (var j = 0; j < cbs.length; j++) {
                            cbs[j].checked = false;
                        }
                        updateBulkBar(selected, bar, countEl);
                    } catch (_) {}
                });
            }
        }
    }

    function setupBulkBar() {
        try {
            var tables = document.querySelectorAll('[data-bulk-select-table]');
            for (var i = 0; i < tables.length; i++) {
                if (tables[i].dataset.bulkBarInit) continue;
                tables[i].dataset.bulkBarInit = '1';
                attachBulkBarBehavior(tables[i]);
            }
        } catch (_) {}
        document.addEventListener('fjui:navigated', function() {
            try {
                var tables = document.querySelectorAll('[data-bulk-select-table]');
                for (var i = 0; i < tables.length; i++) {
                    if (tables[i].dataset.bulkBarInit) continue;
                    tables[i].dataset.bulkBarInit = '1';
                    attachBulkBarBehavior(tables[i]);
                }
            } catch (_) {}
        });
    }
"#;
