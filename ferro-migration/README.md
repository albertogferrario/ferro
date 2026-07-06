# ferro-migration

Backend-portable migration helpers for ferro applications using SeaORM.

## Example

```rust
use ferro_migration::backfill_random_hex;
use sea_orm_migration::prelude::*;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ... add column ...
        backfill_random_hex(manager, "bookings", "checkin_token", 16).await?;
        Ok(())
    }
}
```

## Postgres backend note

`backfill_random_hex` on Postgres uses `encode(gen_random_bytes(N), 'hex')` which
requires the `pgcrypto` extension. DigitalOcean App Platform managed Postgres has
`pgcrypto` enabled by default. Self-hosted Postgres may need:

```sql
CREATE EXTENSION IF NOT EXISTS pgcrypto;
```

`backfill_random_uuid` on Postgres uses `gen_random_uuid()::text`, which is available
in Postgres 13+ without any extension.

## Available helpers

- `backfill_random_hex(manager, table, column, hex_len)` — fills NULL/empty rows with a random hex string of `hex_len` characters.
- `backfill_random_uuid(manager, table, column)` — fills NULL/empty rows with a random UUID v4 string.
- `backfill_current_timestamp(manager, table, column)` — fills NULL rows with the current timestamp.
- `backfill(manager, sql_fn)` — escape hatch; caller provides a closure returning backend-specific SQL.

SQLite and Postgres are supported. MySQL returns an error.
