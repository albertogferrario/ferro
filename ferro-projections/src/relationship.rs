use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Structural cardinality of a service-to-service relationship.
///
/// Standard ER cardinality covering the four relationship types.
/// Each variant maps to a default [`NavigationHint`] via [`Cardinality::default_navigation`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

impl Cardinality {
    /// Returns the default navigation hint for this cardinality.
    ///
    /// - `OneToOne` -> `Inline` (embed related data in current view)
    /// - `ManyToOne` -> `Link` (navigable link to parent entity)
    /// - `OneToMany` -> `Nested` (nested list within current view)
    /// - `ManyToMany` -> `Nested` (nested list within current view)
    pub fn default_navigation(&self) -> NavigationHint {
        match self {
            Cardinality::OneToOne => NavigationHint::Inline,
            Cardinality::ManyToOne => NavigationHint::Link,
            Cardinality::OneToMany => NavigationHint::Nested,
            Cardinality::ManyToMany => NavigationHint::Nested,
        }
    }
}

/// Presentational hint for how a relationship should be rendered in UI.
///
/// Bridges the gap between structural relationships and UI presentation.
/// Defaults are derived from [`Cardinality`] and can be overridden per relationship.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NavigationHint {
    /// Embed related data in current view (e.g., customer name on order card).
    Inline,
    /// Show as navigable link to related entity.
    Link,
    /// Show as separate tab in detail view.
    Tab,
    /// Show as nested list/table within current view.
    Nested,
    /// Relationship exists but not shown in default navigation.
    Hidden,
}

/// A service-to-service relationship declaration.
///
/// Each service declares its own relationships independently. The `inverse` field
/// is a documentation hint, not a hard reference. Relationships carry two dimensions:
/// structural (cardinality) and presentational (navigation hint).
///
/// ```
/// use ferro_projections::{RelationshipDef, Cardinality, NavigationHint};
///
/// let rel = RelationshipDef::new("customer", "customer", Cardinality::ManyToOne)
///     .foreign_key("customer_id")
///     .inverse("orders")
///     .navigation(NavigationHint::Link)
///     .description("Customer who placed this order");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct RelationshipDef {
    /// Relationship name (e.g., "customer", "line_items").
    pub name: String,
    /// Target service name (e.g., "customer", "order_line_item").
    pub target: String,
    /// Structural cardinality of the relationship.
    pub cardinality: Cardinality,
    /// How the renderer should present this relationship.
    pub navigation: NavigationHint,
    /// Foreign key field name on the owning side.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreign_key: Option<String>,
    /// Name of the inverse relationship on the target service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inverse: Option<String>,
    /// Human-readable description of the relationship.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl RelationshipDef {
    /// Creates a new relationship definition with default navigation from cardinality.
    pub fn new(
        name: impl Into<String>,
        target: impl Into<String>,
        cardinality: Cardinality,
    ) -> Self {
        Self {
            name: name.into(),
            target: target.into(),
            navigation: cardinality.default_navigation(),
            cardinality,
            foreign_key: None,
            inverse: None,
            description: None,
        }
    }

    /// Sets the foreign key field name.
    pub fn foreign_key(mut self, fk: impl Into<String>) -> Self {
        self.foreign_key = Some(fk.into());
        self
    }

    /// Sets the inverse relationship name on the target service.
    pub fn inverse(mut self, inverse: impl Into<String>) -> Self {
        self.inverse = Some(inverse.into());
        self
    }

    /// Overrides the default navigation hint.
    pub fn navigation(mut self, hint: NavigationHint) -> Self {
        self.navigation = hint;
        self
    }

    /// Sets the relationship description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}
