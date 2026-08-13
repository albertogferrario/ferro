//! Compile-fail: #[offload] on a method with a &mut parameter (OFFLOAD-01-c).
#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;
extern crate serde;

use ferro::{async_trait, service};

#[derive(Default)]
pub struct Svc;

#[service(Svc)]
#[async_trait]
pub trait MyService {
    #[offload]
    async fn mutate(&self, data: &mut String);
}

fn main() {}
