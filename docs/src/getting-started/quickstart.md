# Quick Start

Build a simple user listing feature in 5 minutes.

## 1. Create Migration

```bash
ferro make:migration create_users_table
```

Edit `src/migrations/m_YYYYMMDD_create_users_table.rs`:

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .col(ColumnDef::new(Users::Id).big_integer().primary_key().auto_increment())
                    .col(ColumnDef::new(Users::Name).string().not_null())
                    .col(ColumnDef::new(Users::Email).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::CreatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Users::Table).to_owned()).await
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Name,
    Email,
    CreatedAt,
}
```

Run the migration:

```bash
ferro db:migrate
```

## 2. Sync Database to Models

```bash
ferro db:sync
```

This generates `src/models/users.rs` with SeaORM entity definitions.

## 3. Create Controller

```bash
ferro make:controller users
```

Edit `src/controllers/users_controller.rs`:

```rust
use ferro::*;
use crate::models::users::Entity as User;

#[handler]
pub async fn index(req: Request) -> Response {
    let db = req.db();
    let users = User::find().all(db).await?;

    Inertia::render(&req, "Users/Index", UsersIndexProps { users })
}

#[derive(InertiaProps)]
pub struct UsersIndexProps {
    pub users: Vec<crate::models::users::Model>,
}
```

## 4. Create Inertia Page

```bash
ferro make:inertia Users/Index
```

Edit `frontend/src/pages/Users/Index.tsx`:

```tsx
import { InertiaProps } from '@/types/inertia-props';

interface User {
  id: number;
  name: string;
  email: string;
}

interface Props {
  users: User[];
}

export default function UsersIndex({ users }: Props) {
  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-4">Users</h1>
      <ul className="space-y-2">
        {users.map((user) => (
          <li key={user.id} className="p-2 border rounded">
            <strong>{user.name}</strong> - {user.email}
          </li>
        ))}
      </ul>
    </div>
  );
}
```

## 5. Add Route

Edit `src/routes.rs`:

```rust
use ferro::*;
use crate::controllers::users_controller;

pub fn routes() -> Router {
    Router::new()
        .get("/users", users_controller::index)
}
```

## 6. Generate TypeScript Types

```bash
ferro generate-types
```

This creates `frontend/src/types/inertia-props.ts` from your Rust props.

## 7. Run the Server

```bash
ferro serve
```

Visit `http://localhost:5173/users` to see your user listing.

## What's Next?

- Add [validation](../features/validation.md) to user creation
- Implement [authentication](../features/authentication.md)
- Add [middleware](../the-basics/middleware.md) for protected routes
