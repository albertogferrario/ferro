pub(super) const SOURCE: &str = r#"
    // ── Dropdown menus ──────────────────────────────────────────────────

    function setupDropdowns() {
        var toggles = document.querySelectorAll('[data-dropdown-toggle]');
        for (var i = 0; i < toggles.length; i++) {
            initDropdownToggle(toggles[i]);
        }
        // Global click-outside: close all open dropdowns (both regular and kanban fixed)
        document.addEventListener('click', function(e) {
            var openMenus = document.querySelectorAll('[data-dropdown]:not(.hidden)');
            for (var j = 0; j < openMenus.length; j++) {
                var menu = openMenus[j];
                if (menu._kanbanFixed) continue; // kanban menus handled by their own click handler
                var toggleId = menu.getAttribute('data-dropdown');
                var toggle = document.querySelector('[data-dropdown-toggle="' + toggleId + '"]');
                if (!menu.contains(e.target) && e.target !== toggle) {
                    menu.classList.add('hidden');
                }
            }
            // Close kanban fixed menus when clicking outside any kanban card
            var kanbanCard = e.target.closest('[data-kanban-card]');
            if (!kanbanCard) {
                var kanbanPanels = document.querySelectorAll('[data-kanban-card] [data-dropdown]');
                for (var k = 0; k < kanbanPanels.length; k++) {
                    kanbanPanels[k].style.display = 'none';
                }
            }
        });
        // Escape key closes all (including kanban fixed)
        document.addEventListener('keydown', function(e) {
            if (e.key === 'Escape') {
                var openMenus = document.querySelectorAll('[data-dropdown]:not(.hidden)');
                for (var k = 0; k < openMenus.length; k++) {
                    openMenus[k].classList.add('hidden');
                }
                var kanbanPanels = document.querySelectorAll('[data-kanban-card] [data-dropdown]');
                for (var m = 0; m < kanbanPanels.length; m++) {
                    kanbanPanels[m].style.display = 'none';
                }
            }
        });
    }

    function initDropdownToggle(btn) {
        var targetId = btn.getAttribute('data-dropdown-toggle');
        var panel = document.querySelector('[data-dropdown="' + targetId + '"]');
        if (!panel) return;
        btn.addEventListener('click', function(e) {
            e.stopPropagation();
            // Close all other dropdowns first
            var allMenus = document.querySelectorAll('[data-dropdown]:not(.hidden)');
            for (var m = 0; m < allMenus.length; m++) {
                if (allMenus[m] !== panel) allMenus[m].classList.add('hidden');
            }
            panel.classList.toggle('hidden');
        });
    }
"#;
