# Phase 7 Plan 1 Summary: MCP Relationship Visibility

## Outcome

**Status:** Complete

All 4 tasks completed successfully with atomic commits. The cancer-mcp server now provides comprehensive relationship visibility tools for understanding route-model-component dependencies.

## What Was Built

### Task 1: route_dependencies Tool (210ba0e)

Analyzes a route handler's source code to identify which models it uses.

**Detection patterns:**
- `Entity::find_by_id`, `Entity::find()` - EntityQuery usage
- `ActiveModel { ... }` - Mutation operations
- `Column::field_name` - Column filtering
- Model imports via `use ... models::`
- Service injection via `App::resolve::<Service>`

**Returns:**
- `models_used`: Vec with model name, usage type, code context
- `inertia_component`: Optional component being rendered
- `services_used`: Vec of injected service names

5 unit tests verify model detection accuracy.

### Task 2: model_usages Tool (0f17a33)

Inverse of route_dependencies - finds all routes that reference a given model.

**Features:**
- Scans all routes via list_routes
- Cross-references with route_dependencies for each route
- Aggregates usage types (query, mutation, filter)

**Returns:**
- `model`: Target model name
- `routes`: Vec with path, method, handler, usage_types, component
- `summary`: total_routes, query_routes, mutation_routes, filter_routes

2 unit tests verify serialization.

### Task 3: dependency_graph Tool (d5b98cb)

Comprehensive dependency graph combining all relationship types.

**Graph nodes:**
- `route:METHOD:/path` - HTTP endpoints
- `model:ModelName` - Database entities
- `component:Component/Path` - Inertia components

**Graph edges:**
- `uses_model` - Route queries/modifies a model
- `belongs_to` - Model FK relationship (source has FK)
- `has_many` - Reverse FK relationship
- `renders` - Route renders Inertia component

**Returns:**
- `nodes`: Vec with id, type, label, metadata
- `edges`: Vec with from, to, relationship, context
- `summary`: total_routes, total_models, total_components, total_relationships, most_used_models

Helper function `table_to_model_name` converts DB table names to PascalCase model names, handling pluralization edge cases (e.g., "status" stays "Status", not "Statu").

5 unit tests verify conversion and serialization.

### Task 4: Documentation Updates (608d1af)

Enhanced CANCER_MCP_INSTRUCTIONS with relationship analysis guidance:

**New workflows:**
- "Understanding Relationships" - Full impact analysis
- "Planning a Model Refactor" - Safe model changes

**New USE guidelines:**
- `route_dependencies` - What models a route needs
- `model_usages` - Impact before model changes
- `dependency_graph` - Architecture visualization

**New tool category:**
- "Relationship Analysis" grouping: route_dependencies, model_usages, dependency_graph, relation_map

**Cross-linking:**
- Updated relation_map to reference dependency_graph

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 210ba0e | feat(07-01): implement route_dependencies tool |
| 2 | 0f17a33 | feat(07-01): implement model_usages tool |
| 3 | d5b98cb | feat(cancer-mcp): add dependency_graph tool for architecture visualization |
| 4 | 608d1af | docs(cancer-mcp): add relationship analysis workflows and tool category |

## Files Created

- `cancer-mcp/src/tools/route_dependencies.rs` - Handler analysis for model detection
- `cancer-mcp/src/tools/model_usages.rs` - Reverse lookup for model references
- `cancer-mcp/src/tools/dependency_graph.rs` - Full architecture graph

## Files Modified

- `cancer-mcp/src/tools/mod.rs` - Export new modules
- `cancer-mcp/src/service.rs` - Tool handlers, params, descriptions, instructions

## Test Results

All 54 cancer-mcp tests pass:
- 5 new route_dependencies tests
- 2 new model_usages tests
- 5 new dependency_graph tests
- All existing tests unchanged

## Decisions Made

1. **Regex caching with once_cell**: Used `once_cell::sync::Lazy` to cache compiled regex patterns for model detection, avoiding recompilation overhead.

2. **Known models validation**: route_dependencies only reports models that exist in `list_models`, reducing false positives from similarly-named types.

3. **Deduplication by (model, usage_type)**: A model appearing multiple times with same usage type is only reported once.

4. **Graph node IDs**: Use prefixed format (`route:`, `model:`, `component:`) for unique, type-safe identification.

5. **Bidirectional FK edges**: Both `belongs_to` and `has_many` edges are created for each FK relationship, enabling traversal in either direction.

## How to Use

**Impact analysis before model changes:**
```
> model_usages User
{
  "model": "User",
  "routes": [
    { "path": "/users", "method": "GET", "usage_types": ["entity_query"] },
    { "path": "/users", "method": "POST", "usage_types": ["active_model"] },
    { "path": "/login", "method": "POST", "usage_types": ["entity_query", "column_filter"] }
  ],
  "summary": { "total_routes": 3, "query_routes": 2, "mutation_routes": 1 }
}
```

**Understanding a specific route:**
```
> route_dependencies /users/{id}
{
  "route": "/users/{id}",
  "method": "GET",
  "models_used": [
    { "model": "User", "usage_type": "entity_query", "context": "User::Entity::find_by_id(id)" }
  ],
  "inertia_component": "Users/Show",
  "services_used": []
}
```

**Full architecture overview:**
```
> dependency_graph
{
  "nodes": [...],
  "edges": [...],
  "summary": {
    "total_routes": 15,
    "total_models": 8,
    "total_components": 12,
    "total_relationships": 45,
    "most_used_models": [
      { "model": "User", "route_count": 5 },
      { "model": "Animal", "route_count": 4 }
    ]
  }
}
```

## Next Steps

Phase 7 Plan 1 complete. Ready to proceed with additional MCP enhancements or the next milestone phase.
