//! Tenant-scoped query filtering for Ferro framework.
//!
//! Provides [`TenantScope`] — a generic [`Scope`] implementation that filters
//! database queries by the current tenant's ID, read from task-local context.
//!
//! # Panics
//!
//! [`TenantScope`] panics at query time if called outside a `TenantMiddleware`
//! scope. This is a programming error — not a runtime condition — because
//! every route that touches tenant-owned data must be behind `TenantMiddleware`.

use crate::database::{QueryBuilder, Scope};
use crate::tenant::context::current_tenant;
use sea_orm::{ColumnTrait, EntityTrait};

/// A query scope that filters by the current tenant's ID.
///
/// Wraps a single [`ColumnTrait`] value representing the tenant foreign-key
/// column. When [`Scope::apply`] is called, it reads the tenant ID from
/// task-local context and appends `.eq(tenant_id)` to the query.
///
/// # Panics
///
/// Panics with a clear message if called outside a `TenantMiddleware` scope.
/// This is intentional — using `TenantScope` without middleware is a
/// programming error, not a recoverable runtime condition.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::TenantScope;
///
/// let posts = post::Entity::scoped(TenantScope(post::Column::TenantId))
///     .all()
///     .await?;
/// ```
pub struct TenantScope<C: ColumnTrait>(pub C);

impl<E, C> Scope<E> for TenantScope<C>
where
    E: EntityTrait,
    E::Model: Send + Sync,
    C: ColumnTrait,
{
    fn apply(self, query: QueryBuilder<E>) -> QueryBuilder<E> {
        let ctx = current_tenant().expect(
            "TenantScope used outside TenantMiddleware scope — ensure this route is behind TenantMiddleware",
        );
        query.filter(self.0.eq(ctx.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Scope as ScopeTrait;
    use crate::tenant::context::{current_tenant, tenant_scope, with_tenant_scope};
    use crate::tenant::TenantContext;
    use sea_orm::{DbBackend, QueryTrait, Value};

    // Minimal SeaORM entity for testing TenantScope.
    // No DB connection needed — we only test query building, not execution.
    mod post {
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "posts")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i64,
            pub tenant_id: i64,
            pub title: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    fn make_tenant(id: i64) -> TenantContext {
        TenantContext {
            id,
            slug: format!("tenant-{id}"),
            name: format!("Tenant {id}"),
            plan: None,
        }
    }

    fn statement_from_query(query: QueryBuilder<post::Entity>) -> sea_orm::Statement {
        query.into_select().build(DbBackend::Sqlite)
    }

    /// Verify TenantScope(column).apply(query) adds a tenant_id filter.
    #[tokio::test]
    async fn tenant_scope_apply_adds_tenant_id_filter() {
        let ctx = tenant_scope();
        {
            let mut guard = ctx.write().await;
            *guard = Some(make_tenant(42));
        }

        let stmt = with_tenant_scope(ctx, async {
            let builder: QueryBuilder<post::Entity> = QueryBuilder::new();
            let scoped = TenantScope(post::Column::TenantId).apply(builder);
            statement_from_query(scoped)
        })
        .await;

        assert!(
            stmt.sql.contains("tenant_id"),
            "Expected SQL to contain tenant_id filter, got: {}",
            stmt.sql
        );
        // Verify the WHERE clause was added
        assert!(
            stmt.sql.contains("WHERE"),
            "Expected SQL to have WHERE clause, got: {}",
            stmt.sql
        );
        // Verify the bound value is 42 (i64)
        let values = stmt.values.expect("expected bound values");
        assert!(
            values
                .0
                .iter()
                .any(|v| matches!(v, Value::BigInt(Some(42)))),
            "Expected bound value 42i64, got: {:?}",
            values.0
        );
    }

    /// Verify TenantScope panics with clear message outside middleware scope.
    #[test]
    #[should_panic(expected = "TenantScope used outside TenantMiddleware scope")]
    fn tenant_scope_panics_outside_middleware_scope() {
        let builder: QueryBuilder<post::Entity> = QueryBuilder::new();
        TenantScope(post::Column::TenantId).apply(builder);
    }

    /// Verify TenantScope works with any ColumnTrait (generic over column type).
    #[tokio::test]
    async fn tenant_scope_is_generic_over_column_type() {
        let ctx = tenant_scope();
        {
            let mut guard = ctx.write().await;
            *guard = Some(make_tenant(7));
        }

        // Use id column to verify generics work beyond tenant_id
        let stmt = with_tenant_scope(ctx, async {
            let builder: QueryBuilder<post::Entity> = QueryBuilder::new();
            let scoped = TenantScope(post::Column::Id).apply(builder);
            statement_from_query(scoped)
        })
        .await;

        assert!(
            stmt.sql.contains("\"id\""),
            "Expected SQL to filter on id column, got: {}",
            stmt.sql
        );
        let values = stmt.values.expect("expected bound values");
        assert!(
            values.0.iter().any(|v| matches!(v, Value::BigInt(Some(7)))),
            "Expected bound value 7i64, got: {:?}",
            values.0
        );
    }

    /// Verify concurrent tasks with different tenant scopes get isolated filters.
    #[tokio::test(flavor = "multi_thread")]
    async fn concurrent_tasks_get_isolated_tenant_scopes() {
        let ctx1 = tenant_scope();
        let ctx2 = tenant_scope();
        {
            let mut g1 = ctx1.write().await;
            *g1 = Some(make_tenant(100));
        }
        {
            let mut g2 = ctx2.write().await;
            *g2 = Some(make_tenant(200));
        }

        let (result1, result2) = tokio::join!(
            with_tenant_scope(ctx1, async { current_tenant().map(|t| t.id) }),
            with_tenant_scope(ctx2, async { current_tenant().map(|t| t.id) }),
        );

        assert_eq!(result1, Some(100), "Task 1 should see tenant 100");
        assert_eq!(result2, Some(200), "Task 2 should see tenant 200");
    }
}
