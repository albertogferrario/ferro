//! HTTP controllers. Each renders a JSON-UI view and assembles only the data
//! that view needs (structure lives in `src/views/*.json`).

pub mod account;
pub mod auth;
pub mod errors;
pub mod home;
pub mod places;
pub mod presence;
pub mod settings;
pub mod trilli;
pub mod utenti;
