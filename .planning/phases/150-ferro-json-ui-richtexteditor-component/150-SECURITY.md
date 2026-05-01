---
phase: 150
slug: ferro-json-ui-richtexteditor-component
status: verified
threats_open: 0
asvs_level: 1
created: 2026-05-01
---

# Phase 150 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Test code → production code | Test functions reference symbols that do not yet exist; compile failure is the design | Compilation-only; no runtime data |
| jsDelivr CDN bytes → ferro-json-ui consumers | Quill JS/CSS loaded from third-party CDN at runtime | Third-party library bytes |
| Handler data (`serde_json::Value`) → rendered HTML | `data_path` resolution flows untrusted handler strings into HTML attributes and text nodes | Untrusted string content |
| Caller-provided `props.value` → host body and hidden inputs | Author-controlled content (typically from DB) flows into `<div data-rte-host>` and hidden inputs | Author-controlled string |
| Caller-provided `props.formats: Vec<String>` → `data-rte-formats` attribute | Format names are author-controlled and rendered into a JSON-encoded HTML attribute | Author-controlled array of strings |
| Server-rendered HTML attributes (`data-rte-*`) → IIFE state | The IIFE reads `data-rte-name`, `data-rte-formats`, `data-rte-theme`, `data-rte-placeholder` from the wrapper | Already html_escape'd strings parsed by JSON.parse |
| User keystrokes / paste → Quill editor → `quill.root.innerHTML` | Quill clipboard module enforces formats allowlist at input time; IIFE post-processes as defense-in-depth | User-supplied rich text content |
| `quill.root.innerHTML` → `sanitizeHtmlByFormats` → `{name}_html` form value | DOM-walker sanitizer output flows into a hidden input for form submission | Sanitized HTML string |
| Form submit event → IIFE submit listener (capture phase) → form serialization | Capture-phase listener guarantees IIFE runs before user-installed bubble-phase listeners | Form field values |
| Public crate surface (`lib.rs` re-exports) → downstream consumers | `RichTextEditorProps` and `RichTextEditorPlugin` form the API contract | Public Rust types |
| `docs/src/json-ui/components.md` → docs.ferro-rs.dev | Documentation site is a public artifact | Reference documentation |
| `ferro-mcp` catalog → AI tooling | The catalog is the AI's mental model of available components | Component schema metadata |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-150-01 | Tampering | `render_rich_text_editor` escaping of `placeholder`, `value`, `label`, `error` into HTML | mitigate | `html_escape` applied to all dynamic attrs; test `render_rich_text_editor_html_escapes_dynamic_attrs` asserts `<script>` cannot survive unescaped | closed |
| T-150-02 | Tampering | `data-rte-formats` attribute (JSON-encoded array) emission | mitigate | `html_escape` applied to the JSON-encoded formats array; attribute boundary injection blocked | closed |
| T-150-03 | Spoofing | Quill CDN asset substitution | mitigate | SHA-384 SRI hashes pinned for quill.js and quill.snow.css; test `render_rich_text_editor_emits_quill_sri_assets_via_pipeline` asserts both assets carry `integrity="sha384-"` | closed |
| T-150-W2-01 | Spoofing | Quill JS/CSS bytes substituted by malicious CDN intermediary | mitigate | SHA-384 SRI hashes computed from live jsDelivr bytes; browsers refuse to execute on hash mismatch | closed |
| T-150-W2-02 | Tampering | Future SRI update without version bump | mitigate | Unit test `quill_urls_pin_to_2_0_3` asserts URL contains `@2.0.3/`; a version bump must update both URL and SRI | closed |
| T-150-W2-03 | Information Disclosure | jsDelivr CDN request leaks visit pattern | accept | Standard CDN posture; no first-party data in request — see Accepted Risks Log | closed |
| T-150-W3-01 | Tampering | Pre-fill of `value` / `data_path` content into editor host body | mitigate | `html_escape(initial_value)` applied before insertion into `<div data-rte-host>{...}</div>` | closed |
| T-150-W3-02 | Tampering | `placeholder`, `name`, `theme`, `label`, `error` emitted into HTML attribute values | mitigate | Every dynamic value passes through `html_escape` before `format!()` into the attribute | closed |
| T-150-W3-03 | Tampering | `formats: Vec<String>` JSON-encoded into `data-rte-formats` attribute | mitigate | Two-layer defense: `serde_json::to_string` JSON-escapes `"` inside format names; `html_escape` converts surrounding `"` to `&quot;` for attribute-context safety | closed |
| T-150-W3-04 | Spoofing | Quill JS/CSS bytes substituted by malicious CDN intermediary (render side) | mitigate | `RichTextEditorPlugin::css_assets` and `::js_assets` carry SHA-384 integrity hashes pinned at Plan 02; browser SRI verification refuses to load on hash mismatch | closed |
| T-150-W3-05 | Information Disclosure | Plugin's unreachable `render()` path producing a server-trusted HTML string | accept | `render()` returns a known-safe static string with no caller-provided content; no injection vector — see Accepted Risks Log | closed |
| T-150-W3-06 | DoS | Multiple `RichTextEditor` instances on one page each causing a Quill load | mitigate | `collect_plugin_assets` deduplicates by URL; N editor instances produce exactly one Quill JS and one CSS load | closed |
| T-150-W4-01 | Tampering | Disallowed HTML tags surviving into `{name}_html` via paste/devtools injection | mitigate | Two-layer defense: Quill `formats` option drops disallowed input at paste/keystroke time; `sanitizeHtmlByFormats` post-processes at submit time | closed |
| T-150-W4-02 | Tampering | `<script>` / `<style>` / `<iframe>` injection via paste | mitigate | All three in `alwaysStripped` — removed entirely via `removeChild` (not unwrapped), along with their text content | closed |
| T-150-W4-03 | Tampering | `onclick=` / `onerror=` / `style=` attributes injected via devtools | mitigate | `stripDisallowedAttributes` removes any attribute starting with `on`, equal to `style`, or `class` without `ql-` prefix | closed |
| T-150-W4-04 | DoS | Pathological deeply-nested HTML in paste causing DOMParser stack overflow | accept | Browser DOMParser enforces depth limits at engine level — see Accepted Risks Log | closed |
| T-150-W4-05 | Spoofing | `wrapper.querySelector('[data-rte-hidden="delta"]')` collision when two editors share a parent | mitigate | All selectors scoped to `wrapper` (per-instance `<div data-rich-text-editor>`), not `document`; no cross-instance collision possible | closed |
| T-150-W4-06 | Information Disclosure | IIFE reads `quill.getText()` at submit-required-check time | accept | Text computed locally; not transmitted unless form submits — same posture as every other client-side form value — see Accepted Risks Log | closed |
| T-150-W5-01 | Tampering | Documentation drift between `RichTextEditorProps` source and docs props table | mitigate | Docs table derived directly from D-03 struct; acceptance criteria include grep checks for every prop name | closed |
| T-150-W5-02 | Tampering | `ferro-mcp` catalog drift between `RichTextEditorProps` source and `CatalogComponent` entry | mitigate | `schemars`-derived `JsonSchema` on `RichTextEditorProps` is the AI tool's actual schema route via `RichTextEditorPlugin::props_schema`; CI asserts component count=42 and name list | closed |
| T-150-W5-03 | Information Disclosure | Docs page reveals internal SRI mechanism details | accept | SRI is a public web standard; documenting it is a feature — see Accepted Risks Log | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-150-01 | T-150-W2-03 | Standard CDN posture — no first-party data in the jsDelivr request, only library bytes. Trade-off accepted in exchange for the zero-build-step contract. | Alberto Ferrario | 2026-05-01 |
| AR-150-02 | T-150-W3-05 | Plugin's `render()` returns a known-safe static debug sentinel with no caller-provided content interpolated. Even if a regression routes through this path, the output is not an injection vector. | Alberto Ferrario | 2026-05-01 |
| AR-150-03 | T-150-W4-04 | Browser DOMParser already enforces depth limits at the engine level. No realistic adversarial path through legitimate paste handling that goes beyond browser-level limits. | Alberto Ferrario | 2026-05-01 |
| AR-150-04 | T-150-W4-06 | `quill.getText()` is computed locally in the browser; not transmitted unless the form submits. Same posture as every other client-side form value. | Alberto Ferrario | 2026-05-01 |
| AR-150-05 | T-150-W5-03 | SRI is a public web standard. Documenting the integrity mechanism in user-facing docs is a feature, not a disclosure risk. | Alberto Ferrario | 2026-05-01 |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-05-01 | 21 | 21 | 0 | gsd-security-auditor (automated) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-05-01
