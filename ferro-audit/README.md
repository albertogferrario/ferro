# ferro-audit

Append-only structured before/after audit log for the Ferro framework.

The crate exposes `AuditEntry::record(action).…write(&conn).await?` — a chainable builder that persists a typed `(AuditActor, AuditTarget, before, after, …)` row to an `audit_log` table, and pairs with `history_for_target(&target, &conn)` + `reconstruct_state(&entries)` for forensic replay. Ships a SeaORM migration consumers register in their own `Migrator`.

Status: part of the [ferro](https://github.com/albertogferrario/ferro) framework workspace.

Documentation: https://docs.rs/ferro-audit

License: MIT
