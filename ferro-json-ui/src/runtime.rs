//! Built-in JavaScript runtime for ferro-json-ui.
//!
//! Provides SSE connection management, live-value replacement, toast
//! display/stacking/auto-dismiss, checklist dismiss, notification dropdown
//! toggle, and sidebar mobile toggle — all via declarative data attributes.
//!
//! Injected once per page as a `<script>` tag by layouts that support live
//! behaviors (e.g., `DashboardLayout`).

/// Vanilla JS runtime (~5-10KB) for ferro-json-ui live behaviors.
///
/// Uses an IIFE and `var` declarations for maximum browser compatibility.
/// Auto-initializes via `DOMContentLoaded`. All behavior is driven by
/// data attributes — no JavaScript configuration objects needed.
///
/// # Data attributes
///
/// - `data-sse-url` on `<body>` — SSE endpoint URL. If present, opens an
///   `EventSource` connection and dispatches incoming messages to live-value
///   and toast handlers.
/// - `data-sse-target="key"` — Element whose `textContent` is updated when
///   an SSE message `{key: "key", value: "..."}` arrives.
/// - `data-toast-container` — Container where toast elements are appended.
/// - `data-dismissible` — Element that can be dismissed via its close button
///   (`data-dismiss-btn`). Hidden when all checkboxes inside are checked.
/// - `data-checklist-key` — Element auto-hidden when all its checkboxes are
///   checked (same as `data-dismissible` checklist behavior).
/// - `data-notification-toggle` — Button that toggles `data-notification-dropdown`.
/// - `data-notification-dropdown` — Dropdown panel shown/hidden by toggle.
/// - `data-tabs` — Tab container. Scopes trigger/panel discovery.
/// - `data-tab="value"` — Tab trigger button. Clicked to switch panels.
/// - `data-tab-panel="value"` — Tab content panel. Shown/hidden by matching trigger.
/// - `data-sidebar-toggle` — Hamburger button that toggles `data-sidebar` on mobile.
/// - `data-sidebar` — Sidebar element toggled for mobile display.
/// - `data-sidebar-backdrop` — Dark overlay shown behind sidebar on mobile; click to close.
/// - `data-form-guard="number-gt-0"` — Form with number input guard. Submit disabled until
///   at least one number input > 0.
pub(crate) const FERRO_RUNTIME_JS: &str = r#"(function() {
    'use strict';

    // ── SSE connection ────────────────────────────────────────────────────

    function connectSSE(url) {
        var es = new EventSource(url);
        es.onmessage = function(event) {
            try {
                var data = JSON.parse(event.data);
                handleSSEMessage(data);
            } catch (e) {
                // Ignore unparseable messages
            }
        };
        es.onerror = function() {
            // EventSource will attempt to reconnect automatically
        };
    }

    function handleSSEMessage(data) {
        if (data && data.key !== undefined && data.value !== undefined) {
            updateLiveValues(data.key, data.value);
        }
        if (data && data.toast) {
            showToast(data.toast);
        }
        if (data && data.reload_kanban) {
            if (window.location.pathname.indexOf('/cassa/ordini') !== -1 ||
                window.location.pathname.indexOf('/prenotazioni') !== -1) {
                window.location.reload();
            }
        }
    }

    // ── Live-value replacement ────────────────────────────────────────────

    function updateLiveValues(key, value) {
        var targets = document.querySelectorAll('[data-sse-target="' + key + '"]');
        for (var i = 0; i < targets.length; i++) {
            targets[i].textContent = value;
        }
    }

    // ── Toast display/stacking/auto-dismiss ───────────────────────────────

    var VARIANT_CLASSES = {
        info: 'bg-primary text-primary-foreground',
        success: 'bg-success text-primary-foreground',
        warning: 'bg-warning text-primary-foreground',
        error: 'bg-destructive text-primary-foreground'
    };

    function showToast(toast) {
        var container = document.querySelector('[data-toast-container]');
        if (!container) return;

        var message = toast.message || '';
        var variant = toast.variant || 'info';
        var timeout = (toast.timeout !== undefined ? toast.timeout : 5) * 1000;
        var colorClass = VARIANT_CLASSES[variant] || VARIANT_CLASSES.info;

        var el = document.createElement('div');
        el.className = 'flex items-start gap-3 px-4 py-3 rounded-lg shadow-lg max-w-sm ' +
            colorClass + ' opacity-0 transition-opacity duration-300';
        el.innerHTML =
            '<span class="flex-1 text-sm">' + escapeHtml(message) + '</span>' +
            '<button class="text-current opacity-70 hover:opacity-100 text-lg leading-none" ' +
            'data-toast-close>&times;</button>';

        var closeBtn = el.querySelector('[data-toast-close]');
        if (closeBtn) {
            closeBtn.addEventListener('click', function() {
                dismissToast(el);
            });
        }

        container.appendChild(el);

        // Fade in
        requestAnimationFrame(function() {
            requestAnimationFrame(function() {
                el.style.opacity = '1';
            });
        });

        if (timeout > 0) {
            setTimeout(function() {
                dismissToast(el);
            }, timeout);
        }
    }

    function dismissToast(el) {
        el.style.opacity = '0';
        setTimeout(function() {
            if (el.parentNode) {
                el.parentNode.removeChild(el);
            }
        }, 300);
    }

    function escapeHtml(str) {
        var div = document.createElement('div');
        div.appendChild(document.createTextNode(str));
        return div.innerHTML;
    }

    // ── Checklist dismiss ─────────────────────────────────────────────────

    function initDismissibles() {
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

    // ── Notification dropdown toggle ──────────────────────────────────────

    function initNotificationToggle() {
        var toggleBtn = document.querySelector('[data-notification-toggle]');
        var dropdown = document.querySelector('[data-notification-dropdown]');
        if (!toggleBtn || !dropdown) return;

        toggleBtn.addEventListener('click', function(e) {
            e.stopPropagation();
            var hidden = dropdown.classList.contains('hidden');
            if (hidden) {
                dropdown.classList.remove('hidden');
            } else {
                dropdown.classList.add('hidden');
            }
        });

        // Close when clicking outside
        document.addEventListener('click', function(e) {
            if (!dropdown.contains(e.target) && e.target !== toggleBtn) {
                dropdown.classList.add('hidden');
            }
        });
    }

    // ── Tab switching ──────────────────────────────────────────────────────

    function initTabs() {
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
    }

    function makeTabHandler(triggers, panels) {
        return function(e) {
            var value = e.currentTarget.getAttribute('data-tab');

            for (var i = 0; i < triggers.length; i++) {
                var t = triggers[i];
                if (t.getAttribute('data-tab') === value) {
                    t.classList.remove('border-transparent', 'text-text-muted', 'hover:text-text');
                    t.classList.add('border-primary', 'text-primary');
                    t.setAttribute('aria-selected', 'true');
                } else {
                    t.classList.remove('border-primary', 'text-primary');
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

    // ── Sidebar mobile toggle ─────────────────────────────────────────────

    function initSidebarToggle() {
        var toggleBtn = document.querySelector('[data-sidebar-toggle]');
        var sidebarEl = document.querySelector('[data-sidebar]');
        var backdropEl = document.querySelector('[data-sidebar-backdrop]');
        if (!toggleBtn || !sidebarEl) return;

        function openSidebar() {
            sidebarEl.classList.remove('hidden');
            if (backdropEl) backdropEl.classList.remove('hidden');
        }

        function closeSidebar() {
            sidebarEl.classList.add('hidden');
            if (backdropEl) backdropEl.classList.add('hidden');
        }

        toggleBtn.addEventListener('click', function() {
            var isHidden = sidebarEl.classList.contains('hidden');
            if (isHidden) { openSidebar(); } else { closeSidebar(); }
        });

        if (backdropEl) {
            backdropEl.addEventListener('click', closeSidebar);
        }

        // Auto-close sidebar when a nav link is clicked (mobile UX)
        var links = sidebarEl.querySelectorAll('a[href]');
        for (var i = 0; i < links.length; i++) {
            links[i].addEventListener('click', function() {
                setTimeout(closeSidebar, 50);
            });
        }
    }

    // ── Form guards ───────────────────────────────────────────────────────

    function initFormGuards() {
        var forms = document.querySelectorAll('[data-form-guard]');
        for (var i = 0; i < forms.length; i++) {
            initFormGuard(forms[i]);
        }
    }

    function initFormGuard(form) {
        var guardType = form.getAttribute('data-form-guard');
        if (guardType === 'number-gt-0') {
            initNumberGuard(form);
        }
    }

    function initNumberGuard(form) {
        var inputs = form.querySelectorAll('input[type="number"]');
        var submitBtn = form.querySelector('button');
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

    // ── Init ──────────────────────────────────────────────────────────────

    function init() {
        var sseUrl = document.body && document.body.getAttribute('data-sse-url');
        if (sseUrl) {
            connectSSE(sseUrl);
        }
        initTabs();
        initDismissibles();
        initNotificationToggle();
        initSidebarToggle();
        initFormGuards();
    }

    document.addEventListener('DOMContentLoaded', init);
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_classes_use_semantic_tokens() {
        assert!(
            FERRO_RUNTIME_JS.contains("bg-primary"),
            "info variant should use bg-primary"
        );
        assert!(
            FERRO_RUNTIME_JS.contains("bg-success"),
            "success variant should use bg-success"
        );
        assert!(
            FERRO_RUNTIME_JS.contains("bg-warning"),
            "warning variant should use bg-warning"
        );
        assert!(
            FERRO_RUNTIME_JS.contains("bg-destructive"),
            "error variant should use bg-destructive"
        );
        assert!(
            !FERRO_RUNTIME_JS.contains("bg-blue-500"),
            "should not contain hardcoded bg-blue-500"
        );
        assert!(
            !FERRO_RUNTIME_JS.contains("bg-green-500"),
            "should not contain hardcoded bg-green-500"
        );
        assert!(
            !FERRO_RUNTIME_JS.contains("bg-yellow-500"),
            "should not contain hardcoded bg-yellow-500"
        );
        assert!(
            !FERRO_RUNTIME_JS.contains("bg-red-500"),
            "should not contain hardcoded bg-red-500"
        );
    }

    #[test]
    fn tab_switcher_uses_semantic_tokens() {
        assert!(
            FERRO_RUNTIME_JS.contains("border-primary"),
            "active tab should use border-primary"
        );
        assert!(
            FERRO_RUNTIME_JS.contains("text-primary"),
            "active tab should use text-primary"
        );
        assert!(
            FERRO_RUNTIME_JS.contains("text-text-muted"),
            "inactive tab should use text-text-muted"
        );
        assert!(
            !FERRO_RUNTIME_JS.contains("border-blue-600"),
            "should not contain hardcoded border-blue-600"
        );
        assert!(
            !FERRO_RUNTIME_JS.contains("text-blue-600"),
            "should not contain hardcoded text-blue-600"
        );
        assert!(
            !FERRO_RUNTIME_JS.contains("text-gray-500"),
            "should not contain hardcoded text-gray-500"
        );
    }

    #[test]
    fn toast_uses_semantic_text_color() {
        assert!(
            FERRO_RUNTIME_JS.contains("text-primary-foreground"),
            "toast should use text-primary-foreground"
        );
        assert!(
            !FERRO_RUNTIME_JS.contains("text-white"),
            "toast should not use hardcoded text-white"
        );
    }
}
