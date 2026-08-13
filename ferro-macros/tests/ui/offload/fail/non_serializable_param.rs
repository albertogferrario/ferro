//! Compile-fail: #[offload] param type is not Serialize/DeserializeOwned (OFFLOAD-02b).
#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;
extern crate serde;

use ferro::{async_trait, service};
use serde::{Deserialize, Serialize};

// NOT serializable — no derive.
pub struct RawHandle;

#[derive(Default)]
pub struct Svc;

#[service(Svc)]
#[async_trait]
pub trait MyService {
    #[offload]
    async fn process(&self, handle: RawHandle);
}

fn main() {}
