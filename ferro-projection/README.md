# ferro-projection

Live read-model runtime: subscribe to domain events, persist per-key
snapshots, broadcast deltas.

**Not the same as `ferro-projections` (plural).** That crate is the
Service Projection abstraction (`ServiceDef → IntentGraph →
JsonUiRenderer`). This crate (`ferro-projection`, singular) is the
live read-model runtime described above. The two abstractions are
orthogonal — most apps will use both for different reasons. See the
crate docs for the full distinction.

Status: part of the [ferro](https://github.com/albertogferrario/ferro)
framework workspace.

Documentation: https://docs.rs/ferro-projection

License: MIT
