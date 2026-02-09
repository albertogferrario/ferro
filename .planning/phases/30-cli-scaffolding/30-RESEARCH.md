# Phase 30: CLI Scaffolding - Research (Retroactive)

**Researched:** 2026-02-09
**Domain:** AI-powered code generation via Anthropic API in Rust CLI
**Confidence:** HIGH

<research_summary>
## Summary

Researched best practices for AI-powered CLI code generation, focusing on Anthropic API optimization, prompt engineering for code output, and how expert frameworks (Laravel Boost) approach the same problem.

The current Ferro `make:json-view` implementation works but leaves significant quality, cost, and UX improvements on the table. The most impactful gaps are: (1) no system prompt separation — everything is jammed into the user message, (2) no few-shot example — the model has zero reference of correct output, (3) no assistant prefill — missing the technique that eliminates markdown fences, (4) no prompt caching — the component catalog is re-sent as new tokens every call, (5) expensive default model — Opus is overkill for structured code generation from templates.

**Primary recommendation:** Use system prompt for role/catalog, add one few-shot example of a correct view, use assistant prefill to force `//!` start, enable prompt caching, default to Sonnet, and add a request timeout.
</research_summary>

<standard_stack>
## Standard Stack

### Core (already used)
| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| reqwest | 0.12 | HTTP client (blocking+json) | Correct choice |
| serde_json | * | JSON body construction | Correct choice |
| regex | * | Model field extraction | Correct choice |

### Recommended Additions
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| reqwest (streaming) | 0.12 | SSE streaming for real-time output | Better UX than spinner-then-dump |
| reqwest-eventsource | 0.6+ | SSE event parsing | Standard for Anthropic streaming API |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| reqwest blocking | reqwest async | Would need tokio runtime in CLI main; blocking is simpler |
| Custom API client | anthropic-sdk-rust | No official Rust SDK exists; custom client is correct |
| regex model scanning | syn crate | syn is heavy/slow for extracting struct fields; regex is pragmatic |
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Pattern 1: System Prompt Separation (MISSING)
**What:** Anthropic best practice is to use the `system` parameter for role definition and static context, keeping the `user` message for task-specific instructions.
**Why it matters:** System prompts improve output quality by clearly separating role/context from task. Anthropic docs: "Use the system parameter to set Claude's role. Put everything else, like task-specific instructions, in the user turn."
**Current state:** Everything (role, catalog, models, routes, instructions) is crammed into a single user message.
**Recommended:**
```json
{
  "system": [
    {"type": "text", "text": "You are a Ferro framework JSON-UI view code generator..."},
    {"type": "text", "text": "<component catalog>", "cache_control": {"type": "ephemeral"}}
  ],
  "messages": [
    {"role": "user", "content": "<project models + routes + instructions + description>"},
    {"role": "assistant", "content": "//!"}
  ]
}
```

### Pattern 2: Assistant Prefill (MISSING)
**What:** Pre-fill the assistant response with the beginning of the expected output to force format compliance and skip preambles.
**Why it matters:** Eliminates need for `strip_markdown_fences()`. The model continues from `//!` directly into code.
**Current state:** Model sometimes wraps output in ```rust fences despite "no fences" instruction; we strip them post-hoc.
**Recommended:** Add `{"role": "assistant", "content": "//!"}` to messages array.
**Limitation:** Not compatible with extended thinking mode (not relevant for this use case).

### Pattern 3: Few-Shot Example (MISSING)
**What:** Include 1+ complete example of correct output in the prompt.
**Why it matters:** Anthropic docs: "Examples reduce misinterpretation. 3-5 diverse examples boost performance." For code generation, even 1 example dramatically improves format consistency.
**Current state:** Zero examples. The model must infer correct output format entirely from the component catalog and instructions.
**Recommended:** Include one complete view file as `<example>` in the system prompt.

### Pattern 4: Prompt Caching (MISSING)
**What:** Mark static content (system prompt with component catalog) with `cache_control: {"type": "ephemeral"}` for automatic caching.
**Why it matters:** Component catalog is ~1500 tokens, identical for every call. Prompt caching reduces cost by 90% and latency by up to 85% for cached content.
**Current state:** Every API call re-processes the full component catalog as new input tokens.
**Minimum cacheable:** 1024 tokens for Sonnet models — our catalog exceeds this.
**Recommended:** Mark system prompt block with `cache_control`.

### Pattern 5: Request Timeout (MISSING)
**What:** Set explicit timeout on the HTTP request.
**Why it matters:** Without timeout, a slow network or API issue hangs the CLI indefinitely.
**Current state:** No timeout configured on reqwest client.
**Recommended:** 60-second timeout via `reqwest::blocking::Client::builder().timeout(Duration::from_secs(60)).build()`.

### Anti-Patterns to Avoid
- **Cramming everything in user message:** Hurts output quality. Use system prompt for role + static context.
- **Post-hoc fence stripping:** Symptom of poor prompting. Prefill eliminates the root cause.
- **No examples for code generation:** Leads to inconsistent output format, wrong import patterns.
- **Defaulting to most expensive model:** Opus is 5x more expensive than Sonnet for a structured code generation task that Sonnet handles well.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Markdown fence stripping | `strip_markdown_fences()` regex | Assistant prefill technique | Eliminates the problem at source — model never adds fences |
| Repeated prompt optimization | Nothing (accept cost) | Prompt caching API feature | 90% cost reduction, 85% latency reduction for cached tokens |
| Output format enforcement | Post-hoc string cleanup | Few-shot examples + prefill | Model produces correct format from the start |
| API client resilience | Hope for the best | Timeout + retry with backoff | Network issues are inevitable in CLI tools |

**Key insight:** Most of the current "workarounds" (fence stripping, spinner-then-dump) exist because the API isn't being used according to Anthropic's own best practices. Using system prompts, prefill, and caching eliminates these problems at the source.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: No Few-Shot Example
**What goes wrong:** AI generates code with wrong import paths, incorrect builder patterns, or non-existent types.
**Why it happens:** Without a concrete example, the model must guess the exact API from a compact catalog description.
**How to avoid:** Include one complete view file as an example in the prompt. Use the static template output as the example — it's already a valid view.
**Warning signs:** Generated code uses `Component::text(...)` instead of `Component::Text(TextProps { ... })`, or imports from wrong paths.

### Pitfall 2: Expensive Model Default
**What goes wrong:** Each `make:json-view` call costs ~$0.10-0.25 with Opus for input+output tokens.
**Why it happens:** Opus (claude-opus-4-6) is the default model. For this structured code generation task, Sonnet produces equivalent quality at 1/5 the cost.
**How to avoid:** Default to `claude-sonnet-4-5` (or latest Sonnet). Let users override with `FERRO_AI_MODEL` for Opus if they want.
**Warning signs:** User complaints about API costs for a scaffolding command.

### Pitfall 3: No Request Timeout
**What goes wrong:** CLI hangs indefinitely on slow network or API outage.
**Why it happens:** Default reqwest client has no timeout.
**How to avoid:** Configure 60s timeout on the client builder.
**Warning signs:** User reports "ferro make:json-view" hangs forever.

### Pitfall 4: Temperature Not Set
**What goes wrong:** Inconsistent output between runs — sometimes correct imports, sometimes hallucinated ones.
**Why it happens:** Default temperature allows more creativity. Code generation benefits from lower temperature (more deterministic).
**How to avoid:** Set `temperature: 0.2` in the API request body. Best practice for code generation per Anthropic docs.
**Warning signs:** Same prompt produces different import styles or component patterns across runs.

### Pitfall 5: No Existing View Context
**What goes wrong:** AI generates views inconsistent with existing views in the project.
**Why it happens:** Only models and routes are scanned for context. Existing views are ignored.
**How to avoid:** Scan `src/views/*.rs` for existing view patterns and include 1-2 as context (like Laravel Boost does).
**Warning signs:** Generated views use different builder patterns or styles than hand-written views in the same project.
</common_pitfalls>

<code_examples>
## Code Examples

### System Prompt + Prefill Pattern (recommended)
```rust
// Recommended API call structure
let body = serde_json::json!({
    "model": model,
    "max_tokens": 8192,
    "temperature": 0.2,
    "system": [
        {
            "type": "text",
            "text": "You are a Ferro framework JSON-UI view code generator. Generate only valid Rust source code."
        },
        {
            "type": "text",
            "text": component_catalog_with_example,
            "cache_control": {"type": "ephemeral"}
        }
    ],
    "messages": [
        {"role": "user", "content": user_prompt},
        {"role": "assistant", "content": "//!"}
    ]
});
```

### Prompt Caching API Format
```json
{
  "system": [
    {
      "type": "text",
      "text": "Role and instructions here..."
    },
    {
      "type": "text",
      "text": "Component catalog + few-shot example here (1500+ tokens)...",
      "cache_control": {"type": "ephemeral"}
    }
  ]
}
```
Cache hit on second call: 90% cheaper, 85% faster for cached prefix.

### Client with Timeout
```rust
let client = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(60))
    .build()
    .map_err(|e| format!("Failed to create HTTP client: {e}"))?;
```

### Few-Shot Example Structure
```
<example>
Input: user_list view showing all users in a table with edit/delete actions
Output:
//! User List JSON-UI view

use ferro::{
    Action, CardProps, ColumnFormat, Component, ComponentNode, JsonUiView, TableColumn, TableProps,
};

pub fn view() -> JsonUiView {
    JsonUiView::new()
        .title("Users")
        .layout("app")
        .component(ComponentNode {
            key: "users_table".to_string(),
            component: Component::Table(TableProps { ... }),
            action: None,
            visibility: None,
        })
}
</example>
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single user message | System + user + assistant prefill | Always (Anthropic best practice) | Better output quality, no fence stripping |
| Pay full price every call | Prompt caching | 2024 (GA) | 90% cost reduction for repeated static content |
| Opus for everything | Sonnet for structured tasks | 2025 (Sonnet 4.5 quality parity for code) | 5x cost reduction |
| Hope for correct format | Few-shot examples | Always (prompt engineering 101) | Dramatically consistent output |
| Default temperature | temperature: 0.2 for code gen | Best practice | More deterministic, fewer hallucinations |

**New tools/patterns to consider:**
- **Structured Outputs (2025-11):** Anthropic now supports JSON schema-enforced outputs. Not directly useful for code gen (we want Rust source, not JSON), but could be used if we structured the output as JSON with a `code` field.
- **Prompt caching 1h TTL:** Useful if the CLI is used less frequently than every 5 minutes. Costs more to write but cache persists 1 hour.
- **Laravel Boost's approach:** MCP-based context assembly with guidelines + skills + documentation API. Ferro already has MCP tools but doesn't use them from the CLI. Worth considering for future enhancement.

**Deprecated/outdated:**
- **Putting everything in user message:** Anthropic explicitly recommends system prompts for role and static context.
- **Not using prompt caching:** There's no reason not to when you have 1000+ tokens of repeated static content.
</sota_updates>

<open_questions>
## Open Questions

1. **Streaming vs blocking for CLI?**
   - What we know: reqwest-eventsource crate exists for SSE streaming. Anthropic streaming API sends tokens as SSE events.
   - What's unclear: Whether the added complexity (async runtime or threaded SSE parsing) is worth the UX improvement in a CLI tool.
   - Recommendation: Keep blocking for now but add timeout. Streaming is a nice-to-have for v2 of this command.

2. **Scan existing views for context?**
   - What we know: Laravel Boost scans the project extensively. Current implementation scans models and routes but not existing views.
   - What's unclear: How much improvement this provides vs added prompt length/cost.
   - Recommendation: Add as optional context (scan `src/views/*.rs` for 1-2 existing views) if the directory exists. Include only the first view found as a "project style reference."

3. **Output validation (compile check)?**
   - What we know: Some AI CLI tools (Codex, Claude Code) run compilation checks on generated code.
   - What's unclear: Whether `cargo check` is fast enough to run inline in a scaffolding command.
   - Recommendation: Don't block on compilation. Optionally warn if basic structure checks fail (no `pub fn view()` found, no `use ferro::` found).
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- [Anthropic Prompt Engineering Docs](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/overview) — system prompts, few-shot examples, prefill technique
- [Anthropic Prompt Caching Docs](https://platform.claude.com/docs/en/build-with-claude/prompt-caching) — cache_control parameter, pricing, minimum token requirements
- [Anthropic Prefill Docs](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/prefill-claudes-response) — assistant prefill technique for format control
- [Anthropic Structured Outputs](https://towardsdatascience.com/hands-on-with-anthropics-new-structured-output-capabilities/) — JSON schema enforcement (beta 2025-11)

### Secondary (MEDIUM confidence)
- [Laravel Boost Docs](https://laravel.com/docs/12.x/boost) — MCP-based AI context assembly patterns, guidelines + skills system
- [Laravel Boost v2.0](https://blog.devgenius.io/laravel-boost-2-0-is-here-skills-better-context-and-a-cleaner-architecture-ffdf9a944d65) — Skills system, on-demand context loading
- [reqwest-eventsource crate](https://docs.rs/reqwest-eventsource/) — SSE streaming for Rust

### Tertiary (LOW confidence - needs validation)
- Few-shot impact on code generation quality claims from blog posts — directionally correct but magnitudes vary
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Anthropic Messages API for code generation
- Ecosystem: reqwest, prompt caching, SSE streaming
- Patterns: System prompts, prefill, few-shot, caching, timeout
- Pitfalls: Model cost, no examples, no timeout, temperature default

**Confidence breakdown:**
- API best practices: HIGH — from official Anthropic docs
- Cost optimization: HIGH — from official pricing docs
- Architecture improvements: HIGH — from official prompt engineering guides
- Laravel Boost comparison: MEDIUM — from official Laravel docs
- Streaming UX: MEDIUM — from crate docs, not tested

**Research date:** 2026-02-09
**Valid until:** 2026-03-09 (30 days — Anthropic API is stable, patterns well-established)
</metadata>

---

*Phase: 30-cli-scaffolding*
*Research completed: 2026-02-09*
*Ready for planning: yes (retroactive improvements)*
