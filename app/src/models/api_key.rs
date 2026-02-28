//! API key model

pub use super::entities::api_keys::*;

use ferro::database::{Model as DatabaseModel, ModelMut, QueryBuilder};

#[allow(dead_code)]
pub type ApiKey = Model;

impl Model {
    #[allow(dead_code)]
    pub fn query() -> QueryBuilder<Entity> {
        QueryBuilder::new()
    }
}

impl DatabaseModel for Entity {}
impl ModelMut for Entity {}
