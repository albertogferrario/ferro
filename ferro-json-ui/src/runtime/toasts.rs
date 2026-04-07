pub(super) const SOURCE: &str = r#"
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

    // ── Toast from URL query param ────────────────────────────────────────

    function initToastFromUrl() {
        var params = new URLSearchParams(window.location.search);
        var msg = params.get('toast');
        if (!msg) return;
        showToast({ message: msg, variant: 'success' });
        params.delete('toast');
        var newUrl = window.location.pathname +
            (params.toString() ? '?' + params.toString() : '') +
            window.location.hash;
        history.replaceState(null, '', newUrl);
    }

    function setupToasts() {
        initToastFromUrl();
    }
"#;
