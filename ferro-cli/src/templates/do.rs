// ============================================================================
// DigitalOcean App Platform Templates
// ============================================================================
//
// Renders `.do/app.yaml` from a typed context. Pure function — no IO.
// See plan 122-05.

use crate::deploy::classify::is_secret;
use crate::deploy::env_example::EnvEntry;
use crate::project::BinEntry;

const APP_YAML_TPL: &str = include_str!("files/do/app.yaml.tpl");

/// Sanitize a Cargo package name into a DO App Platform-compliant app name.
/// Lowercase, `_` → `-`, drop any char not in `[a-z0-9-]`, collapse consecutive `-`.
pub fn sanitize_do_app_name(pkg: &str) -> String {
    let mut out = String::with_capacity(pkg.len());
    let mut prev_dash = false;
    for c in pkg.chars() {
        let c = c.to_ascii_lowercase();
        let mapped = if c == '_' {
            Some('-')
        } else if c.is_ascii_alphanumeric() || c == '-' {
            Some(c)
        } else {
            None
        };
        if let Some(ch) = mapped {
            if ch == '-' {
                if prev_dash {
                    continue;
                }
                prev_dash = true;
            } else {
                prev_dash = false;
            }
            out.push(ch);
        }
    }
    out
}

/// Context driving DO `app.yaml` rendering.
pub struct AppYamlContext<'a> {
    pub package_name: &'a str,
    pub github_repo: &'a str,
    pub region: &'a str,
    pub bins: &'a [BinEntry],
    pub env_entries: &'a [EnvEntry],
}

/// Render an `app.yaml` from a context.
pub fn render_app_yaml(ctx: &AppYamlContext) -> String {
    let envs_block = build_envs_block(ctx.env_entries);
    let has_db = ctx
        .env_entries
        .iter()
        .any(|e| e.key.eq_ignore_ascii_case("DATABASE_URL"));
    let databases_block = if has_db {
        build_databases_block()
    } else {
        String::new()
    };
    let workers_block = build_workers_block(ctx.package_name, ctx.bins);

    APP_YAML_TPL
        .replace("{app_name}", ctx.package_name)
        .replace("{region}", ctx.region)
        .replace("{github_repo}", ctx.github_repo)
        .replace("{envs_block}", &envs_block)
        .replace("{databases_block}", &databases_block)
        .replace("{workers_block}", &workers_block)
}

fn build_envs_block(entries: &[EnvEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }
    let mut out = String::from("    envs:\n");
    for e in entries {
        let is_db = e.key.eq_ignore_ascii_case("DATABASE_URL");
        let placeholder = !is_db && should_placeholder(&e.key, &e.value);
        if placeholder {
            out.push_str(&format!(
                "      # Override in DO App Platform secrets (was: {})\n",
                e.value
            ));
        }
        out.push_str(&format!("      - key: {}\n", e.key));
        out.push_str("        scope: RUN_TIME\n");
        if is_db {
            out.push_str("        value: ${db.DATABASE_URL}\n");
        } else if placeholder {
            out.push_str(&format!("        value: ${{{}}}\n", e.key));
        } else {
            out.push_str(&format!("        value: \"{}\"\n", yaml_escape(&e.value)));
        }
        if is_secret(&e.key) {
            out.push_str("        type: SECRET\n");
        }
    }
    out
}

/// Decide whether an env entry's value should be replaced with a `${KEY}`
/// placeholder in `.do/app.yaml`. Dev defaults (localhost, empty, relative
/// paths, `file:` refs) are unsafe to ship to production.
fn should_placeholder(key: &str, value: &str) -> bool {
    if key.eq_ignore_ascii_case("DATABASE_URL") {
        return false;
    }
    if key.ends_with("_PORT") {
        return false;
    }
    let v = value.trim();
    if v.is_empty() {
        return true;
    }
    if v.contains("localhost") || v.contains("127.0.0.1") || v.contains("0.0.0.0") {
        return true;
    }
    if v.starts_with("file:") {
        return true;
    }
    if v.starts_with("./") || v.starts_with("../") {
        return true;
    }
    if !v.contains("://") && !v.starts_with('/') && v.contains('/') {
        return true;
    }
    false
}

fn build_databases_block() -> String {
    String::from("\ndatabases:\n  - name: db\n    engine: PG\n    production: true\n")
}

/// True if a bin is a test/dev/debug helper that should not ship as a
/// production DO worker.
/// Heuristic: name prefix OR `src/bin/<stem>.rs` stem prefix in
/// {test_, test-, dev_, dev-, debug_, debug-}.
/// TODO(122.1): also honor [[bin]] required-features once
/// project::BinEntry exposes it.
fn is_test_like_bin(b: &BinEntry) -> bool {
    const PREFIXES: &[&str] = &["test_", "test-", "dev_", "dev-", "debug_", "debug-"];
    if PREFIXES.iter().any(|p| b.name.starts_with(p)) {
        return true;
    }
    if let Some(path) = b.path.as_deref() {
        if let Some(stem) = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
        {
            if PREFIXES.iter().any(|p| stem.starts_with(p)) {
                return true;
            }
        }
    }
    false
}

fn build_workers_block(server: &str, bins: &[BinEntry]) -> String {
    let workers: Vec<&BinEntry> = bins
        .iter()
        .filter(|b| b.name != server && !is_test_like_bin(b))
        .collect();
    if workers.is_empty() {
        return String::new();
    }
    let mut out = String::from("\nworkers:\n");
    for w in workers {
        out.push_str(&format!("  - name: {}\n", w.name));
        out.push_str("    dockerfile_path: Dockerfile\n");
        out.push_str("    source_dir: /\n");
        out.push_str(&format!("    run_command: /usr/local/bin/{}\n", w.name));
        out.push_str("    instance_size_slug: apps-s-1vcpu-0.5gb\n");
        out.push_str("    instance_count: 1\n");
    }
    out
}

fn yaml_escape(v: &str) -> String {
    v.replace('\\', "\\\\").replace('"', "\\\"")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn bin(name: &str) -> BinEntry {
        BinEntry {
            name: name.to_string(),
            path: None,
        }
    }

    fn env(k: &str, v: &str) -> EnvEntry {
        EnvEntry {
            key: k.to_string(),
            value: v.to_string(),
        }
    }

    #[test]
    fn scenario_a_single_bin_no_database_fra1() {
        let bins = vec![bin("mkmenu")];
        let envs = vec![env("APP_URL", "https://mkmenu.io"), env("APP_KEY", "abc")];
        let ctx = AppYamlContext {
            package_name: "mkmenu",
            github_repo: "owner/mkmenu",
            region: "fra1",
            bins: &bins,
            env_entries: &envs,
        };
        let out = render_app_yaml(&ctx);
        assert!(out.contains("name: mkmenu"));
        assert!(out.contains("region: fra1"));
        assert!(out.contains("- name: web"));
        assert!(out.contains("dockerfile_path: Dockerfile"));
        assert!(out.contains("source_dir: /"));
        assert!(out.contains("    envs:"));
        assert!(out.contains("- key: APP_URL"));
        assert!(out.contains("- key: APP_KEY"));
        // APP_KEY is a SECRET, APP_URL is not
        let app_url_idx = out.find("- key: APP_URL").unwrap();
        let app_key_idx = out.find("- key: APP_KEY").unwrap();
        let app_url_section = &out[app_url_idx..app_key_idx];
        assert!(!app_url_section.contains("type: SECRET"));
        let app_key_section = &out[app_key_idx..];
        assert!(app_key_section.contains("type: SECRET"));
        assert!(!out.contains("databases:"));
        assert!(!out.contains("\nworkers:"));
    }

    #[test]
    fn scenario_b_multibin_database_nyc() {
        let bins = vec![bin("gestiscilo"), bin("screenshot-worker")];
        let envs = vec![
            env("APP_URL", "https://gestiscilo.it"),
            env("DATABASE_URL", "placeholder"),
            env("STRIPE_SECRET_KEY", "sk_xxx"),
        ];
        let ctx = AppYamlContext {
            package_name: "gestiscilo",
            github_repo: "owner/gestiscilo",
            region: "nyc",
            bins: &bins,
            env_entries: &envs,
        };
        let out = render_app_yaml(&ctx);
        assert!(out.contains("region: nyc"));
        assert!(out.contains("- key: DATABASE_URL"));
        assert!(out.contains("value: ${db.DATABASE_URL}"));
        // DATABASE_URL must be SECRET
        let db_idx = out.find("- key: DATABASE_URL").unwrap();
        let after_db = &out[db_idx..];
        let next_key = after_db[1..]
            .find("- key:")
            .map(|i| i + 1)
            .unwrap_or(after_db.len());
        assert!(after_db[..next_key].contains("type: SECRET"));
        // databases block
        assert!(out.contains("\ndatabases:\n  - name: db"));
        assert!(out.contains("engine: PG"));
        assert!(out.contains("production: true"));
        // workers block with screenshot-worker
        assert!(out.contains("\nworkers:\n"));
        assert!(out.contains("- name: screenshot-worker"));
        assert!(out.contains("run_command: /usr/local/bin/screenshot-worker"));
        // gestiscilo is the server, NOT a worker
        let workers_idx = out.find("\nworkers:").unwrap();
        assert!(!out[workers_idx..].contains("- name: gestiscilo"));
        // STRIPE_SECRET_KEY is SECRET
        let stripe_idx = out.find("- key: STRIPE_SECRET_KEY").unwrap();
        assert!(out[stripe_idx..].contains("type: SECRET"));
        // APP_URL is not SECRET
        let app_url_idx = out.find("- key: APP_URL").unwrap();
        let after_app_url = &out[app_url_idx..];
        let next = after_app_url[1..].find("- key:").map(|i| i + 1).unwrap();
        assert!(!after_app_url[..next].contains("type: SECRET"));
    }

    #[test]
    fn scenario_d_empty_envs_omits_envs_block() {
        let bins = vec![bin("solo")];
        let ctx = AppYamlContext {
            package_name: "solo",
            github_repo: "owner/solo",
            region: "fra1",
            bins: &bins,
            env_entries: &[],
        };
        let out = render_app_yaml(&ctx);
        assert!(out.contains("name: solo"));
        assert!(out.contains("- name: web"));
        // No envs section, no databases, no workers
        assert!(!out.contains("envs:"));
        assert!(!out.contains("databases:"));
        assert!(!out.contains("\nworkers:"));
    }

    #[test]
    fn sanitize_do_app_name_cases() {
        assert_eq!(sanitize_do_app_name("mkmenu_ferro"), "mkmenu-ferro");
        assert_eq!(sanitize_do_app_name("MyApp_v2"), "myapp-v2");
        assert_eq!(sanitize_do_app_name("foo__bar"), "foo-bar");
        assert_eq!(sanitize_do_app_name("app.name"), "appname");
        assert_eq!(sanitize_do_app_name("gestiscilo"), "gestiscilo");
    }

    #[test]
    fn should_placeholder_covers_all_branches() {
        assert!(should_placeholder("APP_URL", "http://localhost:8080"));
        assert!(should_placeholder("APP_URL", "http://127.0.0.1:8080"));
        assert!(should_placeholder("APP_URL", "http://0.0.0.0:8080"));
        assert!(should_placeholder("DATA_DIR", "./data"));
        assert!(should_placeholder("DATA_DIR", "data/local"));
        assert!(should_placeholder("CONFIG", "file:./config.toml"));
        assert!(should_placeholder("SOMETHING", ""));
        assert!(should_placeholder("SOMETHING", "   "));
        assert!(!should_placeholder("APP_URL", "https://gestiscilo.it"));
        assert!(!should_placeholder("SERVER_PORT", "8080"));
        assert!(!should_placeholder("SERVER_PORT", "3000"));
        assert!(!should_placeholder(
            "DATABASE_URL",
            "postgres://localhost/x"
        ));
        assert!(!should_placeholder("STRIPE_SECRET_KEY", "sk_test_xxx"));
    }

    #[test]
    fn envs_block_substitutes_localhost_app_url() {
        let envs = vec![env("APP_URL", "http://localhost:8080")];
        let out = build_envs_block(&envs);
        assert!(out.contains("# Override in DO App Platform secrets (was: http://localhost:8080)"));
        assert!(out.contains("value: ${APP_URL}"));
        assert!(!out.contains("value: \"http://localhost:8080\""));
    }

    #[test]
    fn envs_block_keeps_production_url_literal() {
        let envs = vec![env("APP_URL", "https://gestiscilo.it")];
        let out = build_envs_block(&envs);
        assert!(out.contains("value: \"https://gestiscilo.it\""));
        assert!(!out.contains("# Override"));
        assert!(!out.contains("value: ${APP_URL}"));
    }

    #[test]
    fn envs_block_database_url_unchanged_by_placeholder_logic() {
        let envs = vec![env("DATABASE_URL", "postgres://user:pass@localhost/db")];
        let out = build_envs_block(&envs);
        assert!(out.contains("value: ${db.DATABASE_URL}"));
        assert!(!out.contains("# Override"));
    }

    fn bin_with_path(name: &str, path: &str) -> BinEntry {
        BinEntry {
            name: name.to_string(),
            path: Some(path.to_string()),
        }
    }

    #[test]
    fn workers_block_excludes_test_parser() {
        let bins = vec![bin("mkmenu_ferro"), bin("test_parser")];
        assert_eq!(build_workers_block("mkmenu_ferro", &bins), "");
    }

    #[test]
    fn workers_block_keeps_screenshot_worker() {
        let bins = vec![bin("gestiscilo"), bin("screenshot-worker")];
        let out = build_workers_block("gestiscilo", &bins);
        assert!(out.contains("- name: screenshot-worker"));
        assert!(!out.contains("- name: gestiscilo"));
    }

    #[test]
    fn workers_block_excludes_all_test_dev_debug_prefixes() {
        let bins = vec![
            bin("app"),
            bin("dev_tool"),
            bin("debug-helper"),
            bin("test-foo"),
            bin("real-worker"),
        ];
        let out = build_workers_block("app", &bins);
        assert!(out.contains("- name: real-worker"));
        assert!(!out.contains("dev_tool"));
        assert!(!out.contains("debug-helper"));
        assert!(!out.contains("test-foo"));
    }

    #[test]
    fn workers_block_excludes_by_path_stem() {
        let bins = vec![
            bin("app"),
            bin_with_path("odd_name", "src/bin/test_parser.rs"),
        ];
        assert_eq!(build_workers_block("app", &bins), "");
    }

    #[test]
    fn yaml_escape_handles_quotes_and_backslashes() {
        assert_eq!(yaml_escape("a\"b\\c"), "a\\\"b\\\\c");
    }
}
