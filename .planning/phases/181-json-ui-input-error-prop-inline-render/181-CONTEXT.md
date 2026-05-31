---
phase: 181
name: JSON-UI v1 Input — render `error` prop inline below the field
status: captured (awaiting /gsd-discuss-phase)
discovered: 2026-05-31
discovered-by: gestiscilo Phase 175 UAT (operator product-edit form)
---

# Phase 181 Context

## What's broken

JSON-UI v1 Input elements accept an `error` prop bound via the `{"$data": "/<field>_error"}` reference, but the **v1 Input renderer drops the value on the floor** — the resolved error string is present in the JSON-UI data block at the bottom of the rendered HTML, but no DOM element is emitted near the input to surface it to the operator.

## Concrete repro

`gestiscilo/src/controllers/cassa/products.rs::dettaglio` — operator product-edit page at `/dashboard/cassa/prodotti/{id}/modifica`.

```rust
builder = builder.element(
    "field_overage_threshold",
    Element::new("Input")
        .prop("field", "overage_threshold")
        .prop("label", "Soglia sovrapprezzo")
        .prop("input_type", "number")
        .prop("data_path", /* ... */)
        .prop("error", json!({ "$data": "/overage_threshold_error" })),
);
```

Data plumbing (same handler):

```rust
obj.insert("overage_threshold_error".to_string(),
           json!(req.validation_error("overage_threshold")));
```

After a `ValidationError::new().add("overage_threshold", "Per il sovrapprezzo, ...").with_old_input(&data).redirect_to(...)` round-trip:

- ✅ The data block carries `overage_threshold_error: "Per il sovrapprezzo, compila sia la soglia che il prezzo"`
- ✅ `req.validation_error("overage_threshold")` returns Some(msg)
- ✅ The form value is restored to `value="2"` via the data_path binding
- ❌ **No `<p>` / `<span>` / error element is rendered below the input** — verified by DOM walk:

  ```html
  <div class="space-y-1">
    <label for="overage_threshold">Soglia sovrapprezzo</label>
    <input type="number" id="overage_threshold" value="2">
    <p class="text-sm text-text-muted">Numero di persone oltre cui scatta…</p>
  </div>
  ```

  Only label + input + description render. The `error` prop value is ignored by the renderer.

- ❌ The fallback `?error=generic&msg=Per+il+sovrapprezzo...` query-string flash is the operator's only visible signal.

## Scope of the fix

Extend `ferro-json-ui`'s **v1 Input renderer** so that when the `error` prop resolves to a non-null string the renderer emits a destructive-tone element (e.g. `<p class="text-destructive text-sm mt-1">{error}</p>`) directly below the `<input>` inside the same wrapping `<div class="space-y-1">`.

Same change likely applies to:

- `Select` (combobox)
- `Textarea`
- `Checkbox` (currently no error binding but should accept one)

Possibly also: `<input>` styling on error (add `border-destructive ring-destructive/20` to the input element class chain when error is present), so the field itself is highlighted, not just the message.

## Related dashboard quirk (likely same root cause)

In `gestiscilo/src/controllers/cassa/products.rs::dettaglio`:

```rust
if req.has_validation_errors() {
    root_children.push("toast_validation".to_string());
}
```

This branch is NOT taken even when `req.validation_error("overage_threshold")` returns Some. Result: the toast at the top of the page (`"Controlla i campi evidenziati."`) is also missing.

If `req.has_validation_errors()` and `req.validation_error("X")` are reading from different ferro flash stores (session-cookie vs URL-fallback `?msg=`), this should be reconciled in the same phase. Verify before planning.

## Out of scope

- **JSON-UI v2 element catalog rework** — that's v12.0 territory (Phases 115-121 already planned).
- **Schema-driven projections** — Phase 117.1.
- **Server-side expressions** — Phase 118.

This phase is a minimal v1 renderer patch to close a visible gap that affects every form in every gestiscilo controller with an `error` prop binding.

## Prior art

- Phase 137 (Validator & Old Input — v12.1 Form Validation DX milestone) is the v2 successor planning track. This phase ships the v1 fix so the dashboard doesn't have to wait for v12.x to surface validation messages.

## Discovery context

Surfaced during gestiscilo Phase 175 UAT (2026-05-31) on the operator product-edit form. The paired-nullability gate for `overage_threshold` / `overage_price_cents` fires correctly server-side but the operator only sees a generic URL flash, not an inline field message. Per-plan SUMMARY in `gestiscilo/.planning/phases/175-structured-product-pricing-model-five-typed-columns-on-produ/175-SUMMARY.md`.

## Affected gestiscilo fields (audit)

Every Input/Select/Textarea/Checkbox with `.prop("error", json!({"$data": "/..."}))` in the gestiscilo codebase will surface its inline error after this phase ships. Search:

```bash
rg '\.prop\("error"' ../gestiscilo-it/app/src/
```

Last count before this phase: ~30 bindings across cassa/products, calendario/bookings, settings, staff, documenti.
