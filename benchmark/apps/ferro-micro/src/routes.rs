use ferro::{get, routes};

use crate::controllers;

routes! {
    get!("/json",    controllers::bench::json_handler),
    get!("/db",      controllers::bench::db_handler),
    get!("/queries", controllers::bench::queries),
    get!("/updates", controllers::bench::updates),
}
