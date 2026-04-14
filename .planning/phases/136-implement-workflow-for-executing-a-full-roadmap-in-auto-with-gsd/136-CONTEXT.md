# Phase 136: Implement Workflow for Executing a Full Roadmap in Auto with GSD - Context

**Gathered:** 2026-04-14
**Status:** Ready for planning

<domain>
## Phase Boundary

A GitHub Actions workflow that drives an entire milestone's worth of phases through the GSD pipeline (discuss → plan → execute) automatically, one fresh `claude` CLI invocation per phase. This is the **outer orchestrator** that `/gsd:autonomous` cannot be — it survives context window exhaustion by running outside Claude entirely on GitHub infrastructure.

The workflow takes a milestone name as input and runs all incomplete phases in that milestone sequentially, each in an isolated context window. On failure it stops and opens a GitHub issue.

</domain>

<decisions>
## Implementation Decisions

### Orchestrator Architecture
- **D-01:** GitHub Actions workflow (`.github/workflows/gsd-roadmap.yml`). Runs entirely outside Claude's context window on GitHub infrastructure.
- **D-02:** One fresh `claude` CLI invocation per phase. No batching, no context reuse across phases. Simple and predictable.
- **D-03:** The workflow accepts two `workflow_dispatch` inputs: (1) path to the roadmap file to run (e.g. `.planning/ROADMAP.md`), and (2) milestone name within that roadmap. It runs all incomplete phases in the specified milestone of the specified roadmap.

### State Tracking
- **D-04:** No new state file. Use existing `gsd-tools roadmap analyze` to determine which phases are incomplete. The script re-reads roadmap state before each phase to catch dynamically inserted phases.

### Failure Handling
- **D-05:** On phase failure: stop the roadmap run immediately and open a GitHub issue via `gh issue create`.
- **D-06:** Issue format is minimal: title = "Phase N failed: [phase name]", body = exit code + last few lines of output + link to phase directory.

### Workflow Location
- **D-07:** `.github/workflows/gsd-roadmap.yml` — committed to the repo. Triggered via `workflow_dispatch` from the GitHub Actions UI or `gh workflow run`.

### Claude's Discretion
- Exact `claude` CLI flags and invocation pattern (how to pass `--auto`, how to pipe the commands)
- How to detect success vs failure from Claude CLI exit codes or output parsing
- Whether to use `/gsd:autonomous --from N` or invoke individual `/gsd:discuss-phase`, `/gsd:plan-phase`, `/gsd:execute-phase` per phase
- Runner setup: how to install `claude` CLI, `node`, `gsd-tools` on the GitHub Actions runner
- Whether to use a self-hosted runner or GitHub-hosted runner

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
- `claude` CLI — Installed on the GitHub Actions runner, invoked per phase
- `gh` CLI — Natively available in GitHub Actions runners, used for issue creation on failure
- `GITHUB_TOKEN` — Auto-provided by GitHub Actions, used by `gh` for issue creation
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
