pub(super) const SOURCE: &str = r#"
    // ── Tab switching ──────────────────────────────────────────────────────

    function setupTabs() {
        var containers = document.querySelectorAll('[data-tabs]');
        for (var i = 0; i < containers.length; i++) {
            initTabContainer(containers[i]);
        }
    }

    function initTabContainer(container) {
        var triggers = container.querySelectorAll('[data-tab]');
        var panels = container.querySelectorAll('[data-tab-panel]');
        if (triggers.length === 0) return;

        for (var i = 0; i < triggers.length; i++) {
            triggers[i].addEventListener('click', makeTabHandler(triggers, panels));
        }

        initTabFromUrl(container, triggers, panels);
    }

    // Reads ?tab=<name> from the URL once at boot. If present and the value
    // matches a [data-tab] trigger, activate that tab by invoking the same
    // handler the click path uses — no DOM logic duplicated. Mirrors the
    // initToastFromUrl pattern in toasts.rs (URLSearchParams + DOMContentLoaded;
    // the outer ferroRuntime() IIFE already gates this on DOMContentLoaded so
    // no extra guard is needed).
    function initTabFromUrl(container, triggers, panels) {
        var params = new URLSearchParams(window.location.search);
        var value = params.get('tab');
        if (!value) return;

        var found = false;
        for (var i = 0; i < triggers.length; i++) {
            if (triggers[i].getAttribute('data-tab') === value) {
                found = true;
                break;
            }
        }
        if (!found) return;

        makeTabHandler(triggers, panels)({
            currentTarget: {
                getAttribute: function(attr) {
                    return attr === 'data-tab' ? value : null;
                }
            }
        });
    }

    function makeTabHandler(triggers, panels) {
        return function(e) {
            var value = e.currentTarget.getAttribute('data-tab');

            for (var i = 0; i < triggers.length; i++) {
                var t = triggers[i];
                if (t.getAttribute('data-tab') === value) {
                    t.classList.remove('border-transparent', 'text-text-muted', 'hover:text-text');
                    t.classList.add('border-primary', 'text-primary', 'font-semibold');
                    t.setAttribute('aria-selected', 'true');
                } else {
                    t.classList.remove('border-primary', 'text-primary', 'font-semibold');
                    t.classList.add('border-transparent', 'text-text-muted', 'hover:text-text');
                    t.setAttribute('aria-selected', 'false');
                }
            }

            for (var j = 0; j < panels.length; j++) {
                var p = panels[j];
                if (p.getAttribute('data-tab-panel') === value) {
                    p.classList.remove('hidden');
                } else {
                    p.classList.add('hidden');
                }
            }
        };
    }
"#;
