//! List commands tool - return available CLI commands

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommandsInfo {
    pub commands: Vec<CommandInfo>,
}

#[derive(Debug, Serialize)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
}

pub fn execute() -> CommandsInfo {
    // Static list of available commands
    let commands = vec![
        CommandInfo {
            name: "new".to_string(),
            description: "Create a new Ferro project".to_string(),
        },
        CommandInfo {
            name: "serve".to_string(),
            description: "Start the development servers (backend + frontend)".to_string(),
        },
        CommandInfo {
            name: "generate-types".to_string(),
            description: "Generate TypeScript types from Rust InertiaProps structs".to_string(),
        },
        CommandInfo {
            name: "make:controller".to_string(),
            description: "Generate a new controller".to_string(),
        },
        CommandInfo {
            name: "make:action".to_string(),
            description: "Generate a new action".to_string(),
        },
        CommandInfo {
            name: "make:middleware".to_string(),
            description: "Generate a new middleware".to_string(),
        },
        CommandInfo {
            name: "make:event".to_string(),
            description: "Generate a new domain event".to_string(),
        },
        CommandInfo {
            name: "make:factory".to_string(),
            description: "Generate a new test factory".to_string(),
        },
        CommandInfo {
            name: "make:listener".to_string(),
            description: "Generate a new event listener".to_string(),
        },
        CommandInfo {
            name: "make:job".to_string(),
            description: "Generate a new background job".to_string(),
        },
        CommandInfo {
            name: "make:notification".to_string(),
            description: "Generate a new notification".to_string(),
        },
        CommandInfo {
            name: "make:migration".to_string(),
            description: "Generate a new database migration".to_string(),
        },
        CommandInfo {
            name: "make:policy".to_string(),
            description: "Generate a new authorization policy".to_string(),
        },
        CommandInfo {
            name: "make:task".to_string(),
            description: "Generate a new scheduled task".to_string(),
        },
        CommandInfo {
            name: "make:error".to_string(),
            description: "Generate a new domain error".to_string(),
        },
        CommandInfo {
            name: "make:inertia".to_string(),
            description: "Generate a new Inertia page".to_string(),
        },
        CommandInfo {
            name: "make:seeder".to_string(),
            description: "Generate a new database seeder".to_string(),
        },
        CommandInfo {
            name: "make:scaffold".to_string(),
            description:
                "Generate a complete scaffold (model, migration, controller, Inertia pages)"
                    .to_string(),
        },
        CommandInfo {
            name: "db:migrate".to_string(),
            description: "Run all pending database migrations".to_string(),
        },
        CommandInfo {
            name: "db:rollback".to_string(),
            description: "Rollback the last database migration(s)".to_string(),
        },
        CommandInfo {
            name: "db:status".to_string(),
            description: "Show the status of all migrations".to_string(),
        },
        CommandInfo {
            name: "db:fresh".to_string(),
            description: "Drop all tables and re-run all migrations".to_string(),
        },
        CommandInfo {
            name: "db:sync".to_string(),
            description: "Sync database schema to entity files".to_string(),
        },
        CommandInfo {
            name: "db:query".to_string(),
            description: "Execute a raw SQL query against the database".to_string(),
        },
        CommandInfo {
            name: "schedule:run".to_string(),
            description: "Run all due scheduled tasks once".to_string(),
        },
        CommandInfo {
            name: "schedule:work".to_string(),
            description: "Start the scheduler daemon".to_string(),
        },
        CommandInfo {
            name: "schedule:list".to_string(),
            description: "List all registered scheduled tasks".to_string(),
        },
        CommandInfo {
            name: "storage:link".to_string(),
            description: "Create a symbolic link from public/storage to storage/app/public"
                .to_string(),
        },
        CommandInfo {
            name: "docker:init".to_string(),
            description: "Generate a production-ready Dockerfile".to_string(),
        },
        CommandInfo {
            name: "docker:compose".to_string(),
            description: "Generate docker-compose.yml for local development".to_string(),
        },
        CommandInfo {
            name: "mcp".to_string(),
            description: "Start the MCP server for AI assistant integration".to_string(),
        },
        CommandInfo {
            name: "boost:install".to_string(),
            description: "Install AI development boost features".to_string(),
        },
    ];

    CommandsInfo { commands }
}
