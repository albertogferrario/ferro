//! Compile-fail: #[offload] param is &&T (double reference) — WR-01.
#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;
extern crate serde;

use ferro::{async_trait, service};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inner;

#[derive(Default)]
pub struct Svc;

#[service(Svc)]
#[async_trait]
pub trait MyService {
    #[offload]
    async fn process(&self, data: &&Inner);
}

fn main() {}
