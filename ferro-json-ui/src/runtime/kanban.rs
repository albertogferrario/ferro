pub(super) const SOURCE: &str = r#"
    // ── Kanban card click-to-open-menu (HTML popover API) ──────────────
    //
    // The whole card is the trigger — the kebab inside is hidden. Click
    // anywhere on the card to open the popover; click an item or click
    // outside to close. Browser supplies dismiss + top-layer escape; we
    // only contribute the card-as-trigger gesture, anchor coords, and
    // tracking the card on scroll.

    function setupKanban() {
        injectKanbanScrollbarStyle();
        var cards = document.querySelectorAll('[data-kanban-card]');
        for (var i = 0; i < cards.length; i++) {
            initKanbanCard(cards[i]);
        }
    }

    // Hide scrollbars on kanban column scrollers. Inline `scrollbar-width`
    // covers Firefox; `::-webkit-scrollbar` covers Chrome/Safari/Edge and
    // must live in a stylesheet, so inject one — idempotent via the id check.
    function injectKanbanScrollbarStyle() {
        if (document.getElementById('ferro-kanban-scrollbar-style')) return;
        var s = document.createElement('style');
        s.id = 'ferro-kanban-scrollbar-style';
        s.textContent =
            '.ferro-kanban-scroll::-webkit-scrollbar{display:none;width:0;height:0}';
        document.head.appendChild(s);
    }

    function initKanbanCard(wrapper) {
        var panel = wrapper.querySelector('[data-popover-menu]');
        if (!panel) return;
        panel._kanbanFixed = true; // tells dropdowns.rs to leave this panel alone

        var trigger = wrapper.querySelector('[popovertarget="' + panel.id + '"]');
        if (trigger) trigger.style.display = 'none';

        function track() {
            if (!panel.matches(':popover-open')) return;
            var rect = wrapper.getBoundingClientRect();
            if (rect.bottom < 0 || rect.top > window.innerHeight) {
                panel.hidePopover();
                return;
            }
            positionCenteredOnCard(panel, wrapper);
        }

        wrapper.addEventListener('click', function(e) {
            // Clicks inside the open panel (menu items) pass through.
            if (panel.matches(':popover-open') && panel.contains(e.target)) return;
            e.preventDefault();
            e.stopPropagation();
            if (panel.matches(':popover-open')) {
                panel.hidePopover();
            } else {
                panel.showPopover();
                positionCenteredOnCard(panel, wrapper);
            }
        });

        window.addEventListener('scroll', track, true);
        window.addEventListener('resize', track);
    }

    function positionCenteredOnCard(panel, wrapper) {
        // See dropdowns.rs `positionUnderTrigger` — the UA-supplied
        // `inset: 0; width: fit-content !important` layout doesn't
        // re-evaluate when we clear inset later, so set fit-content
        // inline ourselves to force a fresh content-sized layout.
        panel.style.setProperty('width', 'fit-content', 'important');
        panel.style.setProperty('min-width', '12rem', 'important');
        panel.style.inset = 'auto';
        panel.style.margin = '0';
        panel.style.position = 'fixed';
        void panel.offsetHeight;

        var rect = wrapper.getBoundingClientRect();
        var pw = panel.offsetWidth || 192;
        var ph = panel.offsetHeight || 160;
        // Center horizontally on the card; sit just below its title area.
        var left = rect.left + (rect.width - pw) / 2;
        var top = rect.top + 40;
        if (left < 8) left = 8;
        if (left + pw > window.innerWidth - 8) left = window.innerWidth - pw - 8;
        if (top + ph > window.innerHeight - 8) top = rect.top - ph - 4;
        panel.style.left = left + 'px';
        panel.style.top = top + 'px';
    }
"#;
