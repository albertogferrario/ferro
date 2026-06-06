// Polarity note: the `typeof … === 'undefined'` early-return below is the
// inverse of the guard used in `plugins/map.rs` §306. That site uses a
// positive `!== 'undefined'` wrapper because the guard sits inside a
// per-element callback; here it sits at the top of a setup function, so
// the early-return polarity matches every other sibling primitive.
pub(super) const SOURCE: &str = r#"
    // ── Lazy hero video promotion ─────────────────────────────────────────

    function setupLazyHeroes() {
        if (typeof IntersectionObserver === 'undefined') return;
        var els = document.querySelectorAll('video[preload="none"][data-lazy-hero]:not([data-lazy-hero-promoted])');
        if (!els.length) return;
        var groups = {};
        for (var i = 0; i < els.length; i++) {
            var m = (els[i].getAttribute('data-lazy-hero-margin') || '200px 0px').replace(/^\s+|\s+$/g, '');
            (groups[m] = groups[m] || []).push(els[i]);
        }
        for (var key in groups) {
            var io = new IntersectionObserver(function(entries, obs) {
                for (var j = 0; j < entries.length; j++) {
                    var e = entries[j];
                    if (e.isIntersecting && !e.target.hasAttribute('data-lazy-hero-promoted')) {
                        e.target.setAttribute('preload', 'auto');
                        e.target.setAttribute('data-lazy-hero-promoted', '1');
                        try { e.target.load(); } catch (_) {}
                        obs.unobserve(e.target);
                    }
                }
            }, { rootMargin: key });
            for (var k = 0; k < groups[key].length; k++) {
                io.observe(groups[key][k]);
            }
        }
    }
"#;
