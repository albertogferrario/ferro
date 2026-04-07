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
        out.push_str(&format!("      - key: {}\n", e.key));
        out.push_str("        scope: RUN_TIME\n");
        if e.key.eq_ignore_ascii_case("DATABASE_URL") {
            out.push_str("        value: ${db.DATABASE_URL}\n");
        } else {
            out.push_str(&format!("        value: \"{}\"\n", yaml_escape(&e.value)));
        }
        if is_secret(&e.key) {
            out.push_str("        type: SECRET\n");
        }
    }
    out
}

fn build_databases_block() -> String {
    String::from("\ndatabases:\n  - name: db\n    engine: PG\n    production: true\n")
}

fn build_workers_block(server: &str, bins: &[BinEntry]) -> String {
    let workers: Vec<&BinEntry> = bins.iter().filter(|b| b.name != server).collect();
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
    fn yaml_escape_handles_quotes_and_backslashes() {
        assert_eq!(yaml_escape("a\"b\\c"), "a\\\"b\\\\c");
    }
}
