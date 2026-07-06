//! API route registration

use crate::api::*;
use ferro::*;

pub fn api_routes() -> GroupDef {
    group!("/api/v1", {
        // User CRUD
        get!("/users", user_api::index).name("api.users.index"),
        post!("/users", user_api::store).name("api.users.store"),
        get!("/users/:user", user_api::show).name("api.users.show"),
        put!("/users/:user", user_api::update).name("api.users.update"),
        delete!("/users/:user", user_api::destroy).name("api.users.destroy"),
    })
    .middleware(ApiKeyMiddleware::new())
    .middleware(Throttle::named("api"))
}
