use serde::Serialize;

/// Pagination metadata for API collection responses.
///
/// Computes page information from 1-indexed page input. Designed for use
/// with SeaORM's `PaginatorTrait` — pass `current_page` as the API-facing
/// 1-indexed page number, not SeaORM's 0-indexed offset.
///
/// # Example
///
/// ```rust,ignore
/// let meta = PaginationMeta::new(1, 15, 42);
/// assert_eq!(meta.last_page, 3);
/// assert_eq!(meta.from, 1);
/// assert_eq!(meta.to, 15);
/// ```
#[derive(Serialize, Clone, Debug)]
pub struct PaginationMeta {
    pub current_page: u64,
    pub per_page: u64,
    pub total: u64,
    pub last_page: u64,
    pub from: u64,
    pub to: u64,
}

impl PaginationMeta {
    /// Create pagination metadata from 1-indexed page, items per page, and total item count.
    ///
    /// Computes `last_page` via ceiling division, `from` as 1-based first item
    /// index on the page, and `to` as 1-based last item index. Handles empty
    /// results (from=0, to=0) and partial last pages.
    pub fn new(current_page: u64, per_page: u64, total: u64) -> Self {
        let per_page = per_page.max(1);
        let last_page = if total == 0 {
            1
        } else {
            (total + per_page - 1) / per_page
        };

        let (from, to) = if total == 0 || current_page > last_page {
            (0, 0)
        } else {
            let from = (current_page - 1) * per_page + 1;
            let to = (current_page * per_page).min(total);
            (from, to)
        };

        Self {
            current_page,
            per_page,
            total,
            last_page,
            from,
            to,
        }
    }

    /// Generate pagination links from a request path and optional query string.
    ///
    /// Produces relative URLs (path-based). The `page` query parameter is
    /// replaced or added in each link. Other query parameters are preserved.
    pub fn links(&self, path: &str, query: Option<&str>) -> PaginationLinks {
        let first = build_url(path, query, 1);
        let last = build_url(path, query, self.last_page);

        let prev = if self.current_page > 1 {
            Some(build_url(path, query, self.current_page - 1))
        } else {
            None
        };

        let next = if self.current_page < self.last_page {
            Some(build_url(path, query, self.current_page + 1))
        } else {
            None
        };

        PaginationLinks {
            first,
            last,
            prev,
            next,
        }
    }
}

/// Pagination navigation links for API collection responses.
///
/// Contains URLs for first, last, previous, and next pages.
/// `prev` is `None` on the first page; `next` is `None` on the last page.
#[derive(Serialize, Clone, Debug)]
pub struct PaginationLinks {
    pub first: String,
    pub last: String,
    pub prev: Option<String>,
    pub next: Option<String>,
}

/// Build a URL by replacing or adding the `page` parameter in the query string.
///
/// Preserves existing query parameters and uses `form_urlencoded` for proper encoding.
fn build_url(path: &str, query: Option<&str>, page: u64) -> String {
    let mut params: Vec<(String, String)> = if let Some(q) = query {
        form_urlencoded::parse(q.as_bytes())
            .filter(|(key, _)| key != "page")
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect()
    } else {
        Vec::new()
    };

    params.push(("page".to_string(), page.to_string()));

    let encoded: String = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(&params)
        .finish();

    format!("{}?{}", path, encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_basic() {
        let meta = PaginationMeta::new(1, 15, 45);
        assert_eq!(meta.current_page, 1);
        assert_eq!(meta.per_page, 15);
        assert_eq!(meta.total, 45);
        assert_eq!(meta.last_page, 3);
        assert_eq!(meta.from, 1);
        assert_eq!(meta.to, 15);
    }

    #[test]
    fn test_meta_last_page() {
        let meta = PaginationMeta::new(3, 15, 45);
        assert_eq!(meta.current_page, 3);
        assert_eq!(meta.last_page, 3);
        assert_eq!(meta.from, 31);
        assert_eq!(meta.to, 45);
    }

    #[test]
    fn test_meta_partial_last_page() {
        let meta = PaginationMeta::new(3, 15, 42);
        assert_eq!(meta.last_page, 3);
        assert_eq!(meta.from, 31);
        assert_eq!(meta.to, 42);
    }

    #[test]
    fn test_meta_empty() {
        let meta = PaginationMeta::new(1, 15, 0);
        assert_eq!(meta.last_page, 1);
        assert_eq!(meta.from, 0);
        assert_eq!(meta.to, 0);
    }

    #[test]
    fn test_meta_single_page() {
        let meta = PaginationMeta::new(1, 15, 5);
        assert_eq!(meta.last_page, 1);
        assert_eq!(meta.from, 1);
        assert_eq!(meta.to, 5);
    }

    #[test]
    fn test_links_first_page() {
        let meta = PaginationMeta::new(1, 15, 45);
        let links = meta.links("/users", None);
        assert_eq!(links.first, "/users?page=1");
        assert_eq!(links.last, "/users?page=3");
        assert!(links.prev.is_none());
        assert_eq!(links.next, Some("/users?page=2".to_string()));
    }

    #[test]
    fn test_links_last_page() {
        let meta = PaginationMeta::new(3, 15, 45);
        let links = meta.links("/users", None);
        assert_eq!(links.first, "/users?page=1");
        assert_eq!(links.last, "/users?page=3");
        assert_eq!(links.prev, Some("/users?page=2".to_string()));
        assert!(links.next.is_none());
    }

    #[test]
    fn test_links_middle_page() {
        let meta = PaginationMeta::new(2, 15, 45);
        let links = meta.links("/users", None);
        assert_eq!(links.first, "/users?page=1");
        assert_eq!(links.last, "/users?page=3");
        assert_eq!(links.prev, Some("/users?page=1".to_string()));
        assert_eq!(links.next, Some("/users?page=3".to_string()));
    }

    #[test]
    fn test_links_single_page() {
        let meta = PaginationMeta::new(1, 15, 5);
        let links = meta.links("/users", None);
        assert_eq!(links.first, "/users?page=1");
        assert_eq!(links.last, "/users?page=1");
        assert!(links.prev.is_none());
        assert!(links.next.is_none());
    }

    #[test]
    fn test_links_preserves_query_params() {
        let meta = PaginationMeta::new(1, 15, 45);
        let links = meta.links("/users", Some("sort=name&page=1"));
        assert_eq!(links.first, "/users?sort=name&page=1");
        assert_eq!(links.last, "/users?sort=name&page=3");
        assert!(links.prev.is_none());
        assert_eq!(links.next, Some("/users?sort=name&page=2".to_string()));
    }
}
