## Raw performance (requests/sec)

| Endpoint | laravel-fpm | laravel-octane | ferro | fpm ratio | octane ratio |
|---|---|---|---|---|---|
| /json | 620 | 1,393 | 211,704 | 341.24x | 151.97x |
| /db | 487 | 1,706 | 11,001 | 22.59x | 6.45x |
| /queries | 451 | 1,092 | 1,043 | 2.31x | 0.96x |
| /updates | 239 | 399 | 486 | 2.04x | 1.22x |

## Latency percentiles (ms)

| Endpoint | fpm p50 | fpm p99 | octane p50 | octane p99 | ferro p50 | ferro p99 |
|---|---|---|---|---|---|---|
| /json | 395.5 | 569.1 | 154.1 | 550.9 | 0.9 | 5.0 |
| /db | 507.8 | 999.9 | 130.8 | 343.3 | 11.9 | 151.3 |
| /queries | 547.7 | 903.4 | 230.9 | 317.7 | 202.0 | 668.7 |
| /updates | 1061.9 | 1728.6 | 655.6 | 944.2 | 525.3 | 612.7 |

## Server config

- **laravel-fpm**: php-fpm 8.3 + nginx (supervisord), pm=static, pm.max_children=20, pm.max_requests=1000, opcache on (validate_timestamps=0)
- **laravel-octane**: Laravel Octane 2.17 + RoadRunner v2025.1.14, workers=16 (static), opcache on, pcntl+sockets, php:8.3-cli-bookworm, no nginx
- **ferro**: tokio multi-threaded runtime, --release build

## Static compression

| Metric | laravel | ferro |
|---|---|---|
| code_lines | 1448 | 344 |
| source_tokens | 8976 | 1158 |

## Conformance

All three variants: 4/4 (test_json, test_db, test_queries_clamps, test_updates_returns_k).
