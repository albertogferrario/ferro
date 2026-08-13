//! Compile-pass: .offload() on a derived Job returns OffloadHandle<Output> (OFFLOAD-02a).
#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;
extern crate serde;

use ferro::queue::{OffloadHandle, Offloadable};
use ferro::{async_trait, service};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub total: i64,
}

#[derive(Default)]
pub struct Reports;

#[service(Reports)]
#[async_trait]
pub trait ReportsService {
    #[offload]
    async fn build_monthly(&self, tenant_id: i64) -> Report;
}

#[async_trait]
impl ReportsService for Reports {
    async fn build_monthly(&self, _tenant_id: i64) -> Report {
        Report { total: 0 }
    }
}

fn main() {
    // Structural proof: the derived Job implements Offloadable with Output = Report,
    // so `.offload()` returns OffloadHandle<Report>. Verified at compile time —
    // no runtime dispatch (no App/DB needed).
    fn assert_output_is_report<J: Offloadable<Output = Report>>() {}
    assert_output_is_report::<ReportsServiceBuildMonthlyJob>();

    // And the handle type is what .offload() yields:
    fn _handle_ty(h: OffloadHandle<Report>) -> OffloadHandle<Report> {
        h
    }
}
