use console::style;
use dialoguer::{theme::ColorfulTheme, Input};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::templates;

pub fn run(name: Option<String>, no_interaction: bool, no_git: bool) {
    println!();
    println!("{}", style("Welcome to Ferro!").cyan().bold());
    println!();

    let project_name = get_project_name(name, no_interaction);
    let description = get_description(no_interaction);
    let author = get_author(no_interaction);

    let package_name = to_snake_case(&project_name);

    println!();
    println!(
        "{}",
        style(format!("Creating project '{project_name}'...")).dim()
    );

    if let Err(e) = create_project(&project_name, &package_name, &description, &author, no_git) {
        eprintln!("{} {}", style("Error:").red().bold(), e);
        std::process::exit(1);
    }

    println!("{} Generated project structure", style("✓").green());

    if !no_git {
        println!("{} Initialized git repository", style("✓").green());
    }

    println!("{} Ready to go!", style("✓").green());
    println!();
    println!("Next steps:");
    println!("  {} {}", style("cd").cyan(), project_name);
    println!("  {}", style("ferro serve").cyan());
    println!();
    println!(
        "Backend will be at {}",
        style("http://localhost:8080").underlined()
    );
    println!(
        "Frontend dev server at {}",
        style("http://localhost:5173").underlined()
    );
    println!();
}

fn get_project_name(name: Option<String>, no_interaction: bool) -> String {
    if let Some(n) = name {
        return n;
    }

    if no_interaction {
        return "my-ferro-app".to_string();
    }

    // Refuse to prompt when stdin is not a TTY — dialoguer would panic otherwise.
    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        eprintln!(
            "{} project name required when not running in an interactive terminal.\n  Usage: ferro new <name>",
            style("Error:").red().bold()
        );
        std::process::exit(1);
    }

    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Project name")
        .default("my-ferro-app".to_string())
        .interact_text()
        .unwrap()
}

fn get_description(no_interaction: bool) -> String {
    if no_interaction {
        return "A web application built with Ferro".to_string();
    }

    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Description")
        .default("A web application built with Ferro".to_string())
        .interact_text()
        .unwrap()
}

fn get_author(no_interaction: bool) -> String {
    if no_interaction {
        return String::new();
    }

    let default_author = get_git_author().unwrap_or_default();

    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Author")
        .default(default_author)
        .allow_empty(true)
        .interact_text()
        .unwrap()
}

fn get_git_author() -> Option<String> {
    let name = Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())?;

    let email = Command::new("git")
        .args(["config", "user.email"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())?;

    Some(format!("{name} <{email}>"))
}

fn to_snake_case(s: &str) -> String {
    s.replace('-', "_").to_lowercase()
}

fn to_title_case(s: &str) -> String {
    s.replace(['-', '_'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn create_project(
    project_name: &str,
    package_name: &str,
    description: &str,
    author: &str,
    no_git: bool,
) -> Result<(), String> {
    let project_path = Path::new(project_name);

    if project_path.exists() {
        return Err(format!("Directory '{project_name}' already exists"));
    }

    // Create directory structure
    create_directories(project_path)?;

    // Write backend files
    write_backend_files(project_path, package_name, description, author)?;

    // Write README at project root
    let project_title = to_title_case(project_name);
    write_file(
        project_path,
        "README.md",
        &templates::readme(project_name, &project_title, description),
    )?;

    // Write frontend files
    write_frontend_files(project_path, project_name)?;

    // Initialize git repository
    if !no_git {
        Command::new("git")
            .args(["init"])
            .current_dir(project_path)
            .output()
            .map_err(|e| format!("Failed to initialize git repository: {e}"))?;
    }

    Ok(())
}

fn create_directories(project_path: &Path) -> Result<(), String> {
    let backend_dirs = [
        "src/controllers",
        "src/config",
        "src/middleware",
        "src/actions",
        "src/models",
        "src/migrations",
        "src/events",
        "src/listeners",
        "src/jobs",
        "src/notifications",
        "src/tasks",
        "src/seeders",
        "src/factories",
        "storage/app/public",
        "storage/logs",
        "lang/en",
    ];

    let frontend_dirs = [
        "frontend/src/pages",
        "frontend/src/pages/auth",
        "frontend/src/types",
        "frontend/src/layouts",
        "frontend/src/styles",
        "public/assets",
    ];

    for dir in backend_dirs.iter().chain(frontend_dirs.iter()) {
        fs::create_dir_all(project_path.join(dir))
            .map_err(|e| format!("Failed to create directory {dir}: {e}"))?;
    }

    Ok(())
}

fn write_backend_files(
    project_path: &Path,
    package_name: &str,
    description: &str,
    author: &str,
) -> Result<(), String> {
    // Root files
    write_file(
        project_path,
        "Cargo.toml",
        &templates::cargo_toml(package_name, description, author),
    )?;
    write_file(project_path, ".gitignore", templates::gitignore())?;
    write_file(project_path, ".env", &templates::env(package_name))?;
    write_file(project_path, ".env.example", templates::env_example())?;

    // Main source files
    write_file(
        project_path,
        "src/main.rs",
        &templates::main_rs(package_name),
    )?;
    write_file(project_path, "src/routes.rs", templates::routes_rs())?;
    write_file(project_path, "src/bootstrap.rs", templates::bootstrap())?;
    write_file(project_path, "src/schedule.rs", templates::schedule_rs())?;

    // Controllers
    write_file(
        project_path,
        "src/controllers/mod.rs",
        templates::controllers_mod(),
    )?;
    write_file(
        project_path,
        "src/controllers/home.rs",
        templates::home_controller(),
    )?;
    write_file(
        project_path,
        "src/controllers/auth.rs",
        templates::auth_controller(),
    )?;
    write_file(
        project_path,
        "src/controllers/dashboard.rs",
        templates::dashboard_controller(),
    )?;
    write_file(
        project_path,
        "src/controllers/profile.rs",
        templates::profile_controller(),
    )?;
    write_file(
        project_path,
        "src/controllers/settings.rs",
        templates::settings_controller(),
    )?;

    // Config
    write_file(project_path, "src/config/mod.rs", templates::config_mod())?;
    write_file(
        project_path,
        "src/config/database.rs",
        templates::config_database(),
    )?;
    write_file(project_path, "src/config/mail.rs", templates::config_mail())?;

    // Middleware
    write_file(
        project_path,
        "src/middleware/mod.rs",
        templates::middleware_mod(),
    )?;
    write_file(
        project_path,
        "src/middleware/logging.rs",
        templates::middleware_logging(),
    )?;
    write_file(
        project_path,
        "src/middleware/authenticate.rs",
        templates::authenticate_middleware(),
    )?;

    // Actions
    write_file(project_path, "src/actions/mod.rs", templates::actions_mod())?;
    write_file(
        project_path,
        "src/actions/example_action.rs",
        templates::example_action(),
    )?;

    // Models
    write_file(project_path, "src/models/mod.rs", templates::models_mod())?;
    write_file(project_path, "src/models/user.rs", templates::user_model())?;
    write_file(
        project_path,
        "src/models/password_reset_tokens.rs",
        templates::password_reset_tokens_model(),
    )?;

    // Migrations
    write_file(
        project_path,
        "src/migrations/mod.rs",
        templates::migrations_mod(),
    )?;
    write_file(
        project_path,
        "src/migrations/m20240101_000001_create_users_table.rs",
        templates::create_users_migration(),
    )?;
    write_file(
        project_path,
        "src/migrations/m20240101_000002_create_sessions_table.rs",
        templates::create_sessions_migration(),
    )?;
    write_file(
        project_path,
        "src/migrations/m20240101_000003_create_password_reset_tokens_table.rs",
        templates::create_password_reset_tokens_migration(),
    )?;

    // Events, Listeners, Jobs, Notifications, Tasks
    write_file(project_path, "src/events/mod.rs", templates::events_mod())?;
    write_file(
        project_path,
        "src/listeners/mod.rs",
        templates::listeners_mod(),
    )?;
    write_file(project_path, "src/jobs/mod.rs", templates::jobs_mod())?;
    write_file(
        project_path,
        "src/notifications/mod.rs",
        templates::notifications_mod(),
    )?;
    write_file(project_path, "src/tasks/mod.rs", templates::tasks_mod())?;
    write_file(project_path, "src/seeders/mod.rs", templates::seeders_mod())?;
    write_file(
        project_path,
        "src/factories/mod.rs",
        templates::factories_mod(),
    )?;

    // Storage gitkeep files
    write_file(project_path, "storage/app/.gitkeep", "")?;
    write_file(project_path, "storage/logs/.gitkeep", "")?;

    // Language files
    write_file(
        project_path,
        "lang/en/validation.json",
        templates::lang_validation_json(),
    )?;
    write_file(project_path, "lang/en/app.json", templates::lang_app_json())?;

    Ok(())
}

fn write_frontend_files(project_path: &Path, project_name: &str) -> Result<(), String> {
    let title = to_title_case(project_name);

    // Root frontend files
    write_file(
        project_path,
        "frontend/package.json",
        &templates::package_json(project_name),
    )?;
    write_file(
        project_path,
        "frontend/vite.config.ts",
        templates::vite_config(),
    )?;
    write_file(
        project_path,
        "frontend/tsconfig.json",
        templates::tsconfig(),
    )?;
    write_file(
        project_path,
        "frontend/index.html",
        &templates::index_html(&title),
    )?;

    // Frontend source files
    write_file(project_path, "frontend/src/main.tsx", templates::main_tsx())?;
    write_file(
        project_path,
        "frontend/src/types/inertia-props.ts",
        templates::inertia_props_types(),
    )?;
    write_file(
        project_path,
        "frontend/src/styles/globals.css",
        templates::globals_css(),
    )?;

    // Layouts
    write_file(
        project_path,
        "frontend/src/layouts/AppLayout.tsx",
        templates::app_layout(),
    )?;
    write_file(
        project_path,
        "frontend/src/layouts/AuthLayout.tsx",
        templates::auth_layout(),
    )?;
    write_file(
        project_path,
        "frontend/src/layouts/index.ts",
        templates::layouts_index(),
    )?;

    // Pages
    write_file(
        project_path,
        "frontend/src/pages/Home.tsx",
        templates::home_page(),
    )?;
    write_file(
        project_path,
        "frontend/src/pages/Dashboard.tsx",
        templates::dashboard_page(),
    )?;
    write_file(
        project_path,
        "frontend/src/pages/Profile.tsx",
        templates::profile_page(),
    )?;
    write_file(
        project_path,
        "frontend/src/pages/Settings.tsx",
        templates::settings_page(),
    )?;

    // Auth pages
    write_file(
        project_path,
        "frontend/src/pages/auth/Login.tsx",
        templates::login_page(),
    )?;
    write_file(
        project_path,
        "frontend/src/pages/auth/Register.tsx",
        templates::register_page(),
    )?;
    write_file(
        project_path,
        "frontend/src/pages/auth/ForgotPassword.tsx",
        templates::forgot_password_page(),
    )?;
    write_file(
        project_path,
        "frontend/src/pages/auth/ResetPassword.tsx",
        templates::reset_password_page(),
    )?;

    Ok(())
}

fn write_file(project_path: &Path, relative_path: &str, content: &str) -> Result<(), String> {
    let full_path = project_path.join(relative_path);
    fs::write(&full_path, content).map_err(|e| format!("Failed to write {relative_path}: {e}"))
}
