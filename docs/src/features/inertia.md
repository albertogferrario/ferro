# Inertia.js

Ferro provides first-class Inertia.js integration, enabling you to build modern single-page applications using React while keeping your routing and controllers on the server. This gives you the best of both worlds: the snappy feel of an SPA with the simplicity of server-side rendering.

## How Inertia Works

Inertia.js is a protocol that connects your server-side framework to a client-side framework (React, Vue, or Svelte). Instead of returning HTML or building a separate API:

1. Your controller returns an Inertia response with a component name and props
2. On the first request, a full HTML page is rendered with the initial data
3. On subsequent requests, only JSON is returned
4. The client-side adapter swaps components without full page reloads

## Configuration

### Environment Variables

Configure Inertia in your `.env` file:

```env
# Vite development server URL
VITE_DEV_SERVER=http://localhost:5173

# Frontend entry point
VITE_ENTRY_POINT=src/main.tsx

# Asset version for cache busting
INERTIA_VERSION=1.0

# Development mode (enables HMR)
APP_ENV=development
```

### Bootstrap Setup

In `src/bootstrap.rs`, configure Inertia:

```rust
use ferro::{App, InertiaConfig};

pub async fn register() {
    // Configure from environment
    let config = InertiaConfig::from_env();
    App::set_inertia_config(config);
}
```

### Manual Configuration

```rust
use ferro::InertiaConfig;

let config = InertiaConfig {
    vite_dev_server: "http://localhost:5173".to_string(),
    entry_point: "src/main.tsx".to_string(),
    version: "1.0".to_string(),
    development: true,
    html_template: None,
};
```

## Basic Usage

### Rendering Responses

Use `Inertia::render()` to return an Inertia response:

```rust
use ferro::{handler, Request, Response};
use ferro::inertia::Inertia;
use serde::Serialize;

#[derive(Serialize)]
pub struct HomeProps {
    pub title: String,
    pub message: String,
}

#[handler]
pub async fn index(req: Request) -> Response {
    Inertia::render(&req, "Home", HomeProps {
        title: "Welcome".to_string(),
        message: "Hello from Ferro!".to_string(),
    })
}
```

The component name (`"Home"`) maps to `frontend/src/pages/Home.tsx`.

### The InertiaProps Derive Macro

For automatic camelCase conversion (standard in JavaScript), use the `InertiaProps` derive macro:

```rust
use ferro::InertiaProps;

#[derive(InertiaProps)]
pub struct DashboardProps {
    pub user_name: String,      // Serializes as "userName"
    pub total_posts: i32,       // Serializes as "totalPosts"
    pub is_admin: bool,         // Serializes as "isAdmin"
}

#[handler]
pub async fn dashboard(req: Request) -> Response {
    Inertia::render(&req, "Dashboard", DashboardProps {
        user_name: "John".to_string(),
        total_posts: 42,
        is_admin: true,
    })
}
```

In your React component:

```tsx
interface DashboardProps {
    userName: string;
    totalPosts: number;
    isAdmin: boolean;
}

export default function Dashboard({ userName, totalPosts, isAdmin }: DashboardProps) {
    return <h1>Welcome, {userName}!</h1>;
}
```

### Compile-Time Component Validation

The `inertia_response!` macro validates that your component exists at compile time:

```rust
use ferro::inertia_response;

#[handler]
pub async fn show(req: Request) -> Response {
    // Validates that frontend/src/pages/Users/Show.tsx exists
    inertia_response!(&req, "Users/Show", UserProps { ... })
}
```

If the component doesn't exist, you get a compile error with fuzzy matching suggestions:

```
error: Component "Users/Shwo" not found. Did you mean "Users/Show"?
```

## Shared Props

Shared props are data that should be available to every page component, like authentication state, flash messages, and CSRF tokens.

### Creating the Middleware

```rust
use ferro::{Middleware, Request, Response, Next};
use ferro::inertia::InertiaShared;
use async_trait::async_trait;

pub struct ShareInertiaData;

#[async_trait]
impl Middleware for ShareInertiaData {
    async fn handle(&self, mut request: Request, next: Next) -> Response {
        let mut shared = InertiaShared::new();

        // Add CSRF token
        if let Some(token) = request.csrf_token() {
            shared = shared.csrf(token);
        }

        // Add authenticated user
        if let Some(user) = request.user() {
            shared = shared.auth(AuthUser {
                id: user.id,
                name: user.name.clone(),
                email: user.email.clone(),
            });
        }

        // Add flash messages
        if let Some(flash) = request.session().get::<FlashMessages>("flash") {
            shared = shared.flash(flash);
        }

        // Add custom shared data
        shared = shared.with(serde_json::json!({
            "app_name": "My Application",
            "app_version": "1.0.0",
        }));

        // Store in request extensions
        request.insert(shared);
        next(request).await
    }
}
```

### Registering the Middleware

In `src/bootstrap.rs`:

```rust
use ferro::global_middleware;
use crate::middleware::ShareInertiaData;

pub async fn register() {
    global_middleware!(ShareInertiaData);
}
```

### Using Shared Props in Controllers

When `InertiaShared` is in the request extensions, it's automatically merged:

```rust
#[handler]
pub async fn index(req: Request) -> Response {
    // Shared props (auth, flash, csrf) are automatically included
    Inertia::render(&req, "Home", HomeProps {
        title: "Welcome".to_string(),
    })
}
```

### Accessing Shared Props in React

```tsx
import { usePage } from '@inertiajs/react';

interface SharedProps {
    auth?: {
        id: number;
        name: string;
        email: string;
    };
    flash?: {
        success?: string;
        error?: string;
    };
    csrf?: string;
}

export default function Layout({ children }) {
    const { auth, flash } = usePage<{ props: SharedProps }>().props;

    return (
        <div>
            {auth && <nav>Welcome, {auth.name}</nav>}
            {flash?.success && <div className="alert-success">{flash.success}</div>}
            {children}
        </div>
    );
}
```

## SavedInertiaContext

When you need to consume the request body (e.g., for validation) before rendering, use `SavedInertiaContext`:

```rust
use ferro::{handler, Request, Response};
use ferro::inertia::{Inertia, SavedInertiaContext};
use ferro::validation::{Validator, rules};

#[handler]
pub async fn store(req: Request) -> Response {
    // Save context BEFORE consuming the request
    let ctx = SavedInertiaContext::from_request(&req);

    // Now consume the request body
    let data: serde_json::Value = req.json().await?;

    // Validate
    let errors = Validator::new()
        .rule("title", rules![required(), string(), min(1)])
        .rule("content", rules![required(), string()])
        .validate(&data);

    if errors.fails() {
        // Use saved context to render with validation errors
        return Inertia::render_ctx(&ctx, "Posts/Create", CreatePostProps {
            errors: errors.to_json(),
            old: data,
        });
    }

    // Create the post...
    let post = Post::create(&data).await?;

    redirect!(format!("/posts/{}", post.id))
}
```

## Common Patterns

### Form Handling with Validation

The most common pattern requiring `SavedInertiaContext` is form validation. Here's the complete flow:

```rust
use ferro::{handler, Request, Response};
use ferro::inertia::{Inertia, SavedInertiaContext};
use ferro::validation::{Validator, rules};

#[handler]
pub async fn store(req: Request) -> Response {
    // STEP 1: Save context BEFORE consuming the request body
    // This is required because req.json()/req.input() consumes the body
    let ctx = SavedInertiaContext::from_request(&req);

    // STEP 2: Now safely consume the request body
    let data: CreateItemRequest = req.json().await?;

    // STEP 3: Validate
    let errors = Validator::new()
        .rule("name", rules![required(), string(), min(1)])
        .rule("email", rules![required(), email()])
        .validate(&data);

    // STEP 4: On validation failure, render with saved context
    if errors.fails() {
        return Inertia::render_ctx(&ctx, "Items/Create", FormProps {
            errors: Some(errors.to_json()),
            old: Some(data),
        });
    }

    // STEP 5: On success, redirect
    let item = Item::create(&data).await?;
    Inertia::redirect_ctx(&ctx, &format!("/items/{}", item.id))
}
```

> **Why SavedInertiaContext?** The request body in Rust can only be read once. Once you call `req.json()` or `req.input()`, the body is consumed. But `Inertia::render()` needs request metadata (headers, URL). `SavedInertiaContext` captures this metadata before body consumption.

## Frontend Setup

### Project Structure

```
your-app/
├── src/                    # Rust backend
│   ├── controllers/
│   ├── middleware/
│   └── main.rs
├── frontend/               # React frontend
│   ├── src/
│   │   ├── pages/          # Inertia page components
│   │   │   ├── Home.tsx
│   │   │   ├── Dashboard.tsx
│   │   │   └── Users/
│   │   │       ├── Index.tsx
│   │   │       └── Show.tsx
│   │   ├── components/     # Shared components
│   │   ├── layouts/        # Layout components
│   │   └── main.tsx        # Entry point
│   ├── package.json
│   └── vite.config.ts
└── Cargo.toml
```

### Entry Point (main.tsx)

```tsx
import { createInertiaApp } from '@inertiajs/react';
import { createRoot } from 'react-dom/client';

createInertiaApp({
    resolve: (name) => {
        const pages = import.meta.glob(['./pages/**/*.tsx', '!**/*.test.tsx'], { eager: true });
        return pages[`./pages/${name}.tsx`];
    },
    setup({ el, App, props }) {
        createRoot(el).render(<App {...props} />);
    },
});
```

### Vite Configuration

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
    plugins: [react()],
    server: {
        port: 5173,
        strictPort: true,
    },
    build: {
        manifest: true,
        outDir: '../public/build',
        rollupOptions: {
            input: 'src/main.tsx',
        },
    },
});
```

### Package Dependencies

```json
{
    "dependencies": {
        "@inertiajs/react": "^1.0.0",
        "react": "^18.2.0",
        "react-dom": "^18.2.0"
    },
    "devDependencies": {
        "@types/react": "^18.2.0",
        "@types/react-dom": "^18.2.0",
        "@vitejs/plugin-react": "^4.0.0",
        "typescript": "^5.0.0",
        "vite": "^5.0.0"
    }
}
```

## Links and Navigation

### Inertia Link Component

Use the Inertia `Link` component for client-side navigation:

```tsx
import { Link } from '@inertiajs/react';

export default function Navigation() {
    return (
        <nav>
            <Link href="/">Home</Link>
            <Link href="/about">About</Link>
            <Link href="/users" method="get" as="button">Users</Link>
        </nav>
    );
}
```

### Programmatic Navigation

```tsx
import { router } from '@inertiajs/react';

function handleClick() {
    router.visit('/dashboard');
}

function handleSubmit(data) {
    router.post('/posts', data, {
        onSuccess: () => {
            // Handle success
        },
    });
}
```

## Partial Reloads

Inertia supports partial reloads to refresh only specific props without a full page reload.

### Requesting Partial Data

```tsx
import { router } from '@inertiajs/react';

// Only reload the 'users' prop
router.reload({ only: ['users'] });

// Reload specific props
router.visit('/dashboard', {
    only: ['notifications', 'messages'],
});
```

### Server-Side Handling

Ferro automatically handles partial reload requests. The `X-Inertia-Partial-Data` header specifies which props to return:

```rust
#[handler]
pub async fn dashboard(req: Request) -> Response {
    // All props are computed, but only requested ones are sent
    Inertia::render(&req, "Dashboard", DashboardProps {
        user: get_user().await?,          // Always sent on full load
        notifications: get_notifications().await?,  // Only if requested
        stats: get_stats().await?,        // Only if requested
    })
}
```

## Version Conflict Handling

When your assets change (new deployment), Inertia uses versioning to force a full page reload.

### Checking Version

```rust
use ferro::inertia::Inertia;

#[handler]
pub async fn index(req: Request) -> Response {
    // Check if client version matches
    if let Some(response) = Inertia::check_version(&req, "1.0", "/") {
        return response;  // Returns 409 Conflict
    }

    Inertia::render(&req, "Home", HomeProps { ... })
}
```

### Middleware Approach

```rust
pub struct InertiaVersionCheck;

#[async_trait]
impl Middleware for InertiaVersionCheck {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let current_version = std::env::var("INERTIA_VERSION")
            .unwrap_or_else(|_| "1.0".to_string());

        if let Some(response) = Inertia::check_version(&request, &current_version, "/") {
            return response;
        }

        next(request).await
    }
}
```

## Forms

### Basic Form Handling

```tsx
import { useForm } from '@inertiajs/react';

export default function CreatePost() {
    const { data, setData, post, processing, errors } = useForm({
        title: '',
        content: '',
    });

    function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        post('/posts');
    }

    return (
        <form onSubmit={handleSubmit}>
            <input
                value={data.title}
                onChange={e => setData('title', e.target.value)}
            />
            {errors.title && <span>{errors.title}</span>}

            <textarea
                value={data.content}
                onChange={e => setData('content', e.target.value)}
            />
            {errors.content && <span>{errors.content}</span>}

            <button type="submit" disabled={processing}>
                Create Post
            </button>
        </form>
    );
}
```

### Server-Side Validation Response

```rust
use ferro::inertia::{Inertia, SavedInertiaContext};

#[handler]
pub async fn store(req: Request) -> Response {
    let ctx = SavedInertiaContext::from_request(&req);
    let data: CreatePostRequest = req.json().await?;

    let errors = validate_post(&data);
    if errors.fails() {
        // Return to form with errors
        return Inertia::render_ctx(&ctx, "Posts/Create", CreatePostProps {
            errors: errors.to_json(),
        });
    }

    let post = Post::create(&data).await?;
    redirect!(format!("/posts/{}", post.id))
}
```

## TypeScript Generation

Ferro can generate TypeScript types from your Rust props:

```bash
ferro generate-types
```

This creates type definitions for your InertiaProps structs:

```typescript
// Generated: frontend/src/types/props.d.ts
export interface HomeProps {
    title: string;
    message: string;
}

export interface DashboardProps {
    userName: string;
    totalPosts: number;
    isAdmin: boolean;
}
```

### Automatic Type Generation

When running `ferro serve`, TypeScript types are automatically regenerated whenever you modify a file containing `InertiaProps` structs. Changes are debounced (500ms) to avoid excessive regeneration.

You'll see `[types] Regenerated N type(s)` in the console when types are updated.

To disable automatic regeneration:

```bash
ferro serve --skip-types
```

> **Note:** Type watching is disabled in `--backend-only` mode since there's no frontend to update.

### Custom Types

The type generator automatically discovers structs with `#[derive(InertiaProps)]`. For nested types that don't have this derive, you have two options.

#### Option 1: Manual Type Files (Recommended)

Create manual TypeScript type files for complex domain types:

```typescript
// frontend/src/types/theme-config.ts
export interface ThemeConfig {
  primaryColor?: string;
  fontFamily?: string;
  borderRadius?: number;
}

export interface BottomNavConfig {
  enabled: boolean;
  items: NavItem[];
}
```

Then import in your components:

```tsx
import { ThemeConfig } from '@/types/theme-config';
import { DashboardProps } from '@/types/props';  // Auto-generated

interface Props extends DashboardProps {
  themeConfig: ThemeConfig;
}
```

#### Option 2: Add InertiaProps Derive

For shared types used in multiple props, add the derive:

```rust
#[derive(Serialize, InertiaProps)]
pub struct ThemeConfig {
    pub primary_color: Option<String>,
    pub font_family: Option<String>,
}
```

This will include ThemeConfig in the generated types.

> **Note:** The generator only scans `src/` directory for InertiaProps. Types in libraries or other locations need manual definitions.

### Generated Type Utilities

The generated `props.d.ts` includes utility types:

```typescript
// Arbitrary JSON values
export type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };

// Validation errors
export type ValidationErrors = Record<string, string[]>;
```

Use these in your components:

```tsx
import { JsonValue, ValidationErrors } from '@/types/props';

interface FormProps {
  errors: ValidationErrors | null;
  metadata: JsonValue;
}
```

## Development vs Production

### Development Mode

In development, Ferro serves the Vite dev server with HMR:

```rust
let config = InertiaConfig {
    development: true,
    vite_dev_server: "http://localhost:5173".to_string(),
    // ...
};
```

The rendered HTML includes:

```html
<script type="module" src="http://localhost:5173/@vite/client"></script>
<script type="module" src="http://localhost:5173/src/main.tsx"></script>
```

### Production Mode

In production, Ferro uses the built manifest:

```rust
let config = InertiaConfig {
    development: false,
    // ...
};
```

The rendered HTML includes hashed assets:

```html
<script type="module" src="/build/assets/main-abc123.js"></script>
<link rel="stylesheet" href="/build/assets/main-def456.css">
```

## JSON API Fallback

For testing or API clients that need raw JSON data from Inertia routes, enable JSON fallback:

```rust
use ferro::{handler, Request, Response};
use ferro::inertia::Inertia;

#[handler]
pub async fn show(req: Request, post: Post) -> Response {
    Inertia::render_with_json_fallback(&req, "Posts/Show", ShowProps { post })
}
```

When enabled:
- Requests with `X-Inertia: true` header → Normal Inertia JSON response
- Requests with `Accept: application/json` (no X-Inertia) → Raw props as JSON
- Browser requests → Full HTML page

This is useful for:
- API testing with curl or Postman
- Hybrid apps that sometimes need raw JSON
- Debug tooling

Example with curl:

```bash
# Get raw JSON props
curl -H "Accept: application/json" http://localhost:3000/posts/1

# Get normal Inertia response
curl -H "X-Inertia: true" http://localhost:3000/posts/1

# Get HTML page
curl http://localhost:3000/posts/1
```

> **Note:** This is opt-in per route. Consider security implications before enabling on routes that return sensitive data.

## Example: Complete CRUD

### Routes

```rust
use ferro::{get, post, put, delete};

pub fn routes() -> Vec<Route> {
    vec![
        get!("/posts", controllers::posts::index),
        get!("/posts/create", controllers::posts::create),
        post!("/posts", controllers::posts::store),
        get!("/posts/{post}", controllers::posts::show),
        get!("/posts/{post}/edit", controllers::posts::edit),
        put!("/posts/{post}", controllers::posts::update),
        delete!("/posts/{post}", controllers::posts::destroy),
    ]
}
```

### Controller

```rust
use ferro::{handler, Request, Response, redirect};
use ferro::inertia::{Inertia, SavedInertiaContext};
use ferro::InertiaProps;

#[derive(InertiaProps)]
pub struct IndexProps {
    pub posts: Vec<Post>,
}

#[derive(InertiaProps)]
pub struct ShowProps {
    pub post: Post,
}

#[derive(InertiaProps)]
pub struct FormProps {
    pub post: Option<Post>,
    pub errors: Option<serde_json::Value>,
}

#[handler]
pub async fn index(req: Request) -> Response {
    let posts = Post::all().await?;
    Inertia::render(&req, "Posts/Index", IndexProps { posts })
}

#[handler]
pub async fn create(req: Request) -> Response {
    Inertia::render(&req, "Posts/Create", FormProps {
        post: None,
        errors: None,
    })
}

#[handler]
pub async fn store(req: Request) -> Response {
    let ctx = SavedInertiaContext::from_request(&req);
    let data: CreatePostInput = req.json().await?;

    match Post::create(&data).await {
        Ok(post) => redirect!(format!("/posts/{}", post.id)),
        Err(errors) => Inertia::render_ctx(&ctx, "Posts/Create", FormProps {
            post: None,
            errors: Some(errors.to_json()),
        }),
    }
}

#[handler]
pub async fn show(post: Post, req: Request) -> Response {
    Inertia::render(&req, "Posts/Show", ShowProps { post })
}

#[handler]
pub async fn edit(post: Post, req: Request) -> Response {
    Inertia::render(&req, "Posts/Edit", FormProps {
        post: Some(post),
        errors: None,
    })
}

#[handler]
pub async fn update(post: Post, req: Request) -> Response {
    let ctx = SavedInertiaContext::from_request(&req);
    let data: UpdatePostInput = req.json().await?;

    match post.update(&data).await {
        Ok(post) => redirect!(format!("/posts/{}", post.id)),
        Err(errors) => Inertia::render_ctx(&ctx, "Posts/Edit", FormProps {
            post: Some(post),
            errors: Some(errors.to_json()),
        }),
    }
}

#[handler]
pub async fn destroy(post: Post, _req: Request) -> Response {
    post.delete().await?;
    redirect!("/posts")
}
```

## Redirects

For form submissions (POST, PUT, PATCH, DELETE) that should redirect after success, use `Inertia::redirect()`:

```rust
use ferro::{Inertia, Request, Response, Auth};

pub async fn login(req: Request) -> Response {
    // ... validation and auth logic ...

    Auth::login(user.id);
    Inertia::redirect(&req, "/dashboard")
}

pub async fn logout(req: Request) -> Response {
    Auth::logout();
    Inertia::redirect(&req, "/")
}
```

### Why Not `redirect!()`?

The `redirect!()` macro doesn't have access to the request context, so it can't detect Inertia XHR requests. For non-Inertia routes (API endpoints, traditional forms), `redirect!()` works fine.

For Inertia pages, always use `Inertia::redirect()` which:
- Detects Inertia XHR requests via the `X-Inertia` header
- Uses 303 status for POST/PUT/PATCH/DELETE (forces GET on redirect)
- Includes proper `X-Inertia: true` response header

### With Saved Context

If you've consumed the request with `req.input()`, use the saved context:

```rust
use ferro::{Inertia, Request, Response, SavedInertiaContext};

pub async fn store(req: Request) -> Response {
    let ctx = SavedInertiaContext::from(&req);
    let form: CreateForm = req.input().await?;

    // ... create record ...

    Inertia::redirect_ctx(&ctx, "/items")
}
```

## Best Practices

1. **Use InertiaProps derive** - Automatic camelCase conversion matches JavaScript conventions
2. **Save context before consuming request** - Use `SavedInertiaContext` for validation flows
3. **Share common data via middleware** - Auth, flash, CSRF in `ShareInertiaData`
4. **Organize pages in folders** - `Posts/Index.tsx`, `Posts/Show.tsx` for clarity
5. **Use compile-time validation** - `inertia_response!` macro catches typos early
6. **Handle version conflicts** - Ensure smooth deployments with version checking
7. **Keep props minimal** - Only send what the page needs
8. **Use partial reloads** - Optimize updates by requesting only changed data
9. **Use `Inertia::redirect()` for form success** - Ensures proper 303 status for Inertia XHR requests

## Troubleshooting

### Request Body Already Consumed

**Symptom:** Error when calling `Inertia::render()` after `req.json()` or `req.input()`.

**Cause:** The request body was consumed before rendering. In Rust, request bodies can only be read once.

**Solution:** Use `SavedInertiaContext` to capture request metadata before consuming the body:

```rust
let ctx = SavedInertiaContext::from_request(&req);  // Save first
let data = req.json().await?;                        // Then consume
Inertia::render_ctx(&ctx, "Component", props)        // Use saved context
```

### Validation Errors Not Displaying

**Symptom:** Form validation errors are lost after redirect.

**Cause:** Using `redirect!()` after validation failure instead of re-rendering with errors.

**Solution:** On validation failure, render the form again with errors. On success, redirect:

```rust
if errors.fails() {
    // Re-render form with errors (don't redirect)
    return Inertia::render_ctx(&ctx, "Form", FormProps {
        errors: Some(errors.to_json()),
        old: Some(data),
    });
}

// Only redirect on success
Inertia::redirect_ctx(&ctx, "/success")
```

### Props Not Updating After Navigation

**Symptom:** Page shows stale data after Inertia navigation.

**Cause:** Browser caching or partial reload configuration issue.

**Solution:** Check that your handler returns fresh data and consider using `router.reload()` on the frontend to force a refresh:

```tsx
import { router } from '@inertiajs/react';

// Force reload current page data
router.reload();

// Reload only specific props
router.reload({ only: ['items'] });
```

## MCP Tools

Use these tools to inspect Inertia props structs and manage TypeScript type generation without running the CLI.

### `list_props`

Returns all structs with `#[derive(InertiaProps)]` found in `src/`, including field names, Rust types, their TypeScript equivalents, and which Inertia components use each props struct. Use this to audit the props surface before adding new fields or debugging type mismatches.

### `inspect_props`

Returns a detailed breakdown of a single props struct: source code, TypeScript interface preview, which handlers pass it to `Inertia::render()`, and a validation report comparing the Rust definition to any existing TypeScript interface in `frontend/src/types/`. Use this to catch mismatches between Rust and TypeScript before they cause runtime errors.

### `generate_types`

Generates or regenerates `frontend/src/types/inertia-props.ts` from all `InertiaProps` structs found in the project. Supports a `dry_run` parameter to preview changes. Equivalent to `ferro generate-types` but runs as an MCP tool without requiring the CLI or a running server.
