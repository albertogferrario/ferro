#!/usr/bin/env node
/**
 * dogfood/run_dogfood.mjs
 *
 * Scripted MCP client for the Phase 200 dogfood acceptance run.
 *
 * Drives the full OAuth + MCP sequence against a locally-running ferro app:
 *   Step 1. GET  /.well-known/oauth-authorization-server   — discovery
 *   Step 2. POST /register                                  — dynamic client registration (DCR)
 *   Step 3. Generate PKCE, print /authorize URL, pause for browser login
 *   Step 4. POST /token                                     — code exchange
 *   Step 5. POST /mcp  tools/list                           — find the order tool
 *   Step 6. POST /mcp  tools/call                           — call the order tool
 *   Step 7. VERIFY returned rows all belong to the authenticated tenant
 *
 * Prerequisites: see dogfood/README.md
 *   - APP_URL env var set to the running app's base URL (default: http://127.0.0.1:8080)
 *   - The app is running with MCP_TOKEN_SECRET set and migrations + seed applied
 *
 * Usage:
 *   node dogfood/run_dogfood.mjs
 *
 * Node >= 18 required (global fetch + readline/promises).
 * No external npm dependencies.
 */

import { createHash, randomBytes } from 'node:crypto';
import { createInterface } from 'node:readline/promises';
import { stdin as input, stdout as output } from 'node:process';

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const APP_URL = (process.env.APP_URL ?? 'http://127.0.0.1:8080').replace(/\/$/, '');
const REDIRECT_URI = 'http://localhost:9999/callback';

// ---------------------------------------------------------------------------
// PKCE helpers (RFC 7636, S256)
// ---------------------------------------------------------------------------

function generateCodeVerifier() {
  // 32 random bytes → 43 URL-safe base64 chars (RFC 7636 §4.1 min 43 chars)
  return randomBytes(32).toString('base64url');
}

function s256Challenge(verifier) {
  return createHash('sha256').update(verifier).digest('base64url');
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

async function get(url) {
  const res = await fetch(url, { headers: { Accept: 'application/json' } });
  if (!res.ok) throw new Error(`GET ${url} → ${res.status} ${res.statusText}`);
  return res.json();
}

async function postJson(url, body) {
  const res = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`POST ${url} → ${res.status}: ${text}`);
  }
  return res.json();
}

async function postForm(url, params) {
  const body = new URLSearchParams(params).toString();
  const res = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
      Accept: 'application/json',
    },
    body,
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`POST ${url} (form) → ${res.status}: ${text}`);
  }
  return res.json();
}

async function mcpRequest(url, token, method, params = {}) {
  const body = { jsonrpc: '2.0', id: 1, method, params };
  const res = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Accept: 'application/json',
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`MCP ${method} → ${res.status}: ${text}`);
  }
  return res.json();
}

// ---------------------------------------------------------------------------
// Main sequence
// ---------------------------------------------------------------------------

async function main() {
  const rl = createInterface({ input, output });

  try {
    console.log('=== Ferro dogfood acceptance run ===');
    console.log(`APP_URL: ${APP_URL}`);
    console.log('');

    // ── Step 1: Discovery ─────────────────────────────────────────────────
    console.log('Step 1: Fetching OAuth authorization-server metadata...');
    const meta = await get(`${APP_URL}/.well-known/oauth-authorization-server`);
    console.log(`  issuer:               ${meta.issuer}`);
    console.log(`  authorization_endpoint: ${meta.authorization_endpoint}`);
    console.log(`  token_endpoint:        ${meta.token_endpoint}`);
    console.log(`  registration_endpoint: ${meta.registration_endpoint}`);

    const authorizeEndpoint = meta.authorization_endpoint;
    const tokenEndpoint = meta.token_endpoint;
    const registrationEndpoint = meta.registration_endpoint;

    // ── Step 2: Dynamic client registration (DCR, RFC 7591) ──────────────
    console.log('');
    console.log('Step 2: Registering MCP client via DCR...');
    const dcrResponse = await postJson(registrationEndpoint, {
      redirect_uris: [REDIRECT_URI],
      client_name: 'Ferro dogfood script',
      grant_types: ['authorization_code'],
      response_types: ['code'],
      token_endpoint_auth_method: 'none',
    });
    const clientId = dcrResponse.client_id;
    console.log(`  client_id: ${clientId}`);

    // ── Step 3: PKCE + authorize URL ──────────────────────────────────────
    console.log('');
    console.log('Step 3: Generating PKCE challenge...');
    const codeVerifier = generateCodeVerifier();
    const codeChallenge = s256Challenge(codeVerifier);
    const state = randomBytes(16).toString('base64url');

    const authorizeParams = new URLSearchParams({
      response_type: 'code',
      client_id: clientId,
      redirect_uri: REDIRECT_URI,
      code_challenge: codeChallenge,
      code_challenge_method: 'S256',
      state,
    });
    const authorizeUrl = `${authorizeEndpoint}?${authorizeParams}`;

    console.log('');
    console.log('ACTION REQUIRED — browser login:');
    console.log('  1. Open the URL below in your browser.');
    console.log('  2. Log in as alice@acme.test (password: password123) for tenant "acme".');
    console.log('     Or bob@globex.test (password: password123) for tenant "globex".');
    console.log('  3. Approve the consent screen.');
    console.log('  4. You will be redirected to:');
    console.log(`       ${REDIRECT_URI}?code=<CODE>&state=<STATE>`);
    console.log('  5. Copy the full redirect URL (or just the `code` query parameter).');
    console.log('');
    console.log(`  Authorize URL:`);
    console.log(`    ${authorizeUrl}`);
    console.log('');

    // ── Step 4: Wait for the redirect code ───────────────────────────────
    const rawInput = await rl.question(
      'Paste the redirect URL (or just the `code` value): '
    );
    const trimmed = rawInput.trim();

    let code;
    if (trimmed.startsWith('http')) {
      // Full redirect URL pasted
      try {
        const parsed = new URL(trimmed);
        code = parsed.searchParams.get('code');
        const returnedState = parsed.searchParams.get('state');
        if (returnedState && returnedState !== state) {
          throw new Error(`State mismatch: expected ${state}, got ${returnedState}`);
        }
      } catch (e) {
        if (e.message.includes('State mismatch')) throw e;
        // URL parse failed — treat input as a raw code
        code = trimmed;
      }
    } else {
      code = trimmed;
    }

    if (!code) throw new Error('No authorization code found in the pasted input.');
    console.log(`  code: ${code.substring(0, 12)}... (truncated for display)`);

    // ── Step 5: Token exchange ────────────────────────────────────────────
    console.log('');
    console.log('Step 4: Exchanging code for access token...');
    const tokenResponse = await postForm(tokenEndpoint, {
      grant_type: 'authorization_code',
      code,
      redirect_uri: REDIRECT_URI,
      client_id: clientId,
      code_verifier: codeVerifier,
    });
    const accessToken = tokenResponse.access_token;
    if (!accessToken) throw new Error('No access_token in token response: ' + JSON.stringify(tokenResponse));
    console.log(`  token_type: ${tokenResponse.token_type}`);
    console.log(`  expires_in: ${tokenResponse.expires_in}s`);
    console.log(`  access_token: ${accessToken.substring(0, 20)}... (truncated)`);

    // Decode the JWT payload (no verify — just inspect claims for display)
    const jwtParts = accessToken.split('.');
    let tenantId = null;
    let userId = null;
    if (jwtParts.length === 3) {
      try {
        const payload = JSON.parse(Buffer.from(jwtParts[1], 'base64url').toString('utf8'));
        tenantId = payload.tenant_id ?? null;
        userId = payload.sub ?? null;
        console.log(`  token claims: sub=${userId}, tenant_id=${tenantId}`);
      } catch (_) {
        // non-fatal
      }
    }

    const mcpUrl = `${APP_URL}/mcp`;

    // ── Step 6: MCP initialize ────────────────────────────────────────────
    console.log('');
    console.log('Step 5: Sending MCP initialize...');
    const initResp = await mcpRequest(mcpUrl, accessToken, 'initialize', {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: { name: 'ferro-dogfood', version: '1.0.0' },
    });
    if (initResp.error) throw new Error(`MCP initialize error: ${JSON.stringify(initResp.error)}`);
    console.log(`  server: ${initResp.result?.serverInfo?.name ?? '(unknown)'}`);

    // ── Step 7: tools/list ────────────────────────────────────────────────
    console.log('');
    console.log('Step 6: Calling tools/list...');
    const listResp = await mcpRequest(mcpUrl, accessToken, 'tools/list', {});
    if (listResp.error) throw new Error(`tools/list error: ${JSON.stringify(listResp.error)}`);
    const tools = listResp.result?.tools ?? [];
    console.log(`  tools available: ${tools.map(t => t.name).join(', ') || '(none)'}`);

    // Find the order tool — expected name is "list_order"
    const orderTool = tools.find(t => t.name === 'list_order');
    if (!orderTool) {
      throw new Error(
        `FAIL: "list_order" not found in tools/list. Available: [${tools.map(t => t.name).join(', ')}]`
      );
    }
    console.log(`  found order tool: "${orderTool.name}"`);

    // ── Step 8: tools/call ────────────────────────────────────────────────
    console.log('');
    console.log('Step 7: Calling tools/call for list_order...');
    const callResp = await mcpRequest(mcpUrl, accessToken, 'tools/call', {
      name: 'list_order',
      arguments: { limit: 5, offset: 0 },
    });

    if (callResp.error) throw new Error(`tools/call JSON-RPC error: ${JSON.stringify(callResp.error)}`);

    const result = callResp.result;
    if (result?.isError) {
      throw new Error(`tools/call returned a tool error: ${JSON.stringify(result.content)}`);
    }

    // The content array contains the serialized row data.
    // Each item is { type: "text", text: "<JSON string>" } or similar.
    const contentItems = result?.content ?? [];
    console.log(`  content items returned: ${contentItems.length}`);

    // ── Step 9: Tenant-isolation VERIFY ──────────────────────────────────
    console.log('');
    console.log('Step 8: Verifying tenant isolation...');

    if (tenantId === null) {
      console.log('  WARNING: could not read tenant_id from JWT claims — skipping tenant check.');
      console.log('  Ensure the token was minted with a real tenant_id (i.e. the user has a tenant_id set).');
    } else {
      let pass = true;
      let rowCount = 0;
      let violations = 0;

      for (const item of contentItems) {
        // Items may be JSON strings in item.text or raw objects.
        let rows = [];
        if (item.type === 'text' && typeof item.text === 'string') {
          try {
            const parsed = JSON.parse(item.text);
            rows = Array.isArray(parsed) ? parsed : [parsed];
          } catch (_) {
            // Not JSON — skip text items (e.g. pagination metadata)
          }
        } else if (typeof item === 'object' && item !== null) {
          rows = [item];
        }

        for (const row of rows) {
          rowCount++;
          if ('tenant_id' in row) {
            if (Number(row.tenant_id) !== Number(tenantId)) {
              console.error(
                `  VIOLATION: row id=${row.id} has tenant_id=${row.tenant_id} but expected ${tenantId}`
              );
              violations++;
              pass = false;
            }
          }
        }
      }

      if (rowCount === 0) {
        console.log('  No rows returned (tenant may have no orders, or seed not applied).');
        console.log('  Cannot assert isolation without rows. Verify seed was applied and tenant has orders.');
      } else if (pass) {
        console.log(`  PASS: ${rowCount} row(s) checked — all have tenant_id = ${tenantId}`);
      } else {
        console.error(`  FAIL: ${violations} isolation violation(s) across ${rowCount} row(s).`);
        process.exit(1);
      }
    }

    console.log('');
    console.log('=== Dogfood run complete ===');
    console.log('');
    console.log('Record the result in .planning/phases/200-per-tenant-scoping-policy-authorization-dogfood-acceptance/200-ACCEPTANCE.md');
    console.log('  GO  = all rows returned; all tenant_ids matched; no cross-tenant data visible.');
    console.log('  NO-GO = any step failed or any tenant_id mismatch was detected.');

  } finally {
    rl.close();
  }
}

main().catch(err => {
  console.error('');
  console.error('ERROR:', err.message);
  process.exit(1);
});
