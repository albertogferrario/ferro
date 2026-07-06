# Dogfood: Consumer App MCP Browser Login

Scripted acceptance run for the Phase 200 dogfood gate (SC-4). Drives the full
OAuth + MCP sequence against a locally-running ferro sample app, verifying
per-tenant row isolation end to end.

---

## Prerequisites

All of the following must be true before running the script.

### 1. The sample app is built and its migrations are up to date

```bash
# From the workspace root
cargo build -p app
```

### 2. Environment variables are set

Create or update `app/.env` with:

```dotenv
# Required: base URL the OAuth discovery documents and token audience use.
# Must match the scheme + host the browser can reach.
# The token audience is {APP_URL}/mcp — this value must satisfy the Origin
# check in ferro-mcp-oauth.
APP_URL=http://127.0.0.1:8080

# Required: HS256 signing secret for MCP access tokens.
# Minimum 32 bytes. Used by ferro-mcp-oauth to mint and validate tokens.
# If unset the server refuses to start bearer validation.
MCP_TOKEN_SECRET=<at-least-32-random-bytes>

# Required: database path (SQLite default)
DATABASE_URL=sqlite://./database.db
```

Generate a suitable secret:

```bash
openssl rand -hex 32
```

### 3. The two-tenant seed has been applied

The seed runs automatically on first startup (when the `tenants` table is empty).
Start the app once and it will insert:

| Tenant | Slug   | User                    | Password     |
|--------|--------|-------------------------|--------------|
| Acme   | acme   | alice@acme.test         | password123  |
| Globex | globex | bob@globex.test         | password123  |

Each tenant has 2 seeded orders.

To reseed from scratch (drops and re-runs all migrations):

```bash
./target/debug/app db:fresh
```

---

## Running the dogfood script

### Step 1: Start the app (you run the server)

```bash
# From the workspace root
APP_URL=http://127.0.0.1:8080 \
MCP_TOKEN_SECRET=<your-32-byte-secret> \
cargo run -p app
```

Wait until the server prints that it is listening.

### Step 2: Run the script (in a second terminal)

```bash
node dogfood/run_dogfood.mjs
```

The script will:
1. Fetch OAuth discovery metadata.
2. Register a dynamic client (DCR).
3. Generate a PKCE verifier + S256 challenge.
4. **Print an `/authorize` URL and pause** — this is the human browser-login step.

### Step 3: Complete the browser login (human step)

1. Open the printed `/authorize` URL in your browser.
2. Log in as `alice@acme.test` (tenant "acme") or `bob@globex.test` (tenant "globex").
3. Approve the consent screen.
4. You will be redirected to `http://localhost:9999/callback?code=<CODE>&state=<STATE>`.
   The browser will show "connection refused" (no listener) — that is expected.
5. Copy the full redirect URL from the browser address bar.
6. Paste it back into the script prompt.

### Step 4: Script continues automatically

The script exchanges the code for a token, calls `tools/list`, calls
`tools/call` for `list_order`, and verifies that every returned row's
`tenant_id` matches the authenticated tenant.

**Expected output (GO path):**

```
Step 6: Calling tools/list...
  found order tool: "list_order"
Step 7: Calling tools/call for list_order...
  content items returned: 2
Step 8: Verifying tenant isolation...
  PASS: 2 row(s) checked — all have tenant_id = 1
=== Dogfood run complete ===
```

**NO-GO:** The script exits non-zero if any row's `tenant_id` does not match
the authenticated tenant, or if any step fails.

---

## Claude Desktop path (human-facing confirmation)

After the script passes, verify a real MCP client also works:

Add this entry to your Claude Desktop MCP configuration
(`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "ferro-sample-app": {
      "type": "http",
      "url": "http://127.0.0.1:8080/mcp",
      "authorization": {
        "type": "oauth",
        "authorizationUrl": "http://127.0.0.1:8080/authorize",
        "tokenUrl": "http://127.0.0.1:8080/token",
        "registrationUrl": "http://127.0.0.1:8080/register",
        "scopes": []
      }
    }
  }
}
```

Restart Claude Desktop after saving. Claude will prompt for browser login on
first use. Once authenticated, `list_order` will appear in the tool list.

---

## Acceptance verdict

Record the outcome in
`.planning/phases/200-per-tenant-scoping-policy-authorization-dogfood-acceptance/200-ACCEPTANCE.md`.

| Verdict | Condition |
|---------|-----------|
| **GO**  | Script exits 0; PASS line printed; rows returned; all `tenant_id` values match the authenticated tenant; no cross-tenant data visible. |
| **NO-GO** | Any step fails, script exits non-zero, or any `tenant_id` mismatch is detected. |

A NO-GO blocks phase completion and requires a design revision per the milestone invariant.
Attribution matters: note whether the failure is a design defect or a setup issue
(missing seed, wrong `APP_URL`, `MCP_TOKEN_SECRET` not set, etc.).
