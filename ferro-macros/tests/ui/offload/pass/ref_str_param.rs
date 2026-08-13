//! Compile-pass: &str parameter maps to String field in derived struct (OFFLOAD-01-b).
#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;
extern crate serde;

use ferro::{async_trait, service};

#[derive(Default)]
pub struct Greeter;

#[service(Greeter)]
#[async_trait]
pub trait GreeterService {
    #[offload]
    async fn greet(&self, name: &str);
}

#[async_trait]
impl GreeterService for Greeter {
    async fn greet(&self, _name: &str) {}
}

fn main() {
    // Field is String, proving the &str → String mapping (OFFLOAD-01-b).
    let _job = GreeterServiceGreetJob { name: String::from("x") };
}
