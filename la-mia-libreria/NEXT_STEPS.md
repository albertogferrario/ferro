# Handoff & Next Steps

This is the working note for continuing **La Mia Libreria** in its own
repository. It captures the current state, what was verified, the known
limitations, and where to go next.

## Origin

Built as a standalone Rust project on the **Ferro** framework (`ferro-rs 0.2`).
It was scaffolded and compile-verified inside the Ferro repo container, then
pushed to the branch `claude/ebook-scraping-app-j5fslk` (subfolder
`la-mia-libreria/`) purely as a transport mechanism — repo-creation permissions
were unavailable in that session.

> When moving this to its own repo, **drop the `[workspace]` line** at the top of
> `Cargo.toml`. It exists only to isolate the crate from Ferro's surrounding
> workspace while it lives on the transport branch.

## Current state — MVP complete

A complete vertical slice works end to end:

- Search any book across **Open Library** (universal metadata) + **Project
  Gutenberg** / Gutendex (public-domain, downloadable) — queried in parallel,
  merged, tolerant to one source failing.
- Save results to a personal collection (idempotent on `source + source_id`).
- List / remove from the collection.
- Download the real EPUB for **public-domain** titles into local storage;
  non-public-domain books are explicitly refused.
- Single-page UI (`src/controllers/library_page.html`), no build step.
- `Book` projection (`src/projections/books.rs`) — Ferro's core intent
  abstraction, ready for richer rendering later.

## Verified

- `cargo check` and `cargo clippy --all-targets -- -D warnings` — clean.
- Migration runs; `books` table created.
- Server boots; write path tested live: **create → dedup → list → delete**.

## Known limitation (environment, not code)

External search and file download were **not** exercisable in the build sandbox:
its network policy returns `403` for `openlibrary.org`, `gutendex.com`,
`gutenberg.org`. The code handles upstream failure gracefully (empty results, a
clean error on download). On a machine with open outbound network these work.
**First thing to do in the new repo: run `cargo run`, search a title, and
download a Gutenberg book to confirm the live path.**

## Roadmap

Deliberately left out of the MVP to keep scope tight:

1. **Background download job** — move `download` off the request thread onto
   `ferro-queue` (the code already isolates the fetch+store step). Enables
   batch downloads and retries.
2. **Bulk catalog import** — optional job to ingest a slice of the Open Library
   dump for offline metadata search.
3. **More public-domain sources** — Standard Ebooks (curated editions),
   Internet Archive / Open Library readable editions.
4. **In-app EPUB reader** — currently the file is stored; add a reader view.
5. **Reading workflow** — surface and edit `status`
   (`wanted` / `owned` / `reading` / `read`), progress, notes.
6. **Richer metadata** — fetch descriptions/subjects on import (Open Library
   `works` endpoint) to populate the `description` field.
7. **Tests** — handler tests for store/dedup/delete; a mocked catalog client.
8. **Auth / multi-device** — only if it stops being single-user.

## Map of the code

| Path | Purpose |
|------|---------|
| `src/main.rs` | CLI, config, migrations, server boot |
| `src/bootstrap.rs` | DB pool init |
| `src/routes.rs` | Route table |
| `src/controllers/library.rs` | Handlers |
| `src/controllers/library_page.html` | UI |
| `src/catalog/mod.rs` | Open Library + Gutendex client |
| `src/models/` | `Book` entity + custom finders |
| `src/migrations/` | Schema |
| `src/projections/books.rs` | Ferro service projection |

## Scope reminder

This is a personal **catalog** plus public-domain downloads. The download
endpoint refuses anything not flagged `public_domain` by design — keep it that
way.
