## Performance headline (requests/sec)

| Endpoint | laravel | ferro | ratio |
|---|---|---|---|
| /json | 98 | 148,852 | 1517.59x |
| /db | 74 | 16,571 | 225.28x |

p99 latency (ferro / laravel): /json 8.8ms / 5142.8ms — /db 43.0ms / 7849.3ms

## Static compression

| Metric | laravel | ferro |
|---|---|---|
| code_lines | 1427 | 344 |
| files | 44 | 14 |
| source_tokens | 8874 | 1158 |

---

> Rust vs interpreted-language throughput is expected; see internal.md for the
> full data including warm-up and success rates.

All numbers above are a strict subset of `internal.md` — no value here contradicts
the full internal report. Measured on Apple M1 Pro (8 cores, 16 GB RAM, macOS Darwin
23.6.0) with oha 1.9.0 at c=256, 30s timed run after 5s warmup. PostgreSQL 16.4
shared by both apps. Stock `artisan serve` (no Octane); Ferro release build.
