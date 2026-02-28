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

#[cfg(test)]
mod tests {
    use super::*;

    // -- Type construction --

    #[test]
    fn relationship_def_new_sets_defaults() {
        let rel = RelationshipDef::new("customer", "customer", Cardinality::ManyToOne);
        assert_eq!(rel.name, "customer");
        assert_eq!(rel.target, "customer");
        assert_eq!(rel.cardinality, Cardinality::ManyToOne);
        assert_eq!(rel.navigation, NavigationHint::Link);
        assert!(rel.foreign_key.is_none());
        assert!(rel.inverse.is_none());
        assert!(rel.description.is_none());
    }

    #[test]
    fn relationship_def_builder_chain() {
        let rel = RelationshipDef::new("customer", "customer", Cardinality::ManyToOne)
            .foreign_key("customer_id")
            .inverse("orders")
            .navigation(NavigationHint::Tab)
            .description("Customer who placed this order");

        assert_eq!(rel.name, "customer");
        assert_eq!(rel.target, "customer");
        assert_eq!(rel.cardinality, Cardinality::ManyToOne);
        assert_eq!(rel.navigation, NavigationHint::Tab);
        assert_eq!(rel.foreign_key.as_deref(), Some("customer_id"));
        assert_eq!(rel.inverse.as_deref(), Some("orders"));
        assert_eq!(
            rel.description.as_deref(),
            Some("Customer who placed this order")
        );
    }

    #[test]
    fn cardinality_default_navigation() {
        assert_eq!(
            Cardinality::OneToOne.default_navigation(),
            NavigationHint::Inline
        );
        assert_eq!(
            Cardinality::ManyToOne.default_navigation(),
            NavigationHint::Link
        );
        assert_eq!(
            Cardinality::OneToMany.default_navigation(),
            NavigationHint::Nested
        );
        assert_eq!(
            Cardinality::ManyToMany.default_navigation(),
            NavigationHint::Nested
        );
    }

    // -- Serde round-trip --

    #[test]
    fn relationship_def_serde_round_trip() {
        let rel = RelationshipDef::new("customer", "customer", Cardinality::ManyToOne)
            .foreign_key("customer_id")
            .inverse("orders")
            .navigation(NavigationHint::Link)
            .description("Customer who placed this order");

        let json = serde_json::to_string(&rel).unwrap();
        let parsed: RelationshipDef = serde_json::from_str(&json).unwrap();
        assert_eq!(rel, parsed);
    }

    #[test]
    fn relationship_def_json_omits_none_fields() {
        let rel = RelationshipDef::new("items", "item", Cardinality::OneToMany);
        let json = serde_json::to_string(&rel).unwrap();
        assert!(!json.contains("foreign_key"));
        assert!(!json.contains("inverse"));
        assert!(!json.contains("description"));
        // Required fields are present
        assert!(json.contains("name"));
        assert!(json.contains("target"));
        assert!(json.contains("cardinality"));
        assert!(json.contains("navigation"));
    }

    #[test]
    fn cardinality_serde_values() {
        assert_eq!(
            serde_json::to_string(&Cardinality::OneToOne).unwrap(),
            r#""one_to_one""#
        );
        assert_eq!(
            serde_json::to_string(&Cardinality::OneToMany).unwrap(),
            r#""one_to_many""#
        );
        assert_eq!(
            serde_json::to_string(&Cardinality::ManyToOne).unwrap(),
            r#""many_to_one""#
        );
        assert_eq!(
            serde_json::to_string(&Cardinality::ManyToMany).unwrap(),
            r#""many_to_many""#
        );

        // Round-trip all variants
        for card in [
            Cardinality::OneToOne,
            Cardinality::OneToMany,
            Cardinality::ManyToOne,
            Cardinality::ManyToMany,
        ] {
            let json = serde_json::to_string(&card).unwrap();
            let parsed: Cardinality = serde_json::from_str(&json).unwrap();
            assert_eq!(card, parsed);
        }
    }

    #[test]
    fn navigation_hint_serde_values() {
        assert_eq!(
            serde_json::to_string(&NavigationHint::Inline).unwrap(),
            r#""inline""#
        );
        assert_eq!(
            serde_json::to_string(&NavigationHint::Link).unwrap(),
            r#""link""#
        );
        assert_eq!(
            serde_json::to_string(&NavigationHint::Tab).unwrap(),
            r#""tab""#
        );
        assert_eq!(
            serde_json::to_string(&NavigationHint::Nested).unwrap(),
            r#""nested""#
        );
        assert_eq!(
            serde_json::to_string(&NavigationHint::Hidden).unwrap(),
            r#""hidden""#
        );

        // Round-trip all variants
        for hint in [
            NavigationHint::Inline,
            NavigationHint::Link,
            NavigationHint::Tab,
            NavigationHint::Nested,
            NavigationHint::Hidden,
        ] {
            let json = serde_json::to_string(&hint).unwrap();
            let parsed: NavigationHint = serde_json::from_str(&json).unwrap();
            assert_eq!(hint, parsed);
        }
    }

    // -- JSON Schema --

    #[test]
    fn relationship_def_json_schema() {
        let schema = schemars::schema_for!(RelationshipDef);
        let value = schema.to_value();
        let props = value
            .get("properties")
            .expect("RelationshipDef schema must have properties");
        let obj = props.as_object().unwrap();
        assert!(obj.contains_key("name"), "missing 'name' property");
        assert!(obj.contains_key("target"), "missing 'target' property");
        assert!(
            obj.contains_key("cardinality"),
            "missing 'cardinality' property"
        );
        assert!(
            obj.contains_key("navigation"),
            "missing 'navigation' property"
        );
        assert!(
            obj.contains_key("foreign_key"),
            "missing 'foreign_key' property"
        );
        assert!(obj.contains_key("inverse"), "missing 'inverse' property");
    }
}
