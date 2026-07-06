//! Compile-fail: unterminated `{id` placeholder (missing `}`) in `on_miss`.
//!
//! WR-03 regression fixture: before the fix, `/x/{id` was silently accepted
//! and the malformed format string was passed down to `format!` in generated
//! code, producing a confusing diagnostic far from the attribute.

#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;

use ferro::{async_trait, resource_get, Request, Response, TenantContext, TenantScoped};

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

#[resource_get(Model, on_miss = "/x/{id")]
pub async fn edit(req: &mut Request, tenant: &TenantContext, model: &Model) -> Response {
    let _ = (req, tenant, model);
    Ok(ferro::HttpResponse::new())
}

fn main() {}
