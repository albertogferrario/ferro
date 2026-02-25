//! Dependency graph tool - comprehensive application relationship graph
//!
//! Combines:
//! - Route-to-model relationships (from route_dependencies)
//! - Model-to-model relationships (from relation_map)
//! - Route-to-component relationships (Inertia)

use crate::error::Result;
use crate::tools::{list_models, list_routes, relation_map, route_dependencies};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct DependencyGraph {
    /// All nodes in the graph
    pub nodes: Vec<GraphNode>,
    /// All edges (relationships) in the graph
    pub edges: Vec<GraphEdge>,
    /// Summary statistics
    pub summary: GraphSummary,
}

#[derive(Debug, Serialize, Clone)]
pub struct GraphNode {
    /// Unique identifier for the node
    pub id: String,
    /// Type of node
    #[serde(rename = "type")]
    pub node_type: NodeType,
    /// Human-readable label
    pub label: String,
    /// Additional metadata
    pub metadata: Option<NodeMetadata>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    /// HTTP route/endpoint
    Route,
    /// Database model/entity
    Model,
    /// React/Inertia component
    Component,
}

#[derive(Debug, Serialize, Clone)]
pub struct NodeMetadata {
    /// File path for models/handlers
    pub path: Option<String>,
    /// HTTP method for routes
    pub method: Option<String>,
    /// Table name for models
    pub table: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct GraphEdge {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
    /// Type of relationship
    pub relationship: EdgeRelationship,
    /// Optional context
    pub context: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EdgeRelationship {
    /// Route uses a model (query, insert, update, delete)
    UsesModel,
    /// Model belongs to another model (FK relationship)
    BelongsTo,
    /// Model has many of another model (reverse FK)
    HasMany,
    /// Route renders a component
    Renders,
}

#[derive(Debug, Serialize)]
pub struct GraphSummary {
    /// Total number of routes in the graph
    pub total_routes: usize,
    /// Total number of models in the graph
    pub total_models: usize,
    /// Total number of components in the graph
    pub total_components: usize,
    /// Total number of relationships
    pub total_relationships: usize,
    /// Models with most dependencies
    pub most_used_models: Vec<ModelUsageCount>,
}

#[derive(Debug, Serialize)]
pub struct ModelUsageCount {
    pub model: String,
    pub route_count: usize,
}

pub async fn execute(project_root: &Path) -> Result<DependencyGraph> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seen_nodes: HashSet<String> = HashSet::new();
    let mut model_usage_counts: HashMap<String, usize> = HashMap::new();

    // 1. Collect all models as nodes
    let models = list_models::execute(project_root).unwrap_or_default();
    for model in &models {
        let node_id = format!("model:{}", model.name);
        if !seen_nodes.contains(&node_id) {
            seen_nodes.insert(node_id.clone());
            nodes.push(GraphNode {
                id: node_id,
                node_type: NodeType::Model,
                label: model.name.clone(),
                metadata: Some(NodeMetadata {
                    path: Some(model.path.clone()),
                    method: None,
                    table: model.table.clone(),
                }),
            });
        }
    }

    // 2. Get model-to-model relationships from relation_map
    if let Ok(relations) = relation_map::execute(project_root).await {
        for relation in &relations.relations {
            // Normalize table names to model names
            let from_model = table_to_model_name(&relation.from_table);
            let to_model = table_to_model_name(&relation.to_table);

            let from_id = format!("model:{from_model}");
            let to_id = format!("model:{to_model}");

            // Ensure both nodes exist
            if !seen_nodes.contains(&from_id) {
                seen_nodes.insert(from_id.clone());
                nodes.push(GraphNode {
                    id: from_id.clone(),
                    node_type: NodeType::Model,
                    label: from_model.clone(),
                    metadata: Some(NodeMetadata {
                        path: None,
                        method: None,
                        table: Some(relation.from_table.clone()),
                    }),
                });
            }

            if !seen_nodes.contains(&to_id) {
                seen_nodes.insert(to_id.clone());
                nodes.push(GraphNode {
                    id: to_id.clone(),
                    node_type: NodeType::Model,
                    label: to_model.clone(),
                    metadata: Some(NodeMetadata {
                        path: None,
                        method: None,
                        table: Some(relation.to_table.clone()),
                    }),
                });
            }

            // Add belongs_to edge
            edges.push(GraphEdge {
                from: from_id.clone(),
                to: to_id.clone(),
                relationship: EdgeRelationship::BelongsTo,
                context: Some(format!(
                    "{}.{} -> {}.{}",
                    relation.from_table,
                    relation.from_column,
                    relation.to_table,
                    relation.to_column
                )),
            });

            // Add reverse has_many edge
            edges.push(GraphEdge {
                from: to_id,
                to: from_id,
                relationship: EdgeRelationship::HasMany,
                context: Some(format!(
                    "{} has many {}",
                    relation.to_table, relation.from_table
                )),
            });
        }
    }

    // 3. Collect routes and their dependencies
    let routes_info = list_routes::execute(project_root).await?;
    let mut component_nodes: HashSet<String> = HashSet::new();

    for route in &routes_info.routes {
        if route.handler.is_empty() {
            continue;
        }

        let route_id = format!("route:{}:{}", route.method, route.path);

        // Add route node
        if !seen_nodes.contains(&route_id) {
            seen_nodes.insert(route_id.clone());
            nodes.push(GraphNode {
                id: route_id.clone(),
                node_type: NodeType::Route,
                label: format!("{} {}", route.method, route.path),
                metadata: Some(NodeMetadata {
                    path: Some(route.handler.clone()),
                    method: Some(route.method.clone()),
                    table: None,
                }),
            });
        }

        // Get route dependencies
        if let Ok(deps) = route_dependencies::execute(project_root, &route.path).await {
            // Add edges for model usage
            for model_usage in &deps.models_used {
                let model_id = format!("model:{}", model_usage.model);

                // Track model usage count
                *model_usage_counts
                    .entry(model_usage.model.clone())
                    .or_insert(0) += 1;

                // Ensure model node exists
                if !seen_nodes.contains(&model_id) {
                    seen_nodes.insert(model_id.clone());
                    nodes.push(GraphNode {
                        id: model_id.clone(),
                        node_type: NodeType::Model,
                        label: model_usage.model.clone(),
                        metadata: None,
                    });
                }

                edges.push(GraphEdge {
                    from: route_id.clone(),
                    to: model_id,
                    relationship: EdgeRelationship::UsesModel,
                    context: Some(format!("{:?}", model_usage.usage_type)),
                });
            }

            // Add edge for Inertia component
            if let Some(ref component) = deps.inertia_component {
                let component_id = format!("component:{component}");

                // Add component node if not already added
                if !component_nodes.contains(&component_id) {
                    component_nodes.insert(component_id.clone());
                    seen_nodes.insert(component_id.clone());
                    nodes.push(GraphNode {
                        id: component_id.clone(),
                        node_type: NodeType::Component,
                        label: component.clone(),
                        metadata: Some(NodeMetadata {
                            path: Some(format!("frontend/src/pages/{component}.tsx")),
                            method: None,
                            table: None,
                        }),
                    });
                }

                edges.push(GraphEdge {
                    from: route_id.clone(),
                    to: component_id,
                    relationship: EdgeRelationship::Renders,
                    context: None,
                });
            }
        }
    }

    // Build summary
    let total_routes = nodes
        .iter()
        .filter(|n| matches!(n.node_type, NodeType::Route))
        .count();
    let total_models = nodes
        .iter()
        .filter(|n| matches!(n.node_type, NodeType::Model))
        .count();
    let total_components = nodes
        .iter()
        .filter(|n| matches!(n.node_type, NodeType::Component))
        .count();

    // Get most used models
    let mut usage_vec: Vec<_> = model_usage_counts.into_iter().collect();
    usage_vec.sort_by(|a, b| b.1.cmp(&a.1));
    let most_used_models: Vec<ModelUsageCount> = usage_vec
        .into_iter()
        .take(5)
        .map(|(model, count)| ModelUsageCount {
            model,
            route_count: count,
        })
        .collect();

    let summary = GraphSummary {
        total_routes,
        total_models,
        total_components,
        total_relationships: edges.len(),
        most_used_models,
    };

    Ok(DependencyGraph {
        nodes,
        edges,
        summary,
    })
}

/// Convert a table name to a model name (e.g., "users" -> "User", "animal_tags" -> "AnimalTag")
fn table_to_model_name(table: &str) -> String {
    // Remove trailing 's' for plural tables, but not for words ending in "us" (status, corpus)
    // or "ss" (class, boss) which are typically singular
    let singular = if table.ends_with('s') && !table.ends_with("ss") && !table.ends_with("us") {
        &table[..table.len() - 1]
    } else {
        table
    };

    // Convert snake_case to PascalCase
    singular
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_to_model_name_simple() {
        assert_eq!(table_to_model_name("users"), "User");
        assert_eq!(table_to_model_name("animals"), "Animal");
    }

    #[test]
    fn test_table_to_model_name_compound() {
        assert_eq!(table_to_model_name("animal_tags"), "AnimalTag");
        assert_eq!(table_to_model_name("user_profiles"), "UserProfile");
    }

    #[test]
    fn test_table_to_model_name_no_plural() {
        assert_eq!(table_to_model_name("status"), "Status"); // doesn't end in 's' that should be stripped
    }

    #[test]
    fn test_graph_node_serialization() {
        let node = GraphNode {
            id: "model:User".to_string(),
            node_type: NodeType::Model,
            label: "User".to_string(),
            metadata: Some(NodeMetadata {
                path: Some("src/models/user.rs".to_string()),
                method: None,
                table: Some("users".to_string()),
            }),
        };

        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("\"id\":\"model:User\""));
        assert!(json.contains("\"type\":\"model\""));
    }

    #[test]
    fn test_graph_edge_serialization() {
        let edge = GraphEdge {
            from: "route:GET:/users".to_string(),
            to: "model:User".to_string(),
            relationship: EdgeRelationship::UsesModel,
            context: Some("EntityQuery".to_string()),
        };

        let json = serde_json::to_string(&edge).unwrap();
        assert!(json.contains("\"from\":\"route:GET:/users\""));
        assert!(json.contains("\"relationship\":\"uses_model\""));
    }
}
