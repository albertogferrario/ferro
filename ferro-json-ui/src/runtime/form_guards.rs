pub(super) const SOURCE: &str = r#"
    // ── Form guards ───────────────────────────────────────────────────────

    function setupFormGuards() {
        var forms = document.querySelectorAll('[data-form-guard]');
        for (var i = 0; i < forms.length; i++) {
            initFormGuard(forms[i]);
        }

        // ── Double-submit guard (D-13/D-14/D-15) ─────────────────────────────
        // Binds on the form submit event (not click) so the guard fires once
        // per submission regardless of how the form is submitted.
        // The submitted flag lives ON the button element (btn._submitted) so
        // the pageshow bfcache handler can reset it and the next submit goes
        // through (D-15).
        var disableBtns = document.querySelectorAll('button[data-disable-on-submit]');
        for (var d = 0; d < disableBtns.length; d++) {
            initDisableOnSubmit(disableBtns[d]);
        }
        window.addEventListener('pageshow', function(e) {
            if (!e.persisted) return;
            for (var r = 0; r < disableBtns.length; r++) {
                disableBtns[r]._submitted = false;
                disableBtns[r].removeAttribute('disabled');
                disableBtns[r].classList.remove('opacity-50', 'cursor-not-allowed');
            }
        });

        function initDisableOnSubmit(btn) {
            var form = btn.closest('form');
            if (!form && btn.getAttribute('form')) {
                form = document.getElementById(btn.getAttribute('form'));
            }
            if (!form) return;
            btn._submitted = false;
            form.addEventListener('submit', function(e) {
                if (btn._submitted) { e.preventDefault(); return; }
                btn._submitted = true;
                btn.setAttribute('disabled', 'disabled');
                btn.classList.add('opacity-50', 'cursor-not-allowed');
            });
        }
    }

    // Find this form's submit button. Looks inside the form first; if absent,
    // falls back to a button linked via the HTML5 `form="<id>"` attribute so
    // headers and other external bars can drive the guard.
    function findGuardedSubmit(form) {
        var inside = form.querySelector('button[type="submit"]');
        if (inside) return inside;
        if (form.id) {
            return document.querySelector(
                'button[type="submit"][form="' + form.id + '"]'
            );
        }
        return null;
    }

    function initFormGuard(form) {
        var guardType = form.getAttribute('data-form-guard');
        if (guardType === 'number-gt-0') {
            initNumberGuard(form);
        } else if (guardType && guardType.indexOf('text-equals:') === 0) {
            var expected = guardType.slice('text-equals:'.length);
            initTextEqualsGuard(form, expected);
        }
    }

    function initTextEqualsGuard(form, expected) {
        var input = form.querySelector('input[type="text"]');
        var submitBtn = findGuardedSubmit(form);
        if (!input || !submitBtn) return;

        function check() {
            if (input.value === expected) {
                submitBtn.removeAttribute('disabled');
                submitBtn.classList.remove('opacity-50', 'cursor-not-allowed');
            } else {
                submitBtn.setAttribute('disabled', 'disabled');
                submitBtn.classList.add('opacity-50', 'cursor-not-allowed');
            }
        }

        check();
        input.addEventListener('input', check);
    }

    function initNumberGuard(form) {
        var numberInputs = form.querySelectorAll('input[type="number"]');
        var qtyInputs = form.querySelectorAll('input[data-qty-input]');
        var numpadInputs = form.querySelectorAll('input[data-numpad-input]');
        // Merge all three NodeLists
        var inputs = [];
        for (var n = 0; n < numberInputs.length; n++) inputs.push(numberInputs[n]);
        for (var q = 0; q < qtyInputs.length; q++) inputs.push(qtyInputs[q]);
        for (var p = 0; p < numpadInputs.length; p++) inputs.push(numpadInputs[p]);
        // Find the submit button — inside the form, or linked via the
        // `form="<id>"` attribute from an external chrome (e.g. PageHeader).
        // Skip Tile +/- controls (they have data-qty-* attrs).
        var submitBtn = findGuardedSubmit(form);
        if (!submitBtn) {
            var allBtns = form.querySelectorAll('button:not([data-qty-inc]):not([data-qty-dec])');
            submitBtn = allBtns.length > 0 ? allBtns[allBtns.length - 1] : null;
        }
        if (!submitBtn || inputs.length === 0) return;

        function check() {
            var hasValue = false;
            for (var i = 0; i < inputs.length; i++) {
                if (parseFloat(inputs[i].value) > 0) {
                    hasValue = true;
                    break;
                }
            }
            if (hasValue) {
                submitBtn.removeAttribute('disabled');
                submitBtn.classList.remove('opacity-50', 'cursor-not-allowed');
            } else {
                submitBtn.setAttribute('disabled', 'disabled');
                submitBtn.classList.add('opacity-50', 'cursor-not-allowed');
            }
        }

        check(); // initial state
        for (var j = 0; j < inputs.length; j++) {
            inputs[j].addEventListener('input', check);
        }
    }
"#;
