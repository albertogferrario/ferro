---
name: ferro:new
description: Create a new Ferro project
allowed-tools:
  - Bash
  - Read
  - Write
  - AskUserQuestion
---

<objective>
Create a new Ferro project with complete scaffolding.

Uses the `ferro new` CLI command or manual scaffolding if CLI unavailable.
</objective>

<arguments>
Required:
- `[name]` - Project name (defaults to current directory name)

Optional:
- `--database=TYPE` - Database type: sqlite, postgres, mysql (default: sqlite)
- `--with-inertia` - Include Inertia.js setup
- `--with-auth` - Include authentication scaffolding
- `--minimal` - Minimal setup without extras

Examples:
- `/ferro:new my-app`
- `/ferro:new blog --database=postgres --with-auth`
- `/ferro:new api --minimal`
</arguments>

<process>

<step name="check_cli">

Check if Ferro CLI is installed:

```bash
if command -v ferro &> /dev/null; then
    echo "CLI_AVAILABLE"
    ferro --version
else
    echo "CLI_NOT_AVAILABLE"
fi
```

</step>

<step name="create_project">

**With CLI:**

```bash
ferro new {name} {--database=TYPE} {--with-auth} {--with-inertia}
```

**Without CLI (manual scaffolding):**

1. Create project directory
2. Initialize Cargo.toml with Ferro dependencies
3. Create directory structure
4. Generate initial files

</step>

<step name="scaffold_structure">

Create directory structure:

```
{name}/
├── src/
│   ├── main.rs
│   ├── routes.rs
│   ├── controllers/
│   │   └── mod.rs
│   ├── models/
│   │   ├── mod.rs
│   │   └── entities/
│   │       └── mod.rs
│   ├── middleware/
│   │   └── mod.rs
│   ├── jobs/
│   │   └── mod.rs
│   ├── events/
│   │   └── mod.rs
│   └── config/
│       ├── mod.rs
│       ├── app.rs
│       ├── database.rs
│       └── mail.rs
├── migrations/
├── tests/
├── storage/
│   └── logs/
├── .env
├── .env.example
├── .gitignore
├── Cargo.toml
└── README.md
```

</step>

<step name="generate_cargo_toml">

```toml
[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
ferro-rs = "0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = "0.3"

{if with-inertia}
ferro-inertia = "0.1"
{/if}

{if with-auth}
# Auth dependencies
jsonwebtoken = "9"
bcrypt = "0.15"
{/if}

[dev-dependencies]
tokio-test = "0.4"
```

</step>

<step name="generate_main">

```rust
//! {Name} Application

use ferro_rs::prelude::*;

mod config;
mod controllers;
mod middleware;
mod models;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::init();

    // Build application
    let app = Application::new()
        .routes(routes::register)
        .build()
        .await?;

    // Start server
    let addr = std::env::var("APP_URL").unwrap_or_else(|_| "127.0.0.1:8080".into());
    tracing::info!("Starting server at http://{}", addr);

    app.serve(&addr).await?;

    Ok(())
}
```

</step>

<step name="generate_env">

```env
APP_NAME={name}
APP_ENV=development
APP_DEBUG=true
APP_URL=http://localhost:8080
APP_KEY=base64:generate-a-random-key-here

{if database == "sqlite"}
DATABASE_URL=sqlite:database.sqlite
{else if database == "postgres"}
DATABASE_URL=postgres://user:password@localhost:5432/{name}
{else if database == "mysql"}
DATABASE_URL=mysql://user:password@localhost:3306/{name}
{/if}

LOG_LEVEL=debug
```

</step>

<step name="init_git">

```bash
cd {name}
git init
git add .
git commit -m "Initial commit: Ferro project scaffolding"
```

</step>

<step name="summary">

```
✨ Created Ferro project: {name}

Directory: ./{name}

Next steps:
  1. cd {name}
  2. Copy .env.example to .env and configure
  3. Run migrations: ferro db:migrate
  4. Start server: ferro serve (or cargo run)

Available commands:
  /ferro:serve      - Start development server
  /ferro:routes     - List all routes
  /ferro:model      - Generate a model
  /ferro:controller - Generate a controller
  /ferro:help       - Show all commands

Happy coding! 🦀
```

</step>

</process>
