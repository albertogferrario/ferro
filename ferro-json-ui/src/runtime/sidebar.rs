pub(super) const SOURCE: &str = r#"
    // ── Sidebar mobile toggle ─────────────────────────────────────────────

    function setupSidebar() {
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
"#;
