# Security Considerations

This section documents security properties of the Ferro Projections Protocol and guidance for implementations processing `ServiceDef` data.

## 1. Input Validation

Implementations accepting `ServiceDef` from untrusted sources MUST validate the input against the published [JSON Schema](appendix/json-schema.md) before processing. Malformed input MUST be rejected with an appropriate error.

`ServiceDef::validate()` provides structural validation beyond schema conformance (e.g., undefined guard references, unreachable states). Implementations SHOULD run structural validation after schema validation when processing untrusted input.

## 2. String Injection

Several protocol types contain user-provided string values:

- `FieldMeaning::Custom(String)` -- arbitrary semantic annotations
- `GuardDef.name`, `GuardDef.description` -- guard identifiers and descriptions
- `ActionDef.name`, `ActionDef.display_name` -- action labels
- `StateDef.name`, `StateDef.display_name` -- state labels
- `ServiceDef.name`, `ServiceDef.display_name` -- service identifiers

Implementations MUST NOT interpret these string values as executable code, SQL statements, HTML markup, or shell commands. All string values in a `ServiceDef` are opaque labels with no execution semantics.

Renderers generating HTML or other markup from these strings MUST apply appropriate output encoding to prevent cross-site scripting (XSS) attacks.

## 3. Resource Consumption

A `ServiceDef` with very large arrays (fields, actions, states, transitions, relationships) could be used for resource exhaustion. Implementations SHOULD impose reasonable limits on:

- Number of fields per `ServiceDef`
- Number of states and transitions per `StateMachine`
- Number of actions and their inputs
- Number of relationships
- Total `ServiceDef` size in bytes

The protocol does not prescribe specific limits. Implementations SHOULD document their limits and reject input exceeding them.

## 4. Sensitive Data

`FieldMeaning::Sensitive` indicates a field contains sensitive data such as passwords, tokens, or API keys.

- Renderers MUST NOT display `Sensitive` fields in plaintext in `RenderMode::Display`.
- Implementations SHOULD exclude `Sensitive` fields from log output and debug representations.
- Intent derivation treats `Sensitive` fields as system fields, excluding them from structural analysis.

The `Sensitive` annotation is advisory. It does not replace transport-layer encryption or access control.

## 5. Extension Security

The [extension mechanism](extensions.md) defines critical and non-critical extensions:

- Extensions with `critical: true` that the consumer does not understand MUST cause rejection of the entire `ServiceDef`. This prevents silent bypass of security-relevant extensions (e.g., a GDPR compliance extension).
- Implementations MUST NOT process a `ServiceDef` containing unrecognized critical extensions, even partially.

Extension `data` payloads are arbitrary JSON. Implementations MUST apply the same input validation and string injection protections to extension data as to core protocol fields.

## 6. Transport Security

This protocol does not define a transport mechanism. It describes a data model and transformation rules.

Transport security -- including TLS, authentication, authorization, and session management -- is the responsibility of the transport layer. When `ServiceDef` data is transmitted over a network, implementations SHOULD use encrypted transport (TLS 1.2 or later).

When the protocol is used via MCP, the MCP specification's security model applies. When used via HTTP, standard HTTP security practices (HTTPS, CORS, authentication headers) apply.

## 7. Schema-Only Constraint

The protocol's schema-only design is itself a security property. A `ServiceDef` contains no closures, no executable code, and no runtime logic. It is pure data.

- A `ServiceDef` cannot execute arbitrary code on the consumer.
- A `ServiceDef` cannot access the filesystem, network, or environment variables.
- Guards and actions are string references, not executable functions. They describe what conditions and operations exist, not how to execute them.

This constraint means that receiving and deserializing a `ServiceDef` from an untrusted source carries no code execution risk. The only attack surface is resource consumption (Section 3) and downstream interpretation of string values (Section 2).
