# Extensions

The Ferro Projections Protocol supports two extension mechanisms, modeled after JSON:API's extension/profile system. Both mechanisms preserve forward compatibility: implementations encountering unrecognized extensions continue operating correctly.

## Notational Conventions

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119) / [BCP 14](https://datatracker.ietf.org/doc/html/bcp14).

## Vendor Extensions (x-\* Prefix)

Any field prefixed with `x-` MAY appear on `ServiceDef`, `FieldDef`, `ActionDef`, `RelationshipDef`, or `StateDef`. These are lightweight, informal annotations for tooling-specific metadata.

**Rules:**

- Implementations MUST ignore unrecognized `x-*` fields during deserialization.
- Implementations MUST NOT require consumers to understand `x-*` fields for correct behavior.
- Vendor extensions carry no protocol-level semantics. They are opaque metadata.

**Example:**

```json
{
  "name": "order",
  "display_name": "Order",
  "x-acme-priority": "high",
  "x-acme-team": "fulfillment",
  "fields": [
    {
      "name": "total",
      "data_type": "float",
      "meaning": "money",
      "x-acme-currency": "USD"
    }
  ]
}
```

Vendor extensions are suitable for internal tooling, build-system metadata, or annotations consumed by a specific organization's pipeline. They require no coordination with other implementations.

## Protocol Extensions (URI-Namespaced)

For formal, interoperable extensions, `ServiceDef` supports an optional `extensions` array. Each extension is a structured object with explicit criticality semantics.

**Extension object fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `uri` | string | Yes | Unique identifier for the extension (URI format) |
| `critical` | boolean | Yes | Whether consumers MUST understand this extension |
| `data` | object | No | Arbitrary JSON payload for the extension |

**Criticality rules:**

- If `critical` is `true` and a consumer does not recognize the extension URI, the consumer MUST reject the `ServiceDef`. This prevents silent bypass of extensions that affect correctness or security.
- If `critical` is `false`, the consumer MAY ignore the extension entirely. Processing continues normally.

**Example:**

```json
{
  "name": "order",
  "fields": [...],
  "extensions": [
    {
      "uri": "https://acme.com/ext/audit-trail",
      "critical": false,
      "data": {
        "track_field_changes": true,
        "retention_days": 90
      }
    },
    {
      "uri": "https://acme.com/ext/compliance/gdpr",
      "critical": true,
      "data": {
        "data_residency": "eu-west-1",
        "deletion_policy": "on_request"
      }
    }
  ]
}
```

In this example, a consumer that does not understand the GDPR compliance extension MUST reject the ServiceDef because `critical: true`. The audit trail extension can be safely ignored.

## Extension Registration

There is no central extension registry. Extension URIs are globally unique by convention: the URI domain SHOULD be controlled by the extension author.

**Recommended URI format:**

```
https://{domain}/ext/{extension-name}
```

Extension authors SHOULD publish documentation at the extension URI describing the expected `data` schema and behavioral requirements.

## Future-Proofing

New fields added to protocol types in future protocol versions MUST be optional. Consumers MUST ignore unrecognized fields beyond `x-*` handling. This ensures that a consumer implementing version `0.1.0` can process a ServiceDef produced by a version `0.2.0` producer, provided no critical extensions are present that the consumer does not understand.

This forward-compatibility rule applies to all types defined in the [Data Model](data-model/README.md): `ServiceDef`, `FieldDef`, `ActionDef`, `GuardDef`, `InputDef`, `RelationshipDef`, `StateMachine`, `StateDef`, `Transition`, `Intent`, `IntentScore`, and `IntentHint`.
