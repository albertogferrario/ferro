//! `ferro deploy:init` — interactive scaffolder for
//! `[package.metadata.ferro.deploy]` (Phase 128 D-07..D-11).

use crate::commands::docker_init::{print_dry_run, RenderedFile};
use crate::deploy::bin_detect::detect_web_bin;
use crate::project::find_project_root;
use anyhow::{anyhow, Context};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use toml_edit::{Array, DocumentMut, Item, Table, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnExists {
    Abort,
    Overwrite,
    Merge,
}

#[derive(Debug, Clone)]
pub struct DeployInitOpts {
    pub yes: bool,
    pub dry_run: bool,
    pub on_exists_override: Option<OnExists>,
}

#[derive(Debug, Clone)]
pub struct DeployDefaults {
    pub web_bin: String,
    pub copy_dirs: Vec<String>,
    pub runtime_apt: Vec<String>,
}

pub fn run(yes: bool, dry_run: bool) {
    run_with(DeployInitOpts {
        yes,
        dry_run,
        on_exists_override: None,
    });
}

pub fn run_with(opts: DeployInitOpts) {
    if let Err(e) = execute(opts) {
        eprintln!("{} {e}", style("Error:").red().bold());
        std::process::exit(1);
    }
}

pub fn execute(opts: DeployInitOpts) -> anyhow::Result<()> {
    let root = find_project_root(None)
        .map_err(|_| anyhow!("Cargo.toml not found (searched upward from CWD)"))?;

    let is_tty = std::io::stdin().is_terminal();
    if !is_tty && !opts.yes {
        return Err(anyhow!("ferro deploy:init requires a TTY or --yes"));
    }

    // --- compute defaults ---
    let web_bin = detect_web_bin(&root)
        .context("failed to auto-detect web binary — declare [[bin]] in Cargo.toml or pass --yes with a valid project")?;
    let copy_dirs_candidates = ["migrations", "static"];
    let copy_dirs: Vec<String> = copy_dirs_candidates
        .iter()
        .filter(|d| root.join(d).is_dir())
        .map(|s| s.to_string())
        .collect();
    let mut defaults = DeployDefaults {
        web_bin: web_bin.clone(),
        copy_dirs,
        runtime_apt: Vec::new(),
    };

    // --- interactive refinement ---
    if !opts.yes {
        let theme = ColorfulTheme::default();
        defaults.web_bin = Input::<String>::with_theme(&theme)
            .with_prompt("Web binary name")
            .default(defaults.web_bin.clone())
            .interact_text()?;
        let copy_dirs_str: String = Input::with_theme(&theme)
            .with_prompt("copy_dirs (comma-separated, blank for none)")
            .default(defaults.copy_dirs.join(","))
            .allow_empty(true)
            .interact_text()?;
        defaults.copy_dirs = copy_dirs_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let runtime_apt_str: String = Input::with_theme(&theme)
            .with_prompt("runtime_apt packages (comma-separated, blank for none)")
            .default(String::new())
            .allow_empty(true)
            .interact_text()?;
        defaults.runtime_apt = runtime_apt_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    let block = compute_deploy_toml_block(&defaults);

    // --- dry run ---
    if opts.dry_run {
        let files = [RenderedFile {
            relative_path: PathBuf::from("Cargo.toml ([package.metadata.ferro.deploy])"),
            contents: block,
        }];
        print_dry_run(&files);
        return Ok(());
    }

    // --- resolve on_exists ---
    let cargo_toml = root.join("Cargo.toml");
    let existing = fs::read_to_string(&cargo_toml)?;
    let existing_doc: DocumentMut = existing
        .parse()
        .map_err(|e| anyhow!("failed to parse Cargo.toml: {e}"))?;
    let has_block = existing_doc
        .get("package")
        .and_then(|p| p.as_table_like())
        .and_then(|t| t.get("metadata"))
        .and_then(|m| m.as_table_like())
        .and_then(|t| t.get("ferro"))
        .and_then(|f| f.as_table_like())
        .and_then(|t| t.get("deploy"))
        .is_some();

    let on_exists = if has_block {
        match opts.on_exists_override {
            Some(choice) => choice,
            None if opts.yes => OnExists::Abort,
            None => prompt_on_exists()?,
        }
    } else {
        OnExists::Overwrite // no collision — straight insert
    };

    persist_deploy_block(&cargo_toml, &defaults, on_exists)?;
    println!("{} Updated {}", style("✓").green(), cargo_toml.display());
    print!("{}", deploy_init_footer());
    Ok(())
}

fn prompt_on_exists() -> anyhow::Result<OnExists> {
    let items = &["abort", "overwrite", "merge"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("[package.metadata.ferro.deploy] already exists")
        .default(0)
        .items(items)
        .interact()?;
    Ok(match selection {
        0 => OnExists::Abort,
        1 => OnExists::Overwrite,
        _ => OnExists::Merge,
    })
}

/// Pure compute: format the deploy block as a standalone TOML snippet.
pub fn compute_deploy_toml_block(d: &DeployDefaults) -> String {
    let mut s = String::new();
    s.push_str("[package.metadata.ferro.deploy]\n");
    s.push_str(&format!(
        "runtime_apt = {}\n",
        format_string_array(&d.runtime_apt)
    ));
    s.push_str(&format!(
        "copy_dirs = {}\n",
        format_string_array(&d.copy_dirs)
    ));
    s.push_str(&format!("web_bin = \"{}\"\n", d.web_bin));
    s
}

fn format_string_array(v: &[String]) -> String {
    if v.is_empty() {
        return "[]".to_string();
    }
    let items: Vec<String> = v.iter().map(|s| format!("\"{s}\"")).collect();
    format!("[{}]", items.join(", "))
}

/// Persist the deploy block into the given Cargo.toml using toml_edit,
/// honoring the `on_exists` policy. Aborts with Err on collision when
/// policy is Abort. Preserves all unrelated keys byte-for-byte.
pub fn persist_deploy_block(
    cargo_toml: &Path,
    d: &DeployDefaults,
    on_exists: OnExists,
) -> anyhow::Result<()> {
    let source = fs::read_to_string(cargo_toml)?;
    let mut doc: DocumentMut = source
        .parse()
        .map_err(|e| anyhow!("failed to parse Cargo.toml: {e}"))?;

    let existed = doc
        .get("package")
        .and_then(|p| p.as_table_like())
        .and_then(|t| t.get("metadata"))
        .and_then(|m| m.as_table_like())
        .and_then(|t| t.get("ferro"))
        .and_then(|f| f.as_table_like())
        .and_then(|t| t.get("deploy"))
        .is_some();

    if existed && on_exists == OnExists::Abort {
        return Err(anyhow!(
            "[package.metadata.ferro.deploy] already exists (use --yes with --overwrite policy or run interactively)"
        ));
    }

    // Ensure parent tables exist (package.metadata.ferro.deploy).
    if doc.get("package").is_none() {
        doc["package"] = Item::Table(Table::new());
    }
    let pkg = doc["package"].as_table_mut().expect("package is a table");
    if pkg.get("metadata").is_none() {
        let mut t = Table::new();
        t.set_implicit(true);
        pkg["metadata"] = Item::Table(t);
    }
    let metadata = pkg["metadata"].as_table_mut().expect("metadata table");
    if metadata.get("ferro").is_none() {
        let mut t = Table::new();
        t.set_implicit(true);
        metadata["ferro"] = Item::Table(t);
    }
    let ferro = metadata["ferro"].as_table_mut().expect("ferro table");

    if !existed || on_exists == OnExists::Overwrite {
        let mut deploy = Table::new();
        deploy["runtime_apt"] = toml_edit::value(string_array(&d.runtime_apt));
        deploy["copy_dirs"] = toml_edit::value(string_array(&d.copy_dirs));
        deploy["web_bin"] = toml_edit::value(d.web_bin.clone());
        ferro["deploy"] = Item::Table(deploy);
    } else {
        // Merge: fill in only missing keys.
        let deploy = ferro["deploy"]
            .as_table_mut()
            .ok_or_else(|| anyhow!("[package.metadata.ferro.deploy] is not a table"))?;
        if deploy.get("runtime_apt").is_none() {
            deploy["runtime_apt"] = toml_edit::value(string_array(&d.runtime_apt));
        }
        if deploy.get("copy_dirs").is_none() {
            deploy["copy_dirs"] = toml_edit::value(string_array(&d.copy_dirs));
        }
        if deploy.get("web_bin").is_none() {
            deploy["web_bin"] = toml_edit::value(d.web_bin.clone());
        }
    }

    fs::write(cargo_toml, doc.to_string())?;
    Ok(())
}

fn string_array(v: &[String]) -> Array {
    let mut arr = Array::new();
    for s in v {
        arr.push(Value::from(s.clone()));
    }
    arr
}

fn deploy_init_footer() -> String {
    "\nNext steps:\n  Review Cargo.toml [package.metadata.ferro.deploy].\n  ferro docker:init\n  ferro doctor --deploy\n".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_cargo(root: &Path, body: &str) -> PathBuf {
        let p = root.join("Cargo.toml");
        fs::write(&p, body).unwrap();
        p
    }

    fn defaults() -> DeployDefaults {
        DeployDefaults {
            web_bin: "myapp".into(),
            copy_dirs: vec!["migrations".into()],
            runtime_apt: vec![],
        }
    }

    #[test]
    fn compute_block_formats_expected() {
        let out = compute_deploy_toml_block(&defaults());
        assert!(out.contains("[package.metadata.ferro.deploy]"));
        assert!(out.contains("runtime_apt = []"));
        assert!(out.contains("copy_dirs = [\"migrations\"]"));
        assert!(out.contains("web_bin = \"myapp\""));
    }

    #[test]
    fn persist_inserts_block_when_absent() {
        let td = TempDir::new().unwrap();
        let p = write_cargo(
            td.path(),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n# keep this comment\n\n[dependencies]\nserde = \"1\"\n",
        );
        persist_deploy_block(&p, &defaults(), OnExists::Overwrite).unwrap();
        let out = fs::read_to_string(&p).unwrap();
        assert!(out.contains("[package.metadata.ferro.deploy]"));
        assert!(out.contains("web_bin = \"myapp\""));
        assert!(
            out.contains("# keep this comment"),
            "existing comments preserved"
        );
        assert!(out.contains("serde = \"1\""), "other deps preserved");
    }

    #[test]
    fn persist_aborts_when_table_exists_and_policy_abort() {
        let td = TempDir::new().unwrap();
        let p = write_cargo(
            td.path(),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n[package.metadata.ferro.deploy]\nweb_bin = \"old\"\n",
        );
        let r = persist_deploy_block(&p, &defaults(), OnExists::Abort);
        assert!(r.is_err());
        let out = fs::read_to_string(&p).unwrap();
        assert!(out.contains("web_bin = \"old\""), "file untouched on abort");
    }

    #[test]
    fn persist_merge_fills_missing_fields_only() {
        let td = TempDir::new().unwrap();
        let p = write_cargo(
            td.path(),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n[package.metadata.ferro.deploy]\nweb_bin = \"userpick\"\n",
        );
        persist_deploy_block(&p, &defaults(), OnExists::Merge).unwrap();
        let out = fs::read_to_string(&p).unwrap();
        assert!(
            out.contains("web_bin = \"userpick\""),
            "existing field preserved"
        );
        assert!(out.contains("runtime_apt = []"));
        assert!(out.contains("copy_dirs"));
    }

    #[test]
    fn persist_overwrite_replaces_fields() {
        let td = TempDir::new().unwrap();
        let p = write_cargo(
            td.path(),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n[package.metadata.ferro.deploy]\nweb_bin = \"old\"\n",
        );
        persist_deploy_block(&p, &defaults(), OnExists::Overwrite).unwrap();
        let out = fs::read_to_string(&p).unwrap();
        assert!(out.contains("web_bin = \"myapp\""));
        assert!(!out.contains("web_bin = \"old\""));
    }

    #[test]
    fn dry_run_writes_zero_files() {
        let td = TempDir::new().unwrap();
        let cargo_body =
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n\n[[bin]]\nname = \"x\"\npath = \"src/main.rs\"\n";
        write_cargo(td.path(), cargo_body);
        fs::create_dir_all(td.path().join("src")).unwrap();
        fs::write(td.path().join("src/main.rs"), "fn main() {}\n").unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(td.path()).unwrap();
        let r = execute(DeployInitOpts {
            yes: true,
            dry_run: true,
            on_exists_override: None,
        });
        std::env::set_current_dir(prev).unwrap();
        assert!(r.is_ok(), "dry-run execute failed: {r:?}");
        let after = fs::read_to_string(td.path().join("Cargo.toml")).unwrap();
        assert_eq!(after, cargo_body, "dry-run must not touch Cargo.toml");
    }

    #[test]
    fn footer_mentions_next_steps() {
        let s = deploy_init_footer();
        assert!(s.contains("ferro docker:init"));
        assert!(s.contains("ferro doctor --deploy"));
    }
}
