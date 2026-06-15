## Raw performance (requests/sec)

| Endpoint | laravel | ferro | ratio |
|---|---|---|---|
| /db | 74 | 16,571 | 225.28x |
| /json | 98 | 148,852 | 1517.59x |
| /queries | 66 | 658 | 9.96x |
| /updates | 49 | 333 | 6.78x |

## Static compression

| Metric | laravel | ferro |
|---|---|---|
| code_lines | 1427 | 344 |
| files | 44 | 14 |
| source_tokens | 8874 | 1158 |
