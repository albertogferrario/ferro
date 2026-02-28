//! Service projection definitions for the Ferro framework.

mod error;
mod field;
mod service;

pub use error::Error;
pub use field::{infer_meaning, DataType, FieldDef, FieldMeaning};
pub use service::ServiceDef;
