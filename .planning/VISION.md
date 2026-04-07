# Ferro Vision

The deeper thesis behind ferro. PROJECT.md tracks what is shipped and planned; this file states what ferro is reaching for and why. Aspirational claims are marked as such.

## What ferro is reaching for

Ferro's long-term reach is **media-independent computation**: an application is data plus intent, and a medium is a projection of that pair. The same `ServiceDef` can become a kanban board, a voice flow, a printed report, or a physical control surface — the application does not change, only its projection does.

This is the territory Bret Victor and Alan Kay pointed at. It is not a new framework category; it is a different relationship between program and presentation.

The bridge from "Rust web framework" to that reach is the **projection / intent system** (`ferro-projections`, shipped v9.0): seven structural intents (Browse, Focus, Collect, Process, Summarize, Analyze, Track), signal analyzers that rank them, and renderers that turn ranked intents into rendered views.

**Aspiration vs shipped:**
- Shipped: projection/intent for the visual medium (HTML/CSS via JSON-UI).
- Aspiration: audio, voice, and physical projections. These are deferred to v2.0+ and explicitly named as an unprobed weakness (see below).

At v1.0, ferro is visual-only. The reach is multimodal. Both statements are true.

## Audience

**Agent-assisted humans.** Developers who use AI agents (Cursor, Claude Code, and similar) as their primary build interface. The human stays in the loop for direction and judgment; the agent reads ferro's surface — primarily through MCP — and writes the code.

Ferro is shaped for an agent reading it through tools, not for a human typing it by hand. This is a design constraint, not an accident.

### Two-brand model

Non-developers are not the audience for ferro. They are reached later, by a separate **Builder Brand** (name TBD, project does not exist yet) — a human-facing app builder built ON ferro, with its own approachable aesthetics.

| | Ferro | Builder Brand (future) |
|---|---|---|
| What | Engine, infrastructure | Human-facing app builder |
| Audience | Agent-assisted developers | Non-developers |
| Aesthetic | Industrial, durable, minimal | Beautiful, mass-market |
| Status | Active (pre-1.0) | Does not exist yet |

Ferro is named industrially — iron — on purpose. Non-developers will never see ferro. It does not need to look approachable to them. The Builder Brand does not need to exist for ferro v1.0.

## The killer feature

**Projection / intent.** This is the bet.

A traditional web framework is shaped around a request-response loop and HTML. Ferro is shaped around a service definition that can be projected. The 7 intents are the structural categories of human-computer activity ferro currently models. Signal analyzers rank which intent best fits a given service. Renderers turn the chosen intent into output.

### Why this is the bet

Because every other axis of competition (routing, ORMs, templating, auth) has been solved by every framework in every language for two decades. The differentiator that matters is whether a developer-and-agent pair can express what an application *means* and have ferro decide *how* to render it.

If projection/intent works, ferro is a substrate for a different kind of programming. If it fails, ferro is yet another Rust web framework — and that market is already crowded.

### What "validation" looks like

Four checks (see PROJECT.md for the full v1.0 criterion):

1. Gestiscilo migration — a real product runs on it.
2. Synthetic benchmark suite — canonical app classes covered.
3. Time-to-app test — agent-assisted human reaches a working app within target time.
4. Agent-success-rate test — agents generate correct projections at acceptable rate.

**No stop-loss.** If validation reveals gaps, the response is iteration through more real cases. The bet is not abandoned for a Laravel-shaped fallback.

## Beauty as design criterion

Beauty is not decoration. It is a four-dimensional design criterion. All four must hold for v1.0.

1. **Compressive** — tiny input produces disproportionate output. The defining quality of a substrate worth using. Ferro's strongest direction; the actual bet.
2. **Operational** — it just works. Setup, errors, edges, defaults. Currently ferro's largest debt; actively being paid down.
3. **Conceptual** — small core in the user's mental model. The 20 crates must hold together as one idea, not twenty. Currently mid; coherence is now a continuous tax.
4. **Aesthetic** — visual polish of rendered output, docs, and surface. Currently weakest. Acceptable pre-1.0 because the audience is agent-assisted developers; not acceptable at 1.0.

### Substance-first investment ordering

When time is scarce — and it always is — investment goes in the order:

> compressive → operational → conceptual → aesthetic

This is the priority order for *where time goes when something gives*, not the priority order for *what matters at v1.0*. At v1.0, all four must hold.

## How ferro evolves

### Co-dependent forcing function: gestiscilo

Ferro's evolution is driven by gestiscilo's commercial roadmap. The 20-crate sprawl exists because gestiscilo needed each one. The two projects are co-dependent: gestiscilo gets a substrate, ferro gets a real workload that prevents toy-framework drift.

**Mitigation against overfitting to gestiscilo's shape:**
- Deliberate diversification — apps built in domains gestiscilo does not touch.
- Synthetic catalog of canonical app classes — covers shapes gestiscilo will never encounter.

### Continuous coherence tax

Conceptual coherence is enforced at **write-time, every phase**. Every new feature asks: *does this fit the existing surface, or does the surface evolve to absorb it?* No phase ships without an answer.

Coherence is not a periodic cleanup project. It is a per-phase tax. Phase 113 was the first systematic pass; the standing rule is that there will not need to be a Phase 113 equivalent again, because no phase will be allowed to incur the debt that justified it.

### No stop-loss on projection/intent

The killer feature is the bet. Bets are iterated through, not exited from. If real cases reveal that an intent is wrong, the intent changes. If the seven intents are insufficient, more arrive. If the analyzer ranking is bad, it is fixed. The response to evidence is iteration, not retreat.

## Named weaknesses

These are explicit, not hidden.

### Multimodal generation is unprobed

The 7 intents may be subtly web-shaped. Process = kanban assumes drag-drop and concurrent visibility — both are visual affordances. There is no evidence yet that the same intent maps cleanly onto an audio or voice projection.

The probe is cheap: sketch one intent across audio/voice on paper for one gestiscilo feature. This has not been done. It is deferred to v2.0+ but tracked, not denied.

If the probe reveals that the intents are visually-shaped, the response is to refactor the intent vocabulary. The reach toward media-independent computation is not negotiable; the current intent set is.

### Aesthetic is the weakest of the four beauty dimensions

Acceptable now because agent-assisted developers are the audience. Not acceptable at 1.0.

### Operational debt

Setup, error reporting, edge cases. Real, measured, actively being paid down phase by phase.

## What ferro is NOT

- **Not a Laravel-for-Rust competitor.** Laravel is a fine web framework. Ferro is reaching for a different category. Comparing them on routing or ORM ergonomics is comparing the wrong axis.
- **Not for hand-writing code.** The surface is shaped for an agent reading it through MCP. A human typing into a text editor will find ferro pleasant but not exceptional. An agent introspecting via `ferro-mcp` will find it unusually legible.
- **Not for non-developers directly.** Non-developers are reached via a separate Builder Brand built on ferro. Ferro itself does not need to be approachable to them.
- **Not announced or marketed.** Ferro is published on crates.io as `ferro-rs` and the repo is public. There are no release announcements, no marketing copy, no "what's new for users." Polish exists for first-impression credibility — an agent-assisted developer choosing a substrate forms an opinion fast — not for user comms.
- **Not a bundled agent UX.** The day-one v1.0 experience is: install `ferro-cli`, wire your existing agent (Cursor, Claude Code) to `ferro-mcp` via standard MCP config, let the agent introspect and generate. `ferro-mcp` IS the v1.0 product surface. The Rust crate API is the substrate underneath.
- **Not done.** v1.0 is years away. Pre-1.0 status is real, not modesty.
