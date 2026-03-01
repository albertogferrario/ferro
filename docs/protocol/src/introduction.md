# Introduction

## Ferro Projections Protocol

**Version:** 0.1.0-draft

This document specifies the Ferro Projections Protocol, a transport-agnostic
protocol for deriving user interaction intents from structured service
definitions and rendering them into declarative UI component trees.

## Motivation

The 2026 agent protocol ecosystem has converged on a layered stack addressing
distinct concerns:

- **A2A** (Google / Linux Foundation) handles agent-to-agent coordination.
- **MCP** (Anthropic / Linux Foundation) handles agent-to-tool and context
  exchange.
- **AG-UI** (CopilotKit) handles agent-to-frontend event transport.
- **A2UI** and **Open-JSON-UI** (Google / OpenAI) handle declarative UI
  component specification.

These protocols cover coordination, tool access, event transport, and UI
description. None of them address a prior question: **given a service
definition, what meaningful user interactions exist?**

A2UI describes *what* UI to render. AG-UI describes *how* to transport it. MCP
provides *tools and context*. But no protocol formalizes the derivation of user
intents from a service's structural properties — its fields, state machines,
actions, relationships, and semantic annotations.

The Ferro Projections Protocol fills this gap. It defines a schema for
describing service capabilities, rules for deriving user intents from that
schema, and a trait for rendering those intents into UI component trees.

## Protocol Positioning

The protocol occupies the layer between service definition (MCP tools, OpenAPI
schemas) and UI specification (A2UI, Open-JSON-UI):

```
┌─────────────────────────────────────────────────────────────┐
│  A2A (Google/LF)          Agent ↔ Agent coordination        │
├─────────────────────────────────────────────────────────────┤
│  MCP (Anthropic/LF)       Agent ↔ Tools & Context           │
├─────────────────────────────────────────────────────────────┤
│  ══════════════════════════════════════════════════════════  │
│  Ferro Projections   ServiceDef → IntentGraph → Renderer    │
│  Protocol            "What UI does this service need?"      │
│  ══════════════════════════════════════════════════════════  │
├─────────────────────────────────────────────────────────────┤
│  AG-UI (CopilotKit)       Agent ↔ Frontend event transport  │
├─────────────────────────────────────────────────────────────┤
│  A2UI / Open-JSON-UI      Declarative UI component specs    │
│  (Google / OpenAI)                                          │
└─────────────────────────────────────────────────────────────┘
```

The protocol transforms a structured service description into ranked user
interaction intents. Those intents then drive a pluggable renderer that
produces output in any target format — JSON-UI, A2UI, HTML, or others.

## What This Protocol Defines

This specification defines:

1. **ServiceDef** — A schema for describing service capabilities, composed of
   fields with semantic annotations, actions with preconditions and inputs,
   named guards, inter-service relationships, state machines with transitions,
   and intent hints for manual override.

2. **Intent derivation rules** — A pipeline of analyzers that examine
   structural signals in a ServiceDef to produce a ranked list of IntentScores
   with confidence values. The analyzers consider field meanings, writability
   ratios, state machine complexity, relationship patterns, and action
   signatures.

3. **Renderer trait** — An interface for transforming a ServiceDef, its derived
   IntentScores, and a RenderContext into a framework-independent UI component
   tree. The renderer is pluggable: any output format MAY implement the trait.

4. **Validation rules** — Structural validation for ServiceDef instances,
   producing warnings for non-fatal concerns (unreachable states, unused
   guards) and errors for fatal inconsistencies (undefined references,
   duplicate names).

5. **Extension mechanism** — A two-tier approach allowing vendor-specific data
   via `x-*` prefixed keys and formal protocol extensions via URI-namespaced
   entries, following the JSON:API extension model.

## What This Protocol Does Not Define

The following concerns are explicitly out of scope:

- **Transport.** The protocol does not specify how ServiceDef instances are
  transmitted. Consumers MAY use MCP, HTTP, WebSocket, gRPC, or any other
  transport mechanism. The protocol is transport-agnostic by design.

- **Authentication and authorization.** The protocol does not define how
  service access is controlled. Implementations SHOULD rely on the
  authentication mechanisms provided by their chosen transport layer.

- **Session management.** The protocol does not define user sessions, tokens,
  or state persistence between requests. These concerns belong to the
  application layer.

- **Deployment or hosting.** The protocol does not specify how services are
  deployed, discovered, or scaled. Service discovery MAY be handled by MCP
  tool registration, DNS-based discovery, or other mechanisms external to this
  protocol.

- **Specific UI component libraries.** The Renderer trait produces
  framework-independent output. The protocol does not mandate any particular
  component vocabulary. JSON-UI, A2UI, HTML, and native component trees are
  all valid rendering targets.

## Audience

This specification is intended for:

- **Protocol implementors** building libraries that produce or consume
  ServiceDef instances.
- **Framework authors** integrating service projection capabilities into web
  frameworks or agent platforms.
- **Agent developers** using ServiceDef schemas to drive automated UI
  generation or service introspection.
- **Renderer implementors** creating new rendering backends that transform
  IntentScores into target-specific UI component trees.

## Notational Conventions

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this
document are to be interpreted as described in BCP 14
\[[RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119)\]
\[[RFC 8174](https://datatracker.ietf.org/doc/html/rfc8174)\]
when, and only when, they appear in all capitals, as shown here.

The specification uses these keywords to distinguish between normative
requirements (what implementations MUST or SHOULD do) and informative guidance
(what implementations MAY do). Lowercase uses of these words carry their
ordinary English meaning.
