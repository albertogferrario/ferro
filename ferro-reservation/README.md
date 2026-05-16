# ferro-reservation

Generic hold/commit/release resource reservation kernel for the Ferro framework.

The crate exposes `ReservationKernel<R: Resource>` with `hold` / `commit` / `release` / `extend` / `run_sweep_once` — a typed, race-free state-transition pipeline for any capacity-constrained resource. Consumers implement the `Resource` trait against their own domain model; the kernel composes `ferro-orm::GuardedUpdate` for atomic state changes, emits `ReservationEvent` via `ferro-events`, and writes structured before/after entries to `ferro-audit` on every transition. Ships a SeaORM migration consumers register in their own `Migrator`.

Status: part of the [ferro](https://github.com/albertogferrario/ferro) framework workspace.

Documentation: https://docs.rs/ferro-reservation

License: MIT
