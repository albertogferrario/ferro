pub(super) const SOURCE: &str = r#"
    // ── Sidebar mobile toggle ─────────────────────────────────────────────

    function setupSidebar() {
        var toggleBtn = document.querySelector('[data-sidebar-toggle]');
        var sidebarEl = document.querySelector('[data-sidebar]');
        var backdropEl = document.querySelector('[data-sidebar-backdrop]');
        if (!toggleBtn || !sidebarEl) return;

        // Idempotency guard: toggleBtn lives in the persistent frame (outside
        // #ferro-json-ui) and is never re-created on navigation. ferroRuntime()
        // is called once per navigation, so without this guard each navigation
        // adds another click listener — two listeners make the sidebar open and
        // immediately close on the same click, so the toggle appears broken.
        if (toggleBtn.__fjuiSidebarBound) return;
        toggleBtn.__fjuiSidebarBound = true;

        function openSidebar() {
            sidebarEl.classList.remove('hidden');
            if (backdropEl) backdropEl.classList.remove('hidden');
            // aria-expanded drives the hamburger→X morph (fjui-hamburger skin rule).
            toggleBtn.setAttribute('aria-expanded', 'true');
            toggleBtn.setAttribute('aria-label', 'Chiudi menu');
        }

        function closeSidebar() {
            sidebarEl.classList.add('hidden');
            if (backdropEl) backdropEl.classList.add('hidden');
            toggleBtn.setAttribute('aria-expanded', 'false');
            toggleBtn.setAttribute('aria-label', 'Apri menu');
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
