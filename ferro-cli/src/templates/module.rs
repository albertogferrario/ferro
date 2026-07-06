//! Stub templates for `ferro make:module` — the feature-module convention
//! (controller/model/views/routes). These are injected into user projects, so
//! every snippet must compile as-is inside a fresh Ferro application.

/// `src/modules/<name>/mod.rs` with the default (views included) layout.
pub fn module_mod_rs(name: &str) -> String {
    format!(
        "//! {name} feature module\n\
         \n\
         pub mod controller;\n\
         pub mod model;\n\
         pub mod routes;\n\
         pub mod views;\n"
    )
}

/// `src/modules/<name>/mod.rs` for the `--no-views` (headless) variant.
pub fn module_mod_rs_headless(name: &str) -> String {
    format!(
        "//! {name} feature module (headless)\n\
         \n\
         pub mod controller;\n\
         pub mod model;\n\
         pub mod routes;\n"
    )
}

/// `src/modules/<name>/controller.rs` — minimal compiling handler stub.
pub fn module_controller_rs(name: &str) -> String {
    format!(
        r#"//! {name} controller

use ferro::{{handler, json_response, Request, Response}};

#[handler]
pub async fn index(_req: Request) -> Response {{
    json_response!({{
        "module": "{name}"
    }})
}}
"#
    )
}

/// `src/modules/<name>/model.rs` — empty SeaORM entity stub.
pub fn module_model_rs(name: &str) -> String {
    format!(
        r#"//! {name} model

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "{name}")]
pub struct Model {{
    #[sea_orm(primary_key)]
    pub id: i32,
}}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {{}}

impl ActiveModelBehavior for ActiveModel {{}}
"#
    )
}

/// `src/modules/<name>/views/mod.rs`.
pub fn module_views_mod_rs() -> String {
    "//! Views for this feature module\n\npub mod index;\n".to_string()
}

/// `src/modules/<name>/views/index.rs` — Spec stub (mirrors
/// `json_view_template` in make.rs, scoped to the module).
pub fn module_view_index_rs(name: &str) -> String {
    let title = titlecase(name);
    format!(
        r#"//! {title} index view

use ferro::{{Spec, Element, JsonUi, Response}};

/// Build the {title} index view.
pub async fn view() -> Response {{
    let spec = Spec::builder()
        .title("{title}")
        .layout("app")
        .element(
            "root",
            Element::new("Card")
                .prop("title", "{title}")
                .prop(
                    "description",
                    "Edit src/modules/{name}/views/index.rs to customize this view.",
                )
                .child("heading"),
        )
        .element(
            "heading",
            Element::new("Text")
                .prop("content", "{title}")
                .prop("element", "h1"),
        )
        .build()
        .expect("spec is valid");

    JsonUi::render(&spec, &serde_json::json!({{}}))
}}
"#
    )
}

/// `src/modules/<name>/routes.rs` — canonical `register(router)` hook.
pub fn module_routes_rs(name: &str) -> String {
    format!(
        r#"//! {name} routes

use ferro::Router;

/// Register the {name} module routes onto the given router.
pub fn register(router: Router) -> Router {{
    router.get("/{name}", super::controller::index)
}}
"#
    )
}

/// `migration/src/m_<ts>_create_<name>.rs` — optional SeaORM migration stub.
pub fn module_migration_rs(name: &str, ts: &str) -> String {
    let struct_name = format!("M{ts}CreateTable");
    format!(
        r#"//! create_{name} migration ({ts})

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct {struct_name};

#[async_trait::async_trait]
impl MigrationTrait for {struct_name} {{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .create_table(
                Table::create()
                    .table(Entity::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Column::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Column::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Column::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await
    }}

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .drop_table(Table::drop().table(Entity::Table).to_owned())
            .await
    }}
}}

#[derive(DeriveIden)]
enum Entity {{
    #[sea_orm(iden = "{name}")]
    Table,
}}

#[derive(DeriveIden)]
enum Column {{
    Id,
    CreatedAt,
    UpdatedAt,
}}
"#
    )
}

fn titlecase(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut upper_next = true;
    for ch in name.chars() {
        if ch == '_' || ch == '-' {
            out.push(' ');
            upper_next = true;
        } else if upper_next {
            out.extend(ch.to_uppercase());
            upper_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_rs_declares_views() {
        let out = module_mod_rs("orders");
        assert!(out.contains("pub mod controller;"));
        assert!(out.contains("pub mod model;"));
        assert!(out.contains("pub mod routes;"));
        assert!(out.contains("pub mod views;"));
    }

    #[test]
    fn mod_rs_headless_omits_views() {
        let out = module_mod_rs_headless("orders");
        assert!(out.contains("pub mod controller;"));
        assert!(!out.contains("pub mod views;"));
    }

    #[test]
    fn routes_rs_exposes_register_fn() {
        let out = module_routes_rs("orders");
        assert!(out.contains("pub fn register(router: Router) -> Router"));
        assert!(out.contains("\"/orders\""));
    }

    #[test]
    fn controller_rs_uses_handler_macro() {
        let out = module_controller_rs("orders");
        assert!(out.contains("#[handler]"));
        assert!(out.contains("\"orders\""));
    }

    #[test]
    fn migration_rs_implements_migration_trait() {
        let out = module_migration_rs("orders", "20260407120000");
        assert!(out.contains("MigrationTrait"));
        assert!(out.contains("orders"));
    }
}
