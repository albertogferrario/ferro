# Phase 147: DetailForm component for inline edit — ferro-json-ui — Research

**Researched:** 2026-04-23
**Domain:** ferro-json-ui component system — Rust HTML rendering, server-driven mode toggle (no JS)
**Confidence:** HIGH

## Summary

Phase 147 adds a single new component (`DetailForm`) to `ferro-json-ui`, mechanically identical in shape to phase 146's `KeyValueEditor` addition but with one extra participant per concern: it holds an `Action` (like `Form`), which means it must show up in every `match` over `Component` that recurses into children or actions — most critically the three in `resolve.rs` (`resolve_component_node`, `collect_unresolved_node`, `resolve_errors_node`) plus the two in `render.rs` (`render_component` dispatch, `collect_plugin_types_node`). Missing any of these arms either silently breaks URL resolution or skips a category of introspection.

The component's distinguishing design property — that View and Edit render the same outer HTML scaffold — is mechanically small. `render_detail_form` emits a single `<dl>` using the classes lifted verbatim from `render_description_list` (render.rs:2427-2439). In Edit mode it additionally wraps the `<dl>` in a `<form>` built from the method-spoofing block lifted verbatim from `render_form` (render.rs:971-1011). The only non-trivial question is **what to render inside each `<dd>` in Edit mode** — specifically whether `render_input` (which unconditionally emits a `<label>` at render.rs:1408-1412) will create visual label duplication with the `<dt>`.

After reading the full `render_input` body, the recommended path is **Option A (pass empty label)**: the label line is a single format! that emits `<label>...>...</label>` regardless of whether `props.label` is an empty string. An empty string produces a well-formed but visually-empty `<label>` — no duplication, no new prop on `InputProps`, no scope expansion. This is the choice that preserves D-05 (same outer scaffold) and D-12 (full component surface available in Edit mode) without introducing a new orthogonal concept ("label-less input").

All other open questions resolve cleanly against existing patterns: the serde integration is 100% mechanical (two match arms in Serialize/Deserialize, one match arm in the enum), the `ComponentNode::detail_form` factory is a cut-and-paste from `ComponentNode::form` (component.rs:1245-1252), the COMPONENT_CATALOG entry is a single-paragraph addition, and the resolver arms mirror `Component::Form` verbatim with the recursion target changed from `props.fields` to `props.fields.iter_mut().map(|f| &mut f.input)`.

**Primary recommendation:** Follow the phase 146 four-file playbook exactly (component.rs struct + enum variant + serde arms + factory, render.rs function + dispatch arm, lib.rs re-exports + catalog entry, resolve.rs three arms) plus one addition — the **ferro-mcp `json_ui_catalog.rs` also needs a CatalogComponent entry and hardcoded list insertion**. KeyValueEditor is still missing from there as of phase 146; fixing this for DetailForm is one tiny edit on top of the canonical pattern and should not be deferred.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Core types:**
- **D-01** — `EditMode` enum lives in `ferro-json-ui`. Two variants: `View` (default), `Edit`. Derives: `Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema`. `#[serde(rename_all = "snake_case")]`. Default is `View`.
- **D-02** — `EditMode::from_query(raw: Option<&str>) -> Self` — returns `Edit` when `raw` equals `"edit"` case-insensitively, `View` otherwise.
- **D-03** — `DetailField { label: String, value: String, input: ComponentNode }`. Derives: `Debug, Clone, PartialEq, Serialize, Deserialize` (no JsonSchema — contains `ComponentNode`).
- **D-04** — `DetailFormProps { mode, action, fields, edit_url, cancel_url, edit_label: Option<String>, save_label: Option<String>, cancel_label: Option<String>, method: Option<HttpMethod> }`. Derives: `Debug, Clone, PartialEq, Serialize, Deserialize`. No JsonSchema.

**Structural coherence guarantee:**
- **D-05** — View and Edit render the SAME outer HTML scaffold: `<dl class="grid grid-cols-1 gap-4 …">` with per-row `<div><dt>…</dt><dd>…</dd></div>`. Only `<dd>` contents differ: View = escaped `field.value`, Edit = `render_node(&field.input, data)`.
- **D-06** — Edit mode wraps the `<dl>` in a `<form>` with the same attrs as `render_form` (action, method, spoofing). View mode has no `<form>`.
- **D-07** — Action bar renders OUTSIDE `<dl>` but INSIDE the form element (Edit) or parent wrapper (View). `flex gap-2`.

**View-mode rendering:**
- **D-08** — `<dl>` classes: `grid grid-cols-1 gap-4`. `<dt>`: `text-sm font-medium text-text-muted`. `<dd>`: `mt-1 text-sm text-text`. Mirrors `render_description_list`.
- **D-09** — "Modifica" renders as an `<a>` link (not `<button>`). Outline/secondary styling via `render_button`-style inline classes.
- **D-10** — `edit_url` emitted verbatim (after `html_escape`) as `href`. No resolver.

**Edit-mode rendering:**
- **D-11** — `<form>` uses identical attrs + spoofing as `render_form`:
  - `action` from `props.action.url` (resolver populates); `"#"` fallback
  - `method`: GET for Get, POST for everything else
  - Hidden `_method` input for PUT/PATCH/DELETE
- **D-12** — Each `DetailField.input` rendered via `render_node(&field.input, data)`. Full component surface available.
- **D-13** — Input pre-fill is the caller's job (each input's own `default_value` / `data_path`). `DetailField.value` is View-mode display only.
- **D-14** — "Salva" = `<button type="submit">`, primary. "Annulla" = `<a>` outline, targeting `cancel_url`.

**Action resolution:**
- **D-15** — `Component::DetailForm(props)` participates in resolver like `Component::Form`. Resolver populates `props.action.url` from `props.action.handler`.
- **D-16** — `edit_url` / `cancel_url` are NOT resolved — raw hrefs.

**Serde integration:**
- **D-17** — `Component::DetailForm(DetailFormProps)` variant on the enum. `serialize_tagged(serializer, "DetailForm", p)`. Deserialize match arm for `"DetailForm"`.
- **D-18** — `ComponentNode::detail_form(key, props)` factory.
- **D-19** — `COMPONENT_CATALOG` entry: name `"DetailForm"`, description citing "split-mode detail page with inline edit".

**No runtime JS:**
- **D-20** — No entry in `ferro-json-ui/src/runtime/`. Server-side mode toggle only.

### Claude's Discretion

- Exact Tailwind class lists for buttons and action-bar layout — reuse `render_form` / `render_button` idioms.
- Whether to emit `<section>` or plain `<div>` wrapper around the component.
- "Modifica" link position: default below the `<dl>`, right-aligned.
- Rust doc comments on public types: follow `InputProps` / `FormProps` doc style.
- Tests: follow existing `render_*` patterns in `render.rs` — HTML-substring asserts, one test per mode minimum, plus serde round-trip and `EditMode::from_query`.
- Whether `DetailField` ships a `DetailField::new(label, value, input)` convenience constructor — yes, to match `ComponentNode::input(...)` ergonomics.

### Deferred Ideas (OUT OF SCOPE)

- i18n binding of default button labels via `ferro-lang`
- Handler-based resolution for `edit_url` / `cancel_url`
- Per-field mode override (e.g. read-only in Edit)
- Conditional mode toggle visibility (e.g. `can_edit` flag)
- Nested sections / groups (multi-section DetailForm)
- Form guards (`FormProps.guard` on DetailFormProps)
- Gestiscilo Phase 111 migration (happens downstream in the gestiscilo repo after this phase ships)

</user_constraints>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Props definitions + serde | Rust library (`ferro-json-ui/src/component.rs`) | — | All component schemas live in this file |
| `Component` enum variant + serialize/deserialize arms | Rust library (`component.rs`) | — | Tagged-enum dispatch lives with the enum |
| `ComponentNode::detail_form(...)` factory | Rust library (`component.rs`) | — | Parallel placement to `ComponentNode::form`, `ComponentNode::description_list` |
| HTML rendering (`render_detail_form`) | Rust library (`ferro-json-ui/src/render.rs`) | — | Server-side HTML generation, dispatched from `render_component` |
| URL resolution (`Component::Form`-style arms) | Rust library (`ferro-json-ui/src/resolve.rs`) | — | Resolver walks the component tree to populate `action.url` and error strings |
| Plugin-type collection walk | Rust library (`ferro-json-ui/src/render.rs`) | — | `collect_plugin_types_node` recurses into `DetailField.input` |
| `EditMode::from_query(Option<&str>)` | Rust library (`component.rs`) | Caller (handler) | Enum lives in ferro-json-ui; handler invokes it on `req.query("mode").as_deref()` — request lives in `framework/src/http/request.rs:154` |
| Public API export | Rust library (`ferro-json-ui/src/lib.rs`) | — | User-facing types re-exported from crate root |
| MCP catalog entry | `ferro-mcp/src/tools/json_ui_catalog.rs` | — | Agents introspect catalog via MCP; missing entries fail introspection tests |
| User documentation | `docs/src/json-ui/components.md` | — | Every new component gets a section matching the existing template |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.0 | Props serialization/deserialization | Used by every component in the crate [VERIFIED: ferro-json-ui/Cargo.toml in workspace] |
| serde_json | 1.0 | JSON value resolution, round-trip tests | Already a direct dependency |
| schemars | 1.x | `JsonSchema` derive on `EditMode` | Already used by `HttpMethod`, `DialogVariant`, `ButtonVariant`, etc. |

**No new dependencies needed.** `DetailFormProps` and `DetailField` deliberately skip `JsonSchema` (D-03, D-04) — this matches the existing `FormProps` / `Tab` / `TabsProps` precedent (component.rs:188, 443, 454).

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Shared `<dl>` emission with `render_description_list` | Lift structure; don't call through | D-05 requires per-`<dd>` content divergence; calling through would force constructing a fake `DescriptionListProps`. Duplication is ~6 lines. |
| Label duplication resolved via new `hide_label: Option<bool>` on `InputProps` | Pass empty string as `label` at call site | New prop adds a second orthogonal concept to `InputProps` and cascades into `render_select`, `render_textarea`, etc. Empty-string path is zero-scope (Option A below). |
| `EditMode` in a separate `edit_mode.rs` module | Keep in `component.rs` near the props that use it | Small enum; module overhead not justified. Matches `FormMaxWidth` precedent (component.rs:180-185). |

---

## Architecture Patterns

### System Architecture Diagram

```
URL:  /resource/{id}?mode=edit
              │
              ▼
Handler:  let mode = EditMode::from_query(req.query("mode").as_deref());
          let fields = build_fields(&record);
          render(ComponentNode::detail_form("detail", DetailFormProps {
              mode, action, fields, edit_url, cancel_url, …
          }))
              │
              ▼
resolve_actions(view, resolver)
   └── resolve_component_node walks into Component::DetailForm
         └── resolve_action(&mut props.action, resolver)   // handler -> url
         └── for each field: resolve_component_node(&mut field.input, …)
              │
              ▼
render_to_html(view, data)
   └── render_component(component, data)
         └── match Component::DetailForm(props) => render_detail_form(props, data)
                │
                ▼
         switch mode:
         ┌─ View ─────────────────────────────┐  ┌─ Edit ────────────────────────────┐
         │  <div>                             │  │  <form action="…" method="…">     │
         │    <dl grid-cols-1 gap-4>          │  │    [hidden _method spoofing]       │
         │      per field:                    │  │    <dl grid-cols-1 gap-4>          │
         │        <div>                       │  │      per field:                    │
         │          <dt>{html_escape(label)}  │  │        <div>                       │
         │          <dd>{html_escape(value)}  │  │          <dt>{html_escape(label)}  │
         │        </div>                      │  │          <dd>{render_node(input)}  │
         │    </dl>                           │  │        </div>                      │
         │    <div flex gap-2 justify-end>    │  │    </dl>                           │
         │      <a href=edit_url>Modifica</a> │  │    <div flex gap-2 justify-end>    │
         │    </div>                          │  │      <a href=cancel_url>Annulla</a>│
         │  </div>                            │  │      <button type=submit>Salva</b> │
         │                                    │  │    </div>                          │
         │                                    │  │  </form>                           │
         └────────────────────────────────────┘  └────────────────────────────────────┘
```

### Recommended Project Structure

Files modified:

```
ferro-json-ui/src/
├── component.rs     # +EditMode, +DetailField, +DetailFormProps, +Component::DetailForm,
│                    #  +Serialize arm, +Deserialize arm, +ComponentNode::detail_form factory,
│                    #  +serde round-trip test, +EditMode::from_query test
├── render.rs        # +fn render_detail_form, +dispatch arm, +add DetailForm to leaf list in
│                    #  collect_plugin_types_node, +render_detail_form tests (view/edit modes)
├── lib.rs           # +DetailFormProps, DetailField, EditMode in component re-export block,
│                    #  +DetailForm entry in COMPONENT_CATALOG
└── resolve.rs       # +arm in resolve_component_node (action + children walk),
                     # +arm in collect_unresolved_node (action + children walk),
                     # +arm in resolve_errors_node (children walk)

ferro-mcp/src/tools/
└── json_ui_catalog.rs   # +CatalogComponent entry for DetailForm,
                         # +"DetailForm" in the hardcoded name list around L1115

docs/src/json-ui/
└── components.md    # +## DetailForm section (Rust + JSON example, prop table)
```

### Pattern 1: Tagged-enum serialization (mechanical)

**What:** Custom `Serialize`/`Deserialize` on `Component` that injects `{"type": "<Variant>"}` into the JSON object.

**Source:** `ferro-json-ui/src/component.rs:997-1010` (helper), `:1015-1058` (Serialize arms), `:1072-1201` (Deserialize arms).

**For DetailForm:**
```rust
// Serialize arm (insert after Component::KeyValueEditor at L1056)
Component::DetailForm(p) => serialize_tagged(serializer, "DetailForm", p),

// Deserialize arm (insert before the _ => Plugin catch-all at L1193)
"DetailForm" => serde_json::from_value::<DetailFormProps>(value)
    .map(Component::DetailForm)
    .map_err(de::Error::custom),
```

Ordering is not alphabetical — existing arms are grouped loosely by component family. The safe placement for DetailForm is immediately after `KeyValueEditor` (same family — "form-shaped things that hold an Action") in both blocks.

### Pattern 2: `ComponentNode::<factory>` constructor

**Source:** `ferro-json-ui/src/component.rs:1244-1252` (`ComponentNode::form`).

**For DetailForm:**
```rust
/// Create a DetailForm component node.
///
/// Renders the same outer scaffold in both `View` and `Edit` modes;
/// Edit mode additionally wraps the scaffold in a submittable form.
pub fn detail_form(key: impl Into<String>, props: DetailFormProps) -> Self {
    Self {
        key: key.into(),
        component: Component::DetailForm(props),
        action: None,
        visibility: None,
    }
}
```

Insert near `ComponentNode::form` (component.rs:1244-1252) — grouping by family. Note: phase 146 did **not** add a `ComponentNode::key_value_editor` factory. D-18 explicitly requires this factory for DetailForm, so the inconsistency doesn't carry over. [VERIFIED: grep of `pub fn` in component.rs]

### Pattern 3: Method spoofing (lifted verbatim from `render_form`)

**Source:** `ferro-json-ui/src/render.rs:971-1011`.

**Contract:** For the effective HTTP method:
- `Get` → form `method="get"`, no hidden input
- `Post` → form `method="post"`, no hidden input
- `Put` / `Patch` / `Delete` → form `method="post"`, plus `<input type="hidden" name="_method" value="PUT|PATCH|DELETE">`

**Effective method resolution:** `props.method.as_ref().unwrap_or(&props.action.method)` — i.e. `DetailFormProps.method` override wins, else `props.action.method`.

**Copy-pasteable fragment** for `render_detail_form` Edit mode (identical semantics to `render_form`):

```rust
let effective_method = props
    .method
    .as_ref()
    .unwrap_or(&props.action.method)
    .clone();

let (form_method, needs_spoofing) = match effective_method {
    HttpMethod::Get => ("get", false),
    HttpMethod::Post => ("post", false),
    HttpMethod::Put | HttpMethod::Patch | HttpMethod::Delete => ("post", true),
};

let action_url = props.action.url.as_deref().unwrap_or("#");
let mut html = format!(
    "<form action=\"{}\" method=\"{}\" class=\"…\">",
    html_escape(action_url),
    form_method
);

if needs_spoofing {
    let method_value = match effective_method {
        HttpMethod::Put => "PUT",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Delete => "DELETE",
        _ => unreachable!(),
    };
    html.push_str(&format!(
        "<input type=\"hidden\" name=\"_method\" value=\"{method_value}\">"
    ));
}
```

DetailForm's form classes can differ from `render_form`'s (which is optimized for horizontal-wrap layouts). For DetailForm the form is a plain wrapper — suggested: `class="space-y-4"` (or `class=""` since the `<dl>` inside controls the grid).

### Pattern 4: Description-list scaffold (lifted from `render_description_list`)

**Source:** `ferro-json-ui/src/render.rs:2427-2439`.

**Contract:**
```rust
// Outer:    <dl class="grid grid-cols-{N} gap-4">
// Per row:  <div>
//             <dt class="text-sm font-medium text-text-muted">{html_escape(label)}</dt>
//             <dd class="mt-1 text-sm text-text">{html_escape(value)}</dd>
//           </div>
// Close:    </dl>
```

For DetailForm: D-05 fixes the outer container to `grid-cols-1` (single column). No `columns` prop on `DetailFormProps`. View-mode `<dd>` keeps `html_escape(value)`; Edit-mode `<dd>` substitutes `render_node(&field.input, data)`.

### Pattern 5: Resolver participation — THREE arms required

**Sources:**
- `ferro-json-ui/src/resolve.rs:30-155` (`resolve_component_node`) — URL resolution pass
- `ferro-json-ui/src/resolve.rs:205-328` (`collect_unresolved_node`) — strict-mode error collection
- `ferro-json-ui/src/resolve.rs:377-476` (`resolve_errors_node`) — validation error mapping

**For DetailForm,** each of the three needs a matching arm. The parallel for `Component::Form` (resolve.rs:46-51 / :219-224 / :399-403):

```rust
// resolve_component_node (L46-51 mirror):
Component::DetailForm(props) => {
    resolve_action(&mut props.action, resolver);
    for field in &mut props.fields {
        resolve_component_node(&mut field.input, resolver);
    }
}

// collect_unresolved_node (L219-224 mirror):
Component::DetailForm(props) => {
    collect_unresolved_action(&props.action, unresolved);
    for field in &props.fields {
        collect_unresolved_node(&field.input, unresolved);
    }
}

// resolve_errors_node (L399-403 mirror):
Component::DetailForm(props) => {
    for field in &mut props.fields {
        resolve_errors_node(&mut field.input, errors, all);
    }
}
```

**Gotcha:** the resolver recurses over `ComponentNode` (not `Component`). `DetailField.input` IS a `ComponentNode` — so the recursion is straightforward: iterate `&mut props.fields` and call `resolve_component_node(&mut field.input, …)` on each `.input`.

### Pattern 6: Plugin-type collection walk

**Source:** `ferro-json-ui/src/render.rs:101-197` (`collect_plugin_types_node`).

Current state: this function has a `match` that recurses into every container's children. Form (render.rs:114-118) recurses into `props.fields`. For DetailForm the equivalent is recurse into `props.fields.iter().map(|f| &f.input)`.

```rust
// Insert alongside Component::Form at L114:
Component::DetailForm(props) => {
    for field in &props.fields {
        collect_plugin_types_node(&field.input, types);
    }
}
```

Also must **remove** `Component::DetailForm(_)` from the leaf-components catch-all at render.rs:160-189 — DetailForm is NOT a leaf.

### Pattern 7: Dispatch arm in `render_component`

**Source:** `ferro-json-ui/src/render.rs:288-340` (dispatch match).

```rust
// Insert near Form's line 305:
Component::Form(props) => render_form(props, data),
Component::DetailForm(props) => render_detail_form(props, data),  // NEW
Component::Modal(props) => render_modal(props, data),
```

### Anti-Patterns to Avoid

- **Do not** call `render_description_list` from inside `render_detail_form` to share the scaffold. D-05 requires per-`<dd>` content divergence; forcing a shared call path either requires building a fake `DescriptionListProps` or threading a callback. Lifting the HTML structure directly is ~6 lines.
- **Do not** skip a resolver arm by putting DetailForm in the leaf-components catch-all. The codebase has precedent: `resolve.rs` uses explicit enumeration of leafs so that a missing arm is a compile error. DetailForm is not a leaf — it holds an `Action` AND children (`DetailField.input`).
- **Do not** add a `hide_label: Option<bool>` prop to `InputProps` (or `SelectProps` / `TextareaProps`) to solve label duplication. This leaks DetailForm's UX concern into every form-field component. The empty-string path solves it at the call site without any prop surface change.
- **Do not** allow `render_detail_form` to call itself recursively through `render_node` inside Edit mode's `<dd>`. If the caller passes `DetailField.input = ComponentNode::detail_form(…)`, the inner DetailForm would render *inside* a `<dd>`, which is valid HTML but semantically nonsensical. This phase does not forbid it, but should not demonstrate it in docs/tests.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTML escape of labels, values, URLs | Custom escape function | Existing `html_escape(&str) -> String` in `render.rs` | Already `pub(crate)`, already handles `<`, `>`, `"`, `&`, `'` |
| JSON round-trip tests for the new variant | Hand-rolled JSON assertion comparing full JSON text | `serde_json::to_value` + `from_value` round-trip + `PartialEq` equality, per `key_value_editor_serde_roundtrip` (component.rs:3624-3673) | Exact pattern established phase 146; reads cleanly |
| Method-spoofing form HTML | Custom `<form>` emission logic | Copy the 13-line block from `render_form` (render.rs:972-1011) | Same contract; divergence is a future refactor opportunity |
| `<dl>/<dt>/<dd>` markup | Free-form divs | Lift structure from `render_description_list` (render.rs:2427-2439) | Preserves semantic HTML; matches the design system |
| Button-link rendering | Full `render_button` call with a fake `ButtonProps` | Inline anchor/button emission with copied class strings | `render_button` emits `<button>` not `<a>`; needs different tag for Modifica/Annulla |
| Query-param boolean parsing for `EditMode::from_query` | String-level equality | `raw.map(|s| s.eq_ignore_ascii_case("edit")).unwrap_or(false)` | One line; avoids `String::to_lowercase` allocation |

**Key insight:** Every "hand-roll" candidate in this phase has an existing pattern in the crate already. The phase is all copy-pattern, no novel abstraction.

---

## Button Variant Class Strings (from `render_button`)

Lifted verbatim from `ferro-json-ui/src/render.rs:2029-2117` so `render_detail_form` can emit anchors and submit-buttons with design-system-consistent styling without calling through.

**Shared base (always):**
```
inline-flex items-center justify-center rounded-md font-medium
transition-colors duration-150 motion-reduce:transition-none
focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2
```

**Variant classes:**
| Variant | Class string |
|---------|--------------|
| Default (primary) — "Salva" | `bg-primary text-primary-foreground hover:bg-primary/90` |
| Outline — "Modifica" and "Annulla" | `border border-border bg-background text-text hover:bg-surface` |

**Size — Default:**
```
px-4 py-2 text-sm
```

**Suggested concrete emissions:**
```rust
// "Modifica" (View mode, anchor, outline)
format!(
    "<a href=\"{}\" class=\"inline-flex items-center justify-center rounded-md font-medium \
     transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none \
     focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 \
     border border-border bg-background text-text hover:bg-surface px-4 py-2 text-sm\">{}</a>",
    html_escape(edit_url),
    html_escape(edit_label)
);

// "Salva" (Edit mode, submit button, primary)
format!(
    "<button type=\"submit\" class=\"… bg-primary text-primary-foreground hover:bg-primary/90 \
     px-4 py-2 text-sm\">{}</button>",
    html_escape(save_label)
);

// "Annulla" (Edit mode, anchor, outline — same shape as Modifica with cancel_url / cancel_label)
```

**Action bar wrapper:** `<div class="flex gap-2 justify-end">…</div>` (justify-end to match edit-mode Salva/Annulla placement per Claude's Discretion default).

---

## Label Duplication — The One Hard Question

**Question 6 restated:** In Edit mode, `<dd>` contains `render_node(&field.input, data)`. If `field.input` is `Component::Input(InputProps)`, `render_input` unconditionally emits `<label>{html_escape(&props.label)}</label>` (render.rs:1408-1412). That creates visual label duplication with the `<dt>`.

**Evidence:**

```rust
// ferro-json-ui/src/render.rs:1407-1412
let mut html = String::from("<div class=\"space-y-1\">");
html.push_str(&format!(
    "<label class=\"block text-sm font-medium text-text\" for=\"{}\">{}</label>",
    html_escape(&props.field),
    html_escape(&props.label)
));
```

The `<label>` is always emitted. There is one exception: `InputType::Hidden` (L1385-1393) takes an early return that skips the wrapper and the label entirely. But for visible inputs — text, email, number, textarea, etc. — the label is hard-wired.

The same applies to `render_select`, `render_textarea` (which share the `render_input` code path), and likely `render_checkbox` / `render_switch` (not audited in full but known to emit labels).

**Options:**

| Option | Description | Cost | Recommendation |
|--------|-------------|------|----------------|
| **A** | Caller passes `label: ""` on each `InputProps`. Empty string produces a well-formed but empty `<label>` element: zero visible content, preserves `for=` association with the input for screen readers. | Zero Rust scope change. Documented as a caller convention. | **RECOMMENDED** |
| B | Add `hide_label: Option<bool>` prop to `InputProps`, `SelectProps`, `TextareaProps`, `CheckboxProps`, `SwitchProps`. `render_input` / `render_select` etc. check this flag and skip the `<label>` block. | ~5 props across 5 structs, 5 match-arm edits, leaks DetailForm's concern into every form component. |  |
| C | `render_detail_form` renders inputs inline (no `render_node` call) — emits bare `<input>` / `<select>` without going through the component surface. | Loses D-12 (full component surface in Edit mode); DetailForm becomes coupled to specific input types; breaks plugin components inside `<dd>`. |  |
| D | Accept the duplication (don't fix it). | UX debt; inconsistent visual output across otherwise-identical structural modes; violates "beauty as a design criterion" principle. |  |

**Recommendation: Option A.**

Why:
- **Zero scope creep.** No new prop, no new struct field, no breaking change to `InputProps`.
- **Preserves D-12.** `render_node` still dispatches over the full component surface — plugin components work in Edit mode without any special case.
- **Accessibility preserved.** The empty `<label for="field">` still associates with its input for screen readers; the `<dt>` provides the visible label.
- **Matches idiom.** Rust string defaults are empty string; serde deserializes missing string fields to `""` if annotated. Callers who omit `label` get the right behavior by default when the struct is instantiated with empty string.
- **Documentable.** A one-paragraph note in `docs/src/json-ui/components.md`: *"When an Input is used inside a DetailForm `DetailField.input`, pass `label: "".to_string()` — the `<dt>` provides the visible label."*

Option A does leave an empty `<label class="block text-sm font-medium text-text" for="field"></label>` element in the rendered HTML. That's ~40 bytes of DOM noise per field in Edit mode; it is not visible to users, not announced by screen readers (empty label content), and it does not change layout (empty `<label>` collapses). This is acceptable DOM noise.

**Caveat to document:** If a future phase adds `hide_label: Option<bool>` to `InputProps` for another reason (e.g. inline filter widgets), DetailForm's convention upgrades to `label: "".to_string(), hide_label: Some(true)` for full cleanliness. That future phase is not blocked by this one.

---

## `EditMode::from_query` — Implementation

Recommended minimal implementation:

```rust
/// Which display mode the component uses.
///
/// Set from a URL query parameter — typically `?mode=edit` — by
/// [`EditMode::from_query`]. Defaults to [`EditMode::View`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EditMode {
    /// Read-only display with a "Modifica" action.
    #[default]
    View,
    /// Inline-edit form with "Salva" / "Annulla" actions.
    Edit,
}

impl EditMode {
    /// Parse a URL query parameter value into an `EditMode`.
    ///
    /// Returns [`EditMode::Edit`] when `raw` equals `"edit"`
    /// (ASCII case-insensitive); [`EditMode::View`] otherwise, including
    /// when `raw` is `None` or any other string.
    ///
    /// Handlers typically call this with `req.query("mode").as_deref()`.
    pub fn from_query(raw: Option<&str>) -> Self {
        match raw {
            Some(s) if s.eq_ignore_ascii_case("edit") => EditMode::Edit,
            _ => EditMode::View,
        }
    }
}
```

`#[serde(rename_all = "snake_case")]` + variant names `View` / `Edit` serialize to `"view"` / `"edit"` — matches the query-param string so the same vocabulary is used in URLs and JSON.

**Default test coverage:**
- `EditMode::from_query(Some("edit")) == Edit`
- `EditMode::from_query(Some("EDIT")) == Edit` (case-insensitive per D-02)
- `EditMode::from_query(Some("Edit")) == Edit`
- `EditMode::from_query(None) == View`
- `EditMode::from_query(Some("view")) == View`
- `EditMode::from_query(Some("")) == View`
- `EditMode::from_query(Some("anything-else")) == View`
- `EditMode::default() == View`
- Serde round-trip: `EditMode::Edit <-> "edit"`

---

## Common Pitfalls

### Pitfall 1: Forgetting the third resolver arm

**What goes wrong:** Plan covers `resolve_component_node` but misses `collect_unresolved_node` or `resolve_errors_node`. Unit tests pass (URL resolution works); validation errors silently don't propagate into fields inside DetailForm, or strict-mode never reports unresolved handlers inside DetailForm.
**Root cause:** resolve.rs has three match-over-Component blocks, each with its own leaf-list catch-all. Adding a new container requires an arm in all three.
**Prevention:** Pattern 5 above spells out all three arm insertions. Plan should list them as three separate verifications, not one.
**Warning signs:** Validation error on an input inside DetailForm doesn't render, even though the same input works standalone. `resolve_actions_strict` returns Ok for a DetailForm with an unresolvable handler.

### Pitfall 2: `collect_plugin_types_node` leaf-list drift

**What goes wrong:** Adding a plugin-typed component (`Component::Plugin(...)`) inside a `DetailField.input` produces a rendered HTML that's missing the plugin's `<link>` / `<script>` tags. Only when someone notices the Map-inside-DetailForm case in gestiscilo migration.
**Root cause:** `collect_plugin_types_node` (render.rs:101-197) has its own container-vs-leaf enumeration independent of resolve.rs.
**Prevention:** Add a Container arm for `Component::DetailForm` in `collect_plugin_types_node`, and verify `Component::DetailForm(_)` does NOT appear in the leaf-list at L160-189. Same shape as Form (L114-118).
**Warning signs:** A plugin component placed inside a DetailField.input renders as raw HTML with no styling/interactivity.

### Pitfall 3: `DetailFormProps.method` override not honored

**What goes wrong:** Handler passes `method: Some(HttpMethod::Put)` on the props but renders `method="post"` without spoofing (because effective method reads `props.action.method` which defaults to Post).
**Root cause:** `render_form` uses `props.method.as_ref().unwrap_or(&props.action.method)` — the override is checked first. Easy to invert or drop.
**Prevention:** Copy the exact line from `render_form` (render.rs:973-977). Test with `method: Some(HttpMethod::Put)` + `action.method: HttpMethod::Post`, assert `name="_method" value="PUT"` appears.
**Warning signs:** PUT/PATCH/DELETE forms silently POST without spoofing the method.

### Pitfall 4: `EditMode::from_query` ASCII-case-insensitivity

**What goes wrong:** Use `s.to_lowercase() == "edit"` which allocates for every call and is locale-sensitive (Turkish `I` → `ı`).
**Root cause:** `to_lowercase` is Unicode-aware, allocates, and handles edge cases this code doesn't need.
**Prevention:** Use `s.eq_ignore_ascii_case("edit")`. Zero allocation, ASCII-only (which is fine — the query param is a known literal).
**Warning signs:** Unnecessary allocations per request; clippy might flag with `manual_string_to_lowercase` or similar.

### Pitfall 5: Escaping URLs inside `href` vs. attribute values

**What goes wrong:** `edit_url` / `cancel_url` contain characters the browser interprets as attribute delimiters (`"`, space). Without escaping, a malicious URL `/foo" onmouseover="alert(1)` breaks out of the attribute.
**Root cause:** `render_detail_form` emits URLs into `href` attributes. Must escape even though URLs "look safe".
**Prevention:** `html_escape(edit_url)` and `html_escape(cancel_url)` at every emission site. Same discipline as `render_form` applies to its `action` URL (render.rs:989, 995).
**Warning signs:** XSS test with a `"` character in the URL smuggles out of the attribute.

### Pitfall 6: Missing ferro-mcp catalog entry

**What goes wrong:** Component renders fine at runtime but doesn't appear in `json_ui_catalog` MCP tool output. Agents can't discover it. Silent failure — no compile error.
**Root cause:** `ferro-mcp/src/tools/json_ui_catalog.rs` has a HARDCODED list of component names at L1115 (the exhaustive assertion test) plus a hardcoded array of `CatalogComponent` entries. KeyValueEditor is STILL missing from both as of phase 147 start [VERIFIED: grep in json_ui_catalog.rs].
**Prevention:** Include ferro-mcp catalog updates in the plan. Backfill KeyValueEditor while we're there (one-line cleanup) — or explicitly defer and note.
**Warning signs:** `mcp__ferro__json_ui_catalog` returns a list without `"DetailForm"`; agents hallucinate a different component shape.

### Pitfall 7: Documentation drift

**What goes wrong:** `docs/src/json-ui/components.md` doesn't have a `DetailForm` section. The MCP catalog advertises it but the human-facing docs don't explain when to use it vs. DescriptionList+Form.
**Root cause:** CLAUDE.md rule: *"docs/src/ must reflect current features"*. Easy to skip in a component-add PR.
**Prevention:** Include a new section in components.md matching the DescriptionList section template (L415-475). Minimum: Rust usage example, JSON example, prop table, "when to use" paragraph.

---

## Code Examples

Verified against the current codebase (2026-04-23).

### Example 1: `EditMode::from_query` usage in handler

```rust
use ferro::Request;
use ferro_json_ui::EditMode;

pub async fn show(req: Request, user: User) -> Response {
    // Parse mode from ?mode=edit; default View when absent or any other value.
    let mode = EditMode::from_query(req.query("mode").as_deref());
    // … build fields, action, URLs … then render a view containing a DetailForm.
}
```

### Example 2: Building a DetailForm component tree (Rust)

```rust
use ferro_json_ui::{
    Action, ComponentNode, DetailField, DetailFormProps, EditMode, InputProps, InputType,
};

fn build_detail_form(user: &UserRecord, mode: EditMode) -> ComponentNode {
    ComponentNode::detail_form(
        "user-detail",
        DetailFormProps {
            mode,
            action: Action::new("users.update").method(HttpMethod::Patch),
            fields: vec![
                DetailField::new(
                    "Nome",
                    user.name.clone(),
                    ComponentNode::input(
                        "name",
                        InputProps {
                            field: "name".to_string(),
                            // Empty label — <dt> provides the visible label.
                            label: String::new(),
                            input_type: InputType::Text,
                            default_value: Some(user.name.clone()),
                            // …other fields None / default…
                        },
                    ),
                ),
                DetailField::new(
                    "Email",
                    user.email.clone(),
                    ComponentNode::input(
                        "email",
                        InputProps {
                            field: "email".to_string(),
                            label: String::new(),
                            input_type: InputType::Email,
                            default_value: Some(user.email.clone()),
                        },
                    ),
                ),
            ],
            edit_url: format!("/users/{}?mode=edit", user.id),
            cancel_url: format!("/users/{}", user.id),
            edit_label: None,    // renders "Modifica"
            save_label: None,    // renders "Salva"
            cancel_label: None,  // renders "Annulla"
            method: None,        // use action.method (Patch)
        },
    )
}
```

### Example 3: Serialized JSON (round-trip contract)

```json
{
  "type": "DetailForm",
  "key": "user-detail",
  "mode": "view",
  "action": { "handler": "users.update", "method": "PATCH" },
  "fields": [
    {
      "label": "Nome",
      "value": "Mario Rossi",
      "input": {
        "type": "Input",
        "key": "name",
        "field": "name",
        "label": "",
        "input_type": "text",
        "default_value": "Mario Rossi"
      }
    }
  ],
  "edit_url": "/users/42?mode=edit",
  "cancel_url": "/users/42"
}
```

Note: `mode` serializes to `"view"` or `"edit"` (snake_case). Optional props omitted via `#[serde(skip_serializing_if = "Option::is_none")]`.

### Example 4: Rendered HTML — View mode (pseudocode assertions)

Expected substrings (in order):
```
<dl class="grid grid-cols-1 gap-4">
<dt class="text-sm font-medium text-text-muted">Nome</dt>
<dd class="mt-1 text-sm text-text">Mario Rossi</dd>
</dl>
<div class="flex gap-2 justify-end">
<a href="/users/42?mode=edit"
```

Absent from View mode:
- `<form`
- `<input type="hidden" name="_method"`
- `Salva`
- `Annulla`

### Example 5: Rendered HTML — Edit mode (pseudocode assertions)

Expected substrings:
```
<form action="/users/42" method="post"
<input type="hidden" name="_method" value="PATCH">
<dl class="grid grid-cols-1 gap-4">
<dt class="text-sm font-medium text-text-muted">Nome</dt>
<dd class="mt-1 text-sm text-text"><div class="space-y-1">
<input type="text" id="name" name="name"
value="Mario Rossi"
</dd>
</dl>
<div class="flex gap-2 justify-end">
<a href="/users/42"
<button type="submit"
Salva
</form>
```

Absent from Edit mode:
- `Modifica`
- A Modifica anchor with `?mode=edit`

---

## Test Patterns (from phase 146 precedent)

### Serde round-trip test — template

**Source:** `ferro-json-ui/src/component.rs:3620-3673` (`key_value_editor_serde_roundtrip`).

```rust
#[cfg(test)]
mod detail_form_tests {
    use super::*;

    #[test]
    fn detail_form_serde_roundtrip() {
        let original = Component::DetailForm(DetailFormProps {
            mode: EditMode::Edit,
            action: Action { handler: "users.update".into(), url: None, method: HttpMethod::Patch, … },
            fields: vec![DetailField {
                label: "Nome".into(),
                value: "Mario".into(),
                input: ComponentNode::input("name", InputProps { field: "name".into(), label: "".into(), … }),
            }],
            edit_url: "/users/42?mode=edit".into(),
            cancel_url: "/users/42".into(),
            edit_label: None,
            save_label: None,
            cancel_label: None,
            method: None,
        });

        let serialized = serde_json::to_value(&original).expect("serialize");
        assert_eq!(serialized.get("type").and_then(|v| v.as_str()), Some("DetailForm"));
        assert_eq!(serialized.get("mode").and_then(|v| v.as_str()), Some("edit"));

        let deserialized: Component = serde_json::from_value(serialized).expect("deserialize");
        assert_eq!(original, deserialized);
    }

    #[test]
    fn edit_mode_default_is_view() {
        assert_eq!(EditMode::default(), EditMode::View);
    }

    #[test]
    fn edit_mode_from_query_exact_edit() {
        assert_eq!(EditMode::from_query(Some("edit")), EditMode::Edit);
    }

    #[test]
    fn edit_mode_from_query_case_insensitive() {
        assert_eq!(EditMode::from_query(Some("EDIT")), EditMode::Edit);
        assert_eq!(EditMode::from_query(Some("Edit")), EditMode::Edit);
        assert_eq!(EditMode::from_query(Some("eDiT")), EditMode::Edit);
    }

    #[test]
    fn edit_mode_from_query_none_is_view() {
        assert_eq!(EditMode::from_query(None), EditMode::View);
    }

    #[test]
    fn edit_mode_from_query_unknown_is_view() {
        assert_eq!(EditMode::from_query(Some("")), EditMode::View);
        assert_eq!(EditMode::from_query(Some("view")), EditMode::View);
        assert_eq!(EditMode::from_query(Some("anything-else")), EditMode::View);
    }

    #[test]
    fn edit_mode_serializes_as_snake_case() {
        let json = serde_json::to_value(EditMode::Edit).unwrap();
        assert_eq!(json, serde_json::Value::String("edit".into()));
    }
}
```

### Render test — template

**Source:** `ferro-json-ui/src/render.rs:4263-4372` (form tests), `:8441-8620` (key_value_editor tests). Style: HTML-substring assertions on the full rendered output.

One test per mode minimum, plus discretionary additions:

1. `render_detail_form_view_mode` — assert `<dl>` present, each `<dt>`/`<dd>` pair present with escaped values, Modifica `<a href=` present, NO `<form>`, NO Salva, NO Annulla.
2. `render_detail_form_edit_mode` — assert `<form action=`, `<dl>`, each field's input rendered (e.g. `<input type="text" name="name"`), Salva `<button type="submit">`, Annulla `<a href=`, NO Modifica.
3. `render_detail_form_edit_method_spoofing_put` — `method: Some(HttpMethod::Put)`, assert `method="post"` and `name="_method" value="PUT"`.
4. `render_detail_form_edit_method_spoofing_patch` — same shape, Patch.
5. `render_detail_form_edit_method_spoofing_delete` — same shape, Delete.
6. `render_detail_form_edit_get_no_spoofing` — `action.method: Get`, assert `method="get"`, NO `_method` hidden input.
7. `render_detail_form_view_xss_escapes_label` — label containing `<script>alert(1)</script>`, assert `&lt;script&gt;` in output.
8. `render_detail_form_view_xss_escapes_edit_url` — `edit_url` containing `" onmouseover="x`, assert `&quot;` in output.
9. `render_detail_form_edit_xss_escapes_cancel_url` — same shape for cancel_url.
10. `render_detail_form_custom_labels` — edit_label=Some("Modifica dati"), save_label=Some("OK"), cancel_label=Some("Indietro"); assert substrings match customs, not defaults.
11. `render_detail_form_view_action_bar_below_dl` — assert `</dl>` appears before the Modifica link in the rendered string (for ordering).

### Resolver test — template

**Source:** `ferro-json-ui/src/resolve.rs:593-633` (`resolve_form_action`).

```rust
#[test]
fn resolve_detail_form_action() {
    let mut view = JsonUiView::new().component(ComponentNode::detail_form("df", DetailFormProps {
        mode: EditMode::Edit,
        action: make_action("users.update"),  // url: None
        fields: vec![],
        edit_url: "/users/42?mode=edit".into(),
        cancel_url: "/users/42".into(),
        edit_label: None, save_label: None, cancel_label: None, method: None,
    }));
    resolve_actions(&mut view, |h| match h {
        "users.update" => Some("/users/42".into()),
        _ => None,
    });
    match &view.components[0].component {
        Component::DetailForm(p) => assert_eq!(p.action.url, Some("/users/42".into())),
        _ => panic!("expected DetailForm"),
    }
}

#[test]
fn resolve_errors_propagates_into_detail_form_fields() {
    // Build a DetailForm containing an Input field. Call resolve_errors with that field's
    // error, assert InputProps.error gets populated.
}
```

---

## Runtime State Inventory

> Not applicable — Phase 147 is a greenfield component addition with no rename, migration,
> or refactor of existing state.

---

## Environment Availability

> Skipped — Phase 147 is code-only. No external tools, services, or runtimes beyond
> the existing Rust toolchain and the ferro workspace.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` (no external framework) |
| Config file | `ferro-json-ui/Cargo.toml` (no test-specific config beyond `[dev-dependencies]`) |
| Quick run command | `cargo test -p ferro-json-ui --lib` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-01 | `EditMode` derives Debug/Clone/Copy/PartialEq/Eq/Serialize/Deserialize/JsonSchema | unit (compile-time) | `cargo test -p ferro-json-ui edit_mode_default_is_view` | ❌ Wave 0 |
| D-02 | `EditMode::from_query("edit")`, `("EDIT")`, `(None)`, `(Some("view"))`, etc. | unit | `cargo test -p ferro-json-ui edit_mode_from_query` | ❌ Wave 0 |
| D-03 | `DetailField` constructed + round-trips through serde | unit | `cargo test -p ferro-json-ui detail_form_serde_roundtrip` | ❌ Wave 0 |
| D-04 | `DetailFormProps` round-trips, all Option fields serialize only when present | unit | `cargo test -p ferro-json-ui detail_form_serde_roundtrip` | ❌ Wave 0 |
| D-05 | View and Edit emit same `<dl>` / `<dt>` / `<dd>` scaffold | unit | `cargo test -p ferro-json-ui render_detail_form_view_mode render_detail_form_edit_mode` | ❌ Wave 0 |
| D-06 | Edit wraps `<dl>` in `<form>`; View has no `<form>` | unit | `cargo test -p ferro-json-ui render_detail_form_edit_mode render_detail_form_view_mode` | ❌ Wave 0 |
| D-07 | Action bar is outside `<dl>` but inside `<form>` (Edit) or wrapper (View) | unit | `cargo test -p ferro-json-ui render_detail_form_view_action_bar_below_dl` | ❌ Wave 0 |
| D-08 | `<dl>` uses `grid grid-cols-1 gap-4`; `<dt>`/`<dd>` use documented token classes | unit | `cargo test -p ferro-json-ui render_detail_form_view_mode` | ❌ Wave 0 |
| D-09 | Modifica renders as `<a href=`, outline variant classes | unit | `cargo test -p ferro-json-ui render_detail_form_view_mode` | ❌ Wave 0 |
| D-10 | `edit_url` emitted verbatim after `html_escape` | unit | `cargo test -p ferro-json-ui render_detail_form_view_xss_escapes_edit_url` | ❌ Wave 0 |
| D-11 | Method spoofing: PUT/PATCH/DELETE → POST + `_method` hidden input | unit | `cargo test -p ferro-json-ui render_detail_form_edit_method_spoofing_` | ❌ Wave 0 |
| D-12 | `render_node(&field.input, data)` rendered inside `<dd>` | unit | `cargo test -p ferro-json-ui render_detail_form_edit_mode` | ❌ Wave 0 |
| D-13 | `DetailField.value` NOT threaded into `DetailField.input` defaults (caller's job) | unit + docs | Covered by Edit-mode test: input `value=` comes from `InputProps.default_value`, not `DetailField.value` | ❌ Wave 0 |
| D-14 | Salva = `<button type="submit">` primary; Annulla = `<a>` outline targeting cancel_url | unit | `cargo test -p ferro-json-ui render_detail_form_edit_mode` | ❌ Wave 0 |
| D-15 | `resolve_actions` populates `props.action.url` | unit | `cargo test -p ferro-json-ui resolve_detail_form_action` | ❌ Wave 0 |
| D-16 | `edit_url` / `cancel_url` are NOT touched by resolver | unit | Negative assertion in resolver test: pre-set URLs unchanged after resolve | ❌ Wave 0 |
| D-17 | Tagged-enum serialize emits `"type": "DetailForm"` | unit | `cargo test -p ferro-json-ui detail_form_serde_roundtrip` | ❌ Wave 0 |
| D-18 | `ComponentNode::detail_form(key, props)` compiles and returns the expected variant | unit (compile-time) | Used by the resolver test above | ❌ Wave 0 |
| D-19 | `COMPONENT_CATALOG` contains `"DetailForm"` substring | unit | `cargo test -p ferro-json-ui component_catalog_lists_detail_form` (new) | ❌ Wave 0 |
| D-20 | No new file under `ferro-json-ui/src/runtime/` | filesystem | `! ls ferro-json-ui/src/runtime/detail_form.rs` (verification only, not an automated test) | n/a |
| Integration | `Request::query("mode")` → `EditMode::from_query` pipeline works | manual UAT | Exercise via `app/` sample or gestiscilo after this phase ships | ❌ (N/A — caller's test) |
| MCP catalog | ferro-mcp lists `"DetailForm"` in both the `CatalogComponent` array and the exhaustive name list | unit | `cargo test -p ferro-mcp json_ui_catalog_exhaustive_list` (existing test at L1105+ of json_ui_catalog.rs) | ❌ Wave 0 (edit ferro-mcp) |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-json-ui --lib` (~2-3 seconds; exercises every DetailForm test plus the existing suite)
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green + ferro-mcp crate tests green (catalog exhaustiveness enforced there) before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-json-ui/src/component.rs` — add `#[cfg(test)] mod detail_form_tests` with serde round-trip + 7 `EditMode::from_query` tests
- [ ] `ferro-json-ui/src/render.rs` — add ~11 render tests under the existing `mod tests`
- [ ] `ferro-json-ui/src/resolve.rs` — add `resolve_detail_form_action` + `resolve_errors_propagates_into_detail_form_fields` tests under the existing `mod tests`
- [ ] `ferro-json-ui/src/lib.rs` — update `COMPONENT_CATALOG` literal; no dedicated test but the string `"### DetailForm"` should be grep-assertable in a test
- [ ] `ferro-mcp/src/tools/json_ui_catalog.rs` — add CatalogComponent entry for DetailForm AND `"DetailForm"` to the exhaustive-list assertion at L1115 (also fix KeyValueEditor gap while here — backfill)

No framework installation needed; `cargo test` already present.

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Not in scope — DetailForm is pure rendering |
| V3 Session Management | no | Not in scope |
| V4 Access Control | no (caller responsibility) | Caller decides whether to render DetailForm at all based on authorization |
| V5 Input Validation | yes (server-side) | `html_escape` on every dynamic string → `href`, text node; input field validation stays with the existing validator (ferro-json-ui has no direct role) |
| V6 Cryptography | no | Not applicable |
| V7 Error handling | yes (rendering errors don't leak state) | `action.url.as_deref().unwrap_or("#")` — safe fallback, no panic |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via field `label` / `value` in View mode | Tampering | `html_escape` on both `<dt>` and `<dd>` text content (mirrors render_description_list) |
| XSS via `edit_url` / `cancel_url` in `href` attribute | Tampering | `html_escape` on both URLs before emission — they land in attribute context, identical to Form's action URL handling (render.rs:989, 995) |
| XSS via `save_label` / `cancel_label` / `edit_label` in button text | Tampering | `html_escape` on each before emission (caller-supplied strings even if optional-with-default) |
| XSS via `DetailField.input` — rendered through `render_node` | Tampering | Each component-specific renderer already applies `html_escape` to its own dynamic strings; no new surface |
| CSRF on form submission | Spoofing | Out of scope — CSRF is handled at the framework level (ferro's framework crate), not in DetailForm rendering. DetailForm relies on the same submission pipeline as Form, so whatever CSRF discipline exists applies equally |
| URL injection / open redirect via `edit_url` / `cancel_url` | Tampering | Caller-controlled URLs. If the caller accepts a URL from user input and feeds it in, that's a caller-side flaw; DetailForm renders whatever is passed. Note this in the component docs. |
| Method downgrade — attacker forges `_method=GET` | Tampering | Out of scope; framework-level CSRF + method-spoofing middleware handles this |
| Visible mode toggle — unauthorized user hits `?mode=edit` | Access control | Caller responsibility — D-15/D-16 deliberately don't auto-filter mode visibility. If authorization matters, handler short-circuits before calling `EditMode::from_query`. Document. |

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Handler branches on mode and assembles two different trees (`DescriptionList` vs `Form`) | Single `Component::DetailForm` call, server-side mode toggle | Phase 147 (this phase) | Structural coherence at the view/edit boundary |
| Client-side JS toggle with hidden/shown states | Server-side URL query param (`?mode=edit`) | Phase 147 design (D-20) | Zero runtime JS; simpler; no client-state bugs |

**Deprecated / outdated:**
- Gestiscilo's `EditableField` / `editable_section` helpers (downstream) — to be retired after this phase ships, replaced by `Component::DetailForm`. That migration is NOT part of phase 147.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Empty string as `InputProps.label` renders an empty `<label>` that collapses without layout impact and is silent to screen readers. | Label Duplication — Option A | If incorrect, there would be a visible empty-label gap above each input in Edit mode. Easy to verify with a single screenshot test; low risk. |
| A2 | ferro-mcp `json_ui_catalog.rs` exhaustive test fails if `DetailForm` is missing from the hardcoded list. | Wave 0 Gaps / Pitfall 6 | If the test is not exhaustive, missing DetailForm won't fail CI. Verifiable by running the existing test block at L1115 in a dry-run state. Low risk. |
| A3 | The default Italian labels ("Modifica", "Salva", "Annulla") are the right defaults for v1. | D-04 / CONTEXT.md specifics | User has signed off in CONTEXT.md. Not a research concern. |
| A4 | `render_detail_form` does NOT need to handle `field.input` being itself a `Component::DetailForm` — callers won't nest these. | Anti-Patterns | If nested DetailForms become a real use case, we'd need to add tests for action-bar placement inside nested forms (HTML requires forms not to nest). This is a deliberate scope boundary. |

---

## Open Questions

None blocking. All 10 research questions in the prompt have been answered with pointers to exact source lines. The one design choice (Question 6, label duplication) resolves to Option A with zero scope change.

The only discoverable CODEBASE GAP beyond phase 147's intended scope — ferro-mcp's `json_ui_catalog.rs` is missing KeyValueEditor — should be backfilled in the same plan wave that adds DetailForm. That's a ~2-line edit, prevents the exhaustive-list test from silently missing components, and keeps the MCP catalog honest. Document this as Plan scope, not a new phase.

---

## Project Constraints (from CLAUDE.md)

Actionable directives extracted for the planner:

- **Run before every commit:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **No co-author attribution in commits**
- **Prefer editing existing files** over creating new ones (this phase is 100% edits to 4 existing files in ferro-json-ui + 1 in ferro-mcp + 1 in docs/src)
- **`docs/src/` must reflect framework changes** — components.md needs a DetailForm section
- **Update ferro-mcp when introspection surface changes** — json_ui_catalog.rs is the relevant file
- **Form field rules** — every form field has a proper `default_value`. D-13 defers this to each caller-per-input, consistent with the rule. DetailField.value is NOT the input default; callers set `InputProps.default_value` themselves (usually from `req.old(...)` in edit-after-validation-failure flows).
- **UI design principles** — "Persistent frames are sacred": the outer `<dl>` scaffold is the persistent frame; View/Edit differ only in `<dd>` contents and action bar. D-05/D-06/D-07 directly encode this.
- **"This is always a feature branch"** — add `Component::DetailForm` directly, no compatibility layer.

---

## Sources

### Primary (HIGH confidence)

- `ferro-json-ui/src/component.rs:180-203` — `FormProps`, `FormMaxWidth` (pattern for action-bound form component)
- `ferro-json-ui/src/component.rs:232-263` — `InputProps` (field-level input surface; confirms unconditional label emission)
- `ferro-json-ui/src/component.rs:386-416` — `KeyValueEditorProps` (phase 146 precedent for JsonSchema-present props with complex fields)
- `ferro-json-ui/src/component.rs:425-458` — `DescriptionItem`, `DescriptionListProps`, `Tab`, `TabsProps` (view-mode scaffold + JsonSchema-skipped precedent)
- `ferro-json-ui/src/component.rs:947-991` — `Component` enum (insertion point)
- `ferro-json-ui/src/component.rs:997-1058` — `serialize_tagged` helper + all Serialize arms
- `ferro-json-ui/src/component.rs:1064-1204` — Custom Deserialize match arms (pattern for DetailForm deserialize)
- `ferro-json-ui/src/component.rs:1244-1252` — `ComponentNode::form` (factory pattern for detail_form)
- `ferro-json-ui/src/component.rs:1355-1362` — `ComponentNode::description_list` (parallel factory)
- `ferro-json-ui/src/component.rs:3620-3700` — `key_value_editor_serde_roundtrip` (test pattern to duplicate)
- `ferro-json-ui/src/render.rs:101-197` — `collect_plugin_types_node` (third walk to add DetailForm arm to)
- `ferro-json-ui/src/render.rs:288-340` — `render_component` dispatch (insertion point)
- `ferro-json-ui/src/render.rs:971-1031` — `render_form` (canonical form + spoofing + max-width; direct copy source)
- `ferro-json-ui/src/render.rs:1374-1519` — `render_input` (confirms unconditional label emission at L1408-1412; Hidden is the only exception path)
- `ferro-json-ui/src/render.rs:2029-2117` — `render_button` (variant class strings for anchor/button emission in render_detail_form)
- `ferro-json-ui/src/render.rs:2427-2439` — `render_description_list` (direct copy source for `<dl>/<dt>/<dd>` scaffold)
- `ferro-json-ui/src/render.rs:4261-4372` — Form render tests (canonical substring-assertion style)
- `ferro-json-ui/src/render.rs:8441-8620` — KeyValueEditor render tests (phase 146 substring-assertion style)
- `ferro-json-ui/src/resolve.rs:30-155` — `resolve_component_node` (first of three resolver arms to add)
- `ferro-json-ui/src/resolve.rs:205-328` — `collect_unresolved_node` (second of three resolver arms to add)
- `ferro-json-ui/src/resolve.rs:377-476` — `resolve_errors_node` (third of three resolver arms to add)
- `ferro-json-ui/src/resolve.rs:593-633` — `resolve_form_action` (test pattern for resolve_detail_form_action)
- `ferro-json-ui/src/lib.rs:59-71` — component re-export block (insertion point for `DetailFormProps`, `DetailField`, `EditMode`)
- `ferro-json-ui/src/lib.rs:102-186` — `COMPONENT_CATALOG` literal (insertion point for `### DetailForm` entry)
- `ferro-json-ui/src/action.rs:20-88` — `HttpMethod`, `Action` (structure of the field type DetailFormProps uses)
- `framework/src/http/request.rs:154-160` — `Request::query(name)` signature (caller pipeline for `EditMode::from_query`)
- `ferro-mcp/src/tools/json_ui_catalog.rs:504-522` — `DescriptionList` CatalogComponent entry (pattern to follow)
- `ferro-mcp/src/tools/json_ui_catalog.rs:1115-1154` — exhaustive list of component names (must include DetailForm)
- `docs/src/json-ui/components.md:415-475` — DescriptionList docs section (template for DetailForm section)

### Secondary (MEDIUM confidence)

- `.planning/phases/146-add-keyvalueeditor-component-to-ferro-json-ui-dynamic-key-va/146-RESEARCH.md` — phase 146 research structure (template for this research)
- `.planning/phases/146-add-keyvalueeditor-component-to-ferro-json-ui-dynamic-key-va/146-01-PLAN.md` — phase 146 RED-tests-first wave structure (reused for DetailForm)
- `.planning/PROJECT.md §"Beauty as a design criterion"` — conceptual coherence motivates D-05/D-06/D-07
- `./CLAUDE.md §"Form Field Rules"` — consistent with D-13 (caller sets input defaults)
- `./CLAUDE.md §"UI Design Principles"` — "Persistent frames are sacred" maps to D-05

### Tertiary (LOW confidence)

None. All claims are anchored to current source or to project reference documents.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all patterns established in the crate
- Architecture: HIGH — all insertion points verified against current source with exact line numbers
- Pitfalls: HIGH — the 7 pitfalls listed are direct consequences of observed patterns (3 resolver arms, plugin-type walk, method-spoofing exactness, escape discipline, MCP catalog exhaustiveness, docs drift)
- Label duplication resolution (Option A): MEDIUM-HIGH — recommendation depends on empty `<label>` behaving as described across all target browsers; Rust-side is certain, browser-side is a standard HTML assumption

**Research date:** 2026-04-23
**Valid until:** ~30 days (stable crate; no imminent v2 spec refactor affecting this file set — v12.0 JSON-UI v2 is a separate milestone that will overhaul the entire spec format but is queued behind v13.0 Road to v1.0 per STATE.md)

## RESEARCH COMPLETE
