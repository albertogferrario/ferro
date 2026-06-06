# InlineBudget & RequestTelemetry

Two request-scoped primitives for HTML inline-vs-preload decisioning and in-process sampled telemetry.

## When to use InlineBudget

`InlineBudget` decides at request time whether a chunk of bytes should be inlined into the HTML response or preloaded via `<link rel=preload>`. Use it when:

- The response renders payloads of varying size depending on tenant or context.
- Inlining everything would risk crossing a payload-size cliff (HTML parse cost on the client).
- A pre-built static fallback URL exists for the same byte payload.

`RequestTelemetry` records sampled time-series telemetry into an in-process ring buffer keyed by `(key, scope)`. Use it when:

- Operator dashboards need recent samples of a metric without round-tripping through an external telemetry system.
- Per-tenant or per-route slicing is useful for diagnostics.
- The "lost on process restart" semantic is acceptable.

## Quick example

```rust,ignore
use ferro_rs::{Decision, Request, RequestTelemetry, Sample};
use serde_json::json;

async fn render_page(req: &mut Request, products: Vec<Product>) -> String {
    let products_json = serde_json::to_string(&products).unwrap_or_default();
    let payload_bytes = products_json.len();

    let body = "<div>...</div>";

    let html = match req.inline_budget(
        "products_payload",
        payload_bytes,
        "/_/bootstrap/products.json",
    ) {
        Decision::Inline => format!(
            "<script id='__products' type='application/json'>{products_json}</script>{body}",
        ),
        Decision::Preload(url) => format!(
            r#"<link rel="preload" as="fetch" href="{url}" crossorigin>{body}"#,
        ),
    };

    // Telemetry: record the payload byte count scoped to the tenant.
    req.telemetry_record_scoped(
        "products_payload_size",
        Some("tenant:42"),
        Sample::now(json!({ "bytes": payload_bytes })),
    );

    html
}

// Operator dashboard handler:
async fn ops_payload_distribution() -> Vec<Sample> {
    RequestTelemetry::snapshot("products_payload_size", Some("tenant:42"))
}
```

## The `Decision` enum

```rust,ignore
pub enum Decision {
    Inline,
    Preload(String),
}
```

Match on `Decision::Inline` to emit the bytes inline; match on `Decision::Preload(url)` to emit a `<link rel=preload href=url>` and serve the same bytes from `url` separately. The URL is whatever the caller passed as `fallback_url` — there is no rewriting.

## Threshold configuration

The threshold is read from `AppConfig::inline_budget_threshold_bytes`. Default: `102_400` (100 KiB).

Override via env var:

```sh
INLINE_BUDGET_BYTES=204800  # 200 KiB
```

Or programmatically:

```rust,ignore
use ferro_rs::AppConfigBuilder;

let cfg = AppConfigBuilder::default()
    .inline_budget_threshold_bytes(204_800)
    .build();
```

The threshold is global — there is no per-key override in v1. If your application needs heterogeneous budgets per key, file an issue requesting `req.inline_budget_with_limit(key, bytes, fallback_url, limit)`.

## Warning channel

The first time cumulative bytes for a given `key` crosses the threshold within a single request, `InlineBudget` emits a structured `tracing::warn!` with the following fields:

| Field | Type | Source |
|-------|------|--------|
| `key` | `&str` | the `key` argument |
| `cumulative_bytes` | `usize` | running total for `key` within the request |
| `threshold_bytes` | `usize` | the configured threshold |
| `fallback_url` | `&str` | the `fallback_url` argument |
| `route_pattern` | `String` | `req.route_pattern()` or `""` if not yet matched |

The warning fires **exactly once per `(key, request)`**. Subsequent calls past the threshold for the same key in the same request return `Decision::Preload` silently (no warning spam).

The `fallback_url` field is logged with `Display` formatting. Do NOT pass user-controlled input as `fallback_url`; the field is intended for compile-time-shaped strings designed by the application developer.

## When to use RequestTelemetry

`RequestTelemetry` is a per-key in-process ring buffer for sampled time-series telemetry. Use it for operator-visible metrics that:

- Have low cardinality on the `(key, scope)` axis.
- Benefit from per-tenant or per-route slicing.
- Do not require cross-process aggregation (use Prometheus or OpenTelemetry for that).

Each `(key, scope)` bucket holds at most 128 samples — oldest dropped on overflow. The store is process-global and **lost on process restart**.

## Sample shape

```rust,ignore
pub struct Sample {
    pub recorded_at: std::time::SystemTime,
    pub value: serde_json::Value,
}

impl Sample {
    pub fn now(value: serde_json::Value) -> Self;
    pub fn at(when: std::time::SystemTime, value: serde_json::Value) -> Self;
}
```

`recorded_at` uses `SystemTime` (wall-clock) so samples remain comparable across process restarts. `value` is a `serde_json::Value` so heterogeneous payloads from different writers can share a snapshot reader.

## Writer methods

```rust,ignore
impl Request {
    pub fn telemetry_record(&mut self, key: &str, sample: Sample);
    pub fn telemetry_record_scoped(&mut self, key: &str, scope: Option<&str>, sample: Sample);
}
```

`telemetry_record(key, sample)` is equivalent to `telemetry_record_scoped(key, None, sample)`. Both methods take `&mut self` for API uniformity with `inline_budget`; neither mutates the request itself.

## Reader — `snapshot`

```rust,ignore
impl RequestTelemetry {
    pub fn snapshot(key: &str, scope: Option<&str>) -> Vec<Sample>;
    pub fn keys() -> Vec<(String, Option<String>)>;
    pub fn clear();
}
```

`snapshot` returns a clone of every sample currently in the `(key, scope)` bucket in FIFO order. Empty if no samples have been recorded.

`keys` lists every `(key, scope)` pair that has at least one recorded sample. Useful for operator dashboards that need to enumerate available metrics.

`clear` drops every sample in every bucket — intended for operator-driven resets (e.g. post-deploy).

The reader API is internal Rust. If exposed on an HTTP endpoint, the consumer is responsible for authorization — `RequestTelemetry` itself enforces no access control.

## Scope conventions

`scope` is `Option<&str>`; callers pick the convention. Recommended formats:

| Convention | Example | When to use |
|------------|---------|-------------|
| `tenant:N` | `"tenant:42"` | per-tenant metrics in a multi-tenant app |
| `route:/path` | `"route:/api/products"` | per-route latency / size metrics |
| `region:X` | `"region:eu-west-1"` | per-region metrics in a multi-region deploy |
| (none) | `None` | global metric not sliced by anything |

These are conventions, not constraints — the storage layer treats `scope` as an opaque `Option<&str>`.

## Lost-on-restart semantic

The ring buffer is in-process memory. Process restart drops every sample. This is by design — `RequestTelemetry` is for recent operator-visible signal, not long-term metrics retention. For cross-process aggregation, export selected metrics to Prometheus, OpenTelemetry, or a custom DB sink.

## Key cardinality

The `(key, scope)` axis has no enforced upper bound on cardinality. Callers MUST NOT derive `key` or `scope` strings from user input — a unique key per user creates unbounded growth of the global store. Use a fixed vocabulary controlled by application code.

## End-to-end example

The "Quick example" section above is the canonical end-to-end example. It exercises both primitives in one handler: `inline_budget` decides inline-vs-preload for the products payload, then `telemetry_record_scoped` records the byte count under a per-tenant scope. The operator handler uses `RequestTelemetry::snapshot` to read recent samples for that tenant's bucket.
