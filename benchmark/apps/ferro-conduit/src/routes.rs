use ferro::{get, routes};

use crate::controllers;

routes! {
    get!("/health", controllers::health::show),
}
