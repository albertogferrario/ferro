# Installation

## Requirements

The Ferro CLI installs toolchain-free via Homebrew (below). The following are only needed to
**build and run** a scaffolded app:

- Rust 1.88+ (with Cargo) — to build the app (no OpenSSL needed; the scaffold uses rustls)
- Node.js 18+ (for the frontend dev server)
- PostgreSQL, SQLite, or MySQL — SQLite is the default, no setup required

## Installing the CLI

### Homebrew (macOS and Linux — recommended)

No Rust toolchain required:

```bash
brew install albertogferrario/ferro/ferro
```

### curl installer (macOS and Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/albertogferrario/ferro/main/scripts/install.sh | sh
```

### Cargo (requires Rust)

```bash
cargo install ferro-cli
```

Or build from source:

```bash
git clone https://github.com/albertogferrario/ferro.git
cd ferro
cargo install --path ferro-cli
```

## Creating a New Project

```bash
ferro new my-app
```

This will:
1. Create a new directory `my-app`
2. Initialize a Rust workspace
3. Set up the frontend with React and TypeScript
4. Configure the database
5. Initialize git repository

### Options

```bash
# Skip interactive prompts
ferro new my-app --no-interaction

# Skip git initialization
ferro new my-app --no-git
```

## Starting Development

```bash
cd my-app
ferro serve
```

This starts both the backend (port 8080) and frontend (port 5173) servers.

### Server Options

```bash
# Custom ports
ferro serve --port 3000 --frontend-port 3001

# Backend only
ferro serve --backend-only

# Frontend only
ferro serve --frontend-only

# Skip TypeScript generation
ferro serve --skip-types
```

## AI Development Setup

For AI-assisted development with Claude, Cursor, or VS Code:

```bash
ferro boost:install
```

This configures the MCP server and adds project guidelines for your editor.

## Next Steps

- [Quick Start](quickstart.md) - Build your first feature
- [Directory Structure](directory-structure.md) - Understand the project layout
