//! `ferro ai:explain <target>` — AI-powered service/route/model explanation.
//!
//! Resolves the target in SERVICE → ROUTE → MODEL order (projection-framed is
//! primary; prose fallback when no projection exists). Assembles a prompt from
//! introspected facts and produces prose via a raw `CompletionRequest { schema:
//! None, .. }` call. `--dry-run` prints the assembled prompt without calling
//! the LLM.
//!
//! All resolution and prompt-building logic lives in
//! `ferro_mcp::tools::ai_explain_core`. This module is a thin CLI wrapper:
//! tokio runtime bridge, console output, process exit.

// ---------------------------------------------------------------------------
// Command entry point
// ---------------------------------------------------------------------------

/// Run the `ferro ai:explain <target>` command.
///
/// Thin wrapper over the relocated `ferro_mcp::tools::ai_explain_core` public
/// items. CLI behavior is unchanged: prose-only output, `--dry-run` prints the
/// assembled prompt, no LLM call in dry-run mode.
#[cfg(feature = "projections")]
pub fn run(target: String, type_override: Option<String>, dry_run: bool) {
    use console::style;
    use ferro_ai::client::{Message, Role};
    use ferro_ai::{AiConfig, CompletionRequest};
    use ferro_mcp::tools::ai_explain_core::{
        build_model_prompt, build_route_prompt, build_service_prompt, resolve_max_tokens_with_default,
        resolve_target, ResolvedTarget,
    };

    // 1. Fail-fast: require AI provider unless --dry-run (D-06).
    //    In dry-run mode, AI config is NOT checked — the assembled prompt is
    //    printed and the function returns without calling the LLM or requiring
    //    any env vars to be set.
    let client_result = AiConfig::from_env();
    if !dry_run {
        if let Err(ref e) = client_result {
            eprintln!(
                "{} AI provider not configured: {e}\n  Set FERRO_AI_PROVIDER, FERRO_AI_API_KEY, and FERRO_AI_MODEL.",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    }

    // 2. Tokio runtime bridge
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "{} Failed to create tokio runtime: {e}",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    };

    // 3. Resolve target (service → route → model) via the relocated async core.
    let resolved = rt.block_on(resolve_target(
        std::path::Path::new("."),
        &target,
        type_override.as_deref(),
    ));

    match resolved {
        ResolvedTarget::NotFound(msg) => {
            eprintln!("{} {msg}", style("Error:").red().bold());
            std::process::exit(1);
        }
        resolved => {
            // 4. Build prompt using the relocated pub builders.
            let (system_prompt, user_prompt) = match &resolved {
                ResolvedTarget::Service(d) => build_service_prompt(d),
                ResolvedTarget::Route(r) => build_route_prompt(r),
                ResolvedTarget::Model(m) => build_model_prompt(m),
                ResolvedTarget::NotFound(_) => unreachable!(),
            };

            // 5. --dry-run: print assembled prompt and return (no LLM call)
            if dry_run {
                println!("{system_prompt}");
                println!("---");
                println!("{user_prompt}");
                return;
            }

            // 6. Cost guard (default 2048 for ai:explain)
            let max_tokens = resolve_max_tokens_with_default(2048);

            // 7. Raw prose completion — schema: None (unstructured, no JSON coercion)
            let client = client_result.expect("already validated above");

            let req = CompletionRequest {
                system: Some(system_prompt),
                messages: vec![Message {
                    role: Role::User,
                    content: user_prompt,
                    tool_call_id: None,
                }],
                max_tokens,
                model_override: None,
                schema: None,
                tools: None,
                tool_choice: None,
            };

            match rt.block_on(client.complete(req)) {
                Ok(prose) => {
                    println!("{prose}");
                }
                Err(e) => {
                    eprintln!(
                        "{} LLM completion failed: {e}",
                        style("Error:").red().bold()
                    );
                    std::process::exit(1);
                }
            }
        }
    }
}
