# Directory Structure

A Ferro project follows a convention-based structure inspired by Laravel.

```
my-app/
├── Cargo.toml              # Rust dependencies
├── .env                    # Environment configuration
├── src/
│   ├── main.rs             # Application entry point
│   ├── routes.rs           # Route definitions
│   ├── bootstrap.rs        # Global middleware registration
│   ├── actions/            # Business logic handlers
│   ├── controllers/        # HTTP controllers
│   ├── middleware/         # Custom middleware
│   ├── models/             # Database entities (SeaORM)
│   ├── migrations/         # Database migrations
│   ├── services/           # Service implementations
│   ├── requests/           # Form request validation
│   ├── events/             # Event definitions
│   ├── listeners/          # Event listeners
│   ├── jobs/               # Background jobs
│   ├── notifications/      # Notification classes
│   └── tasks/              # Scheduled tasks
├── frontend/
│   ├── src/
│   │   ├── pages/          # Inertia.js React components
│   │   ├── components/     # Reusable UI components
│   │   ├── types/          # TypeScript type definitions
│   │   └── main.tsx        # Frontend entry point
│   ├── package.json        # Node dependencies
│   └── vite.config.ts      # Vite configuration
├── storage/
│   ├── app/                # Application files
│   └── logs/               # Log files
└── public/
    └── storage/            # Symlink to storage/app/public
```

## Key Directories

### `src/actions/`

Business logic that doesn't fit neatly into controllers. Actions are invocable classes for complex operations.

```rust
// src/actions/create_user.rs
#[derive(Action)]
pub struct CreateUser {
    user_service: Arc<dyn UserService>,
}

impl CreateUser {
    pub async fn execute(&self, data: CreateUserData) -> Result<User> {
        // Business logic here
    }
}
```

### `src/controllers/`

HTTP handlers grouped by resource. Generated with `ferro make:controller`.

```rust
// src/controllers/users_controller.rs
#[handler]
pub async fn index(req: Request) -> Response { ... }

#[handler]
pub async fn store(req: Request, form: CreateUserForm) -> Response { ... }
```

### `src/models/`

SeaORM entity definitions. Generated with `ferro db:sync`.

### `src/middleware/`

Custom middleware for request/response processing.

### `src/events/` and `src/listeners/`

Event-driven architecture. Events dispatch to multiple listeners.

### `src/jobs/`

Background job definitions for queue processing.

### `frontend/src/pages/`

Inertia.js page components. Path determines the route component.

```
pages/Users/Index.tsx  →  Inertia::render("Users/Index", ...)
pages/Dashboard.tsx    →  Inertia::render("Dashboard", ...)
```

## Configuration Files

### `.env`

Environment-specific configuration:

```env
APP_ENV=local
APP_DEBUG=true
DATABASE_URL=sqlite:database.db
REDIS_URL=redis://localhost:6379
```

### `Cargo.toml`

Rust dependencies. Ferro crates are added here.

### `frontend/package.json`

Node.js dependencies for the React frontend.

## Generated Directories

These directories are created automatically:

- `target/` - Rust build artifacts
- `node_modules/` - Node.js dependencies
- `frontend/dist/` - Built frontend assets

## Storage

The `storage/` directory holds application files:

```bash
# Create public storage symlink
ferro storage:link
```

This links `public/storage` → `storage/app/public` for publicly accessible files.
