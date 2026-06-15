## Raw performance (requests/sec)

| Endpoint | laravel | ferro | ratio |
|---|---|---|---|
| /db | 487 | 11,001 | 22.59x |
| /json | 620 | 211,704 | 341.24x |
| /queries | 451 | 1,043 | 2.31x |
| /updates | 239 | 486 | 2.04x |

## Static compression

| Metric | laravel | ferro |
|---|---|---|
| code_lines | 1448 | 344 |
| source_tokens | 8976 | 1158 |
