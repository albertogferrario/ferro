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

## Running the benchmark

### Prerequisites
- Docker (with Compose v2)
- Python 3 with `requests` and `pytest` (`pip install requests pytest`)

### 1. Build and start the stack

```bash
cd benchmark
docker compose up -d --build
```

This pulls postgres:16.4 and builds both app images. The `db` healthcheck gates
app startup — both apps will start only when Postgres is ready.

### 2. Migrate and seed each app

```bash
# Ferro — migrations and seed (10 000 world rows)
docker compose exec ferro-micro app db:migrate
docker compose exec ferro-micro app db:seed

# Laravel — migrations only (world table already seeded by Ferro above)
docker compose exec laravel-micro php artisan migrate --force
```

### 3. Verify both apps pass the contract (acceptance gate)

```bash
# Ferro app is mapped to host port 3001 (container port 3000)
BASE_URL=http://localhost:3001 python3 -m pytest contracts/conformance/test_conformance.py -v

# Laravel app is mapped to host port 8001 (container port 8000)
BASE_URL=http://localhost:8001 python3 -m pytest contracts/conformance/test_conformance.py -v
```

Both must report **4 passed**. If any test fails, the app has a contract bug — fix
the app, do not weaken the test.

### 4. Run load tests (sequential — never concurrent)

```bash
DATE=$(date +%F)
NETWORK=$(docker network ls --filter name=benchmark --format '{{.Name}}' | head -1)

# Ferro first
docker run --rm --network "$NETWORK" -v "$PWD:/work" ferro-bench-toolbox \
  python3 /work/harness/perf/run_perf.py http://ferro-micro:3000 ferro /work/results/$DATE

# Laravel second (after Ferro completes)
docker run --rm --network "$NETWORK" -v "$PWD:/work" ferro-bench-toolbox \
  python3 /work/harness/perf/run_perf.py http://laravel-micro:8000 laravel /work/results/$DATE
```

Writes `results/$DATE/perf-ferro.json` and `results/$DATE/perf-laravel.json`.
Load parameters: 30s timed run, c=256 concurrency, 5s warmup (discarded).

### 5. Static line counts

```bash
docker run --rm -v "$PWD:/work" ferro-bench-toolbox \
  sh -c 'python3 /work/harness/static/count_static.py /work/apps/ferro-micro > /work/results/$DATE/static-ferro.json'

docker run --rm -v "$PWD:/work" ferro-bench-toolbox \
  sh -c 'python3 /work/harness/static/count_static.py /work/apps/laravel-micro > /work/results/$DATE/static-laravel.json'
```

### 6. Build the internal report table

```bash
python3 harness/report/build_tables.py results/$DATE
# Writes results/$DATE/internal.md
```

### 7. Derive the public table

Copy `internal.md` to `public.md`, trim to headline rows, and append the honesty
caveat: _"Rust vs interpreted-language throughput is expected; see internal.md for the
full data including warm-up and success rates."_

### 8. Record hardware metadata

Write `results/$DATE/meta.json` with the actual machine values:

```bash
sysctl -n machdep.cpu.brand_string  # cpu_model
sysctl -n hw.physicalcpu            # physical_cores
sysctl -n hw.memsize                # bytes → ÷1073741824 = GB
uname -sr                           # os
```

See `results/2026-06-15/meta.json` for the schema.

### Re-running from scratch

```bash
docker compose down -v   # remove volumes (drops the DB)
# then repeat from step 1
```
