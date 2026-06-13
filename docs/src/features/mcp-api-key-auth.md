# MCP Per-Tenant API-Key Auth

`ferro-mcp-oauth` supports two authentication paths on the single `/mcp` endpoint: OAuth JWT
(browser-based authorization-code flow and device grant) and a per-tenant API key. Both paths
resolve to the same tenant context and produce the same `BearerCheck::Authenticated` outcome.

## Authentication Paths

The MCP endpoint reads the `Authorization: Bearer <token>` header on every request. The token
shape determines which validation branch runs:

| Token shape | Branch |
|-------------|--------|
| Starts with `ferro_` | API-key validation via `validate_api_key` |
| Anything else | JWT validation via `validate_bearer` |
| Header absent | `BearerCheck::Unauthenticated` → 401 |

Both branches produce `BearerCheck::Authenticated(principal)` on success, where `principal`
is a JSON object with `sub`, `tenant_id`, and `scope` fields. The downstream tool dispatcher
receives the same context regardless of which path was taken.

## The `mcp_api_keys` Table

API keys are stored as SHA-256 hashes — the plaintext key is never persisted.

```sql
CREATE TABLE mcp_api_keys (
    id          BIGINT PRIMARY KEY AUTOINCREMENT,
    tenant_id   BIGINT NOT NULL,
    key_hash    TEXT NOT NULL,          -- SHA-256 hex of the raw key
    scope       TEXT NOT NULL DEFAULT 'read',
    revoked_at  TIMESTAMP WITH TIME ZONE NULL,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at  TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE UNIQUE INDEX idx_mcp_api_keys_key_hash ON mcp_api_keys (key_hash);
CREATE        INDEX idx_mcp_api_keys_tenant_id ON mcp_api_keys (tenant_id);
```

`ferro-mcp-oauth` ships the canonical schema as a migration struct. The consumer app runs it
once at bootstrap:

```rust
use ferro_mcp_oauth::CreateMcpApiKeysTable;

CreateMcpApiKeysTable::up(&db).await?;
```

## Generating and Storing a Key

`generate_mcp_api_key` returns the plaintext key once, alongside the hash to store. The
plaintext is never passed to any persistence layer.

```rust
use ferro_mcp_oauth::{generate_mcp_api_key, CreateMcpApiKeysTable};

// Generate
let (raw_key, key_hash) = generate_mcp_api_key();
// raw_key  — a "ferro_"-prefixed 49-character BASE62 token; present this to the operator
// key_hash — SHA-256 hex; insert this into mcp_api_keys

// Insert (using your preferred query layer)
db.execute(Statement::from_sql_and_values(
    DbBackend::Sqlite,
    "INSERT INTO mcp_api_keys (tenant_id, key_hash, scope, created_at, updated_at)
     VALUES (?, ?, ?, datetime('now'), datetime('now'))",
    [tenant_id.into(), key_hash.into(), "read_write".into()],
)).await?;
```

Generated keys are `ferro_` followed by 43 random BASE62 characters (49 characters total).
The prefix makes API keys unambiguously distinguishable from JWTs at the routing branch point.

## Key Rotation

Rotation uses a soft-revoke pattern: issue a new key and set `revoked_at` on the old one.
Do not delete rows — the audit trail is preserved.

```sql
-- Revoke the old key
UPDATE mcp_api_keys SET revoked_at = NOW() WHERE id = ?;
-- Insert new key (using generate_mcp_api_key as above)
```

`validate_api_key` treats any non-null `revoked_at` as `BearerCheck::Invalid`.

## Scope Model

Each key carries a `scope` field: `read` or `read_write`.

| Scope | `tools/list` | `tools/call` (read tool) | `tools/call` (write tool) |
|-------|-------------|--------------------------|---------------------------|
| `read` | read tools only | allowed | rejected (-32603) |
| `read_write` | all tools | allowed | allowed |

The scope check on `tools/call` is enforced server-side before tool dispatch, independently
of the listing filter. A client that bypasses `tools/list` and calls a write tool directly
with a `read`-scoped key receives a `-32603` error with message `"scope insufficient"`.

**Scope is orthogonal to tenant ability.** `scope` governs the credential's permission (what
this particular key may do). `ServiceDef.mcp_ability` governs the tenant's capability (what
the tenant's account permits). Both checks apply independently.

## Using an API Key

Present the raw key as a Bearer token:

```
Authorization: Bearer ferro_<token>
```

```bash
curl https://your-app.example.com/mcp \
  -H "Authorization: Bearer ferro_Ab3Cd..." \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}'
```

## Security Properties

- **Plaintext never stored.** Only the SHA-256 hex hash of the key is written to the database.
- **Fail-closed.** Any lookup error, unknown key, or revoked key returns
  `BearerCheck::Invalid` — the same rejection path as an invalid JWT.
- **Cross-tenant isolation.** The validator checks `expected_tenant` against the `tenant_id`
  column; a key issued to tenant A returns `BearerCheck::Forbidden` if presented on a
  tenant-B-scoped request.
- **Scope re-check at dispatch.** `tools/call` re-evaluates the key scope before dispatching
  any write tool, so a `tools/list` filter bypass does not grant elevated access.
