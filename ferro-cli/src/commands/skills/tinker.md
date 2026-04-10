---
name: ferro:tinker
description: Interactive database REPL
allowed-tools:
  - Bash
  - Read
---

<objective>
Start an interactive REPL for exploring the database and testing queries.

Uses ferro-mcp `tinker` tool for execution, with fallback to direct database access.
</objective>

<arguments>
Optional:
- `[query]` - Execute single query and exit
- `--sql` - Raw SQL mode (default is model-based)

Examples:
- `/ferro:tinker` - Start interactive session
- `/ferro:tinker "User::all()"` - Execute single query
- `/ferro:tinker --sql "SELECT * FROM users"` - Raw SQL
</arguments>

<process>

<step name="check_connection">

Verify database connection:

```bash
# Check if DATABASE_URL is set
if grep -q "DATABASE_URL" .env 2>/dev/null; then
    echo "Database configured"
else
    echo "ERROR: DATABASE_URL not configured in .env"
    exit 1
fi
```

</step>

<step name="single_query">

**If query argument provided:**

Execute the query using ferro-mcp `tinker` tool and display results:

```
> User::all()

[
  { "id": 1, "email": "admin@example.com", "name": "Admin" },
  { "id": 2, "email": "user@example.com", "name": "User" }
]

2 rows returned
```

</step>

<step name="interactive_mode">

**Start interactive session:**

```
🔧 Ferro Tinker - Interactive REPL

Database: sqlite:database.sqlite
Models: User, Post, Comment

Type expressions to evaluate. Use .help for commands.

ferro> _
```

**Commands:**
- `.help` - Show help
- `.models` - List available models
- `.schema [table]` - Show table schema
- `.sql` - Switch to raw SQL mode
- `.exit` - Exit tinker

**Example session:**

```
ferro> User::count()
=> 5

ferro> User::find(1)
=> Some(User { id: 1, email: "admin@example.com", name: "Admin", ... })

ferro> User::query().filter(Column::Email.contains("@example")).all()
=> [
     User { id: 1, email: "admin@example.com", ... },
     User { id: 2, email: "user@example.com", ... }
   ]

ferro> Post::query().with(Post::user()).first()
=> Some(Post { id: 1, title: "Hello", user: User { id: 1, name: "Admin" } })

ferro> .sql
Switched to SQL mode

sql> SELECT COUNT(*) FROM users
=> 5

sql> SELECT u.name, COUNT(p.id) as post_count
     FROM users u
     LEFT JOIN posts p ON p.user_id = u.id
     GROUP BY u.id
=> [
     { "name": "Admin", "post_count": 10 },
     { "name": "User", "post_count": 3 }
   ]

sql> .exit
Goodbye!
```

</step>

</process>

<available_operations>

## Model Queries
```
Model::all()                    # Get all records
Model::find(id)                 # Find by ID
Model::first()                  # Get first record
Model::count()                  # Count records
Model::query().filter(...).all() # Filtered query
```

## Mutations
```
Model::create({ ... })          # Create record
record.update({ ... })          # Update record
record.delete()                 # Delete record
```

## Relations
```
Model::query().with(Model::relation()).all()
record.relation().await
```

## Raw SQL (in .sql mode)
```
SELECT * FROM table
INSERT INTO table (col) VALUES (val)
UPDATE table SET col = val WHERE id = 1
DELETE FROM table WHERE id = 1
```

</available_operations>
