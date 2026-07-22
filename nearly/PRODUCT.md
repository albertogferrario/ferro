# Nearly — Product Brief

> _"Meet the people around you — in person, not in a chat."_

Nearly is a location-based social app that surfaces the most lively places in your
city in real time and lets you notice — and gently signal — the people near you.
Its defining constraint is what it **deliberately leaves out**: there is no chat,
no direct messages, no inbox of endless text. The only thing you can send is a
**trillo** — a single, wordless "I noticed you, come say hi." Everything else
happens face to face.

This brief is the product source of truth. The engineering plan lives in
[`ARCHITECTURE.md`](./ARCHITECTURE.md); this repository is a reference
implementation of Nearly on the **Ferro** framework.

---

## 1. Idea / Concept

Nearly puts people back in contact by valuing the **human factor** of interaction.
It is a means to start new relationships and acquaintances, and a
communication/information tool for finding where the crowd is right now in your city.

**Value drivers (`+`)**
- Know the busiest / trending places in real time.
- A low-pressure way to open new connections.
- Human-first: because there is **no in-app messaging**, a trillo can only be
  resolved by actually meeting, introducing yourself, starting a conversation.
- A surface for businesses to promote a venue (a **premium** place stays visible
  next to the organic trend area).

**Neutral factors (`~`)**
- Sessions are short and situational (you open it when you're out).

**Risks (`-`) — engineering must mitigate**
- Battery drain from continuous location updates → coarse, throttled presence.
- Location precision → snap to place / neighborhood granularity, never a home address.
- Fake positions → presence is server-timestamped and expires; stale presence disappears.

> **PM note on source material.** The original wireframe deck used crude, sexualized
> placeholder copy for the profile pop-up and trillo notification. That copy is **not**
> part of the product and has been replaced throughout with respectful, on-brand copy.
> Nearly is a place to meet people with dignity; the product voice reflects that.

## 2. Personas

| Persona | Who | Primary job-to-be-done |
|---------|-----|------------------------|
| **Giulia, 26 — the Explorer** | New in town, wants to find where people actually are tonight. | "Show me the lively spots and who's around." |
| **Marco, 31 — the Connector** | Sociable, enjoys meeting people spontaneously. | "Let me signal interest without a cheesy opener." |
| **Bar Luce — the Venue** | A café/bar owner wanting foot traffic. | "Make my place stand out in the trend area." |

## 3. Core user flows

1. **Arrive → Map.** Open Nearly → splash → the map fills the screen, centered on
   you, dotted with nearby **people** (blue pins) and **places** (trend + premium).
2. **Notice someone → Pop-up.** Tap a person pin → a focused profile card:
   photo, name, a short status. One action: **Invia un trillo**.
3. **Send a trillo.** The other person receives a trillo notification:
   _"Qualcuno ti ha inviato un trillo. Rispondi di persona!"_ They can **Accetta**
   (I'm here / say hi) or **Ignora**. There is no text field — by design.
4. **Places.** Browse trending venues; premium venues are highlighted.
5. **Account & Settings.** Edit your profile; toggle **visibility** (go invisible),
   read the about/credits screen.

## 4. Screen map (from the navigability wireframe)

```
Splash  ──▶  Login / Register  ──▶  Home (full-screen Map)
                                     ├─ nav 1: menu (account, settings, places, trilli)
                                     ├─ pop-up: user card ▶ "Invia un trillo"
                                     ├─ trillo notification ▶ Accetta / Ignora
                                     ├─ Account
                                     └─ Settings: visibility · about/credits
```

## 5. Domain model

| Entity | Meaning | Key fields |
|--------|---------|-----------|
| **User** | Account / identity | name, email, password |
| **Profile** | Public presence identity | user_id, display_name, status, avatar_url, **visible** |
| **Presence** | Where you are *now* (throttled, expiring) | user_id, lat, lng, last_seen |
| **Trillo** | A wordless ping A→B | from_user_id, to_user_id, **status** (pending/accepted/declined) |
| **Place** | A venue on the map | name, category, lat, lng, **premium** |

The **projection / intent** system (Ferro's core abstraction) describes each of
these as a `ServiceDef`, from which the framework derives intents and UI. Screen
intents map to Ferro's seven archetypes:

| Screen | Intent |
|--------|--------|
| Map (who/what is around, live) | `track` |
| User pop-up (one person) | `focus` |
| Login / Register / Account / Settings (forms) | `collect` |
| Trilli inbox, Places | `browse` |

## 6. Scope

**v1 (this implementation)**
- Password auth (register / login / logout), session-based.
- Full-screen Leaflet map with people + place markers (seeded demo city: Milan).
- User pop-up page + **send trillo**.
- Trilli inbox with accept / decline.
- Places browse (trend + premium).
- Account (edit profile) + Settings (visibility toggle, about/credits).
- Seeded demo data so the map is alive on first boot.

**Deliberately out of v1**
- **Any messaging / chat** — this is a permanent product principle, not a backlog item.
- Real-time push over WebSocket. Presence is modeled and expiring; live streaming of
  positions is a **v2** direction (Ferro's `ferro-broadcast` / `ferro-projection`),
  consistent with the framework's "multimodal/real-time is a later direction" stance.
- Native GPS capture (the browser demo seeds/updates presence via an endpoint).

## 7. Success criteria

- On first boot the map renders with living pins — no setup required.
- Sending a trillo and accepting/declining works end-to-end.
- Every JSON-UI view lints clean (declares a valid `design.intent`).
- `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test` are green.
</invoke>
