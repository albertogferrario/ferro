# Plugin Surface Audit (Phase 164 D-06..D-07)

**Audited:** 2026-05-17 (paper exercise)
**Conducted by:** Plan 10 (docs pass), confirmed by Plan 11
**Guide audited:** `docs/src/json-ui/plugins.md` (as updated through Plan 10 commit `63529b33`)
**Outcome:** **B — minor gaps found and fixed inline; clean after fixes**

## Purpose

Walk the plugin author guide as if implementing three distinct widgets from scratch. Note any step where the documentation is insufficient to proceed. Fix inline gaps (Outcome B); escalate load-bearing missing primitives to BLOCKER rows in V1-DELETION-AUDIT.md (Outcome C).

This is a paper exercise — no plugins were actually implemented.

## Scenario A — Stripe Payment Status Widget

**Goal:** A plugin that renders a "Connect Stripe" button when no Stripe account is linked, and a "Stripe Connected (acct_xxx)" status pill when one is. Must receive `stripe_account_id: Option<String>` from handler data; render either of two HTML shapes.

**Walk-through result:**

1. **Define the plugin struct and register it:** Covered — `impl JsonUiPlugin for StripeConnectStatus`, `register_plugin(StripeConnectStatus)`. Clear.
2. **Declare component type name:** Covered — `fn component_type(&self) -> &str { "StripeConnectStatus" }`. Clear.
3. **Declare props schema:** Covered — `fn props_schema(&self) -> serde_json::Value`. Example shown. Clear.
4. **Implement `render`:** **Gap found** — `render(props, data)` signature shows `_data` (underscore = unused) in the first example. An author implementing a Stripe widget needs `data` to read `stripe_account_id` from the handler payload. The parameter name `_data` signals "ignored" in Rust convention, which misdirects the author.

   **Fix (Plan 10, commit `63529b33`):** The `render` function example was updated to include:
   ```rust
   fn render(&self, props: &serde_json::Value, _data: &serde_json::Value) -> String {
       // `props` — the element's props object from the spec (already expression-resolved).
       // `data`  — the full spec data payload from the handler (`spec.data`).
       //           Use this to read per-request values not passed explicitly in props.
   ```
   After this fix, an author knows to use `data` to read per-request account state.

5. **Declare assets:** Covered — `css_assets()` and `js_assets()` with examples. Clear.
6. **`init_script()`:** See Scenario B.

**Gaps for Scenario A:** 1 (render `data` param documentation) — **fixed**.

---

## Scenario B — WhatsApp Connection Flow

**Goal:** A plugin that orchestrates a WhatsApp Business API connection: shows a "Link WhatsApp" button → opens a QR code modal → polls a status endpoint → flips to "Connected" state. Needs: assets injection (QR-rendering JS), client-side state, fetch polling.

**Walk-through result:**

1. **Define and register:** Same as Scenario A. Clear.
2. **Asset injection (QR-rendering JS library):** Covered — `js_assets()` returns `Vec<Asset>` with CDN URLs. Deduplication mentioned (two elements on same page load JS once). Clear.
3. **Implement `init_script()` for polling loop:** **Gap found** — the guide showed the `init_script()` method without explaining when it is emitted relative to `js_assets()` tags, and whether it is emitted once or once-per-plugin-instance. For a polling loop, multi-instance emission would create duplicate intervals.

   **Fix (Plan 10, commit `63529b33`):** Prose added after the `init_script()` code block:
   > `init_script()` is emitted once per page regardless of how many instances of the plugin appear in the spec. Use a `querySelectorAll` loop (as above) so the script initializes every instance. The script is injected inline before `</body>`, after all `js_assets()` `<script>` tags have been emitted.

   After this fix, an author knows the script runs once, injection order is guaranteed (assets first, then init), and the `querySelectorAll` pattern handles multi-instance.

4. **Same `render` data gap as Scenario A:** Fixed by same commit. Clear.
5. **Client-side state for QR → Connected transition:** This is a client-side architecture concern beyond the plugin guide's scope. The guide covers the server-side `render()` and asset injection. The author must implement the polling + DOM swap in `init_script()`. No additional framework primitive is needed — `init_script()` + `js_assets()` is sufficient.

**Gaps for Scenario B:** 2 (`render data` param + `init_script` per-page-once semantics) — **both fixed**.

---

## Scenario C — Chart Renderer

**Goal:** A plugin that renders a bar chart from `data_path`-resolved data. Needs: typed props shape; server-side rendering (SVG) AND/OR client-side library hook.

**Walk-through result:**

1. **Implement `props_schema()`:** The guide shows a full `Chart` example with `data_path`, `type`, and `height` props and a JSON Schema object. Clear.
2. **Read from `data_path`:** The author reads the path from `props["data_path"]` and resolves it against `data` in the `render()` method. After the D-06 fix, the `data` parameter is documented as `spec.data` from the handler — an author can use `data.pointer(path)` to resolve the array.
3. **Server-side SVG rendering:** `render()` returns a `String` — an SVG string is valid. The guide's `ChartPlugin` renders a `<canvas>` element (client-side); an author choosing SVG would return an SVG string directly. No gap.
4. **Asset injection (Chart.js):** Covered by the `ChartPlugin` example — `js_assets()` with a CDN URL, `init_script()` with `querySelectorAll` initialization. Clear.

**Gaps for Scenario C:** 0.

---

## Outcome

| Scenario | Gaps Found | Fixed | Escalated to BLOCKER |
|----------|------------|-------|----------------------|
| A — Stripe widget | 1 (render `data` param undocumented) | Yes — Plan 10 `63529b33` | No |
| B — WhatsApp flow | 2 (render `data` param + init_script timing) | Yes — Plan 10 `63529b33` | No |
| C — Chart renderer | 0 | — | No |

**Total gaps:** 2 distinct issues (render `data` param, `init_script` per-page-once semantics), both minor (1-2 doc sentences each), both fixed in `docs/src/json-ui/plugins.md` by Plan 10.

**Outcome: B — minor gaps, fixed inline. No missing load-bearing primitive. No BLOCKER rows added to V1-DELETION-AUDIT.md.**

## V1-DELETION-AUDIT Cross-Reference

The BLOCKER count in V1-DELETION-AUDIT.md remains **0** after this audit. The plugin surface audit does not surface any gap that would prevent Phase 160 (v1 deletion) from proceeding.
