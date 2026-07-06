pub(super) const SOURCE: &str = r#"
    // ── Checklist dismiss ─────────────────────────────────────────────────

    function setupDismissibles() {
        var dismissibles = document.querySelectorAll('[data-dismissible]');
        for (var i = 0; i < dismissibles.length; i++) {
            initDismissible(dismissibles[i]);
        }

        var checklists = document.querySelectorAll('[data-checklist-key]');
        for (var j = 0; j < checklists.length; j++) {
            initChecklist(checklists[j]);
        }
    }

    function initDismissible(el) {
        var btn = el.querySelector('[data-dismiss-btn]');
        if (btn) {
            btn.addEventListener('click', function() {
                el.style.display = 'none';
            });
        }

        var checkboxes = el.querySelectorAll('input[type="checkbox"]');
        if (checkboxes.length > 0) {
            for (var i = 0; i < checkboxes.length; i++) {
                checkboxes[i].addEventListener('change', function() {
                    checkAllChecked(el, checkboxes);
                });
            }
        }
    }

    function initChecklist(el) {
        var checkboxes = el.querySelectorAll('input[type="checkbox"]');
        for (var i = 0; i < checkboxes.length; i++) {
            checkboxes[i].addEventListener('change', function() {
                checkAllChecked(el, checkboxes);
            });
        }
    }

    function checkAllChecked(el, checkboxes) {
        var allChecked = true;
        for (var i = 0; i < checkboxes.length; i++) {
            if (!checkboxes[i].checked) {
                allChecked = false;
                break;
            }
        }
        if (allChecked && checkboxes.length > 0) {
            el.style.display = 'none';
        }
    }
"#;
