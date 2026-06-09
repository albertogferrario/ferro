// Base entity for the `books` table. Mirrors the create_books_table migration.

use ferro::FerroModel;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, FerroModel)]
#[sea_orm(table_name = "books")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub cover_url: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub description: Option<String>,
    pub year: Option<i32>,
    pub source: String,
    pub source_id: String,
    pub public_domain: bool,
    pub download_url: Option<String>,
    pub local_path: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

// Relation enum is required by the DeriveEntityModel macro even when empty.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
