pub(super) const SOURCE: &str = r#"
    // ── Kanban card click-to-open-menu ─────────────────────────────────

    function setupKanban() {
        var cards = document.querySelectorAll('[data-kanban-card]');
        for (var i = 0; i < cards.length; i++) {
            initKanbanCard(cards[i]);
        }
    }

    function initKanbanCard(wrapper) {
        // Hide the kebab trigger button inside kanban cards
        var trigger = wrapper.querySelector('[data-dropdown-toggle]');
        if (trigger) {
            trigger.style.display = 'none';
        }

        // Use fixed positioning so overflow:auto parents don't clip the menu
        var panel = wrapper.querySelector('[data-dropdown]');
        if (panel) {
            panel.style.cssText = 'position:fixed; z-index:9999; min-width:12rem; display:none;';
            panel.classList.remove('hidden');
            panel._kanbanFixed = true;
        }

        function positionPanel() {
            if (!panel) return;
            var rect = wrapper.getBoundingClientRect();
            var pw = panel.offsetWidth || 192;
            var ph = panel.offsetHeight || 160;
            // Center horizontally on card, position below card title area
            var left = rect.left + (rect.width - pw) / 2;
            var top = rect.top + 40;
            // Keep within viewport
            if (left < 8) left = 8;
            if (left + pw > window.innerWidth - 8) left = window.innerWidth - pw - 8;
            if (top + ph > window.innerHeight - 8) top = rect.top - ph - 4;
            panel.style.left = left + 'px';
            panel.style.top = top + 'px';
        }

        wrapper.addEventListener('click', function(e) {
            if (!panel) return;

            // If clicking inside the open panel, let it through
            if (panel.style.display !== 'none' && panel.contains(e.target)) return;

            e.preventDefault();
            e.stopPropagation();

            // Close ALL open kanban menus first
            var allPanels = document.querySelectorAll('[data-kanban-card] [data-dropdown]');
            var wasOpen = panel.style.display !== 'none';
            for (var m = 0; m < allPanels.length; m++) {
                allPanels[m].style.display = 'none';
            }

            if (!wasOpen) {
                positionPanel();
                panel.style.display = '';
            }
        });
    }
"#;
