//! `ferro make:module <name>` — scaffold a feature-module skeleton under
//! `src/modules/<name>/` following the controller/model/views/routes convention
//! used by gestiscilo and mkmenu. Codified to stop the pattern from drifting.

use console::style;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::templates::module as tpl;

/// Public entry: resolve project root then delegate to `run_in`.
pub fn run(name: String, with_migration: bool, no_views: bool, force: bool) {
    let root = match crate::project::find_project_root(None) {
        Ok(p) => p,
        Err(_) => {
            eprintln!(
                "{} Cargo.toml not found (are you inside a Ferro project?)",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    };

    match run_in(&root, &name, with_migration, no_views, force) {
        Ok(report) => {
            for line in &report.created {
                println!("{} Created {}", style("✓").green(), line.display());
            }
            if let Some(updated) = &report.updated_mod {
                println!("{} Updated {}", style("✓").green(), updated.display());
            }
            println!();
            println!(
                "Module {} created successfully!",
                style(&report.snake_name).cyan().bold()
            );
            println!();
            println!("Usage:");
            println!(
                "  Wire into your router: crate::modules::{}::routes::register(router)",
                report.snake_name
            );
            println!();
        }
        Err(RunError::InvalidName(n)) => {
            eprintln!(
                "{} '{}' is not a valid module name",
                style("Error:").red().bold(),
                n
            );
            std::process::exit(1);
        }
        Err(RunError::Exists(p)) => {
            eprintln!(
                "{} {} already exists (use --force to overwrite)",
                style("Error:").red().bold(),
                p.display()
            );
            std::process::exit(1);
        }
        Err(RunError::Io(e)) => {
            eprintln!("{} {}", style("Error:").red().bold(), e);
            std::process::exit(1);
        }
    }
}

/// Deterministic, testable core: operates against a fixed project root.
pub fn run_in(
    root: &Path,
    name: &str,
    with_migration: bool,
    no_views: bool,
    force: bool,
) -> Result<Report, RunError> {
    let snake = to_snake_case(name);
    if !is_valid_identifier(&snake) {
        return Err(RunError::InvalidName(name.to_string()));
    }

    let modules_dir = root.join("src/modules");
    let module_dir = modules_dir.join(&snake);
    let views_dir = module_dir.join("views");

    // Collect planned (path, content) pairs.
    let mut planned: Vec<(PathBuf, String)> = Vec::new();
    planned.push((
        module_dir.join("controller.rs"),
        tpl::module_controller_rs(&snake),
    ));
    planned.push((module_dir.join("model.rs"), tpl::module_model_rs(&snake)));
    planned.push((module_dir.join("routes.rs"), tpl::module_routes_rs(&snake)));

    if no_views {
        planned.push((
            module_dir.join("mod.rs"),
            tpl::module_mod_rs_headless(&snake),
        ));
    } else {
        planned.push((module_dir.join("mod.rs"), tpl::module_mod_rs(&snake)));
        planned.push((views_dir.join("mod.rs"), tpl::module_views_mod_rs()));
        planned.push((
            views_dir.join("index.rs"),
            tpl::module_view_index_rs(&snake),
        ));
    }

    // Pre-check: abort before writing anything if any target exists without --force.
    if !force {
        for (path, _) in &planned {
            if path.exists() {
                return Err(RunError::Exists(path.clone()));
            }
        }
    }

    // Create directories.
    fs::create_dir_all(&module_dir).map_err(RunError::Io)?;
    if !no_views {
        fs::create_dir_all(&views_dir).map_err(RunError::Io)?;
    }

    // Write files.
    let mut created: Vec<PathBuf> = Vec::new();
    for (path, content) in &planned {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(RunError::Io)?;
        }
        fs::write(path, content).map_err(RunError::Io)?;
        created.push(path.clone());
    }

    // Update or create src/modules/mod.rs.
    let modules_mod = modules_dir.join("mod.rs");
    let updated_mod = update_modules_mod(&modules_mod, &snake)?;

    // Optional migration stub.
    if with_migration {
        let migration_src = root.join("migration/src");
        if migration_src.is_dir() {
            let ts = current_timestamp();
            let file = migration_src.join(format!("m_{ts}_create_{snake}.rs"));
            if !file.exists() || force {
                fs::write(&file, tpl::module_migration_rs(&snake, &ts)).map_err(RunError::Io)?;
                created.push(file);
            }
        }
    }

    Ok(Report {
        snake_name: snake,
        created,
        updated_mod,
    })
}

#[derive(Debug)]
pub struct Report {
    pub snake_name: String,
    pub created: Vec<PathBuf>,
    pub updated_mod: Option<PathBuf>,
}

#[derive(Debug)]
pub enum RunError {
    InvalidName(String),
    Exists(PathBuf),
    Io(io::Error),
}

fn update_modules_mod(mod_file: &Path, snake: &str) -> Result<Option<PathBuf>, RunError> {
    let decl = format!("pub mod {snake};");
    if mod_file.exists() {
        let content = fs::read_to_string(mod_file).map_err(RunError::Io)?;
        if content.contains(&decl) {
            return Ok(None);
        }
        let mut new_content = content;
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str(&decl);
        new_content.push('\n');
        fs::write(mod_file, new_content).map_err(RunError::Io)?;
        Ok(Some(mod_file.to_path_buf()))
    } else {
        if let Some(parent) = mod_file.parent() {
            fs::create_dir_all(parent).map_err(RunError::Io)?;
        }
        fs::write(mod_file, format!("//! Feature modules\n\n{decl}\n")).map_err(RunError::Io)?;
        Ok(Some(mod_file.to_path_buf()))
    }
}

fn current_timestamp() -> String {
    chrono::Utc::now().format("%Y%m%d%H%M%S").to_string()
}

fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else if c == '-' {
            result.push('_');
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_project(with_migration_dir: bool) -> TempDir {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        if with_migration_dir {
            fs::create_dir_all(tmp.path().join("migration/src")).unwrap();
        }
        tmp
    }

    #[test]
    fn creates_default_skeleton() {
        let tmp = setup_project(false);
        let report = run_in(tmp.path(), "orders", false, false, false).unwrap();
        assert_eq!(report.snake_name, "orders");

        let base = tmp.path().join("src/modules/orders");
        for rel in [
            "mod.rs",
            "controller.rs",
            "model.rs",
            "routes.rs",
            "views/mod.rs",
            "views/index.rs",
        ] {
            assert!(base.join(rel).exists(), "missing {rel}");
        }

        let modules_mod = fs::read_to_string(tmp.path().join("src/modules/mod.rs")).unwrap();
        assert!(modules_mod.contains("pub mod orders;"));
    }

    #[test]
    fn no_views_flag_skips_views() {
        let tmp = setup_project(false);
        run_in(tmp.path(), "accounts", false, true, false).unwrap();
        assert!(!tmp.path().join("src/modules/accounts/views").exists());
        let mod_rs = fs::read_to_string(tmp.path().join("src/modules/accounts/mod.rs")).unwrap();
        assert!(!mod_rs.contains("pub mod views;"));
    }

    #[test]
    fn with_migration_flag_writes_migration() {
        let tmp = setup_project(true);
        run_in(tmp.path(), "invoices", true, false, false).unwrap();
        let entries: Vec<_> = fs::read_dir(tmp.path().join("migration/src"))
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert!(
            entries.iter().any(|n| n.contains("create_invoices")),
            "migration file not found in {entries:?}"
        );
    }

    #[test]
    fn force_flag_overwrites() {
        let tmp = setup_project(false);
        run_in(tmp.path(), "orders", false, false, false).unwrap();
        let controller = tmp.path().join("src/modules/orders/controller.rs");
        fs::write(&controller, "// tampered\n").unwrap();

        // Without --force → error.
        let err = run_in(tmp.path(), "orders", false, false, false).unwrap_err();
        assert!(matches!(err, RunError::Exists(_)));

        // With --force → controller is rewritten with the template.
        run_in(tmp.path(), "orders", false, false, true).unwrap();
        let content = fs::read_to_string(&controller).unwrap();
        assert!(content.contains("#[handler]"));
    }

    #[test]
    fn rejects_invalid_name() {
        let tmp = setup_project(false);
        let err = run_in(tmp.path(), "123bad", false, false, false).unwrap_err();
        assert!(matches!(err, RunError::InvalidName(_)));
    }
}
