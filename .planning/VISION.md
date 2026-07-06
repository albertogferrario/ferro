# Ferro Design Philosophy

This document describes ferro's architectural rationale. `PROJECT.md` tracks shipped scope and active milestones; this file states what ferro optimizes for and why.

## What ferro optimizes for

Ferro is a Rust web framework optimized for AI-assisted authoring. An application is modeled as data plus intent, and a rendered medium is a projection of that pair. The same `ServiceDef` can be projected as a kanban board, a form, a dashboard, or a report — the application definition does not change, only the projection does.

At v1.0 the supported projection target is visual (HTML/CSS via JSON-UI). The architecture is designed to support additional rendering modalities over time.

## Core abstraction: projection / intent

The primary feature is the **projection / intent system** (`ferro-projections`, shipped in v9.0):

- Seven structural intents: Browse, Focus, Collect, Process, Summarize, Analyze, Track.
- Signal analyzers that score and rank intents for a given service.
- Renderers that turn a ranked intent into a rendered view.

The framework is built around this abstraction. Routing, persistence, validation, and the rest of the surface exist to feed it. The seven intents are the structural categories the framework currently models; they are versioned and may evolve as evidence accumulates.

## Audience

Ferro is designed for AI-assisted authoring. Its surface is shaped for an agent reading the project through tools — primarily through `ferro-mcp`, the introspection layer — rather than for hand-typing into a text editor. The human stays in the loop for direction and judgment; the agent reads the framework's surface and writes the code.

This is well-suited for developers using AI coding agents (Cursor, Claude Code, and similar) as their primary build interface.

## Beauty as a design criterion

Beauty is treated as a four-dimensional design criterion, not as decoration. All four dimensions are required for v1.0.

1. **Compressive** — small inputs produce disproportionate outputs. The defining quality of a useful substrate.
2. **Operational** — setup, errors, edges, and defaults work without surprise.
3. **Conceptual** — the surface holds together as a small, coherent mental model.
4. **Aesthetic** — visual quality of rendered output, documentation, and surface.

### Substance-first ordering

When investment must be prioritized, the ordering is:

> compressive → operational → conceptual → aesthetic

This is the order in which time is spent when something has to give. At v1.0, all four dimensions must hold.

## Evolution

### Continuous conceptual coherence

Conceptual coherence is enforced at write-time. Every new feature asks: *does this fit the existing surface, or does the surface evolve to absorb it?* No phase ships without an answer. Coherence is a discipline applied as code is written, not a periodic cleanup.

### Validation through real-world applications

The projection / intent system is validated against real applications and a synthetic catalog of canonical app classes. Both are used to surface gaps and inform iteration on the intent vocabulary, signal analyzers, and renderers.

## Rendering architecture

The `Renderer` trait in ferro-projections defines the projection boundary: `ServiceDef + IntentScore[] → rendered output`. The trait uses associated types for output and context, so each rendering modality declares its own output format. Visual renderers produce JSON-UI specs. A conversational renderer would produce message payloads. A voice renderer would produce structured scripts. The trait itself carries no modality-specific assumptions.

Renderers live in their output crate, not in ferro-projections. ferro-json-ui provides `JsonUiRenderer`. A future ferro-whatsapp projection would provide `WhatsAppRenderer` behind a feature flag. ferro-projections owns only the trait definition, `derive_intents()`, and `ServiceDef` — the schema layer.

## Roadmap direction

The visual modality is the v1.0 target. Additional rendering modalities (conversational text, voice, structured API) are v2.0+ directions. The Renderer trait is designed to support this without architectural surgery — each new modality adds a renderer implementation in its own crate, reusing the same intent derivation pipeline. The intent vocabulary is expected to evolve as additional modalities are explored.
