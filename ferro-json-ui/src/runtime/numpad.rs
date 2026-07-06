// Price mode contract: the hidden input[data-numpad-input] carries the raw
// digit string as INTEGER CENTS (e.g. "125" for €1.25). Display formatting
// is presentational only. The server must re-validate the submitted cents
// value. Integer arithmetic only — never float money (see PITFALLS.md).

pub(super) const SOURCE: &str = r#"
    // ── Numpad — tap-surface keypad (quantity and price modes) ───────────────
    //
    // Price mode contract: the hidden input[data-numpad-input] carries the raw
    // digit string as INTEGER CENTS (e.g. "125" for €1.25). The display shows
    // two-decimal formatted output for readability. Integer arithmetic only —
    // never float money. The server re-validates the submitted cents value.
    //
    // Attribute contract:
    //   [data-numpad]                     — container (one listener per container)
    //   [data-numpad-target="<field>"]    — names the hidden field to update
    //   [data-numpad-mode="price"|"quantity"] — optional; default is "quantity"
    //   [data-numpad-display]             — display element inside the container
    //   [data-numpad-key="0".."9"|"backspace"|"clear"] — key buttons
    //   input[data-numpad-input="<field>"] — hidden field written on each tap

    function setupNumpad() {
        var pads = document.querySelectorAll('[data-numpad]');
        if (pads.length === 0) return;
        for (var i = 0; i < pads.length; i++) {
            initNumpad(pads[i]);
        }
    }

    function initNumpad(container) {
        var field = container.getAttribute('data-numpad-target');
        // getAttribute returns the decoded raw string: strip characters that
        // would break the attribute selector below (querySelector throws a
        // SyntaxError on `"`, `\` or `]` inside the quoted value).
        if (field) field = field.replace(/["\\\]]/g, '');
        var mode = container.getAttribute('data-numpad-mode') || 'quantity';
        var display = container.querySelector('[data-numpad-display]');
        var input = document.querySelector('input[data-numpad-input="' + field + '"]');
        if (!display || !input) return;

        container.addEventListener('click', function(e) {
            var keyEl = e.target.closest('[data-numpad-key]');
            if (!keyEl) return;
            var key = keyEl.getAttribute('data-numpad-key');
            var raw = input.value || '0';
            var next;

            if (mode === 'price') {
                next = numpadPriceKey(raw, key);
                display.textContent = numpadPriceDisplay(next);
            } else {
                next = numpadQtyKey(raw, key);
                display.textContent = next;
            }

            input.value = next;
            input.dispatchEvent(new Event('input', { bubbles: true }));
        });
    }

    // Quantity mode: integer digit entry with leading-zero collapse; max 9 digits.
    function numpadQtyKey(current, key) {
        var MAX_LEN = 9;
        if (key === 'clear') return '0';
        if (key === 'backspace') {
            var trimmed = current.slice(0, current.length - 1);
            return trimmed === '' ? '0' : trimmed;
        }
        if (current === '0') return key;
        if (current.length >= MAX_LEN) return current;
        return current + key;
    }

    // Price mode: cents-shift entry. Digits shift in from the right.
    // Raw string is integer cents; max 9 digits. No decimal-point key.
    function numpadPriceKey(current, key) {
        var MAX_LEN = 9;
        if (key === 'clear') return '0';
        if (key === 'backspace') {
            var trimmed = current.slice(0, current.length - 1);
            return trimmed === '' ? '0' : trimmed;
        }
        if (current === '0') return key;
        if (current.length >= MAX_LEN) return current;
        return current + key;
    }

    // Format integer cents as a two-decimal display string (e.g. "125" -> "1.25").
    // Presentational only — the hidden field always carries the raw cents string.
    function numpadPriceDisplay(centsStr) {
        var cents = parseInt(centsStr, 10) || 0;
        var whole = Math.floor(cents / 100);
        var frac = cents % 100;
        var fracStr = frac < 10 ? '0' + frac : '' + frac;
        return whole + '.' + fracStr;
    }
"#;
