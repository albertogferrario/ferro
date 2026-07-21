pub(super) const SOURCE: &str = r#"
    // ── Command palette ──────────────────────────────────────────────────────
    //
    // Intercepts the open-command-palette custom event and ⌘K / Ctrl+K keydown.
    // Renders grouped search results via GET /dashboard/search?q=.
    // Recents stored in localStorage; quick actions from static config.
    // ARIA: role="combobox" on input, role="listbox" on results, stable
    // option ids ("fjui-palette-opt-{n}"), aria-activedescendant updated
    // on every arrow-key press (Pitfall D-02 baseline).
    //
    // Persistent-frame: inits exactly once via idempotent guard on element.
    // ES5 only: var, named function declarations, C-style for loops.
    // No arrow functions, no const/let, no template literals.

    function setupCommandPalette() {
        // Persistent-frame: init exactly once. Guard on element presence.
        if (document.getElementById('fjui-palette')) return;

        // ── Config ──────────────────────────────────────────────────────────
        var cfg = window.fjuiPaletteConfig || {};
        var searchUrl = cfg.searchUrl || '/dashboard/search';
        var quickActions = cfg.quickActions || [];

        // ── Recents ─────────────────────────────────────────────────────────
        var RECENTS_KEY = 'fjui-palette-recents';
        var RECENTS_MAX = 8;

        function getRecents() {
            try {
                var raw = localStorage.getItem(RECENTS_KEY);
                return raw ? JSON.parse(raw) : [];
            } catch (_) { return []; }
        }

        function recordRecentFromUrl(url) {
            try {
                // Only record dashboard entity-detail URLs (simple path filter).
                if (!url || url.indexOf('/dashboard/') === -1) return;
                var list = getRecents();
                list = [url].concat(list.filter(function(u) { return u !== url; })).slice(0, RECENTS_MAX);
                localStorage.setItem(RECENTS_KEY, JSON.stringify(list));
            } catch (_) {}
        }

        document.addEventListener('fjui:navigated', function() {
            recordRecentFromUrl(window.location.href);
        });

        // ── Build dialog DOM ─────────────────────────────────────────────────
        var dlg = document.createElement('dialog');
        dlg.id = 'fjui-palette';
        dlg.className = 'fjui-palette';
        dlg.setAttribute('role', 'dialog');
        dlg.setAttribute('aria-modal', 'true');
        dlg.setAttribute('aria-label', 'Cerca');

        var input = document.createElement('input');
        input.type = 'text';
        input.className = 'fjui-palette__input';
        input.setAttribute('role', 'combobox');
        input.setAttribute('aria-expanded', 'false');
        input.setAttribute('aria-controls', 'fjui-palette-listbox');
        input.setAttribute('aria-label', 'Cerca nel dashboard');
        input.setAttribute('autocomplete', 'off');
        input.setAttribute('spellcheck', 'false');
        input.placeholder = 'Cerca clienti, ordini, prenotazioni…';

        var listbox = document.createElement('ul');
        listbox.id = 'fjui-palette-listbox';
        listbox.className = 'fjui-palette__results';
        listbox.setAttribute('role', 'listbox');
        listbox.setAttribute('aria-label', 'Risultati');

        var hint = document.createElement('div');
        hint.className = 'fjui-palette__hint';
        hint.textContent = 'ESC per chiudere';
        hint.setAttribute('aria-hidden', 'true');

        dlg.appendChild(input);
        dlg.appendChild(listbox);
        dlg.appendChild(hint);
        document.body.appendChild(dlg);

        // ── Listbox engine ───────────────────────────────────────────────────
        var optionIndex = 0;

        var engine = createListboxEngine(input, listbox, function(optionEl) {
            var url = optionEl.getAttribute('data-url');
            if (!url) return;
            try { dlg.close(); } catch (_) {}
            input.setAttribute('aria-expanded', 'false');
            input.removeAttribute('aria-activedescendant');
            // Navigate via 248 nav runtime. The nav runtime intercepts
            // same-origin assigns and link clicks automatically. Use
            // window.location.assign so same-origin navigations are
            // picked up by the click/popstate interceptors in nav.rs.
            try { window.location.assign(url); } catch (_) {}
        });

        input.addEventListener('keydown', engine.handleKey);

        // Close clears aria state.
        dlg.addEventListener('close', function() {
            input.setAttribute('aria-expanded', 'false');
            input.removeAttribute('aria-activedescendant');
            input.value = '';
            clearTimeout(paletteTimer);
            if (paletteController) { try { paletteController.abort(); } catch (_) {} }
        });

        // ── Abort controller + debounce ──────────────────────────────────────
        var paletteController = null;
        var paletteTimer = null;

        function fetchResults(q) {
            if (paletteController) { try { paletteController.abort(); } catch (_) {} }
            clearTimeout(paletteTimer);
            paletteTimer = setTimeout(function() {
                paletteController = new AbortController();
                try {
                    fetch(searchUrl + '?q=' + encodeURIComponent(q), {
                        credentials: 'same-origin',
                        signal: paletteController.signal
                    }).then(function(r) { return r.json(); })
                      .then(function(data) {
                          renderResults(data.results || [], q);
                      })
                      .catch(function() { /* silently swallow abort and network errors */ });
                } catch (_) {}
            }, 120);
        }

        // ── Render grouped results ───────────────────────────────────────────
        var GROUP_ORDER = ['clienti', 'prenotazioni', 'ordini', 'prodotti'];
        var GROUP_LABELS = {
            clienti: 'CLIENTI',
            prenotazioni: 'PRENOTAZIONI',
            ordini: 'ORDINI',
            prodotti: 'PRODOTTI'
        };

        function makeOption(url, title, subtitle) {
            var li = document.createElement('li');
            li.setAttribute('role', 'option');
            li.id = 'fjui-palette-opt-' + optionIndex;
            li.className = 'fjui-palette__option';
            li.setAttribute('aria-selected', 'false');
            li.setAttribute('data-url', url);
            optionIndex++;

            var titleEl = document.createElement('span');
            titleEl.className = 'fjui-palette__option-title';
            titleEl.textContent = title;

            li.appendChild(titleEl);

            if (subtitle) {
                var subtitleEl = document.createElement('span');
                subtitleEl.className = 'fjui-palette__option-subtitle';
                subtitleEl.textContent = subtitle;
                li.appendChild(subtitleEl);
            }

            li.addEventListener('click', function() {
                engine.updateOptions();
                var url = li.getAttribute('data-url');
                if (!url) return;
                try { dlg.close(); } catch (_) {}
                input.setAttribute('aria-expanded', 'false');
                input.removeAttribute('aria-activedescendant');
                try { window.location.assign(url); } catch (_) {}
            });

            return li;
        }

        function renderResults(results, q) {
            optionIndex = 0;
            while (listbox.firstChild) { listbox.removeChild(listbox.firstChild); }

            if (results.length === 0) {
                var empty = document.createElement('li');
                empty.className = 'fjui-palette__empty';
                if (q && q.length > 0) {
                    // textContent only — no user strings in markup (T-249-02-01).
                    empty.textContent = 'Nessun risultato per "' + q + '"';
                } else {
                    empty.textContent = 'Inizia a digitare per cercare';
                }
                listbox.appendChild(empty);
                input.setAttribute('aria-expanded', 'false');
                return;
            }

            // Bucket by group in fixed order.
            var buckets = {};
            for (var g = 0; g < GROUP_ORDER.length; g++) {
                buckets[GROUP_ORDER[g]] = [];
            }
            for (var i = 0; i < results.length; i++) {
                var item = results[i];
                if (buckets[item.group]) {
                    buckets[item.group].push(item);
                }
            }

            var hasItems = false;
            for (var gi = 0; gi < GROUP_ORDER.length; gi++) {
                var group = GROUP_ORDER[gi];
                var items = buckets[group];
                if (!items || items.length === 0) continue;
                hasItems = true;

                var groupLi = document.createElement('li');
                groupLi.setAttribute('role', 'group');
                groupLi.setAttribute('aria-label', GROUP_LABELS[group] || group.toUpperCase());

                var header = document.createElement('div');
                header.className = 'fjui-palette__group-label';
                header.textContent = GROUP_LABELS[group] || group.toUpperCase();
                groupLi.appendChild(header);

                var innerUl = document.createElement('ul');
                innerUl.setAttribute('role', 'presentation');

                for (var j = 0; j < items.length; j++) {
                    innerUl.appendChild(makeOption(items[j].url, items[j].title, items[j].subtitle || ''));
                }

                groupLi.appendChild(innerUl);
                listbox.appendChild(groupLi);
            }

            if (hasItems) {
                input.setAttribute('aria-expanded', 'true');
            }
            engine.updateOptions();
        }

        // ── Empty state: recents + quick actions ─────────────────────────────
        function renderEmptyState() {
            optionIndex = 0;
            while (listbox.firstChild) { listbox.removeChild(listbox.firstChild); }

            var recents = getRecents();
            var hasContent = false;

            if (recents.length > 0) {
                hasContent = true;
                var recentsLi = document.createElement('li');
                recentsLi.setAttribute('role', 'group');
                recentsLi.setAttribute('aria-label', 'RECENTI');

                var recentsHeader = document.createElement('div');
                recentsHeader.className = 'fjui-palette__group-label';
                recentsHeader.textContent = 'RECENTI';
                recentsLi.appendChild(recentsHeader);

                var recentsUl = document.createElement('ul');
                recentsUl.setAttribute('role', 'presentation');

                for (var r = 0; r < recents.length; r++) {
                    var recentUrl = recents[r];
                    // Derive a display label from the URL path segments.
                    var segments = recentUrl.replace(/\/$/, '').split('/');
                    var label = segments[segments.length - 1] || recentUrl;
                    recentsUl.appendChild(makeOption(recentUrl, label, recentUrl));
                }

                recentsLi.appendChild(recentsUl);
                listbox.appendChild(recentsLi);
            }

            if (quickActions.length > 0) {
                hasContent = true;
                var actionsLi = document.createElement('li');
                actionsLi.setAttribute('role', 'group');
                actionsLi.setAttribute('aria-label', 'AZIONI RAPIDE');

                var actionsHeader = document.createElement('div');
                actionsHeader.className = 'fjui-palette__group-label';
                actionsHeader.textContent = 'AZIONI RAPIDE';
                actionsLi.appendChild(actionsHeader);

                var actionsUl = document.createElement('ul');
                actionsUl.setAttribute('role', 'presentation');

                for (var a = 0; a < quickActions.length; a++) {
                    var action = quickActions[a];
                    actionsUl.appendChild(makeOption(action.url, action.label, ''));
                }

                actionsLi.appendChild(actionsUl);
                listbox.appendChild(actionsLi);
            }

            if (!hasContent) {
                var emptyEl = document.createElement('li');
                emptyEl.className = 'fjui-palette__empty';
                emptyEl.textContent = 'Inizia a digitare per cercare';
                listbox.appendChild(emptyEl);
            }

            if (hasContent) {
                input.setAttribute('aria-expanded', 'true');
                engine.updateOptions();
            }
        }

        // ── Input event ──────────────────────────────────────────────────────
        input.addEventListener('input', function() {
            var q = input.value.trim();
            if (q === '') {
                clearTimeout(paletteTimer);
                if (paletteController) { try { paletteController.abort(); } catch (_) {} }
                renderEmptyState();
            } else {
                fetchResults(q);
            }
        });

        // ── Open function ────────────────────────────────────────────────────
        function openPalette() {
            try {
                dlg.showModal();
            } catch (_) {}
            try { input.focus(); } catch (_) {}
            renderEmptyState();
        }

        // ── Open triggers ────────────────────────────────────────────────────
        // Header 247 seam.
        document.addEventListener('fjui:open-command-palette', function() {
            openPalette();
        });

        // ⌘K / Ctrl+K global shortcut.
        document.addEventListener('keydown', function(e) {
            if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
                e.preventDefault();
                openPalette();
            }
        });
    }
"#;
