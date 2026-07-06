//! Search docs tool - search local markdown documentation

use crate::error::Result;
use serde::Serialize;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct SearchDocsResult {
    pub query: String,
    pub matches: Vec<DocMatch>,
    pub total_matches: usize,
}

#[derive(Debug, Serialize)]
pub struct DocMatch {
    pub file: String,
    pub title: Option<String>,
    pub excerpt: String,
    pub line_number: usize,
    pub relevance: f32,
}

pub fn execute(project_root: &Path, query: &str) -> Result<SearchDocsResult> {
    let docs_dir = project_root.join("docs");
    let mut matches = Vec::new();

    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    // Search in docs/ directory
    if docs_dir.exists() {
        search_directory(&docs_dir, project_root, &query_words, &mut matches);
    }

    // Also search in README.md if it exists
    let readme = project_root.join("README.md");
    if readme.exists() {
        search_file(&readme, project_root, &query_words, &mut matches);
    }

    // Sort by relevance
    matches.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Limit results
    let total_matches = matches.len();
    matches.truncate(20);

    Ok(SearchDocsResult {
        query: query.to_string(),
        matches,
        total_matches,
    })
}

fn search_directory(
    dir: &Path,
    project_root: &Path,
    query_words: &[&str],
    matches: &mut Vec<DocMatch>,
) {
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "md" || ext == "markdown")
                .unwrap_or(false)
        })
    {
        search_file(entry.path(), project_root, query_words, matches);
    }
}

fn search_file(
    path: &Path,
    project_root: &Path,
    query_words: &[&str],
    matches: &mut Vec<DocMatch>,
) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };

    let relative_path = path
        .strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    // Extract title from first heading
    let title = extract_title(&content);

    let content_lower = content.to_lowercase();

    for (line_idx, line) in content.lines().enumerate() {
        let line_lower = line.to_lowercase();
        let mut word_matches = 0;

        for word in query_words {
            if line_lower.contains(word) {
                word_matches += 1;
            }
        }

        if word_matches > 0 {
            let relevance = calculate_relevance(&line_lower, query_words, &content_lower);

            // Create excerpt with context
            let excerpt = create_excerpt(&content, line_idx);

            matches.push(DocMatch {
                file: relative_path.clone(),
                title: title.clone(),
                excerpt,
                line_number: line_idx + 1,
                relevance,
            });
        }
    }
}

fn extract_title(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            return Some(stripped.trim().to_string());
        }
    }
    None
}

fn calculate_relevance(line: &str, query_words: &[&str], full_content: &str) -> f32 {
    let mut score: f32 = 0.0;

    for word in query_words {
        // Exact word match in line
        if line.contains(word) {
            score += 1.0;
        }

        // Word at start of line (likely heading or important)
        if line.trim_start().starts_with(word) {
            score += 0.5;
        }

        // Multiple occurrences in document
        let occurrences = full_content.matches(word).count();
        score += (occurrences as f32).min(5.0) * 0.1;
    }

    // Boost for heading lines
    if line.trim().starts_with('#') {
        score *= 1.5;
    }

    // Boost for code blocks (likely examples)
    if line.contains("```") || line.starts_with("    ") {
        score *= 1.2;
    }

    score
}

fn create_excerpt(content: &str, target_line: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start = target_line.saturating_sub(1);
    let end = (target_line + 2).min(lines.len());

    lines[start..end]
        .iter()
        .map(|s| s.trim())
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(200)
        .collect::<String>()
}
