//! Cache store implementations.

#[cfg(feature = "memory")]
mod memory;
#[cfg(feature = "memory")]
pub use memory::MemoryStore;

#[cfg(feature = "redis-backend")]
mod redis;
#[cfg(feature = "redis-backend")]
pub use redis::RedisStore;
