# Benchmark Plan 1A — Harness Foundation (Micro-Endpoints) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the reproducible benchmark harness (contracts + static-counter + perf-runner + reporting) and prove it end-to-end on the smallest workload — four micro-endpoints implemented in Ferro and Laravel — producing the first committed results table.

**Architecture:** A `benchmark/` tree containing per-framework apps (each in its own Docker container), a pinned containerized "toolbox" (oha + tokei + python) that runs load tests and static counts against those apps, and a Python reporting step that turns raw result JSON into internal and public markdown tables. Measurement is decoupled from implementations: the runners consume containerized apps + a shared contract and emit raw JSON; reporting reads only `results/`.

**Tech Stack:** Docker + docker compose, PostgreSQL 16 (pinned), `oha` (load gen, JSON output), `tokei` (LoC/file counts), Python 3 + pytest (glue, parsers, reporting, conformance), Ferro (current workspace), Laravel 11 / PHP 8.3.

---

## Decomposition (Phase 1 split into shippable plans)

- **1A (this plan):** harness + contracts + micro-endpoints apps + first results run. Validates the entire pipeline on the cheapest workload.
- **1B:** Ferro Conduit (full RealWorld backend) + vendored `gothinkster/laravel-realworld-example-app` (pinned commit) + Conduit conformance + Conduit results. Plugs into the 1A harness unchanged.
- **1C:** projection-driven static slice (Ferro vs Laravel, `articles` resource) + consolidated Phase-1 internal/public report.

## Resolved defaults (the spec's four open questions)

1. **Database:** PostgreSQL `16.4` in a pinned container, shared by all DB-backed endpoints across frameworks. Rationale: the TechEmpower standard and the fair choice for concurrency (SQLite write-locking distorts load tests). Ferro's SQLite-default story is documented separately in the README, not used for the perf axis.
2. **Laravel source:** for micro-endpoints (1A) there is no community RealWorld app, so a **minimal idiomatic Laravel 11 app** is written, modeled on the TechEmpower Laravel entry — this is acceptable because the endpoints are trivial and idiomatic (the "don't author the competitor" rule binds the realistic Conduit app in 1B, where `gothinkster/laravel-realworld-example-app` is vendored at a pinned commit).
3. **Projection slice resource:** `articles` (reuses the Conduit domain). Deferred to 1C.
4. **Hardware/runner spec:** canonical perf numbers run on a **documented fixed local machine**; every results run records `{cpu_model, physical_cores, ram_gb, os, kernel}` in its metadata. CI (`ubuntu-latest`) runs conformance + a perf *smoke* only (shared runners are too noisy for headline numbers). Tool versions are pinned in the toolbox image, so they are machine-independent.

## File structure

```
benchmark/
  README.md                        # methodology, hardware spec, run instructions, honesty note
  .gitignore                       # ignore app build artifacts; DO NOT ignore results/
  compose.yaml                     # postgres + ferro-micro + laravel-micro + toolbox
  harness/
    Dockerfile.toolbox             # pins oha, tokei, python3, jq
    perf/
      parse_perf.py                # oha --json stdout -> normalized metrics dict
      test_parse_perf.py
      run_perf.py                  # CLI: target URL + label -> results/<date>/perf-<label>.json
    static/
      count_static.py              # tokei JSON + token count over an app dir -> metrics dict
      test_count_static.py
    report/
      build_tables.py              # results/<date>/*.json -> internal.md + public.md
      test_build_tables.py
  contracts/
    micro-endpoints.md             # the 4 endpoint specs
    conformance/
      test_conformance.py          # hits a running app base-URL, asserts the contract
  apps/
    ferro-micro/                   # scaffolded Ferro app, 4 endpoints, Dockerfile
    laravel-micro/                 # minimal Laravel app, 4 endpoints, Dockerfile
  results/
    .gitkeep
```

The four micro-endpoints (the shared contract):

| Path | Behavior |
|------|----------|
| `GET /json` | returns `{"message":"Hello, World!"}`, `Content-Type: application/json` |
| `GET /db` | one row by random id from `world` table → `{"id":N,"randomNumber":M}` |
| `GET /queries?n=K` | K random-row lookups (1≤K≤500, clamp) → array of `{id,randomNumber}` |
| `GET /updates?n=K` | K random reads + writes of `randomNumber` → updated array |

`world` table: `id SERIAL PRIMARY KEY, randomNumber INT NOT NULL`, seeded with 10000 rows.

---

## Task 1: Scaffold the benchmark tree

**Files:**
- Create: `benchmark/README.md`
- Create: `benchmark/.gitignore`
- Create: `benchmark/results/.gitkeep`

- [ ] **Step 1: Create the directory and placeholder files**

```bash
mkdir -p benchmark/harness/perf benchmark/harness/static benchmark/harness/report \
         benchmark/contracts/conformance benchmark/apps benchmark/results
touch benchmark/results/.gitkeep
```

- [ ] **Step 2: Write `benchmark/.gitignore`**

```gitignore
# build artifacts from the sample apps — never the harness or results
apps/*/target/
apps/*/vendor/
apps/*/node_modules/
apps/*/.env
**/__pycache__/
*.pyc
# results/ is intentionally committed (auditable raw data)
```

- [ ] **Step 3: Write `benchmark/README.md` (methodology skeleton)**

```markdown
# Ferro Framework Benchmark

A reproducible comparison of Ferro against mature batteries-included frameworks.
Design: `docs/superpowers/specs/2026-06-15-ferro-framework-benchmark-design.md`.

## What 1A measures
Four micro-endpoints (`/json`, `/db`, `/queries`, `/updates`) in Ferro and Laravel, on two
axes: raw performance (requests/sec, p50/p99 latency, memory) and static compression (LoC,
files, source tokens).

## Reproducibility
- Apps and tooling run in pinned containers (`compose.yaml`, `harness/Dockerfile.toolbox`).
- PostgreSQL 16.4, shared by both apps.
- Perf tool: `oha` (pinned in the toolbox). Static tool: `tokei` (pinned).
- Canonical perf numbers come from a fixed local machine; each results run records the
  hardware. CI runs conformance + a perf smoke only.

## Honesty note
Internal results (`results/<date>/internal.md`) report every number, including where Ferro is
slower or larger. The public table is a subset, never a contradiction, of the internal data.
Rust outperforming interpreted languages on raw throughput is expected and is reported as
such, not as a finding.

## Run it
See "Running the benchmark" below (added in Task 9).
```

- [ ] **Step 4: Commit**

```bash
git add benchmark/
git commit -m "chore(benchmark): scaffold harness tree and methodology readme"
```

---

## Task 2: Write the shared contract

**Files:**
- Create: `benchmark/contracts/micro-endpoints.md`

- [ ] **Step 1: Write the contract**

````markdown
# Micro-endpoints contract (all frameworks implement this identically)

All responses `Content-Type: application/json`. Errors are out of scope (happy path only).

## GET /json
200 → `{"message":"Hello, World!"}`

## GET /db
200 → `{"id":<int 1..10000>,"randomNumber":<int>}` for one random row of `world`.

## GET /queries?n=K
`n` clamped to [1,500] (missing/invalid → 1). 200 → JSON array of K `{"id","randomNumber"}`,
each from an independent random-id lookup.

## GET /updates?n=K
`n` clamped to [1,500]. For each of K: read a random row, set `randomNumber` to a new random
int, persist. 200 → JSON array of the K updated `{"id","randomNumber"}`.

## Schema
`world(id SERIAL PRIMARY KEY, randomNumber INT NOT NULL)`, seeded 10000 rows,
`randomNumber` initialized to a random int in [1,10000].
````

- [ ] **Step 2: Commit**

```bash
git add benchmark/contracts/micro-endpoints.md
git commit -m "docs(benchmark): define micro-endpoints contract"
```

---

## Task 3: Perf parser (TDD)

**Files:**
- Create: `benchmark/harness/perf/parse_perf.py`
- Test: `benchmark/harness/perf/test_parse_perf.py`

- [ ] **Step 1: Write the failing test**

```python
# test_parse_perf.py
import json
from parse_perf import parse_oha

SAMPLE = json.dumps({
    "summary": {"requestsPerSec": 12345.6, "total": 60.0, "successRate": 1.0},
    "latencyPercentiles": {"p50": 0.0012, "p90": 0.004, "p99": 0.009},
})

def test_parse_oha_extracts_core_metrics():
    m = parse_oha(SAMPLE)
    assert m["rps"] == 12345.6
    assert m["p50_ms"] == 1.2
    assert m["p99_ms"] == 9.0
    assert m["success_rate"] == 1.0

def test_parse_oha_rejects_low_success_rate():
    bad = json.dumps({"summary": {"requestsPerSec": 1.0, "successRate": 0.5},
                      "latencyPercentiles": {"p50": 0.1, "p99": 0.2}})
    try:
        parse_oha(bad)
        assert False, "expected ValueError"
    except ValueError:
        pass
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd benchmark/harness/perf && python3 -m pytest test_parse_perf.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'parse_perf'`

- [ ] **Step 3: Write minimal implementation**

```python
# parse_perf.py
import json

def parse_oha(raw: str) -> dict:
    """Normalize `oha --json` output into a flat metrics dict (latency in ms)."""
    d = json.loads(raw)
    s = d["summary"]
    p = d["latencyPercentiles"]
    success = float(s.get("successRate", 1.0))
    if success < 0.99:
        raise ValueError(f"success rate too low to trust results: {success}")
    return {
        "rps": float(s["requestsPerSec"]),
        "p50_ms": round(float(p["p50"]) * 1000, 3),
        "p90_ms": round(float(p.get("p90", 0)) * 1000, 3),
        "p99_ms": round(float(p["p99"]) * 1000, 3),
        "success_rate": success,
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd benchmark/harness/perf && python3 -m pytest test_parse_perf.py -v`
Expected: PASS (2 passed)

- [ ] **Step 5: Commit**

```bash
git add benchmark/harness/perf/parse_perf.py benchmark/harness/perf/test_parse_perf.py
git commit -m "feat(benchmark): perf parser for oha json output"
```

---

## Task 4: Static counter (TDD)

**Files:**
- Create: `benchmark/harness/static/count_static.py`
- Test: `benchmark/harness/static/test_count_static.py`

- [ ] **Step 1: Write the failing test**

```python
# test_count_static.py
import json, subprocess
from count_static import summarize_tokei, count_tokens

TOKEI_JSON = json.dumps({
    "Rust":  {"code": 120, "comments": 10, "blanks": 5, "reports": [{"name":"a.rs"},{"name":"b.rs"}]},
    "Total": {"code": 120, "comments": 10, "blanks": 5},
})

def test_summarize_tokei_totals_code_and_files():
    s = summarize_tokei(TOKEI_JSON)
    assert s["code_lines"] == 120
    assert s["files"] == 2

def test_count_tokens_counts_whitespace_separated(tmp_path):
    f = tmp_path / "x.txt"
    f.write_text("one two three\nfour")
    assert count_tokens([str(f)]) == 4
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd benchmark/harness/static && python3 -m pytest test_count_static.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'count_static'`

- [ ] **Step 3: Write minimal implementation**

```python
# count_static.py
import json, sys, subprocess

def summarize_tokei(raw: str) -> dict:
    d = json.loads(raw)
    files, code = 0, 0
    for lang, v in d.items():
        if lang == "Total":
            continue
        code += v.get("code", 0)
        files += len(v.get("reports", []))
    return {"code_lines": code, "files": files}

def count_tokens(paths: list[str]) -> int:
    total = 0
    for p in paths:
        with open(p, encoding="utf-8", errors="replace") as fh:
            total += len(fh.read().split())
    return total

def run(app_dir: str) -> dict:
    """tokei over app_dir (excluding vendored deps) + a whitespace token count of source."""
    out = subprocess.run(
        ["tokei", "--output", "json", "--exclude", "vendor", "--exclude", "target", app_dir],
        capture_output=True, text=True, check=True,
    ).stdout
    summary = summarize_tokei(out)
    listing = subprocess.run(
        ["tokei", "--files", "--output", "json", "--exclude", "vendor", "--exclude", "target", app_dir],
        capture_output=True, text=True, check=True,
    ).stdout
    paths = [r["name"] for lang, v in json.loads(listing).items()
             if lang != "Total" for r in v.get("reports", [])]
    summary["source_tokens"] = count_tokens(paths)
    return summary

if __name__ == "__main__":
    print(json.dumps(run(sys.argv[1])))
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd benchmark/harness/static && python3 -m pytest test_count_static.py -v`
Expected: PASS (2 passed)

- [ ] **Step 5: Commit**

```bash
git add benchmark/harness/static/count_static.py benchmark/harness/static/test_count_static.py
git commit -m "feat(benchmark): static counter (tokei + token count)"
```

---

## Task 5: Report builder (TDD)

**Files:**
- Create: `benchmark/harness/report/build_tables.py`
- Test: `benchmark/harness/report/test_build_tables.py`

- [ ] **Step 1: Write the failing test**

```python
# test_build_tables.py
from build_tables import render_markdown

DATA = {
  "ferro":   {"perf": {"/json": {"rps": 200000.0, "p99_ms": 1.2}},
              "static": {"code_lines": 40, "files": 3, "source_tokens": 180}},
  "laravel": {"perf": {"/json": {"rps": 9000.0, "p99_ms": 30.0}},
              "static": {"code_lines": 70, "files": 5, "source_tokens": 360}},
}

def test_render_includes_both_frameworks_and_endpoints():
    md = render_markdown(DATA)
    assert "ferro" in md and "laravel" in md
    assert "/json" in md
    assert "200000" in md or "200,000" in md

def test_render_marks_winner_per_metric():
    md = render_markdown(DATA)
    # ferro wins rps; the table notes the ratio honestly
    assert "22.2x" in md or "22.22x" in md
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd benchmark/harness/report && python3 -m pytest test_build_tables.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'build_tables'`

- [ ] **Step 3: Write minimal implementation**

```python
# build_tables.py
import json, sys, glob, os

def render_markdown(data: dict) -> str:
    fw = list(data.keys())
    endpoints = sorted({ep for f in fw for ep in data[f]["perf"]})
    lines = ["## Raw performance (requests/sec)", "",
             "| Endpoint | " + " | ".join(fw) + " | ratio |", "|" + "---|" * (len(fw) + 2)]
    for ep in endpoints:
        rps = [data[f]["perf"][ep]["rps"] for f in fw]
        hi, lo = max(rps), min(rps)
        ratio = f"{hi/lo:.2f}x" if lo else "n/a"
        lines.append("| " + ep + " | " + " | ".join(f"{r:,.0f}" for r in rps) + f" | {ratio} |")
    lines += ["", "## Static compression", "",
              "| Metric | " + " | ".join(fw) + " |", "|" + "---|" * (len(fw) + 1)]
    for metric in ("code_lines", "files", "source_tokens"):
        vals = [str(data[f]["static"][metric]) for f in fw]
        lines.append(f"| {metric} | " + " | ".join(vals) + " |")
    return "\n".join(lines) + "\n"

def load_results(date_dir: str) -> dict:
    data = {}
    for path in glob.glob(os.path.join(date_dir, "*.json")):
        name = os.path.basename(path)[:-5]  # strip .json
        kind, fw = name.split("-", 1) if "-" in name else (name, name)
        with open(path) as fh:
            payload = json.load(fh)
        data.setdefault(fw, {})[ "perf" if kind == "perf" else "static"] = payload
    return data

if __name__ == "__main__":
    date_dir = sys.argv[1]
    md = render_markdown(load_results(date_dir))
    with open(os.path.join(date_dir, "internal.md"), "w") as fh:
        fh.write(md)
    print(md)
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd benchmark/harness/report && python3 -m pytest test_build_tables.py -v`
Expected: PASS (2 passed)

- [ ] **Step 5: Commit**

```bash
git add benchmark/harness/report/build_tables.py benchmark/harness/report/test_build_tables.py
git commit -m "feat(benchmark): markdown report builder with honest ratios"
```

---

## Task 6: Toolbox image + perf runner

**Files:**
- Create: `benchmark/harness/Dockerfile.toolbox`
- Create: `benchmark/harness/perf/run_perf.py`

- [ ] **Step 1: Write the pinned toolbox Dockerfile**

```dockerfile
# Dockerfile.toolbox — pinned load-gen + static-count tooling
FROM rust:1.88.0-slim-bookworm AS build
RUN cargo install oha --version 1.4.7 --locked \
 && cargo install tokei --version 12.1.2 --locked

FROM debian:bookworm-slim
COPY --from=build /usr/local/cargo/bin/oha /usr/local/bin/oha
COPY --from=build /usr/local/cargo/bin/tokei /usr/local/bin/tokei
RUN apt-get update && apt-get install -y --no-install-recommends python3 jq curl \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /work
```

- [ ] **Step 2: Write the perf runner**

```python
# run_perf.py — drive oha against a target URL, write normalized metrics JSON.
import subprocess, sys, json, os
sys.path.insert(0, os.path.dirname(__file__))
from parse_perf import parse_oha

def run(base_url: str, framework: str, out_dir: str,
        endpoints=("/json", "/db", "/queries?n=20", "/updates?n=20"),
        duration="30s", concurrency="256", warmup="5s"):
    perf = {}
    for ep in endpoints:
        url = base_url.rstrip("/") + ep
        # warm-up (discarded)
        subprocess.run(["oha", "-z", warmup, "-c", concurrency, "--no-tui", url],
                       capture_output=True, text=True)
        raw = subprocess.run(
            ["oha", "-z", duration, "-c", concurrency, "--no-tui", "--json", url],
            capture_output=True, text=True, check=True).stdout
        key = ep.split("?")[0]
        perf[key] = parse_oha(raw)
    os.makedirs(out_dir, exist_ok=True)
    path = os.path.join(out_dir, f"perf-{framework}.json")
    with open(path, "w") as fh:
        json.dump(perf, fh, indent=2)
    print(f"wrote {path}")

if __name__ == "__main__":
    # argv: base_url framework out_dir
    run(sys.argv[1], sys.argv[2], sys.argv[3])
```

- [ ] **Step 3: Build the toolbox image to verify it compiles the pins**

Run: `docker build -t ferro-bench-toolbox -f benchmark/harness/Dockerfile.toolbox benchmark/harness`
Expected: image builds; `docker run --rm ferro-bench-toolbox oha --version` prints `oha 1.4.7` and `... tokei --version` prints `tokei 12.1.2`.

- [ ] **Step 4: Commit**

```bash
git add benchmark/harness/Dockerfile.toolbox benchmark/harness/perf/run_perf.py
git commit -m "feat(benchmark): pinned toolbox image + perf runner"
```

---

## Task 7: Ferro micro-endpoints app

**Files:**
- Create: `benchmark/apps/ferro-micro/` (scaffold via CLI, then edit)
- Create: `benchmark/apps/ferro-micro/Dockerfile`

- [ ] **Step 1: Scaffold and reduce to the four endpoints**

Run:
```bash
cd benchmark/apps && ../../target/debug/ferro new ferro-micro --no-interaction
```
Then, using `ferro-mcp` (`list_routes`, `get_handler`, `code_templates`) or the docs to confirm
the current handler/routing API, replace the generated routes with exactly the four contract
endpoints. Verify command names against `ferro-cli/src/commands/` (e.g. migrations via
`ferro db:migrate`). Add a migration creating `world(id SERIAL PRIMARY KEY, randomNumber INT NOT NULL)`
and a seeder inserting 10000 rows.

- [ ] **Step 2: Implement the handlers**

Add a `world` model + handlers. Reference handler shape (adjust to the verified current API):

```rust
// src/controllers/bench.rs
use ferro::prelude::*;
use rand::Rng;
use crate::models::world;

#[handler]
pub async fn json() -> Response {
    Ok(json!({ "message": "Hello, World!" }))
}

#[handler]
pub async fn db(db: Db) -> Response {
    let id = rand::thread_rng().gen_range(1..=10_000);
    let row = world::Entity::find_by_id(id).one(&db).await?.unwrap();
    Ok(json!({ "id": row.id, "randomNumber": row.random_number }))
}

fn clamp(n: Option<i32>) -> i32 { n.unwrap_or(1).clamp(1, 500) }

#[handler]
pub async fn queries(db: Db, req: Request) -> Response {
    let k = clamp(req.query("n").and_then(|s| s.parse().ok()));
    let mut out = Vec::with_capacity(k as usize);
    for _ in 0..k {
        let id = rand::thread_rng().gen_range(1..=10_000);
        let row = world::Entity::find_by_id(id).one(&db).await?.unwrap();
        out.push(json!({ "id": row.id, "randomNumber": row.random_number }));
    }
    Ok(json!(out))
}

#[handler]
pub async fn updates(db: Db, req: Request) -> Response {
    let k = clamp(req.query("n").and_then(|s| s.parse().ok()));
    let mut out = Vec::with_capacity(k as usize);
    for _ in 0..k {
        let id = rand::thread_rng().gen_range(1..=10_000);
        let mut row: world::ActiveModel =
            world::Entity::find_by_id(id).one(&db).await?.unwrap().into();
        let new_n = rand::thread_rng().gen_range(1..=10_000);
        row.random_number = Set(new_n);
        let saved = row.update(&db).await?;
        out.push(json!({ "id": saved.id, "randomNumber": saved.random_number }));
    }
    Ok(json!(out))
}
```

Register the four routes (`GET /json`, `/db`, `/queries`, `/updates`) in the app's route file.

- [ ] **Step 3: Write the Dockerfile (release build, Postgres via env)**

```dockerfile
# benchmark/apps/ferro-micro/Dockerfile
FROM rust:1.88.0-slim-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libpq5 \
 && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/ferro-micro /usr/local/bin/app
ENV APP_PORT=3000
EXPOSE 3000
CMD ["app", "serve", "--host", "0.0.0.0", "--port", "3000"]
```

- [ ] **Step 4: Verify locally against Postgres**

Run (with a scratch Postgres):
```bash
docker run -d --name pg -e POSTGRES_PASSWORD=bench -e POSTGRES_DB=bench -p 5433:5432 postgres:16.4
# build + run the app pointed at DATABASE_URL=postgres://postgres:bench@localhost:5433/bench
# then:
curl -s localhost:3000/json
curl -s "localhost:3000/queries?n=3"
```
Expected: `/json` → `{"message":"Hello, World!"}`; `/queries?n=3` → array of 3 objects.

- [ ] **Step 5: Commit**

```bash
git add benchmark/apps/ferro-micro
git commit -m "feat(benchmark): ferro micro-endpoints app"
```

---

## Task 8: Laravel micro-endpoints app

**Files:**
- Create: `benchmark/apps/laravel-micro/` (minimal Laravel 11 app)
- Create: `benchmark/apps/laravel-micro/Dockerfile`

- [ ] **Step 1: Create a minimal Laravel app with the four routes**

Scaffold Laravel 11 (`composer create-project laravel/laravel laravel-micro "11.*"`), pin the
version in `composer.json`, and define the four routes in `routes/web.php` using the same
`world` schema (migration + seeder of 10000 rows). Idiomatic Eloquent:

```php
// routes/web.php
use Illuminate\Support\Facades\Route;
use App\Models\World;

Route::get('/json', fn () => response()->json(['message' => 'Hello, World!']));

Route::get('/db', function () {
    $w = World::find(random_int(1, 10000));
    return response()->json(['id' => $w->id, 'randomNumber' => $w->randomNumber]);
});

$clamp = fn ($n) => max(1, min(500, (int) ($n ?: 1)));

Route::get('/queries', function (\Illuminate\Http\Request $r) use ($clamp) {
    $k = $clamp($r->query('n'));
    $out = [];
    for ($i = 0; $i < $k; $i++) {
        $w = World::find(random_int(1, 10000));
        $out[] = ['id' => $w->id, 'randomNumber' => $w->randomNumber];
    }
    return response()->json($out);
});

Route::get('/updates', function (\Illuminate\Http\Request $r) use ($clamp) {
    $k = $clamp($r->query('n'));
    $out = [];
    for ($i = 0; $i < $k; $i++) {
        $w = World::find(random_int(1, 10000));
        $w->randomNumber = random_int(1, 10000);
        $w->save();
        $out[] = ['id' => $w->id, 'randomNumber' => $w->randomNumber];
    }
    return response()->json($out);
});
```

- [ ] **Step 2: Write the Dockerfile (php-fpm + a production server)**

```dockerfile
# benchmark/apps/laravel-micro/Dockerfile
FROM php:8.3-cli-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends libpq-dev unzip \
 && docker-php-ext-install pdo_pgsql opcache && rm -rf /var/lib/apt/lists/*
COPY --from=composer:2.7 /usr/bin/composer /usr/bin/composer
WORKDIR /app
COPY . .
RUN composer install --no-dev --optimize-autoloader
ENV APP_ENV=production
EXPOSE 8000
# octane/swoole would be faster, but the plain server keeps the comparison to stock Laravel.
CMD ["php", "artisan", "serve", "--host=0.0.0.0", "--port=8000"]
```

(Record in the README that the Laravel app runs the stock `artisan serve`; an Octane variant
is a documented Phase-2 addition so the numbers stay honest about what is being measured.)

- [ ] **Step 3: Verify locally**

Run the container against the scratch Postgres, then:
```bash
curl -s localhost:8000/json
curl -s "localhost:8000/queries?n=3"
```
Expected: identical shapes to the Ferro app.

- [ ] **Step 4: Commit**

```bash
git add benchmark/apps/laravel-micro
git commit -m "feat(benchmark): minimal laravel micro-endpoints app"
```

---

## Task 9: Conformance test + compose orchestration

**Files:**
- Create: `benchmark/contracts/conformance/test_conformance.py`
- Create: `benchmark/compose.yaml`

- [ ] **Step 1: Write the conformance test (runs against a base URL)**

```python
# test_conformance.py — BASE_URL env points at a running app; asserts the shared contract.
import os, requests

BASE = os.environ["BASE_URL"].rstrip("/")

def test_json():
    r = requests.get(f"{BASE}/json")
    assert r.headers["content-type"].startswith("application/json")
    assert r.json() == {"message": "Hello, World!"}

def test_db():
    o = requests.get(f"{BASE}/db").json()
    assert set(o) == {"id", "randomNumber"} and 1 <= o["id"] <= 10000

def test_queries_clamps():
    assert len(requests.get(f"{BASE}/queries?n=600").json()) == 500
    assert len(requests.get(f"{BASE}/queries?n=0").json()) == 1
    assert len(requests.get(f"{BASE}/queries?n=5").json()) == 5

def test_updates_returns_k():
    assert len(requests.get(f"{BASE}/updates?n=7").json()) == 7
```

- [ ] **Step 2: Write `compose.yaml`**

```yaml
services:
  db:
    image: postgres:16.4
    environment:
      POSTGRES_PASSWORD: bench
      POSTGRES_DB: bench
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 2s
      retries: 15
  ferro-micro:
    build: ./apps/ferro-micro
    environment:
      DATABASE_URL: postgres://postgres:bench@db:5432/bench
    depends_on: { db: { condition: service_healthy } }
    ports: ["3000:3000"]
  laravel-micro:
    build: ./apps/laravel-micro
    environment:
      DB_CONNECTION: pgsql
      DB_HOST: db
      DB_PORT: "5432"
      DB_DATABASE: bench
      DB_USERNAME: postgres
      DB_PASSWORD: bench
    depends_on: { db: { condition: service_healthy } }
    ports: ["8000:8000"]
```

- [ ] **Step 3: Bring it up, migrate+seed, run conformance against both apps**

Run:
```bash
cd benchmark && docker compose up -d --build
# run each app's migrate+seed (ferro: `ferro db:migrate && ferro db:seed`; laravel: `php artisan migrate --seed`)
BASE_URL=http://localhost:3000 python3 -m pytest contracts/conformance/test_conformance.py -v
BASE_URL=http://localhost:8000 python3 -m pytest contracts/conformance/test_conformance.py -v
```
Expected: 4 passed for each app (both honor the identical contract).

- [ ] **Step 4: Commit**

```bash
git add benchmark/contracts/conformance/test_conformance.py benchmark/compose.yaml
git commit -m "test(benchmark): contract conformance + compose orchestration"
```

---

## Task 10: First results run + report + README finalize

**Files:**
- Create: `benchmark/results/2026-06-15/` (raw JSON + `meta.json` + `internal.md` + `public.md`)
- Modify: `benchmark/README.md` (add "Running the benchmark")

- [ ] **Step 1: Record hardware metadata**

Create `benchmark/results/2026-06-15/meta.json` with the canonical machine spec:

```json
{
  "date": "2026-06-15",
  "hardware": {"cpu_model": "<fill from `sysctl -n machdep.cpu.brand_string` / lscpu>",
               "physical_cores": 0, "ram_gb": 0, "os": "<uname -sr>"},
  "tooling": {"oha": "1.4.7", "tokei": "12.1.2", "postgres": "16.4"},
  "load": {"duration": "30s", "concurrency": 256, "warmup": "5s"}
}
```

- [ ] **Step 2: Run perf for both apps (from the toolbox container, on the compose network)**

Run:
```bash
docker run --rm --network benchmark_default -v "$PWD/benchmark:/work" ferro-bench-toolbox \
  python3 /work/harness/perf/run_perf.py http://ferro-micro:3000 ferro /work/results/2026-06-15
docker run --rm --network benchmark_default -v "$PWD/benchmark:/work" ferro-bench-toolbox \
  python3 /work/harness/perf/run_perf.py http://laravel-micro:8000 laravel /work/results/2026-06-15
```
Expected: `perf-ferro.json` and `perf-laravel.json` written with rps/p50/p99 per endpoint.

- [ ] **Step 3: Run static counts for both app source trees**

Run:
```bash
docker run --rm -v "$PWD/benchmark:/work" ferro-bench-toolbox \
  sh -c 'python3 /work/harness/static/count_static.py /work/apps/ferro-micro > /work/results/2026-06-15/static-ferro.json'
docker run --rm -v "$PWD/benchmark:/work" ferro-bench-toolbox \
  sh -c 'python3 /work/harness/static/count_static.py /work/apps/laravel-micro > /work/results/2026-06-15/static-laravel.json'
```
Expected: `static-ferro.json` / `static-laravel.json` with code_lines, files, source_tokens.

- [ ] **Step 4: Build the tables**

Run:
```bash
python3 benchmark/harness/report/build_tables.py benchmark/results/2026-06-15
```
Expected: `internal.md` written and printed; eyeball that Ferro's rps is higher and the ratio
column is populated, and that static numbers are present for both.

- [ ] **Step 5: Derive the public table (subset of internal)**

Copy `internal.md` to `public.md` and trim to the headline rows (json + db rps, one latency
column, the three static metrics). Add the one-line honesty caveat: "Rust vs interpreted-language
throughput is expected; see internal.md for the full data including warm-up and success rates."

- [ ] **Step 6: Finalize README "Running the benchmark" section**

Append to `benchmark/README.md` the exact commands from Tasks 9–10 (compose up, migrate+seed,
conformance, perf, static, report) so a third party can reproduce a run.

- [ ] **Step 7: Commit**

```bash
git add benchmark/results/2026-06-15 benchmark/README.md
git commit -m "feat(benchmark): first micro-endpoints results (ferro vs laravel) + run docs"
```

---

## Self-review notes

- **Spec coverage:** static axis (Tasks 4, 10) ✓; raw-perf axis (Tasks 3, 6, 10) ✓; reproducibility — pinned containers/versions + committed raw data + recorded hardware (Tasks 6, 9, 10) ✓; honesty guardrails — internal-superset/public-subset + "expected, not a finding" caveat (Tasks 1, 5, 10) ✓; "don't author the competitor" — bound to Conduit in 1B, micro-Laravel exception documented (defaults §2, Task 8) ✓. Conduit, projection slice, and the authoring-efficiency axis are explicitly **out of 1A** (deferred to 1B/1C/Phase 3) — that is the decomposition, not a gap.
- **Placeholders:** the only intentional fill-in is the hardware values in `meta.json` Step 1, which are machine-specific and captured at run time by the documented commands — not a design gap.
- **Type/name consistency:** `parse_oha` → keys `rps/p50_ms/p99_ms/success_rate` consumed unchanged by `run_perf.py` and `build_tables.py`; `count_static.run` emits `code_lines/files/source_tokens` consumed unchanged by `build_tables.py`; result filenames `perf-<fw>.json`/`static-<fw>.json` match `load_results` parsing.
- **Open risk to flag at execution:** the Ferro handler code in Task 7 is written against the documented API shape and **must be verified against the live API via `ferro-mcp`** before relying on it (extractor names like `Db`, `req.query`, `ActiveModel`/`Set` may differ). The conformance test (Task 9) is the real gate; treat Task 7 code as a starting point, not gospel.
