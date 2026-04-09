//! `ferro doctor` — runs all 11 health checks (D-01..D-09, D-22).
//!
//! Complementary to `ferro:info` MCP tool — does not replace it.

use crate::doctor::check::{CheckCategory, CheckStatus, Report};
use crate::doctor::registry::default_checks;
use crate::project::find_project_root;
use console::style;

pub fn run(json: bool, deploy_only: bool) {
    let root = match find_project_root(None) {
        Ok(r) => r,
        Err(_) => {
            eprintln!(
                "{} Cargo.toml not found (run `ferro doctor` inside a Ferro project)",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    };

    let all_checks = default_checks();
    let checks: Vec<_> = if deploy_only {
        all_checks
            .into_iter()
            .filter(|c| c.category() == CheckCategory::Deploy)
            .collect()
    } else {
        all_checks
    };
    let results: Vec<_> = checks.iter().map(|c| c.run(&root)).collect();
    let report = Report::build(results);

    if json {
        match serde_json::to_string_pretty(&report) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("{} JSON serialize error: {e}", style("Error:").red().bold());
                std::process::exit(1);
            }
        }
    } else {
        print_human(&report);
    }

    std::process::exit(report.exit_code());
}

fn print_human(report: &Report) {
    for c in &report.checks {
        let icon = match c.status {
            CheckStatus::Ok => style("✓").green().to_string(),
            CheckStatus::Warn => style("⚠").yellow().to_string(),
            CheckStatus::Error => style("✗").red().to_string(),
        };
        println!("{icon} {} — {}", style(c.name).bold(), c.message);
        if let Some(d) = &c.details {
            println!("    {}", style(d).dim());
        }
    }
    println!();
    let s = &report.summary;
    let summary_line = format!("Summary: {} ok, {} warn, {} error", s.ok, s.warn, s.error);
    let styled = match s.overall {
        CheckStatus::Ok => style(summary_line).green().bold(),
        CheckStatus::Warn => style(summary_line).yellow().bold(),
        CheckStatus::Error => style(summary_line).red().bold(),
    };
    println!("{styled}");
}
