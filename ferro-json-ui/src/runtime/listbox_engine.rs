pub(super) const SOURCE: &str = r#"
    // ── Shared listbox keyboard engine (D-05) ────────────────────────────────
    //
    // Single reusable keyboard primitive consumed by command_palette, combobox,
    // and date_picker. Implements the WAI-ARIA combobox/listbox keyboard pattern:
    //   ArrowDown/ArrowUp  — move active option
    //   Home/End           — jump to first/last
    //   Enter              — select active option via onSelect callback
    //   Escape             — clear active, caller handles close
    //
    // ES5 only: var, named function declarations, C-style for loops.
    // No arrow functions, no const/let, no template literals.
    //
    // Usage:
    //   var engine = createListboxEngine(inputEl, listboxEl, function(optionEl) { ... });
    //   inputEl.addEventListener('keydown', engine.handleKey);
    //   // call engine.updateOptions() whenever the visible option set changes
    //   // call engine.setActive(idx) to programmatically focus an option

    function createListboxEngine(input, listbox, onSelect) {
        var activeIdx = -1;
        var options = [];

        function updateOptions() {
            options = Array.prototype.slice.call(
                listbox.querySelectorAll('[role="option"]:not([hidden]):not([aria-disabled="true"])')
            );
        }

        function setActive(idx) {
            if (activeIdx >= 0 && options[activeIdx]) {
                options[activeIdx].setAttribute('aria-selected', 'false');
            }
            activeIdx = idx;
            if (idx >= 0 && options[idx]) {
                var id = options[idx].id;
                input.setAttribute('aria-activedescendant', id);
                options[idx].setAttribute('aria-selected', 'true');
                try { options[idx].scrollIntoView({ block: 'nearest' }); } catch (_) {}
            } else {
                input.removeAttribute('aria-activedescendant');
            }
        }

        function handleKey(event) {
            updateOptions();
            if (event.key === 'ArrowDown') {
                event.preventDefault();
                setActive(Math.min(activeIdx + 1, options.length - 1));
            } else if (event.key === 'ArrowUp') {
                event.preventDefault();
                setActive(Math.max(activeIdx - 1, 0));
            } else if (event.key === 'Home') {
                event.preventDefault();
                setActive(0);
            } else if (event.key === 'End') {
                event.preventDefault();
                setActive(options.length - 1);
            } else if (event.key === 'Enter') {
                event.preventDefault();
                if (activeIdx >= 0 && options[activeIdx]) {
                    onSelect(options[activeIdx]);
                }
            } else if (event.key === 'Escape') {
                activeIdx = -1;
                input.removeAttribute('aria-activedescendant');
                // caller handles close
            }
        }

        return { handleKey: handleKey, updateOptions: updateOptions, setActive: setActive };
    }
"#;
