//! Compile-fail: `#[resource_post]` with no `redirect_to` — required arg is missing.

#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;

use ferro::{async_trait, resource_post, ActionResult, Request, TenantContext, TenantScoped};

#[derive(Clone)]
struct Model {
    pub id: i64,
}

#[async_trait]
impl TenantScoped for Model {
    type Id = i64;

    async fn find_for_tenant(
        id: i64,
        _tenant_id: i64,
    ) -> Result<Option<Self>, ferro::FrameworkError> {
        Ok(None)
    }
}

#[resource_post(Model, form_url = "/x")]
pub async fn save(req: &mut Request, tenant: &TenantContext, model: &Model) -> ActionResult {
    let _ = (req, tenant, model);
    Ok(())
}

fn main() {}
