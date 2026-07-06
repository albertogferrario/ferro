//! Todo model
//!
//! This file contains custom implementations for the Todo model.
//! The base entity is auto-generated in src/models/entities/todos.rs
//!
//! This file is NEVER overwritten by `ferro db:sync` - your custom code is safe here.

// Re-export the auto-generated entity (includes FerroModel-generated boilerplate)
pub use super::entities::todos::*;

/// Type alias for convenient access
#[allow(dead_code)]
pub type Todo = Model;

// ============================================================================
// CUSTOM METHODS
// Add your custom query and mutation methods below
// ============================================================================

// Example custom finder:
// impl Model {
//     pub async fn find_by_email(email: &str) -> Result<Option<Self>, ferro::FrameworkError> {
//         Self::query().filter(Column::Email.eq(email)).first().await
//     }
// }

// ============================================================================
// RELATIONS
// Define relationships to other entities here
// ============================================================================

// Example: One-to-Many relation
// impl Entity {
//     pub fn has_many_posts() -> RelationDef {
//         Entity::has_many(super::posts::Entity).into()
//     }
// }

// Example: Belongs-To relation
// impl Entity {
//     pub fn belongs_to_user() -> RelationDef {
//         Entity::belongs_to(super::users::Entity)
//             .from(Column::UserId)
//             .to(super::users::Column::Id)
//             .into()
//     }
// }
