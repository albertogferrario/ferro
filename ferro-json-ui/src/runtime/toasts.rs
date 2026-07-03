pub(super) const SOURCE: &str = r#"
    // ── Toast display/stacking/auto-dismiss ───────────────────────────────

    // Tone classes are shared verbatim with the SSR render_toast — see the
    // TOAST_TONE_* consts in render/classes.rs. Lockstep is asserted in
    // runtime::tests::toast_tone_classes_match_ssr.
    var VARIANT_CLASSES = {
        neutral: 'bg-primary/70 text-primary-foreground',
        success: 'bg-success/70 text-primary-foreground',
        warning: 'bg-warning/70 text-primary-foreground',
        destructive: 'bg-destructive/70 text-primary-foreground'
    };

    function showToast(toast) {
        var container = document.querySelector('[data-toast-container]');
        if (!container) return;

        var message = toast.message || '';
        var tone = toast.tone || 'neutral';
        var timeout = (toast.timeout !== undefined ? toast.timeout : 5) * 1000;
        var colorClass = VARIANT_CLASSES[tone] || VARIANT_CLASSES.neutral;

        var el = document.createElement('div');
        el.className = 'flex items-start gap-3 px-4 py-3 rounded-lg shadow-lg max-w-sm backdrop-blur-md ' +
            colorClass + ' opacity-0 transition-opacity duration-base ease-base';
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
        // Removal keyed on the fade-out transition (duration-base is themable).
        // Reduced motion collapses durations to 0.01ms — not `none` — so
        // transitionend still fires; the 500ms timer is a fallback bound in
        // case the event is missed (e.g. element hidden mid-transition).
        var removed = false;
        function removeNode() {
            if (removed) return;
            removed = true;
            if (el.parentNode) {
                el.parentNode.removeChild(el);
            }
        }
        el.addEventListener('transitionend', removeNode);
        setTimeout(removeNode, 500);
        el.style.opacity = '0';
    }

    function escapeHtml(str) {
        var div = document.createElement('div');
        div.appendChild(document.createTextNode(str));
        return div.innerHTML;
    }

    // ── Toast from URL query param ────────────────────────────────────────

    function initToastFromUrl() {
        var params = new URLSearchParams(window.location.search);
        var msg = params.get('toast');
        if (!msg) return;
        showToast({ message: msg, tone: 'success' });
        params.delete('toast');
        var newUrl = window.location.pathname +
            (params.toString() ? '?' + params.toString() : '') +
            window.location.hash;
        history.replaceState(null, '', newUrl);
    }

    // Server-rendered toasts: wire the optional [data-toast-close] button
    // (emitted when ToastProps.dismissible is true) and schedule auto-dismiss
    // based on data-toast-timeout (seconds). A timeout <= 0 means persistent
    // until manually dismissed — the renderer clamps timeout to >= 1 when
    // dismissible is false, so every toast has at least one way out.
    function setupServerToasts() {
        var toasts = document.querySelectorAll('[data-toast-tone]:not([data-toast-handled])');
        for (var i = 0; i < toasts.length; i++) {
            var t = toasts[i];
            t.setAttribute('data-toast-handled', '');
            var closeBtn = t.querySelector('[data-toast-close]');
            if (closeBtn) {
                (function(el, btn) {
                    btn.addEventListener('click', function() { dismissToast(el); });
                })(t, closeBtn);
            }
            var timeoutAttr = t.getAttribute('data-toast-timeout');
            var timeout = parseInt(timeoutAttr, 10);
            if (!isFinite(timeout) || timeout <= 0) continue;
            (function(el, ms) {
                setTimeout(function() { dismissToast(el); }, ms);
            })(t, timeout * 1000);
        }
    }

    function setupToasts() {
        initToastFromUrl();
        setupServerToasts();
    }
"#;
