---
name: ferro:db
description: Database operations (migrate, rollback, seed, schema)
allowed-tools:
  - Bash
  - Read
  - Glob
---

<objective>
Execute database operations for Ferro applications.

Subcommands:
- `migrate` - Run pending migrations
- `rollback` - Rollback last migration(s)
- `seed` - Run database seeders
- `fresh` - Drop all tables and re-migrate
- `schema` - Show database schema
- `sync` - Sync models with database (generate entities)
</objective>

<arguments>
Required:
- `<subcommand>` - One of: migrate, rollback, seed, fresh, schema, sync

Optional (varies by subcommand):
- `--step=N` - Number of migrations to rollback (rollback only)
- `--seed` - Run seeders after fresh (fresh only)
- `--force` - Force operation in production

Examples:
- `/ferro:db migrate`
- `/ferro:db rollback --step=2`
- `/ferro:db fresh --seed`
- `/ferro:db schema`
- `/ferro:db sync`
</arguments>

<process>

<step name="parse_subcommand">

Parse the subcommand and route to appropriate handler.

</step>

<step name="migrate">

**`/ferro:db migrate`**

Run pending migrations:

```bash
cargo run -- db:migrate
```

Or using ferro CLI directly:
```bash
ferro db:migrate
```

Show result:
```
Running migrations...

✓ 20240115120000_create_users_table
✓ 20240115120100_create_posts_table
✓ 20240116090000_add_avatar_to_users

3 migrations completed.
```

</step>

<step name="rollback">

**`/ferro:db rollback [--step=N]`**

Rollback migrations:

```bash
cargo run -- db:rollback {--step=N}
```

Default rolls back last batch. With `--step=N`, rolls back N migrations.

```
Rolling back migrations...

✓ Rolled back: 20240116090000_add_avatar_to_users

1 migration rolled back.
```

</step>

<step name="seed">

**`/ferro:db seed`**

Run database seeders:

```bash
cargo run -- db:seed
```

```
Running seeders...

✓ UsersSeeder: 10 records
✓ PostsSeeder: 50 records

Seeding complete.
```

</step>

<step name="fresh">

**`/ferro:db fresh [--seed]`**

Drop all tables and re-run all migrations:

```bash
cargo run -- db:fresh {--seed}
```

⚠️ This is destructive! Confirm before running:

"This will DROP ALL TABLES. Are you sure? (y/N)"

```
Dropping all tables...
Running migrations...

✓ 20240115120000_create_users_table
✓ 20240115120100_create_posts_table
✓ 20240116090000_add_avatar_to_users

{if --seed}
Running seeders...
✓ UsersSeeder: 10 records
✓ PostsSeeder: 50 records
{/if}

Database refreshed.
```

</step>

<step name="schema">

**`/ferro:db schema`**

Show database schema using ferro-mcp `database_schema` tool.

If MCP unavailable, query database directly:

```bash
# For SQLite
sqlite3 database.sqlite ".schema"

# For PostgreSQL
psql $DATABASE_URL -c "\dt+" -c "\d+ users"
```

Display as formatted table:
```
# Database Schema

## users
┌─────────────┬──────────────┬──────────┬─────────┐
│ Column      │ Type         │ Nullable │ Default │
├─────────────┼──────────────┼──────────┼─────────┤
│ id          │ INTEGER      │ NO       │ AUTO    │
│ email       │ VARCHAR(255) │ NO       │         │
│ name        │ VARCHAR(255) │ NO       │         │
│ password    │ VARCHAR(255) │ NO       │         │
│ created_at  │ TIMESTAMP    │ NO       │ NOW()   │
│ updated_at  │ TIMESTAMP    │ NO       │ NOW()   │
└─────────────┴──────────────┴──────────┴─────────┘

Indexes: PRIMARY (id), UNIQUE (email)

## posts
...
```

</step>

<step name="sync">

**`/ferro:db sync`**

Sync models with database - generates SeaORM entities:

```bash
cargo run -- db:sync
```

This:
1. Reads current database schema
2. Generates entity files in `src/models/entities/`
3. Updates `src/models/entities/mod.rs`

```
Syncing database entities...

✓ Generated: src/models/entities/users.rs
✓ Generated: src/models/entities/posts.rs
✓ Updated: src/models/entities/mod.rs

Sync complete. 2 entities generated.
```

</step>

</process>

<tips>
- Always backup before running `fresh` in non-dev environments
- Use `--force` flag carefully - it bypasses production safeguards
- Run `db:sync` after adding new migrations to regenerate entities
- Check `database_schema` MCP tool for detailed schema info
</tips>
