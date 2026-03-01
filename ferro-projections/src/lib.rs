//! Service projection definitions for the Ferro framework.

mod action;
mod derive;
mod error;
mod field;
mod intent;
mod relationship;
pub mod render;
mod service;
mod state;

pub use action::{ActionDef, GuardDef, InputDef};
pub use derive::derive_intents;
pub use error::Error;
pub use field::{infer_meaning, DataType, FieldDef, FieldMeaning};
pub use intent::{Intent, IntentHint, IntentScore};
pub use relationship::{Cardinality, NavigationHint, RelationshipDef};
pub use render::json_ui::JsonUiRenderer;
pub use render::{RenderContext, RenderMode, Renderer};
pub use service::ServiceDef;
pub use state::{StateDef, StateMachine, Transition, Warning};
