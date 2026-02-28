//! API key model

pub use super::entities::api_keys::*;

use ferro::database::{Model as DatabaseModel, ModelMut, QueryBuilder};

pub type ApiKey = Model;

impl Model {
    pub fn query() -> QueryBuilder<Entity> {
        QueryBuilder::new()
    }
}

impl DatabaseModel for Entity {}
impl ModelMut for Entity {}
