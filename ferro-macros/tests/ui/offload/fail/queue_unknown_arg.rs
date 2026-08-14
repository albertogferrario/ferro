//! Compile-fail: #[offload] with an unrecognized argument (D-04 negative gate).
//!
//! The macro must reject any argument other than `queue = "name"` and emit:
//!   unknown #[offload] argument; expected `queue = "name"`
//!
//! This fixture uses `retries = 3`, which is not a recognized #[offload] argument.
//! Plan 02 regenerates the matching `.stderr` snapshot via:
//!   TRYBUILD=overwrite cargo test -p ferro-macros --test offload_macro
#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;
extern crate serde;

use ferro::{async_trait, service};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Month(pub u32);

#[derive(Default)]
pub struct ReportBuilder;

#[service(ReportBuilder)]
#[async_trait]
pub trait Reports {
    #[offload(retries = 3)]
    async fn build_monthly(&self, month: Month);
}

#[async_trait]
impl Reports for ReportBuilder {
    async fn build_monthly(&self, _month: Month) {}
}

fn main() {}
