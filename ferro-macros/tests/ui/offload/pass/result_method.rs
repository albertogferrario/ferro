//! Compile-pass: Result<T, E> return type on an #[offload] method (OFFLOAD-01-b/D-06).
#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;
extern crate serde;

use ferro::{async_trait, service};

#[derive(Default)]
pub struct Exporter;

#[service(Exporter)]
#[async_trait]
pub trait ExporterService {
    #[offload]
    async fn export(&self, id: i64) -> Result<(), String>;
}

#[async_trait]
impl ExporterService for Exporter {
    async fn export(&self, _id: i64) -> Result<(), String> {
        Ok(())
    }
}

fn main() {
    // Derived struct carries the i64 field; Result branch in handle() is verified
    // by compilation (the impl job handles the Result<(), String> return).
    let _job = ExporterServiceExportJob { id: 7 };
}
