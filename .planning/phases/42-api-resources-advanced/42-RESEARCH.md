# Phase 42: API Resources Advanced - Research

**Researched:** 2026-02-10
**Domain:** Relationship inclusion, pagination, and collection resources for Ferro API Resources
**Confidence:** HIGH

<research_summary>
## Summary

Researched how to extend Ferro's Phase 41 API Resources with relationship inclusion, paginated responses, and resource collections. Phase 42 builds entirely on existing Ferro infrastructure — no new external dependencies required.

Key finding: Ferro's explicit batch-loading model (HashMap-based, no lazy loading) makes Laravel's `whenLoaded()` pattern unnecessary. Instead, relationships are passed alongside the parent model as explicit `Option<T>` or `Vec<T>` fields on the resource struct. This is more type-safe than Laravel's runtime relationship-loaded checks and fits Rust's explicit ownership model.

Pagination integrates naturally via SeaORM's `PaginatorTrait` (already re-exported). A `ResourceCollection` type wraps `Vec<T: Resource>` with pagination metadata (links, meta) and produces the standard `{"data": [...], "links": {...}, "meta": {...}}` envelope. The critical limitation: SeaORM's `find_with_related()` (`SelectTwoMany`) does NOT support `.paginate()` — so the pattern is always: paginate parent → batch-load relations → map to resources.

**Primary recommendation:** Build `ResourceCollection<T>` with pagination support, add `when_loaded` / `when_loaded_many` convenience methods to `ResourceMap` for the batch-loaded HashMap pattern, and add `Resource::collection()` convenience for simple collection cases. Keep relationship handling explicit (no magic) to match Ferro's and Rust's design philosophy.
</research_summary>

<standard_stack>
## Standard Stack

### Core (Internal — No External Dependencies)
| Component | Purpose | Why This Approach |
|-----------|---------|-------------------|
| `ResourceCollection<T>` | Wraps `Vec<T: Resource>` with pagination meta | Mirrors Laravel's ResourceCollection; enables `{"data": [...], "links": {...}, "meta": {...}}` |
| `PaginationMeta` | Pagination metadata struct (current_page, total, per_page, etc.) | Type-safe pagination info, produced from SeaORM's `Paginator` |
| `PaginationLinks` | Pagination links struct (first, last, prev, next URLs) | Standard API pagination links from request URL + page info |
| `ResourceMap::when_loaded` | Convenience for belongs_to batch-load pattern | Checks `HashMap<K, Model>` by key; includes field only if found |
| `ResourceMap::when_loaded_many` | Convenience for has_many batch-load pattern | Checks `HashMap<K, Vec<Model>>` by key; includes field only if found |

### Supporting (Existing Dependencies — Already In Workspace)
| Library | Version | Purpose | Already In Use |
|---------|---------|---------|----------------|
| `sea-orm` (`PaginatorTrait`) | workspace | Pagination queries | Yes (re-exported) |
| `serde` | workspace | Serialization | Yes |
| `serde_json` | workspace | JSON value construction | Yes |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `ResourceCollection<T>` | Generic `Vec<T>` with free functions | Collection loses ability to carry metadata, pagination context |
| `PaginationMeta` struct | Raw `serde_json::Value` | Struct enforces correct fields, enables compile-time checks |
| `when_loaded` on ResourceMap | Just use `when_some` with manual HashMap lookup | `when_loaded` is ergonomic sugar that communicates intent; `when_some` requires wrapping in Option |
| Builder pattern for collection | Constructor with all params | Builder is more flexible for optional pagination, additional metadata |

### No New Dependencies Required
All building blocks already exist. SeaORM pagination is re-exported. The `Resource` trait, `ResourceMap`, and `ApiResource` derive macro from Phase 41 are the foundation.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Module Structure (extends Phase 41)
```
framework/src/http/resources/
├── mod.rs                  # Add ResourceCollection, PaginationMeta, PaginationLinks exports
├── resource.rs             # Add Resource::collection() convenience method
├── resource_map.rs         # Add when_loaded, when_loaded_many methods
├── resource_collection.rs  # NEW: ResourceCollection<T> with pagination
└── pagination.rs           # NEW: PaginationMeta, PaginationLinks structs
```

### Pattern 1: ResourceCollection with Pagination
**What:** A wrapper around `Vec<T: Resource>` that adds pagination metadata and produces the standard paginated response envelope.
**When to use:** Any endpoint returning a list of resources, especially paginated.
**Design:**
```rust
pub struct ResourceCollection<T: Resource> {
    items: Vec<T>,
    pagination: Option<PaginationMeta>,
    additional: Option<serde_json::Value>,
}

impl<T: Resource> ResourceCollection<T> {
    /// Create from a vec of resources (no pagination).
    pub fn new(items: Vec<T>) -> Self;

    /// Create from items + SeaORM pagination info.
    pub fn paginated(items: Vec<T>, meta: PaginationMeta) -> Self;

    /// Add extra top-level fields (merged alongside data/meta/links).
    pub fn additional(mut self, value: serde_json::Value) -> Self;

    /// Produce the JSON response.
    /// Without pagination: {"data": [...]}
    /// With pagination: {"data": [...], "links": {...}, "meta": {...}}
    pub fn to_response(&self, req: &Request) -> HttpResponse;
}
```

### Pattern 2: PaginationMeta from SeaORM Paginator
**What:** A struct that captures pagination state from SeaORM's Paginator and produces standard pagination JSON.
**When to use:** After calling `paginator.fetch_page()` and `paginator.num_items_and_pages()`.
**Design:**
```rust
pub struct PaginationMeta {
    pub current_page: u64,
    pub per_page: u64,
    pub total: u64,
    pub last_page: u64,
    pub from: u64,    // first item index on current page (1-based)
    pub to: u64,      // last item index on current page (1-based)
}

pub struct PaginationLinks {
    pub first: String,
    pub last: String,
    pub prev: Option<String>,
    pub next: Option<String>,
}

impl PaginationMeta {
    /// Create from SeaORM pagination result + current page + per_page.
    pub fn new(current_page: u64, per_page: u64, total: u64) -> Self;

    /// Generate links from the request URL.
    pub fn links(&self, req: &Request) -> PaginationLinks;
}
```

### Pattern 3: Relationship Inclusion via Resource Struct Fields
**What:** Instead of Laravel's `whenLoaded()` runtime check, relationships are explicit fields on the resource struct. The batch-loaded HashMap is queried during resource construction.
**When to use:** Always — this is Ferro's Rust-idiomatic approach.
**Design:**
```rust
// Resource struct includes relationship data as explicit fields
pub struct UserResource {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub posts: Option<Vec<PostResource>>,    // has_many, batch-loaded
    pub department: Option<DepartmentResource>, // belongs_to, batch-loaded
}

// Construction from model + batch-loaded maps
impl UserResource {
    pub fn from_with_relations(
        model: users::Model,
        posts: &HashMap<i32, Vec<posts::Model>>,
        departments: &HashMap<i32, departments::Model>,
    ) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            email: model.email.clone(),
            posts: posts.get(&model.id).map(|p|
                p.iter().map(|m| PostResource::from(m.clone())).collect()
            ),
            department: departments.get(&model.department_id)
                .map(|d| DepartmentResource::from(d.clone())),
        }
    }
}

// Resource impl uses when_some for optional relations
impl Resource for UserResource {
    fn to_resource(&self, req: &Request) -> Value {
        ResourceMap::new()
            .field("id", json!(self.id))
            .field("name", json!(self.name))
            .field("email", json!(self.email))
            .when_some("posts", &self.posts.as_ref().map(|p|
                p.iter().map(|r| r.to_resource(req)).collect::<Vec<_>>()
            ))
            .when_some("department", &self.department.as_ref()
                .map(|d| d.to_resource(req)))
            .build()
    }
}
```

### Pattern 4: ResourceMap when_loaded Convenience
**What:** Ergonomic methods on ResourceMap that check batch-loaded HashMaps and include related resource(s) only when present.
**When to use:** In manual `Resource::to_resource()` implementations that use batch-loaded data.
**Design:**
```rust
impl ResourceMap {
    /// Include a belongs_to/has_one relation from a batch-loaded HashMap.
    /// If the key exists in the map, includes the resource; otherwise omits the field.
    pub fn when_loaded<K, M, R>(
        self,
        key: &str,
        lookup_key: &K,
        map: &HashMap<K, M>,
        transform: impl FnOnce(&M) -> Value,
    ) -> Self
    where
        K: Eq + Hash;

    /// Include a has_many relation from a batch-loaded HashMap.
    /// If the key exists, maps each item through the transform; otherwise omits.
    pub fn when_loaded_many<K, M, R>(
        self,
        key: &str,
        lookup_key: &K,
        map: &HashMap<K, Vec<M>>,
        transform: impl FnOnce(&[M]) -> Value,
    ) -> Self
    where
        K: Eq + Hash;
}
```

Usage in a handler:
```rust
#[handler]
pub async fn show(req: Request) -> Response {
    let user = User::find_or_fail(1).await?;

    // Batch load relations (explicit — user decides what to load)
    let posts = batch_load_has_many::<posts::Entity, _, _, _>(
        [user.id], posts::Column::UserId, |p| p.user_id
    ).await?;

    let resource = ResourceMap::new()
        .field("id", json!(user.id))
        .field("name", json!(user.name))
        .when_loaded_many("posts", &user.id, &posts, |items| {
            json!(items.iter().map(|p| {
                ResourceMap::new()
                    .field("id", json!(p.id))
                    .field("title", json!(p.title))
                    .build()
            }).collect::<Vec<_>>())
        })
        .build();

    Ok(HttpResponse::json(json!({"data": resource})))
}
```

### Pattern 5: Resource::collection() Convenience
**What:** A static method on the Resource trait for creating simple (non-paginated) collections.
**When to use:** When returning a list without pagination metadata.
**Design:**
```rust
// On the Resource trait:
fn collection(items: &[Self], req: &Request) -> Vec<Value>
where
    Self: Sized,
{
    items.iter().map(|item| item.to_resource(req)).collect()
}
```

### Pattern 6: Full Paginated Handler Flow
**What:** The complete pattern from query → pagination → batch load → resource collection → response.
**When to use:** Any paginated API list endpoint.
**Design:**
```rust
#[handler]
pub async fn index(req: Request) -> Response {
    let page = req.query("page").unwrap_or(1);
    let per_page = req.query("per_page").unwrap_or(15);
    let db = DB::connection()?;

    // Step 1: Paginate parent entity
    let paginator = User::find()
        .order_by_asc(users::Column::Id)
        .paginate(db.inner(), per_page);

    let users = paginator.fetch_page(page - 1).await?;
    let totals = paginator.num_items_and_pages().await?;

    // Step 2: Batch load relations for this page
    let department_ids: Vec<_> = users.iter().map(|u| u.department_id).collect();
    let departments = Department::batch_load(department_ids).await?;

    // Step 3: Map to resources
    let resources: Vec<UserResource> = users.into_iter()
        .map(|u| UserResource::from_with_relations(u, &departments))
        .collect();

    // Step 4: Build paginated response
    let meta = PaginationMeta::new(page, per_page, totals.number_of_items);
    let collection = ResourceCollection::paginated(resources, meta);

    Ok(collection.to_response(&req))
}
```

### Anti-Patterns to Avoid
- **Lazy-loading emulation:** Don't try to replicate Eloquent's `whenLoaded()` with runtime "was this loaded?" state. Rust's type system makes relationships explicit. Embrace it.
- **Paginating after join:** Don't use `find_with_related().paginate()` — it doesn't compile (`SelectTwoMany` doesn't impl `PaginatorTrait`). Always paginate parent then batch-load.
- **N+1 in resource mapping:** Don't call database queries inside `to_resource()`. All data should be loaded before resource construction.
- **Mixing pagination and collection methods:** Don't put pagination logic inside `ResourceCollection::new()`. Keep construction and pagination data injection separate.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Pagination queries | Manual LIMIT/OFFSET SQL | SeaORM's `PaginatorTrait` (.paginate/.fetch_page/.num_items_and_pages) | Handles edge cases, integrates with Select queries |
| Batch relation loading | Loop queries per item (N+1) | Ferro's `batch_load_has_many` / `BatchLoad` / `BatchLoadMany` | Single query, HashMap grouping already implemented |
| JSON envelope construction | Manual `json!({...})` wrapping per handler | `ResourceCollection::to_response()` | Consistent envelope format across all endpoints |
| Pagination link generation | String concatenation of URLs | `PaginationMeta::links(&req)` using Request URL | Handles query parameter merging, base URL extraction |
| Collection mapping | `items.iter().map(|i| i.to_resource(req)).collect()` in every handler | `Resource::collection()` or `ResourceCollection` | Reduces boilerplate, centralizes mapping logic |

**Key insight:** All the building blocks exist. SeaORM handles pagination queries. Ferro's eager loading handles batch relation loading. Phase 41's Resource/ResourceMap handles individual item transformation. Phase 42 wires these together with `ResourceCollection` and convenience methods — no novel algorithms needed.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Paginating Joined Queries
**What goes wrong:** Attempting `Entity::find().find_with_related(Related).paginate(db, 20)` fails to compile.
**Why it happens:** SeaORM's `SelectTwoMany` does not implement `PaginatorTrait`. JOINs inflate row counts (one parent repeated per child), making LIMIT/OFFSET produce wrong page sizes.
**How to avoid:** Always paginate the parent entity first, then batch-load relations for the fetched page. This is actually more efficient: 2 queries (parent page + relations) vs 1 inflated JOIN.
**Warning signs:** Compiler error about missing `PaginatorTrait` implementation.

### Pitfall 2: N+1 Queries Inside Resource Construction
**What goes wrong:** Database queries inside `to_resource()` or inside collection mapping closures cause N+1.
**Why it happens:** It's tempting to load related data during resource serialization, especially when porting Laravel patterns where models have lazy-loading.
**How to avoid:** All related data must be loaded BEFORE creating resources. The handler loads everything, then maps to resources purely from in-memory data.
**Warning signs:** Database calls inside `Resource::to_resource()`, `From<Model>` implementations, or resource construction closures.

### Pitfall 3: Wrong Page Numbers (Off-By-One)
**What goes wrong:** Page 1 returns empty results or page 2 duplicates page 1 items.
**Why it happens:** SeaORM's `fetch_page()` is 0-indexed, but API consumers expect 1-indexed pages.
**How to avoid:** `PaginationMeta::new()` should accept 1-indexed page from the API, and internally convert to 0-indexed for SeaORM. Document this clearly.
**Warning signs:** First page returns nothing, or `from` field shows 0 instead of 1.

### Pitfall 4: Missing Total Count Query
**What goes wrong:** Pagination metadata shows total=0 or is missing.
**Why it happens:** Forgetting to call `paginator.num_items_and_pages()` (which executes a separate COUNT query). Or calling it after consuming the paginator.
**How to avoid:** Call `num_items_and_pages()` on the same paginator, before or after `fetch_page()`. The paginator is borrowed, not consumed, by `fetch_page()`.
**Warning signs:** Pagination meta has total=0 or last_page=0 despite data existing.

### Pitfall 5: Inconsistent Collection Wrapping
**What goes wrong:** Some endpoints return `{"data": [...]}`, others return `[...]`, others return `{"users": [...]}`.
**Why it happens:** Without a standard collection type, each handler wraps differently.
**How to avoid:** Always use `ResourceCollection::to_response()` for collections. The wrapping is consistent: `{"data": [...]}` without pagination, `{"data": [...], "links": {...}, "meta": {...}}` with pagination.
**Warning signs:** Frontend code needs special cases per endpoint for parsing collections.
</common_pitfalls>

<code_examples>
## Code Examples

### ResourceCollection Basic (No Pagination)
```rust
// Source: Ferro design — extends Resource trait from Phase 41
use ferro::{Resource, ResourceCollection, Request, Response};

#[handler]
pub async fn index(req: Request) -> Response {
    let users = User::all().await?;
    let resources: Vec<UserResource> = users.into_iter()
        .map(UserResource::from)
        .collect();

    let collection = ResourceCollection::new(resources);
    Ok(collection.to_response(&req))
    // Output: {"data": [{"id": 1, ...}, {"id": 2, ...}]}
}
```

### ResourceCollection with Pagination
```rust
// Source: Ferro design — integrates SeaORM PaginatorTrait
use ferro::{Resource, ResourceCollection, PaginationMeta, Request, Response};
use sea_orm::PaginatorTrait;

#[handler]
pub async fn index(req: Request) -> Response {
    let page: u64 = req.query("page").unwrap_or(1);
    let per_page: u64 = req.query("per_page").unwrap_or(15);
    let db = DB::connection()?;

    let paginator = User::find()
        .order_by_asc(users::Column::Id)
        .paginate(db.inner(), per_page);

    let users = paginator.fetch_page(page - 1).await
        .map_err(|e| FrameworkError::database(e.to_string()))?;
    let totals = paginator.num_items_and_pages().await
        .map_err(|e| FrameworkError::database(e.to_string()))?;

    let resources: Vec<UserResource> = users.into_iter()
        .map(UserResource::from)
        .collect();

    let meta = PaginationMeta::new(page, per_page, totals.number_of_items);
    let collection = ResourceCollection::paginated(resources, meta);

    Ok(collection.to_response(&req))
    // Output: {
    //   "data": [{...}, {...}],
    //   "meta": {"current_page": 1, "per_page": 15, "total": 42, "last_page": 3, "from": 1, "to": 15},
    //   "links": {"first": "/?page=1", "last": "/?page=3", "prev": null, "next": "/?page=2"}
    // }
}
```

### Relationship Inclusion with Batch Loading
```rust
// Source: Ferro design — combines Phase 41 Resource with Ferro eager loading
use ferro::{Resource, ResourceMap, ResourceCollection, PaginationMeta};
use ferro::database::{batch_load_has_many, BatchLoad};

pub struct UserWithPostsResource {
    user: users::Model,
    posts: Vec<posts::Model>,
}

impl Resource for UserWithPostsResource {
    fn to_resource(&self, req: &Request) -> Value {
        ResourceMap::new()
            .field("id", json!(self.user.id))
            .field("name", json!(self.user.name))
            .field("posts", json!(
                self.posts.iter().map(|p| {
                    ResourceMap::new()
                        .field("id", json!(p.id))
                        .field("title", json!(p.title))
                        .build()
                }).collect::<Vec<_>>()
            ))
            .build()
    }
}

#[handler]
pub async fn index(req: Request) -> Response {
    let page: u64 = req.query("page").unwrap_or(1);
    let per_page: u64 = 15;
    let db = DB::connection()?;

    // 1. Paginate parent
    let paginator = User::find().paginate(db.inner(), per_page);
    let users = paginator.fetch_page(page - 1).await?;
    let totals = paginator.num_items_and_pages().await?;

    // 2. Batch load relations for this page only
    let user_ids: Vec<i32> = users.iter().map(|u| u.id).collect();
    let posts_map = batch_load_has_many::<posts::Entity, _, _, _>(
        user_ids, posts::Column::UserId, |p| p.user_id
    ).await?;

    // 3. Map to resources with relations
    let resources: Vec<UserWithPostsResource> = users.into_iter()
        .map(|u| {
            let posts = posts_map.get(&u.id)
                .cloned()
                .unwrap_or_default();
            UserWithPostsResource { user: u, posts }
        })
        .collect();

    // 4. Return paginated collection
    let meta = PaginationMeta::new(page, per_page, totals.number_of_items);
    Ok(ResourceCollection::paginated(resources, meta).to_response(&req))
}
```

### when_loaded Convenience on ResourceMap
```rust
// Source: Ferro design — ergonomic API for batch-loaded relations
use ferro::{Resource, ResourceMap};
use std::collections::HashMap;

impl Resource for UserResource {
    fn to_resource(&self, req: &Request) -> Value {
        // self.departments and self.posts are pre-loaded HashMaps
        ResourceMap::new()
            .field("id", json!(self.model.id))
            .field("name", json!(self.model.name))
            .when_loaded("department", &self.model.department_id,
                &self.departments, |dept| {
                    DepartmentResource::from(dept.clone()).to_resource(req)
                })
            .when_loaded_many("posts", &self.model.id,
                &self.posts, |items| {
                    json!(items.iter().map(|p|
                        PostResource::from(p.clone()).to_resource(req)
                    ).collect::<Vec<_>>())
                })
            .build()
    }
}
```

### Collection with Additional Metadata
```rust
// Source: Ferro design — mirrors Laravel's additional() pattern
#[handler]
pub async fn index(req: Request) -> Response {
    let users = User::all().await?;
    let resources: Vec<UserResource> = users.into_iter()
        .map(UserResource::from)
        .collect();

    let collection = ResourceCollection::new(resources)
        .additional(json!({
            "meta": {
                "version": "v1",
                "filters_applied": ["active"]
            }
        }));

    Ok(collection.to_response(&req))
    // Output: {"data": [...], "meta": {"version": "v1", "filters_applied": ["active"]}}
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Laravel `whenLoaded()` runtime checks | Explicit Option/Vec fields on resource struct | N/A (Rust design) | Type-safe at compile time; no runtime "is loaded?" state |
| SeaORM `find_with_related` for paginated lists | Paginate parent + batch load (LoaderTrait or Ferro BatchLoad) | SeaORM design | `SelectTwoMany` can't paginate; batch approach is correct |
| Manual JSON envelope per handler | `ResourceCollection::to_response()` | Phase 42 | Consistent `{data, links, meta}` across all endpoints |
| SeaORM offset pagination only | Offset + cursor pagination available | SeaORM 0.9+ | Cursor pagination for large datasets (no total count) |

**Relevant for Phase 42:**
- **SeaORM `LoaderTrait`:** Provides `load_one()`, `load_many()`, `load_many_to_many()` — parallel to Ferro's `BatchLoad`/`BatchLoadMany`. Could be used as alternative, but Ferro's batch loading is more ergonomic with its HashMap return type.
- **Cursor pagination:** SeaORM supports it via `.cursor_by()`. Phase 42 could optionally support cursor-based pagination in `ResourceCollection`, but offset pagination covers the standard API case. Cursor pagination can be deferred if scope is tight.

**Not applicable to Phase 42:**
- **`whenPivotLoaded()`:** Only relevant for many-to-many pivot data. Can be added later if Ferro apps use pivot tables with extra columns.
- **`preserveKeys`:** Laravel-specific for keyed collections. Ferro returns arrays (Vec), not keyed maps.
</sota_updates>

<open_questions>
## Open Questions

1. **Cursor-based pagination in Phase 42?**
   - What we know: SeaORM supports it via `.cursor_by()`. Useful for large datasets. No total count available.
   - What's unclear: Do Ferro users need cursor pagination now, or can it wait?
   - Recommendation: Defer cursor pagination. Offset pagination covers the standard API use case. Cursor can be added as a separate `CursorPaginationMeta` in a future phase if needed.

2. **ResourceCollection as derive macro or manual?**
   - What we know: Individual resources have `#[derive(ApiResource)]`. Collections are typically used in handlers, not defined as standalone types.
   - What's unclear: Should there be a `#[derive(ApiResourceCollection)]` macro?
   - Recommendation: No derive macro for collections. Collections are constructed in handlers from `Vec<T: Resource>`. The derive macro complexity isn't justified — `ResourceCollection::new(items)` is already simple.

3. **Pagination link URL construction**
   - What we know: Links need the current request URL with `page` query parameter updated.
   - What's unclear: How to extract the base URL from Ferro's `Request` type. Does it carry the full URL including host?
   - Recommendation: Use the request path + existing query parameters, replacing/adding `page=N`. If the host is not available, use relative URLs (e.g., `/users?page=2`). This is the safest approach and works behind reverse proxies.

4. **`with()` method on ResourceCollection**
   - What we know: Laravel's `with()` adds static metadata that's always included.
   - What's unclear: Is this needed alongside `.additional()`?
   - Recommendation: Start with just `.additional()` (set at call site). If a pattern emerges where collections always need the same metadata, add a `with()` trait method later.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- Ferro codebase — Phase 41 implementation: Resource trait, ResourceMap, ApiResource derive macro (`framework/src/http/resources/`)
- Ferro codebase — eager loading: `BatchLoad`, `BatchLoadMany`, `batch_load_has_many` (`framework/src/database/eager_loading.rs`)
- Ferro codebase — query builder: `QueryBuilder` with `all_with` methods (`framework/src/database/query_builder.rs`)
- [Laravel 12.x Eloquent: API Resources](https://laravel.com/docs/12.x/eloquent-resources) — full feature reference for relationship inclusion, collections, pagination, conditional attributes
- [SeaORM PaginatorTrait](https://docs.rs/sea-orm/latest/sea_orm/trait.PaginatorTrait.html) — pagination API, `fetch_page`, `num_items_and_pages`
- [SeaORM Cursor pagination](https://docs.rs/sea-orm/latest/sea_orm/struct.Cursor.html) — cursor-based alternative
- [SeaORM LoaderTrait](https://www.sea-ql.org/SeaORM/docs/relation/entity-loader/) — batch relation loading

### Secondary (MEDIUM confidence)
- [SeaORM SelectTwoMany](https://docs.rs/sea-orm/latest/sea_orm/query/struct.SelectTwoMany.html) — confirmed does NOT implement PaginatorTrait (verified via docs.rs)
- [Laravel API Resources with Relations](https://laraveldaily.com/post/laravel-api-resources-relations-when-methods) — whenLoaded patterns and N+1 prevention strategies

### Tertiary (LOW confidence — needs validation)
- None. All findings verified against official sources or codebase inspection.
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Ferro Resource/ResourceMap (Phase 41) + SeaORM pagination + Ferro batch loading
- Ecosystem: No new external dependencies
- Patterns: ResourceCollection, pagination envelope, explicit relationship inclusion, batch-load-then-map
- Pitfalls: Paginating joins, N+1 in resource construction, off-by-one pages

**Confidence breakdown:**
- Standard stack: HIGH — all internal, extends verified Phase 41 code
- Architecture: HIGH — patterns match existing Ferro conventions and SeaORM capabilities
- Pitfalls: HIGH — verified SeaORM limitations (SelectTwoMany pagination) against docs.rs
- Code examples: HIGH — designed from existing Ferro patterns and verified SeaORM API

**Research date:** 2026-02-10
**Valid until:** 2026-03-10 (30 days — Ferro internal patterns stable, SeaORM pagination API stable)
</metadata>

---

*Phase: 42-api-resources-advanced*
*Research completed: 2026-02-10*
*Ready for planning: yes*
