//! Compile-pass: minimal `#[resource_post]` fixture.
//!
//! Defines a local model + TenantScoped impl. The macro folds the prelude +
//! validation-redirect envelope; the user body receives real typed params and
//! `__form_url: &str` as an injected binding.

#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;

use ferro::{
    async_trait, resource_post, ActionResult, Request, TenantContext, TenantScoped,
};

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

#[resource_post(Model, redirect_to = "/list", form_url = "/list/{id}/edit")]
pub async fn save(req: &mut Request, tenant: &TenantContext, model: &Model) -> ActionResult {
    let _ = (req, tenant, model, __form_url);
    Ok(())
}

fn main() {}
