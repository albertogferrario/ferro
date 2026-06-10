use ferro_projections::render::Renderer;
use ferro_projections::{Error as ProjError, IntentScore, ServiceDef};
use rmcp::model::Tool;

/// Context for MCP rendering. Carries no state in Phase 197;
/// Phase 200 will extend with tenant/policy context.
#[derive(Debug, Clone, Default)]
pub struct McpContext;

/// Renders a `ServiceDef` projection into an MCP tool definition.
pub struct McpRenderer;

impl Renderer for McpRenderer {
    type Output = Tool;
    type Context = McpContext;

    fn render(
        &self,
        _service: &ServiceDef,
        _intents: &[IntentScore],
        _ctx: &McpContext,
    ) -> std::result::Result<Tool, ProjError> {
        // Implemented in plan 02.
        Err(ProjError::Render("not yet implemented".into()))
    }
}
