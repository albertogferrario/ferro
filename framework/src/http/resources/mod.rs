/// Cursor and offset pagination envelope types.
pub mod pagination;
/// Single-resource transformation trait.
pub mod resource;
/// Paginated resource collection with metadata.
pub mod resource_collection;
/// Key-value resource builder with conditional fields.
pub mod resource_map;

pub use pagination::{PaginationLinks, PaginationMeta};
pub use resource::Resource;
pub use resource_collection::ResourceCollection;
pub use resource_map::ResourceMap;
