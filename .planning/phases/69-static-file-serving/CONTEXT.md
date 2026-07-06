# Phase 69: Static File Serving

## Problem

Ferro has no built-in static file serving. In development, `ferro serve` spawns Vite which serves assets. In production, the binary serves only registered routes — requests to `/assets/main.js` (Vite build output) return 404.

This was discovered during mkmenu production deployment. A manual `/assets/{path}` controller was added as a workaround, but every Ferro app will hit this.

## Requirements

- Serve files from `public/` directory for any request that doesn't match a registered route
- Production-ready: correct MIME types, cache headers, directory traversal protection
- Zero configuration — works out of the box for all Ferro apps
- In development mode (when Vite dev server is running), this should be a no-op since Vite handles assets

## Scope

- Built-in middleware or fallback handler in the framework (`ferro-rs` crate)
- Serves from `public/` relative to working directory
- Immutable cache headers for hashed assets (`/assets/*.js`)
- No-cache for root files (`/favicon.ico`, `/robots.txt`)
- Should integrate with the existing server.rs request handler (check filesystem before returning 404)

## Reference

- Ferro server: `framework/src/server.rs` (lines 154-202) — route matching + fallback
- Inertia production HTML: `ferro-inertia/src/response.rs` (lines 432-449) — references `/assets/main.js`, `/assets/main.css`
- Vite build output: `public/assets/` (configured in `frontend/vite.config.ts`)
- mkmenu workaround: `src/controllers/public/assets.rs` in gestiscilo-it/mkmenu
