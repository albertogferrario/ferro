//! Storage drivers.

mod local;
mod memory;

pub use local::LocalDriver;
pub use memory::MemoryDriver;

#[cfg(feature = "s3")]
mod s3;
#[cfg(feature = "s3")]
pub use s3::S3Driver;
