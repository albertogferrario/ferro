//! List authorization policies tool - scan for impl `Policy<Model>` patterns

use crate::error::Result;
use serde::Serialize;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct PoliciesInfo {
    pub policies: Vec<PolicyInfo>,
}

#[derive(Debug, Serialize)]
pub struct PolicyInfo {
    pub name: String,
    pub model: String,
    pub path: String,
    pub abilities: Vec<String>,
}

/// Known Policy trait methods that represent authorization abilities.
const POLICY_ABILITIES: &[&str] = &[
    "before",
    "view_any",
    "view",
    "create",
    "update",
    "delete",
    "restore",
    "force_delete",
];

pub fn execute(project_root: &Path) -> Result<PoliciesInfo> {
    let src_path = project_root.join("src");
    let mut all_policies = Vec::new();

    if src_path.exists() {
        scan_directory(&src_path, &mut all_policies, project_root);
    }

    Ok(PoliciesInfo {
        policies: all_policies,
    })
}

fn scan_directory(dir: &Path, policies: &mut Vec<PolicyInfo>, project_root: &Path) {
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            // Quick check before parsing: skip files without Policy impl
            if !content.contains("impl Policy<") {
                continue;
            }

            let relative_path = entry
                .path()
                .strip_prefix(project_root)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .to_string();

            extract_policies(&content, &relative_path, policies);
        }
    }
}

/// Extract policy information from file content using string matching.
///
/// Looks for `impl Policy<ModelName> for StructName` patterns and
/// extracts method names within the impl block.
fn extract_policies(content: &str, path: &str, policies: &mut Vec<PolicyInfo>) {
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        // Match `impl Policy<ModelName> for StructName`
        if let Some((model, name)) = parse_policy_impl(line) {
            let abilities = extract_abilities_from_impl(&lines, i);

            policies.push(PolicyInfo {
                name,
                model,
                path: path.to_string(),
                abilities,
            });
        }
    }
}

/// Parse a line for `impl Policy<Model> for Name` pattern.
/// Returns (model, policy_name) if found.
fn parse_policy_impl(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();

    // Match: impl Policy<X> for Y { or impl Policy<X> for Y
    let policy_idx = trimmed.find("impl Policy<")?;
    let after_impl = &trimmed[policy_idx + "impl Policy<".len()..];

    // Extract model name (inside angle brackets)
    let bracket_end = after_impl.find('>')?;
    let model = after_impl[..bracket_end].trim().to_string();

    // Extract policy struct name after "for"
    let after_bracket = &after_impl[bracket_end + 1..];
    let for_idx = after_bracket.find(" for ")?;
    let after_for = after_bracket[for_idx + " for ".len()..].trim();

    // Name ends at whitespace, '{', or end of line
    let name_end = after_for
        .find(|c: char| c.is_whitespace() || c == '{')
        .unwrap_or(after_for.len());
    let name = after_for[..name_end].trim().to_string();

    if model.is_empty() || name.is_empty() {
        return None;
    }

    Some((model, name))
}

/// Extract ability method names from the impl block starting at the given line.
fn extract_abilities_from_impl(lines: &[&str], start_line: usize) -> Vec<String> {
    let mut abilities = Vec::new();
    let mut brace_depth: i32 = 0;
    let mut in_block = false;

    for line in &lines[start_line..] {
        for ch in line.chars() {
            if ch == '{' {
                brace_depth += 1;
                in_block = true;
            } else if ch == '}' {
                brace_depth -= 1;
            }
        }

        // Check for fn declarations that match known abilities
        let trimmed = line.trim();
        let fn_part = if let Some(stripped) = trimmed.strip_prefix("pub fn ") {
            Some(stripped)
        } else {
            trimmed.strip_prefix("fn ")
        };

        if let Some(fn_part) = fn_part {
            // Extract function name
            if let Some(paren) = fn_part.find('(') {
                let fn_name = fn_part[..paren].trim();
                if POLICY_ABILITIES.contains(&fn_name) {
                    abilities.push(fn_name.to_string());
                }
            }
        }

        // End of impl block
        if in_block && brace_depth == 0 {
            break;
        }
    }

    abilities
}
