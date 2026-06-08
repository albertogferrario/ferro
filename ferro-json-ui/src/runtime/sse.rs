pub(super) const SOURCE: &str = r#"
    // ── SSE connection ────────────────────────────────────────────────────

    function setupSSE() {
        var sseUrl = document.body && document.body.getAttribute('data-sse-url');
        if (!sseUrl) return;
        var es = new EventSource(sseUrl);
        es.onmessage = function(event) {
            try {
                var data = JSON.parse(event.data);
                handleSSEMessage(data);
            } catch (e) {
                // Ignore unparseable messages
            }
        };
        es.onerror = function() {
            // EventSource will attempt to reconnect automatically
        };
    }

    function handleSSEMessage(data) {
        if (data && data.key !== undefined && data.value !== undefined) {
            updateLiveValues(data.key, data.value);
        }
        if (data && data.toast) {
            showToast(data.toast);
        }
        if (data && data.reload_kanban) {
            if (window.location.pathname.indexOf('/cassa/ordini') !== -1 ||
                window.location.pathname.indexOf('/prenotazioni') !== -1) {
                window.location.reload();
            }
        }
        // App-registered SSE handlers. Apps push functions onto
        // window.__ferroSSEHandlers (creating it if absent, so registration is
        // ordering-safe relative to this IIFE). Each receives the parsed frame
        // and may act on custom keys the built-in handlers ignore. A throwing
        // handler must not break sibling handlers or the stream.
        if (window.__ferroSSEHandlers) {
            for (var hi = 0; hi < window.__ferroSSEHandlers.length; hi++) {
                try {
                    window.__ferroSSEHandlers[hi](data);
                } catch (e) {
                    // Ignore handler errors — isolate app code from the stream
                }
            }
        }
    }

    function updateLiveValues(key, value) {
        var targets = document.querySelectorAll('[data-sse-target="' + key + '"]');
        for (var i = 0; i < targets.length; i++) {
            targets[i].textContent = value;
        }
    }
"#;
