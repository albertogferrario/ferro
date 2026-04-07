pub(super) const SOURCE: &str = r#"
    // ── Product tile quantity controls ───────────────────────────────────

    function setupProductTiles() {
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
            var display = document.querySelector('[data-qty-display="' + field + '"]');
            var input = document.querySelector('[data-qty-input="' + field + '"]');
            if (!display || !input) return;
            var current = parseInt(input.value, 10) || 0;
            var next = current + delta;
            if (next < 0) next = 0;
            input.value = next;
            display.textContent = next;
            // Notify form guards of the change
            input.dispatchEvent(new Event('input', { bubbles: true }));
        });
    }
"#;
