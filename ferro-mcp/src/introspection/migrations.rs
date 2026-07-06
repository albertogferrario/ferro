//! Migration introspection

use crate::tools::list_migrations::MigrationInfo;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn scan_migration_files(migrations_dir: &Path) -> Vec<MigrationInfo> {
    let mut migrations = Vec::new();

    if !migrations_dir.exists() {
        return migrations;
    }

    for entry in WalkDir::new(migrations_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .file_name()
                .map(|f| f.to_string_lossy().starts_with('m'))
                .unwrap_or(false)
                && e.path().extension().map(|ext| ext == "rs").unwrap_or(false)
        })
    {
        if let Some(name) = entry.path().file_stem() {
            let name_str = name.to_string_lossy().to_string();
            if name_str != "mod" {
                migrations.push(MigrationInfo {
                    name: name_str,
                    status: "unknown".to_string(),
                    applied_at: None,
                });
            }
        }
    }

    // Also check mod.rs for migration registration
    let mod_file = migrations_dir.join("mod.rs");
    if mod_file.exists() {
        if let Ok(content) = fs::read_to_string(&mod_file) {
            for line in content.lines() {
                if line.trim().starts_with("mod m") {
                    if let Some(name) = line
                        .trim()
                        .trim_start_matches("mod ")
                        .trim_end_matches(';')
                        .split_whitespace()
                        .next()
                    {
                        if !migrations.iter().any(|m| m.name == name) {
                            migrations.push(MigrationInfo {
                                name: name.to_string(),
                                status: "unknown".to_string(),
                                applied_at: None,
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort by name (which includes timestamp prefix)
    migrations.sort_by(|a, b| a.name.cmp(&b.name));

    migrations
}
