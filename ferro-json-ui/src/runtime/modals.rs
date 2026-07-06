pub(super) const SOURCE: &str = r#"
    // ── Modal dialog wiring ───────────────────────────────────────────────

    function setupModals() {
        var openers = document.querySelectorAll('[data-modal-open]');
        for (var i = 0; i < openers.length; i++) {
            (function(btn) {
                var id = btn.getAttribute('data-modal-open');
                var dialog = document.getElementById(id);
                if (!dialog) return;
                btn.addEventListener('click', function() {
                    dialog.showModal();
                });
                dialog.addEventListener('click', function(e) {
                    var rect = dialog.getBoundingClientRect();
                    var inDialog = (
                        e.clientX >= rect.left && e.clientX <= rect.right &&
                        e.clientY >= rect.top && e.clientY <= rect.bottom
                    );
                    if (!inDialog) { dialog.close(); }
                });
                var closeBtn = dialog.querySelector('[data-modal-close]');
                if (closeBtn) {
                    closeBtn.addEventListener('click', function() {
                        dialog.close();
                    });
                }
            })(openers[i]);
        }
    }
"#;
