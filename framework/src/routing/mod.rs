mod group;
mod macros;
mod router;

pub use group::{GroupBuilder, GroupRouter};
pub use macros::{
    // Internal functions used by macros (hidden from docs)
    __box_handler,
    __delete_impl,
    __fallback_impl,
    __get_impl,
    __patch_impl,
    __post_impl,
    __put_impl,
    validate_route_path,
    FallbackDefBuilder,
    GroupDef,
    GroupItem,
    GroupRoute,
    HttpMethod,
    IntoGroupItem,
    ResourceAction,
    ResourceDef,
    ResourceRoute,
    RouteDefBuilder,
};
pub use router::{
    get_registered_routes, register_route_name, route, route_with_params, BoxedHandler,
    RouteBuilder, RouteInfo, Router,
};
