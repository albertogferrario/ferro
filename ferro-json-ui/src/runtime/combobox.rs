pub(super) const SOURCE: &str = r#"
    // ── Searchable combobox ──────────────────────────────────────────────────
    //
    // Progressive enhancement of native <select> elements marked with
    // data-combobox. The native <select> (data-combobox-native) remains the
    // hidden form value carrier (D-06). The visible widget is a text input +
    // listbox overlay backed by the shared createListboxEngine (D-05).
    //
    // Client-side filter only — NO server fetch on keystroke (D-06 lock).
    // Idempotent via dataset.comboboxInit guard. Re-scans on fjui:navigated.
    //
    // ARIA contract (from render_select combobox branch):
    //   input role="combobox" aria-expanded aria-controls aria-activedescendant
    //         aria-autocomplete="list"
    //   ul    role="listbox"
    //   li    role="option" id="fjui-combo-opt-{field}-{i}" aria-selected
    //
    // Security (T-249-03-01, T-249-03-02):
    //   Native select value is set directly via .value (no innerHTML).
    //   Display text is written via textContent only.

    function setupCombobox() {
        var els = document.querySelectorAll('[data-combobox]');
        for (var i = 0; i < els.length; i++) {
            if (els[i].dataset.comboboxInit) continue;
            els[i].dataset.comboboxInit = '1';
            try { attachComboboxBehavior(els[i]); } catch (_) {}
        }

        document.addEventListener('fjui:navigated', function() {
            var newEls = document.querySelectorAll('[data-combobox]');
            for (var i = 0; i < newEls.length; i++) {
                if (newEls[i].dataset.comboboxInit) continue;
                newEls[i].dataset.comboboxInit = '1';
                try { attachComboboxBehavior(newEls[i]); } catch (_) {}
            }
        });
    }

    function attachComboboxBehavior(wrapper) {
        // The native <select data-combobox-native> is emitted as a sibling of the
        // wrapper (outer .relative div contains both). Look in the parent container.
        var nativeSelect = wrapper.querySelector('[data-combobox-native]')
            || (wrapper.parentElement ? wrapper.parentElement.querySelector('[data-combobox-native]') : null);
        var input = wrapper.querySelector('[data-combobox-input]');
        var listbox = wrapper.querySelector('[role="listbox"]');

        if (!nativeSelect || !input || !listbox) return;

        // Initialize display value from the native select's currently-selected
        // option (reflects pre-filled form values — Form Field Rules).
        var currentOpt = nativeSelect.options[nativeSelect.selectedIndex];
        if (currentOpt && currentOpt.value !== '') {
            input.value = currentOpt.text;
        }

        // Persist all option elements for filter restoration.
        var allOptions = Array.prototype.slice.call(listbox.querySelectorAll('[role="option"]'));

        var engine = createListboxEngine(input, listbox, function(opt) {
            // On selection: sync native <select> value, dispatch change event,
            // set display text, close listbox. T-249-03-02: textContent only.
            var val = opt.getAttribute('data-value');
            nativeSelect.value = val;
            try {
                nativeSelect.dispatchEvent(new Event('change', { bubbles: true }));
            } catch (_) {}
            // T-249-03-02: textContent only, never innerHTML
            input.value = opt.textContent;
            listbox.hidden = true;
            input.setAttribute('aria-expanded', 'false');
            input.removeAttribute('aria-activedescendant');
        });

        // Click on option (mouse support alongside keyboard).
        listbox.addEventListener('click', function(e) {
            var opt = e.target.closest('[role="option"]');
            if (opt && !opt.hidden) {
                var val = opt.getAttribute('data-value');
                nativeSelect.value = val;
                try {
                    nativeSelect.dispatchEvent(new Event('change', { bubbles: true }));
                } catch (_) {}
                input.value = opt.textContent;
                listbox.hidden = true;
                input.setAttribute('aria-expanded', 'false');
                input.removeAttribute('aria-activedescendant');
            }
        });

        // Input event: client-side filter (D-06 — NO server fetch).
        input.addEventListener('input', function() {
            var q = input.value.toLowerCase();
            var hasVisible = false;
            for (var i = 0; i < allOptions.length; i++) {
                var matches = allOptions[i].textContent.toLowerCase().indexOf(q) !== -1;
                allOptions[i].hidden = !matches;
                if (matches) hasVisible = true;
            }

            // Show empty state when no options match.
            var emptyEl = listbox.querySelector('[data-combobox-empty]');
            if (emptyEl) {
                emptyEl.hidden = hasVisible;
            }

            listbox.hidden = false;
            input.setAttribute('aria-expanded', 'true');
            engine.updateOptions();
        });

        // Keydown: delegate to engine; Escape closes.
        input.addEventListener('keydown', function(e) {
            if (e.key === 'Escape') {
                listbox.hidden = true;
                input.setAttribute('aria-expanded', 'false');
                input.removeAttribute('aria-activedescendant');
                return;
            }
            engine.handleKey(e);
        });

        // Open listbox on focus if there is content.
        input.addEventListener('focus', function() {
            if (allOptions.length > 0) {
                // Reset filter to show all options.
                for (var i = 0; i < allOptions.length; i++) {
                    allOptions[i].hidden = false;
                }
                listbox.hidden = false;
                input.setAttribute('aria-expanded', 'true');
                engine.updateOptions();
            }
        });

        // Close on outside click.
        document.addEventListener('click', function(e) {
            if (!wrapper.contains(e.target)) {
                listbox.hidden = true;
                input.setAttribute('aria-expanded', 'false');
            }
        });
    }
"#;
