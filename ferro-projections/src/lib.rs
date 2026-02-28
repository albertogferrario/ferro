//! Service projection definitions for the Ferro framework.

mod error;
mod field;

pub use error::Error;
pub use field::{infer_meaning, DataType, FieldDef, FieldMeaning};
