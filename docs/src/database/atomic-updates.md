# Atomic Updates

The `ferro-orm` crate exposes `GuardedUpdate<E>`, a typed builder that compiles to a single `UPDATE … WHERE …` SQL statement. It replaces the hand-rolled `read → check → write` pattern wherever a column's value is conditionally mutated — counter decrements, status transitions, optimistic concurrency checks. The database engine's per-statement atomicity (SQLite serial writer, Postgres `READ COMMITTED`) is the entire correctness mechanism; `GuardedUpdate` adds the chainable surface and the rows-affected → `GuardedError` mapping on top.

## The Anti-Pattern: `read → check → write`

A typical conditional update without `GuardedUpdate` looks like this:

```rust,ignore
// Anti-pattern — do not write this.
let unit = inventory_units::Entity::find_by_id(unit_id)
    .one(&conn)
    .await?
    .ok_or(Error::NotFound)?;

if unit.quantity < needed {
    return Err(Error::InsufficientCapacity);
}

let mut active: inventory_units::ActiveModel = unit.into();
active.quantity = Set(active.quantity.unwrap() - needed);
active.update(&conn).await?;
```

Two concurrent callers can both pass the `if unit.quantity < needed` check before either writes. Both then write the decrement, and capacity is exceeded. The read-check-write round trip leaves a race window between the SELECT and the UPDATE.

## The Replacement: `GuardedUpdate`

The same operation as a single statement:

```rust,ignore
use ferro_orm::{GuardedUpdate, ColumnTrait};
use sea_orm::sea_query::Expr;

GuardedUpdate::new(inventory_units::Entity)
    .filter(inventory_units::Column::Id.eq(unit_id))
    .filter(inventory_units::Column::Quantity.gte(needed))
    .set_expr(
        inventory_units::Column::Quantity,
        Expr::col(inventory_units::Column::Quantity).sub(needed),
    )
    .exec_one(&conn)
    .await?;
```

One round trip, one statement. The database atomically tests both `id = unit_id` and `quantity >= needed`; if both hold, the row is decremented in the same statement. If either fails, `exec_one` returns `Err(GuardedError::NoRowsAffected)` — capacity is exhausted or the row no longer exists.

## API

### `GuardedUpdate::new(entity)`

Construct an empty builder targeting a SeaORM entity.

### `.filter(condition)`

Add a filter expression. Multiple `.filter(...)` calls are AND-combined onto an internal `Condition`. Accepts anything implementing `IntoCondition` — typically `Column::field.eq(value)` or any `SimpleExpr`.

### `.set_expr(column, expression)` and `.set_value(column, value)`

Set a column. `set_expr` takes a `SimpleExpr` (for value-derived updates such as `Expr::col(Column::Quantity).sub(1)`); `set_value` takes a `Value` (for literal assignments). Both are chainable; multiple calls set multiple columns in one statement. Insertion order is preserved; later sets to the same column override earlier ones.

### `.exec_one(&conn)` vs `.exec_at_most_one(&conn)`

Both methods compile and execute the `UPDATE` against a `&impl ConnectionTrait` (works equally with `&DatabaseConnection` or `&DatabaseTransaction`):

| Outcome | `exec_one` | `exec_at_most_one` |
|---------|------------|--------------------|
| 1 row matched | `Ok(())` | `Ok(true)` |
| 0 rows matched | `Err(NoRowsAffected)` | `Ok(false)` |
| >1 rows matched | `Err(TooManyRows { affected })` | `Err(TooManyRows { affected })` |
| DB error | `Err(Db(_))` | `Err(Db(_))` |
| Empty builder (no `set_*` calls) | `Err(EmptyUpdate)` | `Err(EmptyUpdate)` |

Use `exec_one` when predicate failure is the operative signal (capacity exhausted, pre-condition unmet) and should surface as an error. Use `exec_at_most_one` when predicate failure is a normal outcome that should not pollute error logs (e.g., refreshing a session that may have expired).

## Common Patterns

### Counter decrement

The canonical race-free counter decrement (inventory, queue capacity, rate limit budget):

```rust,ignore
GuardedUpdate::new(counters::Entity)
    .filter(counters::Column::Id.eq(counter_id))
    .filter(counters::Column::Quantity.gte(needed))
    .set_expr(
        counters::Column::Quantity,
        Expr::col(counters::Column::Quantity).sub(needed),
    )
    .exec_one(&conn)
    .await?;
```

### Status transition

Atomic state machine transition with multi-column update — set the new status and the timestamp in the same statement:

```rust,ignore
use ferro_orm::Value;
use chrono::Utc;

GuardedUpdate::new(reservations::Entity)
    .filter(reservations::Column::Id.eq(handle_id))
    .filter(reservations::Column::Status.eq("held"))
    .set_value(
        reservations::Column::Status,
        Value::String(Some(Box::new("committed".into()))),
    )
    .set_value(
        reservations::Column::CommittedAt,
        Value::ChronoDateTimeUtc(Some(Box::new(Utc::now()))),
    )
    .exec_one(&conn)
    .await?;
```

### Optimistic update (predicate failure is normal)

Refresh a session's `last_seen_at` if the token is valid and not expired. If the session expired between issue and now, this is a normal outcome, not an error log:

```rust,ignore
let updated = GuardedUpdate::new(sessions::Entity)
    .filter(sessions::Column::Token.eq(&token))
    .filter(sessions::Column::ExpiresAt.gt(now))
    .set_value(
        sessions::Column::LastSeenAt,
        Value::ChronoDateTimeUtc(Some(Box::new(now))),
    )
    .exec_at_most_one(&conn)
    .await?;

if !updated {
    return Err(AuthError::SessionExpired);
}
```

## Atomicity Guarantee (and Its Limit)

`GuardedUpdate` guarantees atomicity *per statement*, not *per builder*. The single `UPDATE … WHERE …` SQL statement is atomic at the database engine level — SQLite's serial writer and Postgres's `READ COMMITTED` semantics both ensure that the predicate test and the row mutation cannot interleave with another concurrent statement on the same row.

However, a caller building `.set_expr(qty - 1)` and then reading the resulting `qty` value in a *separate* query without a transaction re-introduces a race window between the two queries. The crate's responsibility is to make the conditional UPDATE itself race-free; bracketing it in a transaction (when post-update inspection is required) is the caller's responsibility.

If the post-update row state is needed, wrap the `GuardedUpdate` and the subsequent SELECT in the same `DatabaseTransaction`:

```rust,ignore
use sea_orm::TransactionTrait;

let txn = conn.begin().await?;
GuardedUpdate::new(counters::Entity)
    .filter(counters::Column::Id.eq(1))
    .set_expr(
        counters::Column::Quantity,
        Expr::col(counters::Column::Quantity).sub(1),
    )
    .exec_one(&txn)
    .await?;
let row = counters::Entity::find_by_id(1).one(&txn).await?;
txn.commit().await?;
```

## Errors

| Variant | When | Notes |
|---------|------|-------|
| `NoRowsAffected` | Predicate matched zero rows | The operative "capacity exhausted" / "pre-condition unmet" signal for `exec_one` |
| `TooManyRows { affected }` | Predicate matched more than one row | Indicates the filter is not unique-key-equivalent — typically an index or uniqueness bug at the call site |
| `EmptyUpdate` | `exec_*` called with no `set_*` calls | Programming error; surfaces immediately without touching the database |
| `Db(sea_orm::DbErr)` | Underlying SeaORM error | Connection failure, constraint violation, deadlock, etc. |

Every variant's `Display` impl is prefixed `"guarded: "` for log greppability — matches the workspace convention used by other ferro crate error types (e.g. `"wallet: "`, `"config: "`).

## Postgres vs SQLite

`GuardedUpdate` is dialect-agnostic; the SQL it emits is standard `UPDATE … WHERE …`. SQLite enforces per-statement atomicity via its serial writer; Postgres enforces it via `READ COMMITTED` (the default isolation level) — both are sufficient for the contract.

`UPDATE … RETURNING` is not currently supported; SeaORM does not yet abstract it cleanly across dialects. Callers needing the post-update row should re-fetch inside the same transaction (see above).
