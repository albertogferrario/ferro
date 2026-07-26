pub(super) const SOURCE: &str = r#"
    // ── Notification dropdown toggle ──────────────────────────────────────

    function setupNotifications() {
        var toggleBtn = document.querySelector('[data-notification-toggle]');
        var dropdown = document.querySelector('[data-notification-dropdown]');
        if (!toggleBtn || !dropdown) return;

        // Idempotency guard: notification toggle is in the persistent frame
        // (outside #ferro-json-ui). ferroRuntime() runs on every navigation;
        // without this check each navigation adds another listener pair —
        // the second toggle listener immediately re-hides the dropdown.
        if (toggleBtn.__fjuiNotifBound) return;
        toggleBtn.__fjuiNotifBound = true;

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
