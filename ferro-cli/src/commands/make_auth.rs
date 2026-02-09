use chrono::Local;
use console::style;
use std::fs;
use std::path::Path;

use crate::templates;

pub fn run(force: bool) {
    let controllers_dir = Path::new("src/controllers");
    let migrations_dir = find_migrations_dir();

    // Check we're in a Ferro project
    if !controllers_dir.exists() {
        eprintln!(
            "{} Controllers directory not found at src/controllers",
            style("Error:").red().bold()
        );
        eprintln!(
            "{}",
            style("Make sure you're in a Ferro project root directory.").dim()
        );
        std::process::exit(1);
    }

    if migrations_dir.is_none() {
        eprintln!(
            "{} Migrations directory not found",
            style("Error:").red().bold()
        );
        eprintln!(
            "{}",
            style("Expected src/migrations or src/database/migrations").dim()
        );
        std::process::exit(1);
    }

    let migrations_dir = migrations_dir.unwrap();
    println!(
        "Scaffolding authentication system...\n"
    );

    // 1. Generate migration
    let migration_created = generate_migration(&migrations_dir, force);

    // 2. Generate auth controller
    let controller_created = generate_auth_controller(controllers_dir, force);

    // 3. Update controllers/mod.rs
    let mod_updated = update_controllers_mod(controllers_dir);

    // Print summary
    println!();
    if migration_created {
        println!(
            "{} Created migration in {}",
            style("Created:").green().bold(),
            migrations_dir.display()
        );
    }
    if controller_created {
        println!(
            "{} src/controllers/auth_controller.rs",
            style("Created:").green().bold()
        );
    }
    if mod_updated {
        println!(
            "{} src/controllers/mod.rs",
            style("Updated:").green().bold()
        );
    }

    // 4. Print next steps
    print_next_steps();
}

fn find_migrations_dir() -> Option<&'static Path> {
    if Path::new("src/migrations").exists() {
        Some(Path::new("src/migrations"))
    } else if Path::new("src/database/migrations").exists() {
        Some(Path::new("src/database/migrations"))
    } else {
        None
    }
}

fn generate_migration(migrations_dir: &Path, force: bool) -> bool {
    // Check if an auth migration already exists
    if !force {
        if let Ok(entries) = fs::read_dir(migrations_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains("add_auth_fields") || name.contains("auth_fields") {
                    println!(
                        "{} Auth migration already exists: {}",
                        style("Skip:").yellow().bold(),
                        name
                    );
                    return false;
                }
            }
        }
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let migration_name = format!("m{}_add_auth_fields_to_users", timestamp);
    let file_path = migrations_dir.join(format!("{}.rs", migration_name));

    let content = templates::auth_migration_template();

    if let Err(e) = fs::write(&file_path, content) {
        eprintln!(
            "{} Failed to write migration: {}",
            style("Error:").red().bold(),
            e
        );
        return false;
    }

    println!("{} {}", style("✓").green(), file_path.display());

    // Register migration in mod.rs
    register_migration(migrations_dir, &migration_name);

    true
}

fn register_migration(migrations_dir: &Path, migration_name: &str) {
    let mod_path = migrations_dir.join("mod.rs");

    if !mod_path.exists() {
        eprintln!(
            "{} migrations/mod.rs not found, skipping registration",
            style("Warning:").yellow().bold()
        );
        return;
    }

    let content = match fs::read_to_string(&mod_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "{} Failed to read mod.rs: {}",
                style("Warning:").yellow().bold(),
                e
            );
            return;
        }
    };

    let mod_decl = format!("mod {};", migration_name);
    if content.contains(&mod_decl) {
        return;
    }

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Find position to insert mod declaration (after other mod declarations)
    let mut last_mod_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("mod ") && !line.contains("mod tests") {
            last_mod_idx = Some(i);
        }
        if line.trim().starts_with("pub mod m") {
            last_mod_idx = Some(i);
        }
    }

    let insert_idx = match last_mod_idx {
        Some(idx) => idx + 1,
        None => {
            let mut idx = 0;
            for (i, line) in lines.iter().enumerate() {
                if line.contains("sea_orm_migration") || line.is_empty() {
                    idx = i + 1;
                } else if line.starts_with("mod ") || line.starts_with("pub struct") {
                    break;
                }
            }
            idx
        }
    };
    lines.insert(insert_idx, mod_decl);

    // Add to migrations() vec
    let box_new_line = format!("            Box::new({}::Migration),", migration_name);
    let mut insert_vec_idx = None;

    for (i, line) in lines.iter().enumerate() {
        if line.contains("vec![]") {
            lines[i] = line.replace("vec![]", &format!("vec![\n{}\n        ]", box_new_line));
            if let Err(e) = fs::write(&mod_path, lines.join("\n")) {
                eprintln!(
                    "{} Failed to update mod.rs: {}",
                    style("Warning:").yellow().bold(),
                    e
                );
            }
            return;
        }
        if line.contains("vec![") && !line.contains("vec![]") {
            for (j, inner_line) in lines.iter().enumerate().skip(i + 1) {
                if inner_line.trim() == "]" || inner_line.trim().starts_with(']') {
                    insert_vec_idx = Some(j);
                    break;
                }
            }
            break;
        }
    }

    if let Some(idx) = insert_vec_idx {
        lines.insert(idx, box_new_line);
    }

    if let Err(e) = fs::write(&mod_path, lines.join("\n")) {
        eprintln!(
            "{} Failed to update mod.rs: {}",
            style("Warning:").yellow().bold(),
            e
        );
    }
}

fn generate_auth_controller(controllers_dir: &Path, force: bool) -> bool {
    let file_path = controllers_dir.join("auth_controller.rs");

    if file_path.exists() && !force {
        println!(
            "{} Auth controller already exists at {}",
            style("Skip:").yellow().bold(),
            file_path.display()
        );
        return false;
    }

    let content = templates::auth_controller_template();

    if let Err(e) = fs::write(&file_path, content) {
        eprintln!(
            "{} Failed to write auth controller: {}",
            style("Error:").red().bold(),
            e
        );
        return false;
    }

    println!(
        "{} src/controllers/auth_controller.rs",
        style("✓").green()
    );
    true
}

fn update_controllers_mod(controllers_dir: &Path) -> bool {
    let mod_path = controllers_dir.join("mod.rs");

    if !mod_path.exists() {
        let content = "pub mod auth_controller;\n";
        if let Err(e) = fs::write(&mod_path, content) {
            eprintln!(
                "{} Failed to create mod.rs: {}",
                style("Error:").red().bold(),
                e
            );
            return false;
        }
        return true;
    }

    let content = match fs::read_to_string(&mod_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "{} Failed to read mod.rs: {}",
                style("Error:").red().bold(),
                e
            );
            return false;
        }
    };

    let mod_decl = "pub mod auth_controller;";
    if content.contains(mod_decl) {
        return false;
    }

    // Find position to insert (after other pub mod declarations)
    let mut lines: Vec<&str> = content.lines().collect();
    let mut last_pub_mod_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("pub mod ") {
            last_pub_mod_idx = Some(i);
        }
    }

    let insert_idx = match last_pub_mod_idx {
        Some(idx) => idx + 1,
        None => 0,
    };
    lines.insert(insert_idx, mod_decl);

    let new_content = lines.join("\n");
    if let Err(e) = fs::write(&mod_path, new_content) {
        eprintln!(
            "{} Failed to update mod.rs: {}",
            style("Error:").red().bold(),
            e
        );
        return false;
    }

    true
}

fn print_next_steps() {
    println!("\n{}", style("Next steps:").bold());
    println!(
        "\n  {} Update your auth provider (src/providers/auth_provider.rs):",
        style("1.").dim()
    );
    println!();
    println!("{}", style("     use ferro::{{Auth, Authenticatable, UserProvider, FrameworkError, verify}};").cyan());
    println!("{}", style("     use crate::models::users::{{self, Entity, Column, Model as User}};").cyan());
    println!("{}", style("     use sea_orm::prelude::*;").cyan());
    println!("{}", style("     use std::sync::Arc;").cyan());
    println!();
    println!("{}", style("     pub struct DatabaseUserProvider;").cyan());
    println!();
    println!("{}", style("     #[async_trait::async_trait]").cyan());
    println!("{}", style("     impl UserProvider for DatabaseUserProvider {").cyan());
    println!("{}", style("         async fn retrieve_by_id(&self, id: i64)").cyan());
    println!("{}", style("             -> Result<Option<Arc<dyn Authenticatable>>, FrameworkError> {").cyan());
    println!("{}", style("             let user = User::query()").cyan());
    println!("{}", style("                 .filter(Column::Id.eq(id as i32))").cyan());
    println!("{}", style("                 .first().await?;").cyan());
    println!("{}", style("             Ok(user.map(|u| Arc::new(u) as Arc<dyn Authenticatable>))").cyan());
    println!("{}", style("         }").cyan());
    println!();
    println!("{}", style("         async fn retrieve_by_credentials(&self, credentials: &serde_json::Value)").cyan());
    println!("{}", style("             -> Result<Option<Arc<dyn Authenticatable>>, FrameworkError> {").cyan());
    println!("{}", style("             let email = credentials[\"email\"].as_str().unwrap_or_default();").cyan());
    println!("{}", style("             let user = User::find_by_email(email).await?;").cyan());
    println!("{}", style("             Ok(user.map(|u| Arc::new(u) as Arc<dyn Authenticatable>))").cyan());
    println!("{}", style("         }").cyan());
    println!();
    println!("{}", style("         async fn validate_credentials(&self, user: &dyn Authenticatable,").cyan());
    println!("{}", style("             credentials: &serde_json::Value) -> Result<bool, FrameworkError> {").cyan());
    println!("{}", style("             let password = credentials[\"password\"].as_str().unwrap_or_default();").cyan());
    println!("{}", style("             let user = user.as_any().downcast_ref::<User>()").cyan());
    println!("{}", style("                 .ok_or_else(|| FrameworkError::internal(\"Invalid user type\".into()))?;").cyan());
    println!("{}", style("             verify(password, &user.password)").cyan());
    println!("{}", style("         }").cyan());
    println!("{}", style("     }").cyan());

    println!(
        "\n  {} Add auth routes to src/routes.rs:",
        style("2.").dim()
    );
    println!();
    println!("{}", style("     use crate::controllers::auth_controller;").cyan());
    println!("{}", style("     use ferro::{{AuthMiddleware, GuestMiddleware, group, post}};").cyan());
    println!();
    println!("{}", style("     // Guest-only routes (login/register)").cyan());
    println!("{}", style("     group!(\"/auth\")").cyan());
    println!("{}", style("         .middleware(GuestMiddleware::redirect_to(\"/\"))").cyan());
    println!("{}", style("         .routes([").cyan());
    println!("{}", style("             post!(\"/register\", auth_controller::register),").cyan());
    println!("{}", style("             post!(\"/login\", auth_controller::login),").cyan());
    println!("{}", style("         ])").cyan());
    println!();
    println!("{}", style("     // Authenticated routes").cyan());
    println!("{}", style("     post!(\"/auth/logout\", auth_controller::logout)").cyan());
    println!("{}", style("         .middleware(AuthMiddleware::new())").cyan());

    println!(
        "\n  {} Run the migration:",
        style("3.").dim()
    );
    println!("     {}", style("ferro db:migrate").cyan());
    println!();
}
