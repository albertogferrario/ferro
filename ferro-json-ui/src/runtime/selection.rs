// Security notes (T-256-12..T-256-15):
//   T-256-14: The running total is display-only. The confirm POST carries only
//   hidden-input quantities; the server re-derives the authoritative total from
//   products × quantities.
//   T-256-15: All arithmetic is integer-cents. The only float appears in the
//   presentational (n/100).toFixed(2) display string, never in a stored or
//   POSTed value.
//   T-256-12: Field names are sanitized before use in attribute selectors.
//   T-256-13: Name/qty/total are written via textContent, never innerHTML.

pub(super) const SOURCE: &str = r#"
    // ── Selection panel — live cart view ────────────────────────────────────
    //
    // Reconciliation is input-event-driven (D-07): any bubbling 'input' from
    // a [data-qty-input] in the panel's form scope triggers a full reconcile.
    // qty > 0 ensures a line (cloned from <template>) with name + qty + line
    // total; qty = 0 removes the line; running total always recomputed in
    // integer cents; EmptyState toggles. All mutations funnel through one path.
    //
    // Attribute contract (D-06..D-15):
    //   [data-selection-panel]                   — panel root
    //   [data-selection-form="{form_id}"]        — scope isolator
    //   [data-selection-line-template]           — <template> for line markup
    //   [data-selection-lines]                   — lines container (scrollable)
    //   [data-selection-empty]                   — EmptyState (toggled by runtime)
    //   [data-selection-total]                   — running total display
    //   [data-selection-currency="{symbol}"]     — currency symbol (on total el)
    //   [data-selection-line]                    — line wrapper (in template)
    //   [data-selection-line-field="{field}"]    — lookup key set by runtime on cloned lines
    //   [data-selection-line-name]               — item name text node (in template)
    //   [data-selection-line-qty]                — quantity display span (in template)
    //   [data-selection-line-total]              — per-line subtotal text node (in template)
    //   [data-selection-inc="{field}"]           — per-line inc button (delegated click)
    //   [data-selection-dec="{field}"]           — per-line dec button (delegated click)
    //   [data-selection-remove="{field}"]        — per-line remove button (delegated click)
    //   [data-filter-text]                       — tile root (display name source, D-09)
    //   [data-unit-price]                        — integer cents on tile root (D-09)
    //   [data-qty-input="{field}"]               — hidden form input (single source of truth)

    function setupSelection() {
        var panels = document.querySelectorAll('[data-selection-panel]');
        if (panels.length === 0) return;
        for (var i = 0; i < panels.length; i++) {
            initSelectionPanel(panels[i]);
        }
    }

    function initSelectionPanel(panel) {
        // D-11: resolve form scope — getElementById fallback to document
        var formId = panel.getAttribute('data-selection-form');
        var form = formId ? document.getElementById(formId) : null;
        if (!form) form = document;

        var tmpl = panel.querySelector('[data-selection-line-template]');
        var linesEl = panel.querySelector('[data-selection-lines]');
        var emptyEl = panel.querySelector('[data-selection-empty]');
        var totalEl = panel.querySelector('[data-selection-total]');
        var currency = totalEl ? (totalEl.getAttribute('data-selection-currency') || '') : '';

        // D-07: delegated input listener — any [data-qty-input] change triggers reconcile
        form.addEventListener('input', function(e) {
            if (e.target && e.target.getAttribute && e.target.getAttribute('data-qty-input')) {
                reconcile();
            }
        });

        // D-10: delegated click for per-line controls (post-load cloned lines, no per-element binding)
        panel.addEventListener('click', function(e) {
            var field, input;
            // Optional secondary stepper: clamp to the tile's [min,max], then reconcile.
            var sIncBtn = e.target.closest('[data-selection-secondary-inc]');
            var sDecBtn = e.target.closest('[data-selection-secondary-dec]');
            if (sIncBtn || sDecBtn) {
                var sf = (sIncBtn || sDecBtn).getAttribute(sIncBtn ? 'data-selection-secondary-inc' : 'data-selection-secondary-dec');
                if (sf) sf = sf.replace(/["\\\]]/g, '');
                var sInp = form.querySelector('[data-secondary-input="' + sf + '"]');
                var tileEl = sInp ? sInp.closest('[data-secondary-field]') : null;
                if (sInp && tileEl) {
                    var smn = parseInt(tileEl.getAttribute('data-secondary-min'), 10) || 0;
                    var smx = parseInt(tileEl.getAttribute('data-secondary-max'), 10) || 0;
                    var scur = parseInt(sInp.value, 10);
                    if (isNaN(scur)) scur = smn;
                    var snext = scur + (sIncBtn ? 1 : -1);
                    if (snext < smn) snext = smn;
                    if (smx > 0 && snext > smx) snext = smx;
                    sInp.value = snext;
                    reconcile();
                }
                return;
            }
            var incBtn = e.target.closest('[data-selection-inc]');
            var decBtn = e.target.closest('[data-selection-dec]');
            var remBtn = e.target.closest('[data-selection-remove]');
            if (incBtn) {
                field = incBtn.getAttribute('data-selection-inc');
                // T-256-12: sanitize field name before building attribute selector
                if (field) field = field.replace(/["\\\]]/g, '');
                input = form.querySelector('[data-qty-input="' + field + '"]');
                if (input) {
                    input.value = (parseInt(input.value, 10) || 0) + 1;
                    input.dispatchEvent(new Event('input', { bubbles: true }));
                }
            } else if (decBtn) {
                field = decBtn.getAttribute('data-selection-dec');
                if (field) field = field.replace(/["\\\]]/g, '');
                input = form.querySelector('[data-qty-input="' + field + '"]');
                if (input) {
                    input.value = Math.max(0, (parseInt(input.value, 10) || 0) - 1);
                    input.dispatchEvent(new Event('input', { bubbles: true }));
                }
            } else if (remBtn) {
                field = remBtn.getAttribute('data-selection-remove');
                if (field) field = field.replace(/["\\\]]/g, '');
                input = form.querySelector('[data-qty-input="' + field + '"]');
                if (input) {
                    input.value = 0;
                    input.dispatchEvent(new Event('input', { bubbles: true }));
                }
            }
        });

        // D-07: initial pass — covers server-rendered default_quantity > 0 state
        reconcile();

        function reconcile() {
            if (!tmpl || !linesEl) return;
            var inputs = form.querySelectorAll('[data-qty-input]');
            var totalCents = 0;
            var hasLines = false;
            var i, input, field, qty, tile, name, unit, lineCents, lineEl, qtyEl, lineTotalEl;
            var clone, newLine, incEl, decEl, remEl, nameEl;
            var sField, secEl, sLabelEl, sIncEl, sDecEl, sInput, sCountEl;

            for (i = 0; i < inputs.length; i++) {
                input = inputs[i];
                field = input.getAttribute('data-qty-input');
                if (!field) continue;
                // T-256-12: sanitize field name before building attribute selector
                field = field.replace(/["\\\]]/g, '');
                qty = parseInt(input.value, 10) || 0;

                if (qty > 0) {
                    hasLines = true;
                    // D-09: read display name and unit price from the tile DOM
                    tile = input.closest('[data-filter-text]');
                    name = tile ? tile.getAttribute('data-filter-text') : field;
                    unit = tile ? (parseInt(tile.getAttribute('data-unit-price'), 10) || 0) : 0;
                    // T-256-15: integer-cents arithmetic throughout
                    lineCents = unit * qty;
                    totalCents = totalCents + lineCents;

                    // Find existing line or create via template clone (D-08)
                    lineEl = linesEl.querySelector('[data-selection-line-field="' + field + '"]');
                    if (!lineEl) {
                        clone = tmpl.content.cloneNode(true);
                        newLine = clone.querySelector('[data-selection-line]');
                        if (newLine) {
                            newLine.setAttribute('data-selection-line-field', field);
                            // Wire delegated-click field attr onto template buttons (D-10)
                            incEl = clone.querySelector('[data-selection-inc]');
                            decEl = clone.querySelector('[data-selection-dec]');
                            remEl = clone.querySelector('[data-selection-remove]');
                            if (incEl) incEl.setAttribute('data-selection-inc', field);
                            if (decEl) decEl.setAttribute('data-selection-dec', field);
                            if (remEl) remEl.setAttribute('data-selection-remove', field);
                            nameEl = clone.querySelector('[data-selection-line-name]');
                            // T-256-13: textContent only, never innerHTML
                            if (nameEl) nameEl.textContent = name || '';
                            // Optional secondary stepper: reveal + wire only when the
                            // line's tile declares data-secondary-* (opt-in per line).
                            sField = tile ? tile.getAttribute('data-secondary-field') : null;
                            secEl = clone.querySelector('[data-selection-secondary]');
                            if (sField && secEl) {
                                secEl.style.display = '';
                                sLabelEl = clone.querySelector('[data-selection-secondary-label]');
                                if (sLabelEl) sLabelEl.textContent = tile.getAttribute('data-secondary-label') || '';
                                sIncEl = clone.querySelector('[data-selection-secondary-inc]');
                                sDecEl = clone.querySelector('[data-selection-secondary-dec]');
                                if (sIncEl) sIncEl.setAttribute('data-selection-secondary-inc', sField);
                                if (sDecEl) sDecEl.setAttribute('data-selection-secondary-dec', sField);
                            }
                        }
                        linesEl.appendChild(clone);
                        // Re-query after append (fragment was moved into linesEl)
                        lineEl = linesEl.querySelector('[data-selection-line-field="' + field + '"]');
                    }
                    if (lineEl) {
                        qtyEl = lineEl.querySelector('[data-selection-line-qty]');
                        lineTotalEl = lineEl.querySelector('[data-selection-line-total]');
                        // T-256-13: textContent only
                        if (qtyEl) qtyEl.textContent = qty;
                        if (lineTotalEl) lineTotalEl.textContent = formatMoney(lineCents, currency);
                        // Refresh the secondary count display from its hidden input.
                        sField = tile ? tile.getAttribute('data-secondary-field') : null;
                        if (sField) {
                            sInput = form.querySelector('[data-secondary-input="' + sField.replace(/["\\\]]/g, '') + '"]');
                            sCountEl = lineEl.querySelector('[data-selection-secondary-count]');
                            if (sInput && sCountEl) sCountEl.textContent = parseInt(sInput.value, 10) || 0;
                        }
                    }
                } else {
                    // qty === 0: remove the line if it exists
                    lineEl = linesEl.querySelector('[data-selection-line-field="' + field + '"]');
                    if (lineEl) lineEl.parentNode.removeChild(lineEl);
                }
            }

            // D-12: update running total display (display-only — T-256-14)
            if (totalEl) totalEl.textContent = formatMoney(totalCents, currency);

            // D-13: toggle EmptyState vs lines container via style.display
            if (emptyEl) emptyEl.style.display = hasLines ? 'none' : '';
            if (linesEl) linesEl.style.display = hasLines ? '' : 'none';
        }
    }

    // D-12: presentational integer-cents formatter.
    // T-256-15: the (n/100).toFixed(2) float is display-only; no stored/POSTed value
    // is ever derived from this string. The hidden input always carries integer cents.
    // Italian decimal convention: comma separator (€1.234,56), not period.
    function formatMoney(cents, symbol) {
        var n = parseInt(cents, 10) || 0;
        var parts = (n / 100).toFixed(2).split('.');
        var whole = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, '.');
        var s = whole + ',' + parts[1];
        return symbol ? symbol + s : s;
    }
"#;

#[cfg(test)]
mod tests {
    /// The reconcile + delegated-click runtime must read the tile's
    /// `data-secondary-*`, drive `data-selection-secondary-inc/dec`, and write the
    /// `data-secondary-input` hidden field — the optional per-line secondary stepper.
    #[test]
    fn runtime_handles_secondary_stepper() {
        let js = super::SOURCE;
        assert!(
            js.contains("data-secondary-field"),
            "reconcile must read tile secondary metadata"
        );
        assert!(
            js.contains("data-selection-secondary-inc"),
            "must handle secondary inc"
        );
        assert!(
            js.contains("data-selection-secondary-dec"),
            "must handle secondary dec"
        );
        assert!(
            js.contains("data-secondary-input"),
            "must write the secondary hidden input"
        );
    }
}
