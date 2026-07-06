//! Order model

pub use super::entities::orders::*;

#[allow(dead_code)]
pub type Order = Model;

use ferro::async_trait;
use ferro::tenant::TenantScoped;
use ferro::FrameworkError;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

/// Tenant-scoped lookup for the Order model.
///
/// `find_for_tenant(id, tenant_id)` filters by both primary key and
/// `tenant_id`, preventing cross-tenant reads through the generated-handler path.
/// This is the load-bearing cross-tenant denial primitive used by the write executor (D-03).
#[async_trait]
impl TenantScoped for Model {
    type Id = i32;

    async fn find_for_tenant(id: i32, tenant_id: i64) -> Result<Option<Self>, FrameworkError> {
        use super::entities::orders::{Column, Entity};
        Entity::find_by_id(id)
            .filter(Column::TenantId.eq(tenant_id))
            .one(
                ferro::DB::connection()
                    .map_err(|e| FrameworkError::Database(e.to_string()))?
                    .inner(),
            )
            .await
            .map_err(|e| FrameworkError::Database(e.to_string()))
    }
}
