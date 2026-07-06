pub(super) const SOURCE: &str = r#"
    // ── Tile quantity controls ───────────────────────────────────────────

    function setupTiles() {
        var incBtns = document.querySelectorAll('[data-qty-inc]');
        for (var i = 0; i < incBtns.length; i++) {
            initQtyButton(incBtns[i], 1);
        }
        var decBtns = document.querySelectorAll('[data-qty-dec]');
        for (var j = 0; j < decBtns.length; j++) {
            initQtyButton(decBtns[j], -1);
        }
    }

    function initQtyButton(btn, delta) {
        btn.addEventListener('click', function() {
            var field = btn.getAttribute(delta > 0 ? 'data-qty-inc' : 'data-qty-dec');
            // getAttribute returns the decoded raw string: strip characters
            // that would break the attribute selectors below (querySelector
            // throws a SyntaxError on `"`, `\` or `]` inside the quoted value).
            if (field) field = field.replace(/["\\\]]/g, '');
            var display = document.querySelector('[data-qty-display="' + field + '"]');
            var input = document.querySelector('[data-qty-input="' + field + '"]');
            // display may be null for tap-to-add tiles (D-02: no on-tile qty display).
            // Only input is required; display is updated only when present.
            if (!input) return;
            var current = parseInt(input.value, 10) || 0;
            var next = current + delta;
            if (next < 0) next = 0;
            input.value = next;
            if (display) display.textContent = next;
            // Notify form guards of the change
            input.dispatchEvent(new Event('input', { bubbles: true }));
        });
    }
"#;
