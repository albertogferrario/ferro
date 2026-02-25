# Static Files

Ferro serves static files from the `public/` directory for any request that doesn't match a registered route. This works automatically with zero configuration.

## How It Works

When a GET or HEAD request doesn't match any route, Ferro checks if a corresponding file exists in `public/`. If found, it serves the file with the correct MIME type and cache headers. If not, the request falls through to the fallback handler (e.g., Inertia SPA catch-all) or returns 404.

```
Request: GET /assets/main-abc123.js
  1. Route matching → no match
  2. Static file check → public/assets/main-abc123.js exists
  3. Serve file with Content-Type: application/javascript
```

Only GET and HEAD requests trigger static file checks. POST, PUT, DELETE, and other methods skip filesystem checks entirely.

## Cache Strategy

Ferro applies differentiated cache headers based on the request path:

| Path pattern | Cache-Control | Rationale |
|---|---|---|
| `/assets/*` | `public, max-age=31536000, immutable` | Vite hashed output — content hash in filename means the URL changes when content changes |
| Everything else | `public, max-age=0, must-revalidate` | Root files like `favicon.ico`, `robots.txt` may change without URL change |

This means Vite build output (`public/assets/`) is cached for one year with no revalidation, while root-level files are always revalidated.

## Security

Static file serving includes the following protections:

- **Dotfile rejection**: Paths containing segments starting with `.` are rejected (prevents serving `.env`, `.git/config`, `.planning/`, etc.)
- **Directory traversal protection**: Paths are canonicalized and verified to remain within `public/`. Symlinks and `..` segments that resolve outside `public/` are blocked.
- **Null byte rejection**: Paths containing null bytes are rejected.
- **Directory listing disabled**: Requests to directories return nothing (falls through to fallback/404).

## Development vs Production

In development, `ferro serve` starts the Vite dev server which handles asset serving via HMR. The HTML references `http://localhost:5173/src/main.tsx`, not `/assets/main.js`. Since `public/assets/` doesn't exist until `vite build` runs, static file serving is effectively a no-op in development.

In production, `vite build` outputs hashed files to `public/assets/`. The compiled Ferro binary serves these files directly.

## Large Files

Static file serving reads entire files into memory. This is appropriate for typical web assets (JS, CSS, fonts, images under 1MB).

For large files (video, datasets, user uploads), use [Storage](storage.md) with a CDN or object storage service instead of placing files in `public/`.
