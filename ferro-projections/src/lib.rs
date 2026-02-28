//! Service projection definitions for the Ferro framework.

mod action;
mod error;
mod field;
mod relationship;
mod service;
mod state;

pub use action::{ActionDef, GuardDef, InputDef};
pub use error::Error;
pub use field::{infer_meaning, DataType, FieldDef, FieldMeaning};
pub use relationship::{Cardinality, NavigationHint, RelationshipDef};
pub use service::ServiceDef;
pub use state::{StateDef, StateMachine, Transition, Warning};
