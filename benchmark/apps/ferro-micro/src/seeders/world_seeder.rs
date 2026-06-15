//! World table seeder — inserts 10 000 rows with random numbers

use ferro::{async_trait, FrameworkError, Seeder};
use sea_orm::{DatabaseConnection, EntityTrait, Set};

use crate::models::world;

pub struct WorldSeeder;

#[async_trait]
impl Seeder for WorldSeeder {
    async fn run(&self, db: &DatabaseConnection) -> Result<(), FrameworkError> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let rows: Vec<world::ActiveModel> = (1..=10_000)
            .map(|_| world::ActiveModel {
                random_number: Set(rng.gen_range(1..=10_000)),
                ..Default::default()
            })
            .collect();
        world::Entity::insert_many(rows)
            .exec(db)
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))?;
        Ok(())
    }
}
