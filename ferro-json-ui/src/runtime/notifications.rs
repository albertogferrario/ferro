pub(super) const SOURCE: &str = r#"
    // ── Notification dropdown toggle ──────────────────────────────────────

    function setupNotifications() {
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
"#;
