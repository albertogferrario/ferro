---
name: ferro:serve
description: Start the Ferro development server (backend + frontend)
allowed-tools:
  - Bash
---

<objective>
Start the Ferro development server. Backend runs under an in-process supervisor
with an optional file-watcher; the Vite frontend runs alongside. Manual reload
is always available via the `r` key; auto-reload is opt-in via `--watch`.
</objective>

<arguments>
Optional:
- `--port=PORT` — backend port (default 8080)
- `--frontend-port=PORT` — frontend port (default 5173)
- `--backend-only` — run only the Rust backend
- `--frontend-only` — run only the Vite frontend
- `--skip-types` — skip TypeScript type regeneration
- `--watch` — enable file-watch auto-reload (500ms trailing-edge debounce); off by default

Examples:
- `/ferro:serve`
- `/ferro:serve --watch`
- `/ferro:serve --port=3000 --frontend-port=5174`
- `/ferro:serve --backend-only`
</arguments>

<process>

<step name="start_server">

Default (manual reload — use when you want explicit control over rebuild timing):

```bash
ferro serve
```

With auto-reload (file watcher, 500ms trailing-edge debounce):

```bash
ferro serve --watch
```

Custom ports:

```bash
ferro serve --port 3000 --frontend-port 5174
```

Backend only (no Vite):

```bash
ferro serve --backend-only
```

</step>

<step name="runtime_keys">

When stdin is a TTY, the supervisor listens for two keys:

- `r` — rebuild the backend and regenerate types. If a build is in flight, it is
  cancelled and a fresh build starts.
- `q` or Ctrl-C — graceful shutdown of backend, frontend, and types watcher.

When stdin is not a TTY, `r` is unavailable; Ctrl-C still works via the signal.

</step>

</process>

<tips>
- The file watcher runs in-process via `notify-debouncer-mini`; no external watcher is installed.
- Use `ferro serve` (no `--watch`) during intensive editing; press `r` when ready to rebuild.
- Use `ferro serve --watch` when every save should trigger a rebuild.
- Check `.env` for backend config (`SERVER_HOST`, `SERVER_PORT`, `VITE_PORT`).
</tips>
