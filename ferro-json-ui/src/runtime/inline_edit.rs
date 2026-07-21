pub(super) const SOURCE: &str = r#"
    // ── Inline edit ────────────────────────────────────────────────────────
    //
    // Double-click or pencil icon activates an in-place input overlay on
    // DescriptionList <dd> elements with data-inline-edit-field. Enter commits
    // via POST; Escape and blur cancel without saving (D-09). One field in
    // edit at a time. Idempotent via dataset.inlineEditInit guard.

    function cancelInlineEdit(dd) {
        try {
            var input = dd.querySelector('.fjui-inline-edit__input');
            var valueEl = dd.querySelector('.fjui-inline-edit__value');
            var errEl = dd.querySelector('.fjui-inline-edit__error');
            if (input && valueEl) {
                input.parentNode.replaceChild(valueEl, input);
            }
            if (errEl) {
                errEl.textContent = '';
                errEl.style.display = 'none';
            }
            dd.removeAttribute('data-inline-edit-active');
            dd.removeAttribute('aria-invalid');
        } catch (_) {}
    }

    function commitInlineEdit(dd, input, endpoint, field) {
        // In-flight guard: mark loading to prevent double-submit (T-249-04-05).
        input.classList.add('fjui-inline-edit__input--loading');
        input.style.cursor = 'wait';
        input.setAttribute('aria-busy', 'true');

        var body = 'field=' + encodeURIComponent(field) + '&value=' + encodeURIComponent(input.value);
        try {
            fetch(endpoint, {
                method: 'POST',
                credentials: 'same-origin',
                headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                body: body
            }).then(function(r) {
                if (!r.ok) {
                    cancelInlineEdit(dd);
                    return null;
                }
                return r.json();
            }).then(function(data) {
                if (!data) return;
                if (data.ok) {
                    // Success: update displayed value span via textContent only (T-249-04-02).
                    var valueEl = dd.querySelector('.fjui-inline-edit__value');
                    if (!valueEl) {
                        valueEl = document.createElement('span');
                        valueEl.className = 'fjui-inline-edit__value';
                    }
                    valueEl.textContent = data.value;
                    cancelInlineEdit(dd);
                    // Restore value span after cancel (cancel removes input, restores span).
                    // cancelInlineEdit already restores valueEl; but if the span was replaced,
                    // re-insert it:
                    if (!dd.querySelector('.fjui-inline-edit__value')) {
                        dd.insertBefore(valueEl, dd.querySelector('.fjui-inline-edit__pencil'));
                    }
                    dd.querySelector('.fjui-inline-edit__value') && (dd.querySelector('.fjui-inline-edit__value').textContent = data.value);
                } else {
                    // Validation error: keep edit mode, show inline error (T-249-04-02).
                    var errEl = dd.querySelector('.fjui-inline-edit__error');
                    if (!errEl) {
                        errEl = document.createElement('span');
                        errEl.className = 'fjui-inline-edit__error';
                        var fieldAttr = dd.getAttribute('data-inline-edit-field') || '';
                        errEl.id = 'fjui-ie-error-' + fieldAttr;
                        errEl.setAttribute('role', 'alert');
                        errEl.setAttribute('aria-live', 'polite');
                        dd.appendChild(errEl);
                    }
                    errEl.textContent = data.error || 'Valore non valido. Riprova.';
                    errEl.style.display = '';
                    input.setAttribute('aria-invalid', 'true');
                    input.classList.remove('fjui-inline-edit__input--loading');
                    input.style.cursor = '';
                    input.removeAttribute('aria-busy');
                }
            }).catch(function() {
                // Network/500 error: show error message and exit edit mode (T-249-04-05).
                try {
                    var errEl = dd.querySelector('.fjui-inline-edit__error');
                    if (!errEl) {
                        errEl = document.createElement('span');
                        errEl.className = 'fjui-inline-edit__error';
                        dd.appendChild(errEl);
                    }
                    errEl.textContent = 'Errore durante il salvataggio. Riprova.';
                    errEl.style.display = '';
                } catch (_) {}
                cancelInlineEdit(dd);
            });
        } catch (_) {
            cancelInlineEdit(dd);
        }
    }

    function activateInlineEdit(dd) {
        // One-at-a-time guard (D-09).
        if (document.querySelector('[data-inline-edit-active]')) return;
        dd.setAttribute('data-inline-edit-active', '1');

        var field = dd.getAttribute('data-inline-edit-field') || '';
        var endpoint = dd.getAttribute('data-inline-edit-endpoint') || '';
        var kind = dd.getAttribute('data-inline-edit-kind') || 'text';

        var valueEl = dd.querySelector('.fjui-inline-edit__value');
        var currentValue = valueEl ? valueEl.textContent : '';

        var input;
        if (kind === 'textarea') {
            input = document.createElement('textarea');
            input.rows = 3;
        } else {
            input = document.createElement('input');
            input.type = kind === 'number' ? 'number' : 'text';
        }
        input.className = 'fjui-inline-edit__input';
        input.value = currentValue;
        input.setAttribute('aria-label', 'Modifica ' + (dd.closest('div') && dd.closest('div').querySelector('dt') ? dd.closest('div').querySelector('dt').textContent : field));
        input.setAttribute('aria-describedby', 'fjui-ie-error-' + field);

        // Replace value span with the input (no layout shift).
        if (valueEl) {
            dd.replaceChild(input, valueEl);
        } else {
            dd.insertBefore(input, dd.firstChild);
        }
        try { input.focus(); input.select(); } catch (_) {}

        // Blur = cancel (never save) — D-09, Italian operator convention.
        input.addEventListener('blur', function() {
            // Slight delay to let click events fire first (e.g. click-outside detection).
            setTimeout(function() {
                if (dd.getAttribute('data-inline-edit-active')) {
                    cancelInlineEdit(dd);
                    // Restore value span.
                    if (valueEl && !dd.querySelector('.fjui-inline-edit__value')) {
                        var pencil = dd.querySelector('.fjui-inline-edit__pencil');
                        if (pencil) dd.insertBefore(valueEl, pencil);
                        else dd.appendChild(valueEl);
                    }
                }
            }, 100);
        });

        // Enter = commit; Escape = cancel.
        input.addEventListener('keydown', function(e) {
            if (e.key === 'Enter' && kind !== 'textarea') {
                e.preventDefault();
                input.removeEventListener('blur', arguments.callee);
                commitInlineEdit(dd, input, endpoint, field);
            }
            if (e.key === 'Escape') {
                e.preventDefault();
                cancelInlineEdit(dd);
                if (valueEl && !dd.querySelector('.fjui-inline-edit__value')) {
                    var pencil = dd.querySelector('.fjui-inline-edit__pencil');
                    if (pencil) dd.insertBefore(valueEl, pencil);
                    else dd.appendChild(valueEl);
                }
                var pencilBtn = dd.querySelector('.fjui-inline-edit__pencil');
                if (pencilBtn) { try { pencilBtn.focus(); } catch (_) {} }
            }
        });
    }

    function setupInlineEdit() {
        var els = document.querySelectorAll('[data-inline-edit-field]');
        for (var i = 0; i < els.length; i++) {
            if (els[i].dataset.inlineEditInit) continue;
            els[i].dataset.inlineEditInit = '1';
            (function(dd) {
                dd.addEventListener('dblclick', function() { activateInlineEdit(dd); });
                var pencil = dd.querySelector('.fjui-inline-edit__pencil');
                if (pencil) {
                    pencil.addEventListener('click', function(e) {
                        e.stopPropagation();
                        activateInlineEdit(dd);
                    });
                    pencil.addEventListener('keydown', function(e) {
                        if (e.key === 'Enter' || e.key === ' ') {
                            e.preventDefault();
                            activateInlineEdit(dd);
                        }
                    });
                }
            })(els[i]);
        }
        // Click-outside: cancel any active inline editor when clicking outside its <dd>.
        if (!document.dataset.inlineEditOutsideInit) {
            document.dataset.inlineEditOutsideInit = '1';
            document.addEventListener('click', function(e) {
                var active = document.querySelector('[data-inline-edit-active]');
                if (!active) return;
                if (!active.contains(e.target)) {
                    cancelInlineEdit(active);
                    // Restore value span if it was detached.
                    var valueEl = active.querySelector('.fjui-inline-edit__value');
                    if (!valueEl) {
                        var fallback = document.createElement('span');
                        fallback.className = 'fjui-inline-edit__value';
                        var pencilBtn = active.querySelector('.fjui-inline-edit__pencil');
                        if (pencilBtn) active.insertBefore(fallback, pencilBtn);
                        else active.appendChild(fallback);
                    }
                }
            });
        }
        document.addEventListener('fjui:navigated', setupInlineEdit);
    }
"#;
