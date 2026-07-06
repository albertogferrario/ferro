# ferro-orm

Atomic conditional updates and ORM primitives for the Ferro framework.

The crate exposes `GuardedUpdate<E>` — a chainable builder that compiles to a single `UPDATE … WHERE …` SQL statement, replacing the hand-rolled `read → check → write` pattern wherever a column's value is conditionally mutated. The database engine's per-statement atomicity (SQLite serial writer, Postgres `READ COMMITTED`) is the entire correctness mechanism; `GuardedUpdate` adds the chainable surface and the rows-affected → `GuardedError` mapping on top.

Status: part of the [ferro](https://github.com/albertogferrario/ferro) framework workspace.

Documentation: https://docs.rs/ferro-orm

License: MIT
