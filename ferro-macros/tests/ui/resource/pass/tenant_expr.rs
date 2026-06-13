//! Compile-pass: `tenant = "expr"` escape hatch on both macros.
//!
//! Exercises the D-02 escape-hatch path — consumer supplies an owned TenantContext
//! expression rather than relying on `current_tenant()`. The macro must bind
//! `let __tenant = { expr };` (no explicit type annotation) so that `&__tenant`
//! at the delegation site produces `&TenantContext`, not `&&TenantContext`.
//!
//! WR-01 regression fixture: before the fix, the escape-hatch arm emitted
//! `let __tenant: #tenant_ty = { expr };` where `#tenant_ty` was `&TenantContext`,
//! causing the delegation `&__tenant` to become `&&TenantContext`.

#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;

use ferro::{async_trait, resource_get, resource_post, ActionResult, Request, Response, TenantContext, TenantScoped};

#[derive(Clone)]
struct Widget {
    pub id: i64,
}

#[async_trait]
impl TenantScoped for Widget {
    type Id = i64;

    async fn find_for_tenant(
        id: i64,
        _tenant_id: i64,
    ) -> Result<Option<Self>, ferro::FrameworkError> {
        let _ = id;
        Ok(None)
    }
}

fn mock_tenant() -> TenantContext {
    TenantContext::new(1, "test".to_string(), "Test".to_string(), None)
}

#[resource_get(Widget, on_miss = "/widgets", tenant = "mock_tenant()")]
pub async fn show(req: &mut Request, tenant: &TenantContext, widget: &Widget) -> Response {
    let _ = (req, tenant, widget);
    Ok(ferro::HttpResponse::new())
}

#[resource_post(Widget, redirect_to = "/widgets", tenant = "mock_tenant()")]
pub async fn update(req: &mut Request, tenant: &TenantContext, widget: &Widget) -> ActionResult {
    let _ = (req, tenant, widget, __form_url);
    Ok(())
}

fn main() {}
