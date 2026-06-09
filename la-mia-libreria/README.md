# La Mia Libreria

A personal book library built on the [Ferro](https://crates.io/crates/ferro-rs)
Rust web framework. Search essentially every book that exists, keep the ones you
care about in your own collection, and download **public-domain** titles to read
offline.

## What it does

- **Search any book** — queries [Open Library](https://openlibrary.org)
  (~40M editions of metadata) and [Project Gutenberg](https://gutenberg.org)
  (via the [Gutendex](https://gutendex.com) API) in parallel and merges the
  results.
- **Build a collection** — save any search result to your personal library and
  track its status (`wanted` / `owned` / `reading` / `read`).
- **Download public-domain books** — for Project Gutenberg titles (which are in
  the public domain) it fetches the real EPUB into local storage so you keep the
  file.

### A note on copyright

This app only downloads files that are **legally free to download** — public-domain
works from Project Gutenberg. For every other book it stores metadata only
(title, author, cover, etc.). The download endpoint explicitly refuses any book
not flagged `public_domain`. It is a personal *catalog*, not a piracy tool.

## Running it

```bash
cp .env.example .env        # adjust if you like; SQLite works out of the box
cargo run                   # migrates the DB, then serves on http://127.0.0.1:8080
```

Then open <http://127.0.0.1:8080>.

Other commands:

```bash
cargo run -- db:migrate     # run migrations only
cargo run -- serve --no-migrate
```

> Note: searching and downloading require outbound internet access to
> `openlibrary.org`, `gutendex.com` and `gutenberg.org`. In restricted/sandboxed
> environments those hosts may be blocked; the app degrades gracefully (an empty
> result set, a clear error on download) rather than crashing.

## Structure

| Path | Purpose |
|------|---------|
| `src/main.rs` | Entry point: CLI, config, migrations, server |
| `src/routes.rs` | Route registration |
| `src/controllers/library.rs` | HTTP handlers (search, collection, download) |
| `src/controllers/library_page.html` | Self-contained single-page UI (no build step) |
| `src/catalog/mod.rs` | Open Library + Gutendex client |
| `src/models/` | `Book` entity and model |
| `src/migrations/` | Database schema |
| `src/projections/books.rs` | Ferro service projection for the Book entity |

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/` | Single-page UI |
| `GET` | `/library/search?q=` | Search external catalogs |
| `GET` | `/library/books` | List the saved collection |
| `POST` | `/library/books` | Save a search result (idempotent on source) |
| `DELETE` | `/library/books/:book` | Remove a book |
| `POST` | `/library/books/:book/download` | Download a public-domain file |

## License

MIT
