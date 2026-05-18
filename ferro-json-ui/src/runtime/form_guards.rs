pub(super) const SOURCE: &str = r#"
    // ── Form guards ───────────────────────────────────────────────────────

    function setupFormGuards() {
        var forms = document.querySelectorAll('[data-form-guard]');
        for (var i = 0; i < forms.length; i++) {
            initFormGuard(forms[i]);
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
        // Merge both NodeLists
        var inputs = [];
        for (var n = 0; n < numberInputs.length; n++) inputs.push(numberInputs[n]);
        for (var q = 0; q < qtyInputs.length; q++) inputs.push(qtyInputs[q]);
        // Find the submit button — inside the form, or linked via the
        // `form="<id>"` attribute from an external chrome (e.g. PageHeader).
        // Skip ProductTile +/- controls (they have data-qty-* attrs).
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
