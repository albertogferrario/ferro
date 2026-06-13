//! Compile-pass: full CRUD reference fixture — both `#[resource_get]` and `#[resource_post]`.
//!
//! Exercises `ferro::resource_get` + `ferro::resource_post` + `ferro::TenantScoped` +
//! `validate_or_redirect` on the same model via the `ferro::` facade path. Proves CRUD-06:
//! the four phase artifacts compose into one coherent CRUD surface a downstream crate can import.
//!
//! Security note (T-212-01): both macros call `find_for_tenant(id, tenant_id)` — the
//! `Customer::find_for_tenant` stub below intentionally receives `tenant_id` in its signature to
//! show the required tenant-scoped contract.

#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;

use ferro::{ActionResult, Request, Response, TenantContext};

/// Minimal customer model for the reference fixture.
#[derive(Clone)]
struct Customer {
    pub id: i64,
    pub name: String,
}

/// T-212-01: the lookup always receives `tenant_id`; an un-scoped lookup is structurally
/// impossible through these macros.
#[ferro::async_trait]
impl ferro::TenantScoped for Customer {
    type Id = i64;

    async fn find_for_tenant(
        id: i64,
        tenant_id: i64,
    ) -> Result<Option<Self>, ferro::FrameworkError> {
        let _ = (id, tenant_id);
        Ok(None)
    }
}

/// GET handler — displays the customer edit form.
///
/// The macro folds id-extraction, tenant resolution, tenant-scoped lookup, and the 302-on-miss
/// redirect into one attribute. `tenant` and `customer` remain real typed parameters; IDE
/// jump-to-def keeps working.
#[ferro::resource_get(Customer, on_miss = "/dashboard/clienti")]
pub async fn edit(req: &mut Request, tenant: &TenantContext, customer: &Customer) -> Response {
    let _ = (req, tenant, customer);
    Ok(ferro::HttpResponse::new())
}

/// POST handler — saves the customer form submission.
///
/// Extends the GET prelude with a validation-redirect envelope: the inner fn receives the injected
/// `__form_url: &str` binding, which `validate_or_redirect` uses to redirect back on failure.
#[ferro::resource_post(
    Customer,
    redirect_to = "/dashboard/clienti",
    form_url = "/dashboard/clienti/{id}/modifica"
)]
pub async fn save(req: &mut Request, tenant: &TenantContext, customer: &Customer) -> ActionResult {
    let _ = (req, tenant, customer);
    let data = ferro::serde_json::json!({ "name": "test" });
    ferro::Validator::new(&data)
        .rules("name", ferro::rules![ferro::required()])
        .validate_or_redirect(__form_url)?;
    Ok(())
}

fn main() {}
