pub(super) const SOURCE: &str = r#"
    // ── Peek-card hover overlay ─────────────────────────────────────────────
    //
    // Opens a tooltip-style card on entity reference links that carry
    // data-peek-entity + data-peek-id attributes. Uses the Popover API
    // (top-layer, no stacking context conflicts) with 300 ms open delay and
    // 150 ms leave grace. Touch devices: tap navigates, no peek fires.
    // Cache per-navigation to avoid refetch storms on cursor drift.

    var peekTimer = null;
    var peekGrace = null;
    var peekController = null;
    var peekCache = {};

    function setupPeek() {
        if (document.getElementById('fjui-peek-card')) return;

        var card = document.createElement('div');
        card.id = 'fjui-peek-card';
        card.className = 'fjui-peek-card';
        card.setAttribute('role', 'tooltip');
        card.setAttribute('aria-live', 'off');

        var hasPopover = ('popover' in document.createElement('div'));
        if (hasPopover) {
            card.setAttribute('popover', '');
        } else {
            card.style.position = 'fixed';
            card.style.display = 'none';
        }
        card.hidden = true;
        document.body.appendChild(card);

        document.addEventListener('mouseover', function(e) {
            try {
                var link = e.target && e.target.closest ? e.target.closest('[data-peek-entity]') : null;
                if (!link) return;
                if (e.sourceCapabilities && e.sourceCapabilities.firesTouchEvents) return;
                clearTimeout(peekTimer);
                peekTimer = setTimeout(function() { openPeek(link); }, 300);
            } catch (_) {}
        });

        document.addEventListener('mouseout', function(e) {
            try {
                var link = e.target && e.target.closest ? e.target.closest('[data-peek-entity]') : null;
                if (!link) return;
                clearTimeout(peekTimer);
                peekGrace = setTimeout(function() { closePeek(); }, 150);
            } catch (_) {}
        });

        card.addEventListener('mouseover', function() {
            try {
                clearTimeout(peekGrace);
            } catch (_) {}
        });

        card.addEventListener('mouseout', function() {
            try {
                peekGrace = setTimeout(function() { closePeek(); }, 150);
            } catch (_) {}
        });

        document.addEventListener('keydown', function(e) {
            try {
                if (e.key === 'Escape') closePeek();
            } catch (_) {}
        });

        document.addEventListener('fjui:navigated', function() {
            try {
                peekCache = {};
                closePeek();
            } catch (_) {}
        });
    }

    function openPeek(link) {
        try {
            var entity = link.getAttribute('data-peek-entity');
            var id = link.getAttribute('data-peek-id');
            var url = '/dashboard/peek/' + encodeURIComponent(entity) + '/' + encodeURIComponent(id);
            if (peekCache[url]) {
                renderPeek(peekCache[url], link);
                return;
            }
            if (peekController) {
                try { peekController.abort(); } catch (_) {}
            }
            peekController = new AbortController();
            try {
                fetch(url, { credentials: 'same-origin', signal: peekController.signal })
                    .then(function(r) { return r.json(); })
                    .then(function(data) {
                        peekCache[url] = data;
                        renderPeek(data, link);
                    })
                    .catch(function() {});
            } catch (_) {}
        } catch (_) {}
    }

    function renderPeek(data, link) {
        try {
            var card = document.getElementById('fjui-peek-card');
            if (!card) return;

            while (card.firstChild) {
                card.removeChild(card.firstChild);
            }

            if (data.title) {
                var titleEl = document.createElement('div');
                titleEl.className = 'fjui-peek-card__title';
                titleEl.textContent = data.title;
                card.appendChild(titleEl);
            }

            if (data.fields && data.fields.length) {
                for (var i = 0; i < data.fields.length; i++) {
                    var field = data.fields[i];
                    var row = document.createElement('div');
                    var labelEl = document.createElement('span');
                    labelEl.className = 'fjui-peek-card__label';
                    labelEl.textContent = field.label || '';
                    var valueEl = document.createElement('span');
                    valueEl.className = 'fjui-peek-card__value';
                    valueEl.textContent = field.value || '';
                    row.appendChild(labelEl);
                    row.appendChild(valueEl);
                    card.appendChild(row);
                }
            }

            try {
                var rect = link.getBoundingClientRect();
                var top = rect.bottom + window.scrollY + 4;
                var left = rect.left + window.scrollX;
                card.style.top = top + 'px';
                card.style.left = left + 'px';
            } catch (_) {}

            card.hidden = false;
            var hasPopover = ('popover' in document.createElement('div'));
            if (hasPopover) {
                try { card.showPopover(); } catch (_) {}
            } else {
                card.style.display = 'block';
            }
        } catch (_) {}
    }

    function closePeek() {
        try {
            var card = document.getElementById('fjui-peek-card');
            if (!card) return;
            var hasPopover = ('popover' in document.createElement('div'));
            if (hasPopover) {
                try { card.hidePopover(); } catch (_) {}
            } else {
                card.style.display = 'none';
            }
            card.hidden = true;
        } catch (_) {}
    }
"#;
