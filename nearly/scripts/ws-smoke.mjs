// Nearly real-time smoke test — verifies the full ferro-broadcast stack live.
//
// Usage (Node 22+, no deps):
//   BROADCAST_SECRET=test-secret APP_ENV=production APP_NAME=Nearly \
//     DATABASE_URL="sqlite://$PWD/smoke.db" ./target/debug/nearly &   # from repo root
//   node nearly/scripts/ws-smoke.mjs
//
// Asserts: live PresenceUpdated on the public `nearby` channel; live
// TrilloReceived on a recipient's signed private channel; and that a client
// cannot obtain a token for (403) or forge a subscription to someone else's
// channel. Exits non-zero on any failure.
const BASE = 'http://127.0.0.1:8080'
const WS = 'ws://127.0.0.1:8080/_ferro/ws'
const sleep = (ms) => new Promise((r) => setTimeout(r, ms))

function jar() {
  return {
    c: {},
    hdr() { return Object.entries(this.c).map(([k, v]) => `${k}=${v}`).join('; ') },
    set(res) {
      const sc = (res.headers.getSetCookie && res.headers.getSetCookie()) || []
      for (const s of sc) { const kv = s.split(';')[0]; const i = kv.indexOf('='); this.c[kv.slice(0, i)] = kv.slice(i + 1) }
    },
  }
}
async function get(path, j) { const r = await fetch(BASE + path, { headers: { cookie: j.hdr() }, redirect: 'manual' }); j.set(r); return r }
async function post(path, body, j, csrf) {
  const r = await fetch(BASE + path, {
    method: 'POST', redirect: 'manual',
    headers: { cookie: j.hdr(), 'content-type': 'application/json', ...(csrf ? { 'x-csrf-token': csrf } : {}) },
    body: JSON.stringify(body),
  }); j.set(r); return r
}
const metaCsrf = (h) => (h.match(/name="csrf-token" content="([^"]+)"/) || [])[1]
function dataPage(h) {
  const m = h.match(/data-page="([^"]+)"/); if (!m) return null
  const s = m[1].replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/&#x27;/g, "'").replace(/&amp;/g, '&')
  return JSON.parse(s)
}

let failures = 0
const ok = (c, m) => { console.log(`${c ? '✅' : '❌'} ${m}`); if (!c) failures++ }

async function main() {
  // --- Session A: log in as the demo user alex ---
  const A = jar()
  let h = await (await get('/login', A)).text()
  await post('/login', { email: 'alex@nearly.app', password: 'password123' }, A, metaCsrf(h))
  h = await (await get('/map', A)).text()
  const csrfA = metaCsrf(h)
  const alexId = dataPage(h)?.props?.auth?.id
  ok(Number.isInteger(alexId), `logged in as alex (id=${alexId})`)

  // --- Session B: register a fresh user (auto-logged-in) ---
  const B = jar()
  let hb = await (await get('/register', B)).text()
  const email = `bruno_${process.pid}@example.com`
  await post('/register', { name: 'Bruno Test', email, password: 'password123' }, B, metaCsrf(hb))
  hb = await (await get('/map', B)).text()
  const csrfB = metaCsrf(hb)
  const bId = dataPage(hb)?.props?.auth?.id
  ok(Number.isInteger(bId) && bId !== alexId, `registered second user Bruno (id=${bId})`)

  // --- Security: A cannot get a token for someone else's private channel ---
  const forbidden = await post('/broadcasting/auth', { channel_name: `private-user.${bId}`, socket_id: 'x' }, A, csrfA)
  ok(forbidden.status === 403, `auth for another user's private channel → 403 (got ${forbidden.status})`)

  // --- Open A's WebSocket and subscribe to nearby + own private channel ---
  const ws = new WebSocket(WS)
  const events = []
  let socketId = null
  const subs = new Set()
  ws.addEventListener('message', async (ev) => {
    const msg = JSON.parse(ev.data)
    if (msg.type === 'connected') {
      socketId = msg.socket_id
      ws.send(JSON.stringify({ type: 'subscribe', channel: 'nearby' }))
      // Signed subscribe to own private channel.
      const res = await post('/broadcasting/auth', { channel_name: `private-user.${alexId}`, socket_id: socketId }, A, csrfA)
      const auth = await res.json()
      ws.send(JSON.stringify({ type: 'subscribe', channel: `private-user.${alexId}`, auth: auth.auth, channel_data: auth.channel_data }))
      // Forge attempt: try to ride B's private channel with a bogus signature.
      ws.send(JSON.stringify({ type: 'subscribe', channel: `private-user.${bId}`, auth: 'deadbeef' }))
    } else if (msg.type === 'subscribed') { subs.add(msg.channel) }
    else if (msg.type === 'subscription_error') { events.push({ err: msg.channel }) }
    else if (msg.type === 'event') { events.push({ event: msg.event, data: msg.data }) }
  })
  await new Promise((r) => ws.addEventListener('open', r))
  await sleep(600)

  ok(subs.has('nearby'), 'subscribed to public nearby')
  ok(subs.has(`private-user.${alexId}`), 'subscribed to own private channel (signed)')
  ok(events.some((e) => e.err === `private-user.${bId}`), 'forged subscription to another private channel → rejected')

  // --- Live presence: A moves; expect a PresenceUpdated broadcast ---
  await post('/presence', { lat: 45.4642, lng: 9.19 }, A, csrfA)
  // --- Live trillo: B trillos A; expect a TrilloReceived on A's private channel ---
  await post('/trilli', { to_user_id: alexId }, B, csrfB)
  await sleep(800)

  const presence = events.find((e) => e.event === 'PresenceUpdated' && e.data?.user_id === alexId)
  ok(!!presence, `received live PresenceUpdated (lat=${presence?.data?.lat})`)
  const trillo = events.find((e) => e.event === 'TrilloReceived')
  ok(!!trillo, `received live TrilloReceived (from=${trillo?.data?.from})`)

  ws.close()
  console.log(failures === 0 ? '\nALL PASS' : `\n${failures} FAILED`)
  process.exit(failures === 0 ? 0 : 1)
}
main().catch((e) => { console.error(e); process.exit(1) })
