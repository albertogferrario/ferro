#!/usr/bin/env node
/**
 * dogfood/run_dogfood_auto.mjs
 *
 * Fully-autonomous variant of run_dogfood.mjs — replaces the human browser-login
 * pause with a programmatic cookie-jar login + consent, matching the method used
 * for the Phase 200 acceptance run. Drives the entire OAuth + MCP sequence and
 * verifies per-tenant row isolation, with no GUI step.
 *
 *   discovery → DCR → PKCE → POST /auth/login → GET /authorize (scrape CSRF)
 *   → POST /authorize (approve) → POST /token → MCP initialize/tools/list/tools/call
 *   → verify every returned row's tenant_id == authenticated tenant
 *
 * Usage:
 *   APP_URL=http://127.0.0.1:8090 node dogfood/run_dogfood_auto.mjs <email> <password> <expectedTenantId>
 *
 * Exit 0 = GO for this direction; non-zero = NO-GO.
 */

import { createHash, randomBytes } from 'node:crypto';

const APP_URL = (process.env.APP_URL ?? 'http://127.0.0.1:8090').replace(/\/$/, '');
const REDIRECT_URI = 'http://localhost:9999/callback';

const [email, password, expectedTenantArg] = process.argv.slice(2);
if (!email || !password) {
  console.error('Usage: node run_dogfood_auto.mjs <email> <password> <expectedTenantId>');
  process.exit(2);
}
const expectedTenant = expectedTenantArg != null ? Number(expectedTenantArg) : null;

// --- cookie jar -----------------------------------------------------------
const jar = new Map();
function storeCookies(res) {
  const set = res.headers.getSetCookie?.() ?? [];
  for (const c of set) {
    const [pair] = c.split(';');
    const idx = pair.indexOf('=');
    if (idx > 0) jar.set(pair.slice(0, idx).trim(), pair.slice(idx + 1).trim());
  }
}
function cookieHeader() {
  return [...jar.entries()].map(([k, v]) => `${k}=${v}`).join('; ');
}

// --- PKCE -----------------------------------------------------------------
const codeVerifier = randomBytes(32).toString('base64url');
const codeChallenge = createHash('sha256').update(codeVerifier).digest('base64url');
const state = randomBytes(16).toString('base64url');

const log = (...a) => console.log(...a);

async function main() {
  log(`=== autonomous dogfood run (${email}) ===`);
  log(`APP_URL: ${APP_URL}`);

  // 1. discovery
  const meta = await (await fetch(`${APP_URL}/.well-known/oauth-authorization-server`, {
    headers: { Accept: 'application/json' },
  })).json();
  log(`1. discovery OK — issuer=${meta.issuer}`);

  // 2. DCR
  const dcr = await (await fetch(meta.registration_endpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
    body: JSON.stringify({
      redirect_uris: [REDIRECT_URI],
      client_name: 'Ferro autonomous dogfood',
      grant_types: ['authorization_code'],
      response_types: ['code'],
      token_endpoint_auth_method: 'none',
    }),
  })).json();
  const clientId = dcr.client_id;
  log(`2. DCR OK — client_id=${clientId}`);

  // 3. login (cookie jar)
  const loginRes = await fetch(`${APP_URL}/auth/login`, {
    method: 'POST',
    redirect: 'manual',
    headers: { 'Content-Type': 'application/json', Accept: 'application/json', Cookie: cookieHeader() },
    body: JSON.stringify({ email, password }),
  });
  storeCookies(loginRes);
  if (loginRes.status >= 400) {
    throw new Error(`login failed → ${loginRes.status}: ${await loginRes.text()}`);
  }
  log(`3. login OK — status=${loginRes.status}, session cookie ${jar.size ? 'set' : 'MISSING'}`);

  // 4. GET /authorize → consent HTML, scrape CSRF _token
  const authParams = new URLSearchParams({
    response_type: 'code',
    client_id: clientId,
    redirect_uri: REDIRECT_URI,
    code_challenge: codeChallenge,
    code_challenge_method: 'S256',
    state,
  });
  const consentRes = await fetch(`${APP_URL}/authorize?${authParams}`, {
    headers: { Accept: 'text/html', Cookie: cookieHeader() },
    redirect: 'manual',
  });
  storeCookies(consentRes);
  if (consentRes.status === 302) {
    throw new Error(`GET /authorize redirected to ${consentRes.headers.get('location')} — not authenticated (session cookie not honored)`);
  }
  const html = await consentRes.text();
  const csrf = (html.match(/name="_token"\s+value="([^"]*)"/) || [])[1];
  if (!csrf) throw new Error(`no _token CSRF found in consent page (status ${consentRes.status})`);
  log(`4. consent page OK — CSRF token scraped`);

  // 5. POST /authorize approve → capture redirect code
  const approveRes = await fetch(`${APP_URL}/authorize`, {
    method: 'POST',
    redirect: 'manual',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded', Cookie: cookieHeader() },
    body: new URLSearchParams({
      _token: csrf,
      client_id: clientId,
      redirect_uri: REDIRECT_URI,
      code_challenge: codeChallenge,
      code_challenge_method: 'S256',
      state,
      response_type: 'code',
      action: 'approve',
    }).toString(),
  });
  storeCookies(approveRes);
  const loc = approveRes.headers.get('location');
  if (!loc) throw new Error(`approve did not redirect (status ${approveRes.status}): ${await approveRes.text()}`);
  const code = new URL(loc, APP_URL).searchParams.get('code');
  if (!code) throw new Error(`no code in approve redirect: ${loc}`);
  log(`5. consent approved — auth code issued`);

  // 6. token exchange
  const tokenJson = await (await fetch(meta.token_endpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded', Accept: 'application/json' },
    body: new URLSearchParams({
      grant_type: 'authorization_code',
      code,
      redirect_uri: REDIRECT_URI,
      client_id: clientId,
      code_verifier: codeVerifier,
    }).toString(),
  })).json();
  const token = tokenJson.access_token;
  if (!token) throw new Error(`no access_token: ${JSON.stringify(tokenJson)}`);
  const claims = JSON.parse(Buffer.from(token.split('.')[1], 'base64url').toString('utf8'));
  log(`6. token OK — claims sub=${claims.sub} tenant_id=${claims.tenant_id} aud=${claims.aud}`);

  const mcp = async (method, params = {}) => {
    const r = await fetch(`${APP_URL}/mcp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Accept: 'application/json', Authorization: `Bearer ${token}` },
      body: JSON.stringify({ jsonrpc: '2.0', id: 1, method, params }),
    });
    if (!r.ok) throw new Error(`MCP ${method} → ${r.status}: ${await r.text()}`);
    return r.json();
  };

  // 7. initialize + tools/list
  await mcp('initialize', { protocolVersion: '2024-11-05', capabilities: {}, clientInfo: { name: 'auto', version: '1' } });
  const list = await mcp('tools/list');
  const tools = (list.result?.tools ?? []).map((t) => t.name);
  log(`7. tools/list OK — tools: ${tools.join(', ') || '(none)'}`);
  if (!tools.includes('list_order')) throw new Error('list_order not present in tools/list');

  // 8. tools/call list_order
  const call = await mcp('tools/call', { name: 'list_order', arguments: { limit: 5, offset: 0 } });
  if (call.result?.isError) throw new Error(`tool error: ${JSON.stringify(call.result.content)}`);
  const items = call.result?.content ?? [];
  const rows = [];
  for (const it of items) {
    if (it.type === 'text' && typeof it.text === 'string') {
      try {
        const p = JSON.parse(it.text);
        (Array.isArray(p) ? p : [p]).forEach((r) => rows.push(r));
      } catch { /* non-JSON text item */ }
    } else if (it && typeof it === 'object') rows.push(it);
  }
  log(`8. tools/call OK — ${rows.length} row(s) returned`);

  // 9. verify tenant isolation
  let violations = 0;
  for (const r of rows) {
    log(`     order id=${r.id} customer=${r.customer_name ?? r.customer ?? '?'} tenant_id=${r.tenant_id}`);
    if (expectedTenant != null && 'tenant_id' in r && Number(r.tenant_id) !== expectedTenant) violations++;
  }
  if (expectedTenant != null && Number(claims.tenant_id) !== expectedTenant) {
    throw new Error(`token tenant_id=${claims.tenant_id} != expected ${expectedTenant}`);
  }
  if (rows.length === 0) throw new Error('no rows returned');
  if (violations > 0) throw new Error(`${violations} cross-tenant row(s) leaked`);

  log(`GO: PASS — ${rows.length} row(s), all tenant_id=${expectedTenant}; list_order present; full OAuth+MCP flow completed.`);
}

main().catch((e) => {
  console.error(`NO-GO: ${e.message}`);
  process.exit(1);
});
