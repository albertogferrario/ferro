use console::style;
use std::path::Path;
use std::process::Command;

/// Invoke the unified `ferro` binary's `json-ui:schema` subcommand from the
/// workspace root via `cargo run --quiet`.
///
/// Mirrors the `db_status.rs` shell-out pattern. Requires a `Cargo.toml` in
/// the current directory to confirm the caller is inside a Ferro project.
pub fn run(output: Option<String>, pretty: bool, component: Option<String>) {
    if !Path::new("Cargo.toml").exists() {
        eprintln!(
            "{} This command must be run from a Ferro project root (no Cargo.toml found)",
            style("Error:").red().bold()
        );
        std::process::exit(1);
    }

    let mut args: Vec<String> = vec![
        "run".into(),
        "--quiet".into(),
        "--".into(),
        "json-ui:schema".into(),
    ];
    if let Some(o) = output.as_deref() {
        args.push("--output".into());
        args.push(o.to_string());
    }
    if pretty {
        args.push("--pretty".into());
    }
    if let Some(c) = component.as_deref() {
        args.push("--component".into());
        args.push(c.to_string());
    }

    let status = Command::new("cargo")
        .args(&args)
        .status()
        .expect("Failed to execute cargo command");

    if !status.success() {
        eprintln!(
            "{} Schema export failed",
            style("Error:").red().bold()
        );
        std::process::exit(1);
    }
}
