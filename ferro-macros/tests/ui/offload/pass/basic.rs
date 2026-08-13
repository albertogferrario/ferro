//! Compile-pass: minimal #[offload] on an async trait method (OFFLOAD-01-a).
#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;
extern crate serde;

use ferro::{async_trait, service};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Month(pub u32);

#[derive(Default)]
pub struct ReportBuilder;

// Note: positional syntax used here because `impl` is a keyword that cannot be
// parsed as an Ident; the positional form is the backwards-compatible path.
#[service(ReportBuilder)]
#[async_trait]
pub trait Reports {
    #[offload]
    async fn build_monthly(&self, month: Month);
}

#[async_trait]
impl Reports for ReportBuilder {
    async fn build_monthly(&self, _month: Month) {}
}

fn main() {
    // The derived struct is public and nameable (proves OFFLOAD-01-a emitted it).
    let _job = ReportsBuildMonthlyJob { month: Month(1) };
}
