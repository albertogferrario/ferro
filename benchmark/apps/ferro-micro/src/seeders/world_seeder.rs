//! World table seeder — inserts 10 000 rows with random numbers

use ferro::{async_trait, FrameworkError, Seeder};
use sea_orm::{DatabaseConnection, EntityTrait, Set};

use crate::models::world;

#[derive(Default)]
pub struct WorldSeeder;

#[async_trait]
impl Seeder for WorldSeeder {
    async fn run(&self, db: &DatabaseConnection) -> Result<(), FrameworkError> {
        use rand::Rng;
        // Collect all random numbers synchronously before any await point.
        // ThreadRng is !Send so it must not be held across .await.
        let rows: Vec<world::ActiveModel> = {
            let mut rng = rand::thread_rng();
            (1..=10_000)
                .map(|_| world::ActiveModel {
                    random_number: Set(rng.gen_range(1..=10_000)),
                    ..Default::default()
                })
                .collect()
        };
        world::Entity::insert_many(rows)
            .exec(db)
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))?;
        Ok(())
    }
}
