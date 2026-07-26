//! LiveFragment client runtime (Phase 260, D-03).
//!
//! Opens the existing `/_ferro/ws` WebSocket, subscribes to each
//! `[data-live-fragment]` channel found on the page, and swaps the
//! container's `innerHTML` when a server-pushed `fragment` event arrives.
//! No WASM, no client-side state, no eval — subscribe + swap only (SC3).

pub(super) const SOURCE: &str = r#"
    // ── LiveFragment WebSocket subscriptions ──────────────────────────────

    function setupLiveFragments() {
        var fragments = document.querySelectorAll('[data-live-fragment]');
        if (!fragments.length) return;

        // One shared socket for all fragments on the page (D-03).
        var ws = new WebSocket(
            (location.protocol === 'https:' ? 'wss://' : 'ws://') +
            location.host + '/_ferro/ws'
        );

        ws.addEventListener('open', function() {
            for (var i = 0; i < fragments.length; i++) {
                var ch = fragments[i].getAttribute('data-channel');
                if (ch) {
                    ws.send(JSON.stringify({ type: 'subscribe', channel: ch }));
                }
            }
        });

        ws.addEventListener('message', function(e) {
            try {
                var msg = JSON.parse(e.data);
                // ServerMessage::Event → { type: "event", event: "...",
                // channel: "...", data: {...} } (ferro-broadcast message.rs).
                // Filter on event === 'fragment' so raw 'delta' is ignored.
                if (msg.type === 'event' && msg.event === 'fragment' &&
                    msg.data && msg.data.html) {
                    var target = document.querySelector(
                        '[data-live-fragment][data-channel="' + msg.channel + '"]'
                    );
                    if (target) { target.innerHTML = msg.data.html; }
                }
            } catch (_) {}
        });

        ws.addEventListener('error', function() {
            // Browser does not auto-reconnect a closed WebSocket.
            // Reconnect strategy deferred to a future phase.
        });
    }
"#;

#[cfg(test)]
mod tests {
    #[test]
    fn live_fragment_runtime_no_wasm_no_state() {
        let src = super::SOURCE;
        assert!(
            src.contains("setupLiveFragments"),
            "must define the setup fn"
        );
        assert!(
            src.contains("data-live-fragment"),
            "must scan for the marker"
        );
        assert!(src.contains("/_ferro/ws"), "must use the fixed WS path");
        assert!(src.contains("innerHTML"), "must swap innerHTML");
        // SC3: no WASM, no client-side reactive state, no eval.
        assert!(!src.contains("WebAssembly"), "SC3: no WASM");
        assert!(!src.contains("useState"), "SC3: no signal/state store");
        assert!(!src.contains("eval("), "SC3: no eval");
    }
}
