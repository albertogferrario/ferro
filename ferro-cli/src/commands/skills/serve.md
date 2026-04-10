---
name: ferro:serve
description: Start development server
allowed-tools:
  - Bash
---

<objective>
Start the Ferro development server with hot reload and request logging.
</objective>

<arguments>
Optional:
- `--port=PORT` - Port to listen on (default: 8080)
- `--host=HOST` - Host to bind to (default: 127.0.0.1)
- `--release` - Run in release mode
- `--watch` - Enable hot reload (default)
- `--no-watch` - Disable hot reload

Examples:
- `/ferro:serve`
- `/ferro:serve --port=3000`
- `/ferro:serve --release`
</arguments>

<process>

<step name="check_deps">

Check if cargo-watch is installed for hot reload:

```bash
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch for hot reload..."
    cargo install cargo-watch
fi
```

</step>

<step name="start_server">

**With hot reload (default):**

```bash
cargo watch -x run
```

Or with specific features:
```bash
cargo watch -x "run --all-features"
```

**Without hot reload (--no-watch):**

```bash
cargo run
```

**Release mode (--release):**

```bash
cargo run --release
```

**Custom port/host:**

Set environment variables before running:
```bash
APP_PORT={port} APP_HOST={host} cargo watch -x run
```

</step>

<step name="output">

Display server info:

```
🚀 Ferro Development Server

   Local:    http://127.0.0.1:8080
   Network:  http://192.168.1.100:8080

   Mode:     development
   Reload:   enabled

   Press Ctrl+C to stop

─────────────────────────────────────────
```

Then show request logs as they come in:

```
[2024-01-15 10:30:45] GET /api/users 200 12ms
[2024-01-15 10:30:46] POST /api/users 201 45ms
[2024-01-15 10:30:47] GET /api/users/1 200 8ms
```

</step>

</process>

<tips>
- The server automatically reloads when you save files
- Check `.env` for configuration (DATABASE_URL, APP_KEY, etc.)
- Use `/ferro:logs` to view detailed logs
- Use `/ferro:routes` to see available endpoints
</tips>
