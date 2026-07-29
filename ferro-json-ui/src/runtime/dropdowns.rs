pub(super) const SOURCE: &str = r#"
    // ── Dropdown menus (HTML popover API) ───────────────────────────────
    //
    // Markup:
    //   <button popovertarget="m1" ...>⋮</button>
    //   <div popover id="m1" data-popover-menu ...>…items…</div>
    //
    // The browser handles open/close, click-outside dismiss, Escape, and
    // promotes the panel to the top layer (escapes any overflow:hidden /
    // overflow:auto ancestor and any z-index stacking context for free).
    //
    // We supply only:
    //   - anchor positioning under the trigger on open
    //   - reposition on scroll/resize so the panel tracks its anchor
    //   - close when the trigger leaves the viewport
    //
    // Kanban cards override this in runtime/kanban.rs: there the whole
    // card is the trigger, not the kebab.

    function setupDropdowns() {
        var menus = document.querySelectorAll('[data-popover-menu]');
        for (var i = 0; i < menus.length; i++) {
            initPopoverMenu(menus[i]);
        }
    }

    function initPopoverMenu(panel) {
        var trigger = document.querySelector('[popovertarget="' + panel.id + '"]');
        if (!trigger) return;

        function track() {
            if (!panel.matches(':popover-open') || panel._kanbanFixed) return;
            var rect = trigger.getBoundingClientRect();
            // Close if the trigger has scrolled fully out of view.
            if (rect.bottom < 0 || rect.top > window.innerHeight) {
                panel.hidePopover();
                return;
            }
            positionUnderTrigger(panel, trigger);
        }

        // Position BEFORE the browser paints the panel. `beforetoggle` fires
        // synchronously while the panel is still hidden, so anchoring here means
        // the first open never flashes at the UA default (centered) position —
        // the blink that otherwise only showed on the first click, before a
        // prior inline left/top existed. The panel is display:none at this point
        // so its width can't be measured; anchor by the RIGHT edge to the
        // trigger (exact without a width) and drop below. `toggle` then refines
        // vertical flip + edge clamping with real measured dimensions.
        panel.addEventListener('beforetoggle', function(e) {
            if (e.newState !== 'open' || panel._kanbanFixed) return;
            var rect = trigger.getBoundingClientRect();
            panel.style.position = 'fixed';
            panel.style.inset = 'auto';
            panel.style.margin = '0';
            panel.style.setProperty('min-width', '12rem', 'important');
            panel.style.setProperty('width', 'fit-content', 'important');
            // clientWidth (not innerWidth) excludes the scrollbar, matching the
            // coordinate system of getBoundingClientRect / position:fixed so the
            // right edge lands exactly on the trigger (no first-open nudge).
            var vw = document.documentElement.clientWidth;
            panel.style.left = 'auto';
            panel.style.right = Math.max(8, vw - rect.right) + 'px';
            panel.style.top = (rect.bottom + 4) + 'px';
        });
        panel.addEventListener('toggle', function(e) {
            if (e.newState === 'open' && !panel._kanbanFixed) {
                positionUnderTrigger(panel, trigger);
            }
        });
        // Capture=true so scrolls inside overflow:auto ancestors also fire.
        window.addEventListener('scroll', track, true);
        window.addEventListener('resize', track);
    }

    function positionUnderTrigger(panel, trigger) {
        // The popover opens with UA `inset: 0` which stretches the panel
        // to the viewport. The UA's `width: fit-content !important` is the
        // same value as what we want, but the layout doesn't re-evaluate
        // when we later clear `inset` to auto — so set it inline ourselves
        // to force a fresh sizing pass against the new (unconstrained) box.
        panel.style.setProperty('width', 'fit-content', 'important');
        panel.style.setProperty('min-width', '12rem', 'important');
        panel.style.inset = 'auto';
        panel.style.margin = '0';
        panel.style.position = 'fixed';
        void panel.offsetHeight; // flush layout so offsetWidth is accurate

        var rect = trigger.getBoundingClientRect();
        var pw = panel.offsetWidth || 192;
        var ph = panel.offsetHeight || 100;
        // Right-edge align with trigger, drop below by default.
        var left = rect.right - pw;
        var top = rect.bottom + 4;
        if (left < 8) left = 8;
        if (left + pw > window.innerWidth - 8) left = window.innerWidth - pw - 8;
        if (top + ph > window.innerHeight - 8) {
            var flipped = rect.top - ph - 4;
            if (flipped >= 8) top = flipped;
        }
        panel.style.left = left + 'px';
        panel.style.top = top + 'px';
    }
"#;
