# Projection-Compression Slice (Design 1C)

How little **authored** code Ferro's projection/intent system needs to produce a
working, data-bound resource UI, versus the equivalent hand-written Laravel
resource. Static + render-evidence comparison — no Docker, no load tests.

The killer-feature claim is **authoring compression of a working render
pipeline**, stated accurately and with explicit caveats. Read the caveats — they
bound what this number means.

## Resource

`product`, scalar fields only so the create form is fully fair:

| field    | type     | meaning      |
| -------- | -------- | ------------ |
| `id`     | Integer  | Identifier   |
| `name`   | String   | EntityName   |
| `price`  | Float    | Money        |
| `stock`  | Integer  | Quantity     |
| `status` | String   | Status       |

`status` drives the Process/Kanban DISPLAY grouping (a scalar string); the create
form needs only `name` / `price` / `stock`, all scalar Inputs.

## Authored line / token count

Counts cover the files a developer **writes**, never framework scaffold. Both
sides exclude their framework-provided HTML shell (Ferro: `render_layout`;
Laravel: `layouts/app.blade.php`, marked excluded in-file).

### Ferro — files counted

| file                 | total | code | tokens |
| -------------------- | ----- | ---- | ------ |
| `ferro/src/service.rs` (the `ServiceDef` declaration) | 34 | 19 | 168 |

The `service_def()` builder chain is **17 code lines** — 10 for the scalar field
declaration, 7 for the optional `state_machine` block that exists only to feed
the Process/Kanban DISPLAY surface. A consumer who skips Process writes ~10 lines.

### Laravel — files counted

| file                                                | total | code | tokens |
| --------------------------------------------------- | ----- | ---- | ------ |
| `app/Models/Product.php`                            | 20    | 16   | 31     |
| `database/migrations/..._create_products_table.php` | 25    | 22   | 39     |
| `app/Http/Controllers/ProductController.php`        | 65    | 45   | 177    |
| `routes/web.php`                                    | 11    | 9    | 23     |
| `resources/views/products/index.blade.php`          | 28    | 25   | 44     |
| `resources/views/products/show.blade.php`           | 21    | 15   | 28     |
| `resources/views/products/create.blade.php`         | 31    | 24   | 64     |
| `resources/views/products/summary.blade.php`        | 8     | 7    | 13     |
| `resources/views/products/board.blade.php`          | 19    | 17   | 38     |
| **TOTAL**                                           | 228   | 180  | 457    |

### Ratio (Laravel / Ferro)

| metric      | Laravel | Ferro | ratio     |
| ----------- | ------- | ----- | --------- |
| total lines | 228     | 34    | **6.7x**  |
| code lines  | 180     | 19    | **9.5x**  |
| tokens      | 457     | 168   | **2.7x**  |

Tokens are whitespace-delimited; the ratio is lower than the line ratio because
each Ferro line is denser (a builder call vs. a `<td>` or a `$table->` line). The
line-count ratio is the headline; the token ratio is the conservative floor.

## What each side produces from the counted code

**Ferro** — from the single `ServiceDef` declaration, `derive_intents()` +
`JsonUiRenderer` produce five data-bound surfaces (render evidence in
`ferro/rendered/`):

| surface     | intent    | root element | data-bound? |
| ----------- | --------- | ------------ | ----------- |
| list        | Browse    | DataTable    | yes — product rows |
| detail      | Focus     | Card + DescriptionList | yes |
| stat        | Summarize | StatCard     | yes — `value_path /data/product/price` |
| kanban      | Process   | KanbanBoard  | yes — products bucketed into draft/active/discontinued lanes |
| create form | Collect   | Form         | yes — scalar Inputs |

**Laravel** — the same five surfaces, hand-written across 9 files (model,
migration, controller, routes, 5 Blade views).

## Render evidence

`ferro/rendered/` contains, per surface, the JSON-UI `Spec` (`.json`) and a
data-bound HTML snapshot (`.html`) produced by binding a fixed 3-product dataset.
Regenerate with:

```
cd ferro && cargo run --bin render-evidence
```

The HTML snapshots contain the literal data values (e.g. `Aeron Chair`, `1395`),
proving real binding rather than placeholders — verifiable by grepping the files.

## HONEST CAVEATS (read these)

These bound the claim. The comparison is fair only within them.

**(a) Enum / foreign-key form fields are not auto-populated.** The projection's
form `Select` for `status` renders with `"options": []` (visible in
`ferro/rendered/collect.json`) — `build_select_props` emits an empty option list
today. **The comparison therefore covers scalar-field create only** (`name` /
`price` / `stock`, which render as text/number Inputs). The Laravel `create`
view, by contrast, hand-writes the three status `<option>`s. A full
relational/enum create round-trip is NOT at parity and is excluded from the
claim.

**(b) The form submit route is a placeholder.** The projection emits
`action: POST /product` (see `collect.json`) — a convention the consumer wires to
a real route by hand. The Laravel side hand-writes both the route
(`routes/web.php`) and the `store()` action. The Ferro count does **not** include
wiring that submit route, because the projection does not author it; this is a
known consumer responsibility, not generated code.

**(c) The render evidence is Specs + HTML snapshots, not a wired HTTP app.** The
five surfaces are individually rendered and data-bound in the evidence harness,
but there is **not yet a single wired web route in a sample app serving a
projection page end-to-end** — the pieces are tested and rendered, not assembled
into one HTTP flow. This measures **authoring compression of a working render
pipeline**, not a turnkey scaffolded application. The Laravel side, by contrast,
is a complete (if minimal) request→controller→view flow.

## Interpretation

Within the caveats, a Ferro developer authors ~19 code lines (~10 if Process is
skipped) to obtain five data-bound surfaces; the idiomatic Laravel equivalent is
~180 code lines across 9 files — a **9.5x line / 2.7x token** authoring
compression. The compression is real and reproducible; it is not a claim of
feature-for-feature parity (caveats a–c), and it is not a turnkey-app claim
(caveat c).
