# Phase 136: Implement Workflow for Executing a Full Roadmap in Auto with GSD - Context

**Gathered:** 2026-04-14
**Status:** Ready for planning

<domain>
## Phase Boundary

A shell script that drives an entire milestone's worth of phases through the GSD pipeline (discuss → plan → execute) automatically, one fresh `claude` CLI invocation per phase. This is the **outer orchestrator** that `/gsd:autonomous` cannot be — it survives context window exhaustion by running outside Claude entirely.

The script takes a milestone name as parameter and runs all incomplete phases in that milestone sequentially, each in an isolated context window.

</domain>

<decisions>
## Implementation Decisions

### Orchestrator Architecture
- **D-01:** Shell script (bash), not a node command or Claude skill. Runs entirely outside Claude's context window.
- **D-02:** One fresh `claude` CLI invocation per phase. No batching, no context reuse across phases. Simple and predictable.
- **D-03:** The script accepts a milestone name as a required parameter. It runs all incomplete phases in that milestone.

### State Tracking
- **D-04:** No new state file. Use existing `gsd-tools roadmap analyze` to determine which phases are incomplete. The script re-reads roadmap state before each phase to catch dynamically inserted phases.

### Failure Handling
- **D-05:** On phase failure: stop the roadmap run immediately and open a GitHub issue via `gh issue create`.
- **D-06:** Issue format is minimal: title = "Phase N failed: [phase name]", body = exit code + last few lines of output + link to phase directory.

### Script Location
- **D-07:** Standalone shell script at `~/.claude/get-shit-done/bin/gsd-roadmap-run.sh`. No skill wrapper, no gsd-tools subcommand. Minimal packaging.

### Claude's Discretion
- Exact `claude` CLI flags and invocation pattern (how to pass `--auto`, how to pipe the `/gsd:autonomous` or `/gsd:discuss-phase --auto` command)
- How to detect success vs failure from Claude CLI exit codes or output parsing
- Whether to pass `--from N` to `/gsd:autonomous` or invoke individual `/gsd:discuss-phase`, `/gsd:plan-phase`, `/gsd:execute-phase` per phase

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### GSD Workflows
- `~/.claude/get-shit-done/workflows/autonomous.md` — Existing per-session autonomous workflow (discuss→plan→execute loop). The new script orchestrates around this.
- `~/.claude/get-shit-done/workflows/discuss-phase.md` — Discuss workflow with `--auto` flag support
- `~/.claude/get-shit-done/workflows/plan-phase.md` — Plan workflow
- `~/.claude/get-shit-done/workflows/execute-phase.md` — Execute workflow

### GSD Tools
- `~/.claude/get-shit-done/bin/gsd-tools.cjs` — CLI tool with `roadmap analyze`, `init phase-op`, `init milestone-op` commands used for state checking

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `gsd-tools.cjs roadmap analyze` — Returns JSON with phase list, statuses, completion state. Already used by `/gsd:autonomous`.
- `gsd-tools.cjs init milestone-op` — Returns milestone metadata (version, name, phase count, completed count).
- `gsd-tools.cjs init phase-op N` — Returns per-phase state (has_context, has_plans, has_verification).
- `/gsd:autonomous` workflow — Contains the per-phase logic (smart discuss, plan, execute, verification routing). The shell script can delegate to this or to individual phase commands.

### Established Patterns
- `--auto` flag on discuss-phase auto-selects recommended defaults for all gray areas
- Auto-advance chain: discuss → plan → execute within one session via `workflow._auto_chain_active` config flag
- `--no-transition` flag on execute-phase prevents inter-phase transition prompts

### Integration Points
- `claude` CLI — The shell script spawns this as a subprocess per phase
- `gh` CLI — Used to create GitHub issues on failure
- `.planning/ROADMAP.md` — Source of truth for phase list and completion status

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 136-implement-workflow-for-executing-a-full-roadmap-in-auto-with-gsd*
*Context gathered: 2026-04-14*
