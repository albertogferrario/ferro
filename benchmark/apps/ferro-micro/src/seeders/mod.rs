mod world_seeder;
pub use world_seeder::WorldSeeder;

pub fn register() -> ferro::SeederRegistry {
    ferro::SeederRegistry::new().add::<WorldSeeder>()
}
