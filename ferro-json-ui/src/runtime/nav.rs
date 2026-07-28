pub(super) const SOURCE: &str = r#"
    // ── Instant navigation runtime ────────────────────────────────────────
    //
    // Intercepts same-origin GET <a> clicks on the dashboard, fetches the
    // destination page, and swaps only the #ferro-json-ui content region.
    // The sidebar, header, toast container, and SSE EventSource are never
    // re-rendered (persistent frame — NAV-01).
    //
    // Prefetch on pointerdown / hover-dwell so warm navigations apply
    // immediately with no visible loading state (NAV-02).
    //
    // History, scroll position, and focus are managed explicitly:
    //   - history.scrollRestoration = 'manual'
    //   - scroll state stored in history.state (main.scrollTop)
    //   - focus moves to PageHeader h2 after forward navigation (NAV-03)
    //
    // SSE EventSource is never torn down. Scripts in swapped content are
    // re-executed by cloning into fresh <script> nodes (same-origin only).
    // fjui:navigated fires after re-execution; fjui:before-navigate fires
    // before the swap so page-scoped EventSources can close (NAV-04).
    //
    // POST forms, modified clicks, target=_blank, download, hash-only, and
    // cross-origin links are never intercepted (D-14).

    // ── Progress hairline ─────────────────────────────────────────────────

    function setupProgressHairline() {
        if (document.querySelector('.fjui-nav-progress')) return;
        var bar = document.createElement('div');
        bar.className = 'fjui-nav-progress';
        bar.setAttribute('aria-hidden', 'true');
        document.body.appendChild(bar);

        function showHairline() {
            bar.classList.add('fjui-nav-progress--active');
        }
        function doneHairline() {
            bar.classList.remove('fjui-nav-progress--active');
            bar.classList.add('fjui-nav-progress--done');
            setTimeout(function() {
                bar.classList.remove('fjui-nav-progress--done');
            }, 300);
        }
        function resetHairline() {
            bar.classList.remove('fjui-nav-progress--active');
            bar.classList.remove('fjui-nav-progress--done');
        }

        // Expose helpers for setupNav (same closure scope).
        window.__fjuiHairline = {
            show: showHairline,
            done: doneHairline,
            reset: resetHairline
        };
    }

    // ── Navigation runtime ────────────────────────────────────────────────

    function setupNav() {
        if (!document.getElementById('ferro-json-ui')) return;

        // Disable browser scroll restoration — we manage it manually (D-07).
        history.scrollRestoration = 'manual';

        // The actual scroll container (HIDE_SCROLLBARS_CSS pins html/body to
        // overflow:hidden; only <main> scrolls).
        var mainEl = document.querySelector('body > div.flex.flex-col > main');

        // In-memory prefetch cache. Each entry: { promise, controller, ts }.
        var prefetchCache = {};
        var inflight = 0;
        var MAX_INFLIGHT = 2;
        var PREFETCH_TTL = 5000;
        var HOVER_DWELL = 80;

        // Track the URL that a click actually intends to navigate to, so a
        // late-arriving prefetch for a superseded URL cannot be applied (Pitfall 4).
        var intendedUrl = null;

        function evictStale(url) {
            var entry = prefetchCache[url];
            if (entry && (Date.now() - entry.ts) > PREFETCH_TTL) {
                delete prefetchCache[url];
                return true;
            }
            return false;
        }

        function prefetch(url) {
            if (prefetchCache[url]) {
                evictStale(url);
                if (prefetchCache[url]) return; // still fresh
            }
            if (inflight >= MAX_INFLIGHT) return;
            var controller = new AbortController();
            inflight++;
            var entry = {
                promise: fetch(url, {
                    credentials: 'same-origin',
                    headers: { 'X-FJUI-Nav': '1', 'X-FJUI-Target': 'ferro-json-ui' },
                    signal: controller.signal
                }).then(function(r) {
                    inflight--;
                    return r;
                }).catch(function() {
                    inflight--;
                    delete prefetchCache[url];
                }),
                controller: controller,
                ts: Date.now()
            };
            prefetchCache[url] = entry;
        }

        function abortAllPrefetchesExcept(keepUrl) {
            for (var k in prefetchCache) {
                if (k !== keepUrl && prefetchCache[k] && prefetchCache[k].controller) {
                    try { prefetchCache[k].controller.abort(); } catch (_) {}
                    delete prefetchCache[k];
                }
            }
        }

        function shouldIntercept(a, event) {
            if (!a || !a.href) return false;
            if (event.defaultPrevented) return false;
            if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return false;
            if (event.button !== 0) return false;
            if (a.target && a.target !== '' && a.target !== '_self') return false;
            if (a.hasAttribute('download')) return false;
            var href = a.href;
            // Hash-only: same page, only fragment changes.
            if (href.indexOf('#') !== -1) {
                try {
                    var u = new URL(href);
                    if (u.pathname === window.location.pathname && u.search === window.location.search) {
                        return false;
                    }
                } catch (_) { return false; }
            }
            try {
                var urlObj = new URL(href);
                if (urlObj.origin !== window.location.origin) return false;
                if (urlObj.protocol !== 'http:' && urlObj.protocol !== 'https:') return false;
            } catch (_) {
                return false;
            }
            return true;
        }

        function findAnchor(el) {
            var node = el;
            while (node && node !== document.body) {
                if (node.tagName === 'A') return node;
                node = node.parentElement;
            }
            return null;
        }

        function navigate(url, isPopstate) {
            var hairline = window.__fjuiHairline;
            var timer = null;

            // Only show hairline (after 150ms delay) for navigations that aren't
            // already resolved in the prefetch cache.
            var hasCached = prefetchCache[url] && !evictStale(url);
            if (!hasCached && hairline) {
                timer = setTimeout(function() { hairline.show(); }, 150);
            }

            // Abort all other in-flight prefetches — their responses must not
            // overwrite this navigation's result (D-05, Pitfall 4).
            abortAllPrefetchesExcept(url);

            var responsePromise;
            if (prefetchCache[url] && !evictStale(url)) {
                responsePromise = prefetchCache[url].promise;
                delete prefetchCache[url];
            } else {
                var controller = new AbortController();
                responsePromise = fetch(url, {
                    credentials: 'same-origin',
                    headers: { 'X-FJUI-Nav': '1', 'X-FJUI-Target': 'ferro-json-ui' },
                    signal: controller.signal
                });
            }

            responsePromise.then(function(response) {
                var isFragment = response.headers.get('X-FJUI-Fragment') === '1';
                var fragTitle = isFragment ? response.headers.get('X-FJUI-Title') : null;
                var fragBodyClass = isFragment ? response.headers.get('X-FJUI-Body-Class') : null;

                // Swappable check (D-03).
                var contentType = response.headers.get('content-type') || '';
                if (!response.ok || contentType.indexOf('text/html') === -1) {
                    if (timer) clearTimeout(timer);
                    if (hairline) hairline.reset();
                    clearBusy();
                    window.location.assign(url);
                    return;
                }
                // Response-URL correlation: check origin is still same-origin
                // (guards against unexpected redirects to external sites).
                try {
                    var responseUrl = new URL(response.url);
                    if (responseUrl.origin !== window.location.origin) {
                        if (timer) clearTimeout(timer);
                        if (hairline) hairline.reset();
                        clearBusy();
                        window.location.assign(url);
                        return;
                    }
                } catch (_) {
                    if (timer) clearTimeout(timer);
                    if (hairline) hairline.reset();
                    clearBusy();
                    window.location.assign(url);
                    return;
                }

                response.text().then(function(html) {
                    // Concurrency guard: if another click superseded this one, discard.
                    if (!isPopstate && url !== intendedUrl) {
                        if (timer) clearTimeout(timer);
                        if (hairline) hairline.reset();
                        clearBusy();
                        return;
                    }

                    var doc = new DOMParser().parseFromString(html, 'text/html');
                    var newEl = doc.getElementById('ferro-json-ui');
                    if (!newEl) {
                        if (timer) clearTimeout(timer);
                        if (hairline) hairline.reset();
                        clearBusy();
                        window.location.assign(url);
                        return;
                    }

                    // Capture the departing page's scroll position BEFORE the
                    // swap mutates the DOM: replacing tall content with shorter
                    // content makes the browser clamp mainEl.scrollTop, so reading
                    // it after replaceChildren would save 0 (B-03).
                    var departScrollTop = mainEl ? mainEl.scrollTop : 0;

                    // Before-swap cleanup hook (D-13): page scripts can listen
                    // to this event to close transient EventSources.
                    try {
                        document.dispatchEvent(new CustomEvent('fjui:before-navigate', {
                            detail: { url: url }
                        }));
                    } catch (_) {}

                    // Swap inner content of #ferro-json-ui (swap target only — never
                    // an ancestor node; sidebar/header node identity preserved).
                    var target = document.getElementById('ferro-json-ui');
                    if (target) {
                        var children = Array.prototype.slice.call(newEl.childNodes);
                        target.replaceChildren.apply(target, children);
                    }

                    // Sync body class from destination page (NAV-05: fill_viewport
                    // toggling — ferro-fill must be added/removed so the CSS chain
                    // activates on POS pages and deactivates on standard pages).
                    if (fragBodyClass !== null) {
                        document.body.className = fragBodyClass;
                    } else if (doc.body) {
                        document.body.className = doc.body.className;
                    }

                    // Update document title.
                    if (fragTitle !== null) {
                        document.title = fragTitle;
                    } else {
                        var titleEl = doc.querySelector('title');
                        if (titleEl) document.title = titleEl.textContent;
                    }

                    // Sidebar active-item update (D-03, Pitfall 6).
                    var oldActive = document.querySelector('.fjui-sidebar__nav-item--active');
                    if (oldActive) {
                        oldActive.classList.remove('fjui-sidebar__nav-item--active');
                    }
                    try {
                        var newPath = new URL(url).pathname;
                        var navLinks = document.querySelectorAll('.fjui-sidebar__nav-item[href]');
                        for (var j = 0; j < navLinks.length; j++) {
                            if (navLinks[j].getAttribute('href') === newPath) {
                                navLinks[j].classList.add('fjui-sidebar__nav-item--active');
                                break;
                            }
                        }
                    } catch (_) {}

                    // History + scroll management (D-07).
                    if (!isPopstate) {
                        // Save current scroll position into the current history entry
                        // before pushing the new state.
                        try {
                            history.replaceState(
                                { scrollTop: departScrollTop },
                                document.title
                            );
                            history.pushState({ scrollTop: 0 }, document.title, url);
                        } catch (_) {}
                        if (mainEl) mainEl.scrollTop = 0;
                    } else {
                        // popstate: restore scroll from event state (set by caller).
                        // Caller passes the state value as the third arg via closure.
                    }

                    // Hairline done + clear busy state on the initiating link.
                    if (timer) clearTimeout(timer);
                    if (hairline) hairline.done();
                    clearBusy();

                    // Script re-execution (D-12, B-02): scripts set via innerHTML/
                    // replaceChildren are inert; clone into fresh <script> nodes.
                    // Same-origin guard: never execute scripts from external origins.
                    // Type guard: skip non-executable script types (data islands,
                    // JSON-LD, templates, module scripts) — only classic JS
                    // (no type, or type="text/javascript") is re-executed.
                    var pendingLoads = 0;
                    function dispatchNavigated() {
                        try {
                            document.dispatchEvent(new CustomEvent('fjui:navigated'));
                        } catch (_) {}
                        // D-16: re-initialize page-scoped components after every
                        // content swap. All setup functions are idempotent via their
                        // own guards, so re-invoking ferroRuntime() is always safe.
                        // Do NOT call ferroRuntime() from inside a setup function
                        // (infinite-loop anti-pattern).
                        try { if (typeof ferroRuntime === 'function') ferroRuntime(); } catch (_) {}
                    }
                    if (newEl) {
                        var scripts = newEl.querySelectorAll('script');
                        for (var i = 0; i < scripts.length; i++) {
                            var srcType = scripts[i].type;
                            // Skip non-executable script types.
                            if (srcType && srcType !== '' && srcType !== 'text/javascript') {
                                continue;
                            }
                            var s = document.createElement('script');
                            if (scripts[i].src) {
                                try {
                                    if (new URL(scripts[i].src).origin !== window.location.origin) {
                                        continue;
                                    }
                                } catch (_) {
                                    continue;
                                }
                                // External-src scripts load asynchronously.
                                // Track pending loads so fjui:navigated fires only
                                // after all same-origin external scripts have executed
                                // (or failed). A 5 s timeout ensures a hung script
                                // cannot block the event indefinitely.
                                s.src = scripts[i].src;
                                pendingLoads++;
                                (function(node) {
                                    var settled = false;
                                    function settle() {
                                        if (settled) return;
                                        settled = true;
                                        pendingLoads--;
                                        if (pendingLoads === 0) dispatchNavigated();
                                    }
                                    node.onload = settle;
                                    node.onerror = settle;
                                    setTimeout(settle, 5000);
                                }(s));
                            } else {
                                s.textContent = scripts[i].textContent;
                            }
                            document.head.appendChild(s);
                        }
                    }

                    // Fire navigated event after all inline scripts have executed
                    // (synchronous) and all same-origin external-src scripts have
                    // loaded (or timed out after 5 s). If there are no external-src
                    // scripts, fires immediately here (pendingLoads === 0).
                    // NOTE: fjui:navigated fires after all inline scripts in the
                    // swapped content have executed. External-src scripts (same-origin
                    // only) are appended and awaited via onload; a 5 s fallback
                    // prevents a hung script from blocking the event indefinitely.
                    // Phase-249 init hooks may be inline or same-origin external-src.
                    if (pendingLoads === 0) {
                        dispatchNavigated();
                    }

                    // Focus PageHeader h2 on forward navigation (D-09).
                    if (!isPopstate) {
                        try {
                            var h2 = document.querySelector('#ferro-json-ui h2.fjui-text--display');
                            if (h2) {
                                h2.setAttribute('tabindex', '-1');
                                h2.focus({ preventScroll: true });
                            }
                        } catch (_) {}
                    }

                }).catch(function() {
                    if (timer) clearTimeout(timer);
                    if (hairline) hairline.reset();
                    clearBusy();
                    window.location.assign(url);
                });

            }).catch(function() {
                if (timer) clearTimeout(timer);
                if (hairline) hairline.reset();
                clearBusy();
                window.location.assign(url);
            });
        }

        // pointerdown: start prefetch early (D-04).
        document.addEventListener('pointerdown', function(event) {
            var a = findAnchor(event.target);
            if (a && shouldIntercept(a, event)) {
                prefetch(a.href);
            }
        }, true);

        // mouseover with dwell timer (D-04 secondary trigger).
        var hoverTimer = null;
        var hoverUrl = null;
        document.addEventListener('mouseover', function(event) {
            var a = findAnchor(event.target);
            if (a && shouldIntercept(a, { button: 0 })) {
                if (a.href !== hoverUrl) {
                    if (hoverTimer) clearTimeout(hoverTimer);
                    hoverUrl = a.href;
                    hoverTimer = setTimeout(function() {
                        prefetch(hoverUrl);
                    }, HOVER_DWELL);
                }
            } else {
                if (hoverTimer) clearTimeout(hoverTimer);
                hoverTimer = null;
                hoverUrl = null;
            }
        });
        document.addEventListener('mouseout', function() {
            if (hoverTimer) clearTimeout(hoverTimer);
            hoverTimer = null;
            hoverUrl = null;
        });

        // Track the link currently showing a busy state so it can be cleared
        // after the swap completes or on failure. Only one link is ever busy
        // at a time (the most recent intercepted click).
        var busyAnchor = null;

        function setBusy(a) {
            if (busyAnchor && busyAnchor !== a) {
                try { busyAnchor.removeAttribute('aria-busy'); } catch (_) {}
            }
            busyAnchor = a;
            try { a.setAttribute('aria-busy', 'true'); } catch (_) {}
        }

        function clearBusy() {
            if (busyAnchor) {
                try { busyAnchor.removeAttribute('aria-busy'); } catch (_) {}
                busyAnchor = null;
            }
        }

        // click: intercept same-origin GET <a> clicks (D-14).
        document.addEventListener('click', function(event) {
            // D-12 / T-249-04-04: never intercept clicks while an inline editor is active.
            try {
                if (document.querySelector('[data-inline-edit-active]')) return;
                if (event.target && event.target.closest && event.target.closest('[data-inline-edit-active]')) return;
            } catch (_) {}
            var a = findAnchor(event.target);
            if (!a) return;
            if (!shouldIntercept(a, event)) return;
            event.preventDefault();
            intendedUrl = a.href;
            setBusy(a);
            navigate(intendedUrl, false);
        }, true);

        // popstate: re-fetch the page and restore scroll (D-08).
        window.addEventListener('popstate', function(event) {
            var url = window.location.href;
            var scrollTop = event.state && typeof event.state.scrollTop === 'number'
                ? event.state.scrollTop : 0;
            intendedUrl = url;
            navigate(url, true);
            // Scroll is restored after the swap completes; pass via closure.
            // Override: patch scroll restoration into the navigate callback.
            // Since navigate is async, we defer the scroll restore via a
            // listener on fjui:navigated (fired after the swap).
            var restoreOnce = function() {
                if (mainEl) mainEl.scrollTop = scrollTop;
                document.removeEventListener('fjui:navigated', restoreOnce);
            };
            document.addEventListener('fjui:navigated', restoreOnce);
        });
    }
"#;

#[cfg(test)]
mod nav_source_tests {
    use super::SOURCE;

    #[test]
    fn nav_sends_fjui_target_header() {
        assert!(
            SOURCE.contains("'X-FJUI-Target': 'ferro-json-ui'"),
            "nav.js must send X-FJUI-Target header"
        );
    }

    #[test]
    fn nav_reads_fragment_headers() {
        assert!(
            SOURCE.contains("X-FJUI-Fragment"),
            "nav.js must read X-FJUI-Fragment response header"
        );
        assert!(
            SOURCE.contains("X-FJUI-Title"),
            "nav.js must read X-FJUI-Title response header"
        );
        assert!(
            SOURCE.contains("X-FJUI-Body-Class"),
            "nav.js must read X-FJUI-Body-Class response header"
        );
    }

    #[test]
    fn nav_still_sends_fjui_nav_header() {
        // Backward compat: both headers sent.
        assert!(
            SOURCE.contains("'X-FJUI-Nav': '1'"),
            "nav.js must still send X-FJUI-Nav for backward compat"
        );
    }
}
