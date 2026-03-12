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
            if (window.location.pathname.indexOf('/cassa/ordini') !== -1) {
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
        info: 'bg-blue-500',
        success: 'bg-green-500',
        warning: 'bg-yellow-500',
        error: 'bg-red-500'
    };

    function showToast(toast) {
        var container = document.querySelector('[data-toast-container]');
        if (!container) return;

        var message = toast.message || '';
        var variant = toast.variant || 'info';
        var timeout = (toast.timeout !== undefined ? toast.timeout : 5) * 1000;
        var colorClass = VARIANT_CLASSES[variant] || VARIANT_CLASSES.info;

        var el = document.createElement('div');
        el.className = 'flex items-start gap-3 px-4 py-3 rounded-lg shadow-lg text-white max-w-sm ' +
            colorClass + ' opacity-0 transition-opacity duration-300';
        el.innerHTML =
            '<span class="flex-1 text-sm">' + escapeHtml(message) + '</span>' +
            '<button class="text-white opacity-70 hover:opacity-100 text-lg leading-none" ' +
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
                    t.classList.remove('border-transparent', 'text-gray-500', 'hover:text-gray-700');
                    t.classList.add('border-blue-600', 'text-blue-600');
                    t.setAttribute('aria-selected', 'true');
                } else {
                    t.classList.remove('border-blue-600', 'text-blue-600');
                    t.classList.add('border-transparent', 'text-gray-500', 'hover:text-gray-700');
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
        if (!toggleBtn || !sidebarEl) return;

        toggleBtn.addEventListener('click', function() {
            var hidden = sidebarEl.classList.contains('hidden');
            if (hidden) {
                sidebarEl.classList.remove('hidden');
            } else {
                sidebarEl.classList.add('hidden');
            }
        });
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
    }

    document.addEventListener('DOMContentLoaded', init);
})();
"#;
