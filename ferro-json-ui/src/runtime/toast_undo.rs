pub(super) const SOURCE: &str = r#"
    // ── Undo toast ──────────────────────────────────────────────────────────
    //
    // Reads ?flash_undo_msg + ?flash_undo_url from the current URL on page
    // load and after each nav swap. When both params are present, shows an
    // undo toast with an "Annulla" action button that POSTs the undo URL.
    // Strips the flash params immediately via history.replaceState so
    // back-nav does not re-show the toast (Pitfall 4).
    //
    // Reuses showToast / dismissToast / escapeHtml from toasts.rs (assembled
    // before this module in the bundle).

    function setupUndo() {
        checkUndoFlash();
        document.addEventListener('fjui:navigated', checkUndoFlash);
    }

    function checkUndoFlash() {
        try {
            var params = new URLSearchParams(window.location.search);
            var msg = params.get('flash_undo_msg');
            var undoUrl = params.get('flash_undo_url');
            if (!msg || !undoUrl) return;
            showUndoToast(msg, undoUrl);
            try {
                var clean = window.location.pathname + window.location.hash;
                history.replaceState(history.state, '', clean);
            } catch (_) {}
        } catch (_) {}
    }

    function showUndoToast(msg, undoUrl) {
        try {
            var container = document.querySelector('[data-toast-container]');
            if (!container) return;

            // Enforce max 3 visible toasts — dismiss oldest before inserting a 4th.
            var existing = container.querySelectorAll('.fjui-toast');
            if (existing.length >= 3) {
                try { dismissToast(existing[existing.length - 1]); } catch (_) {}
            }

            var toast = document.createElement('div');
            toast.className = 'fjui-toast fjui-toast--neutral';
            toast.setAttribute('role', 'status');
            toast.setAttribute('aria-live', 'polite');
            toast.setAttribute('aria-atomic', 'true');
            toast.innerHTML =
                '<span class="fjui-toast__msg">' + escapeHtml(msg) + '</span>' +
                '<button type="button" class="fjui-toast__action fjui-toast__undo" ' +
                'aria-label="Annulla l\'ultima azione">Annulla</button>' +
                '<button type="button" class="fjui-toast__close" aria-label="Chiudi notifica">' +
                '<span aria-hidden="true">\xD7</span></button>';
            container.insertBefore(toast, container.firstChild);

            var timer = setTimeout(function() { dismissToast(toast); }, 7000);

            toast.querySelector('.fjui-toast__undo').addEventListener('click', function() {
                clearTimeout(timer);
                dismissToast(toast);
                postUndo(undoUrl);
            });
            toast.querySelector('.fjui-toast__close').addEventListener('click', function() {
                clearTimeout(timer);
                dismissToast(toast);
            });
        } catch (_) {}
    }

    function postUndo(url) {
        try {
            var form = document.createElement('form');
            form.method = 'POST';
            form.action = url;
            document.body.appendChild(form);
            form.submit();
        } catch (_) {}
    }
"#;
