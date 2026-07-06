//! Compile-pass: minimal `#[resource_get]` fixture.
//!
//! Defines a local model + TenantScoped impl. The macro folds id-extraction,
//! tenant resolution, tenant-scoped lookup, and 404-on-miss into one attribute;
//! the user body receives real typed params.

#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;

use ferro::{async_trait, resource_get, ActionResult, Request, Response, TenantContext, TenantScoped};

#[derive(Clone)]
struct Model {
    pub id: i64,
    pub name: String,
}

#[async_trait]
impl TenantScoped for Model {
    type Id = i64;

    async fn find_for_tenant(
        id: i64,
        _tenant_id: i64,
    ) -> Result<Option<Self>, ferro::FrameworkError> {
        // Minimal stub: always miss so no DB needed
        Ok(None)
    }
}

#[resource_get(Model, on_miss = "/list")]
pub async fn edit(req: &mut Request, tenant: &TenantContext, model: &Model) -> Response {
    let _ = (req, tenant, model);
    Ok(ferro::HttpResponse::new())
}

fn main() {}
