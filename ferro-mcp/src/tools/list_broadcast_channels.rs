//! List broadcast channels tool - scan for broadcasting configuration and channel usage

use crate::error::Result;
use serde::Serialize;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct BroadcastInfo {
    pub config_found: bool,
    pub auth_route: Option<String>,
    pub channels: Vec<ChannelInfo>,
}

#[derive(Debug, Serialize)]
pub struct ChannelInfo {
    pub name: String,
    pub source_file: String,
    pub line: usize,
    pub channel_type: String,
}

pub fn execute(project_root: &Path) -> Result<BroadcastInfo> {
    let src_path = project_root.join("src");
    let mut config_found = false;
    let mut auth_route = None;
    let mut channels = Vec::new();

    if src_path.exists() {
        scan_directory(
            &src_path,
            &mut config_found,
            &mut auth_route,
            &mut channels,
            project_root,
        );
    }

    Ok(BroadcastInfo {
        config_found,
        auth_route,
        channels,
    })
}

fn scan_directory(
    dir: &Path,
    config_found: &mut bool,
    auth_route: &mut Option<String>,
    channels: &mut Vec<ChannelInfo>,
    project_root: &Path,
) {
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            let relative_path = entry
                .path()
                .strip_prefix(project_root)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .to_string();

            // Check for BroadcastConfig usage
            if content.contains("BroadcastConfig") {
                *config_found = true;
            }

            // Check for broadcasting_auth route registration
            if content.contains("broadcasting_auth") {
                extract_auth_route(&content, &relative_path, auth_route);
            }

            // Check for channel usage patterns
            let has_broadcast_send = content.contains("Broadcaster::send(")
                || content.contains("broadcaster.send(")
                || content.contains("broadcast(")
                || content.contains(".channel(");

            if has_broadcast_send {
                extract_channel_usage(&content, &relative_path, channels);
            }
        }
    }
}

/// Extract the broadcasting auth route path if found.
fn extract_auth_route(content: &str, path: &str, auth_route: &mut Option<String>) {
    for line in content.lines() {
        let trimmed = line.trim();
        // Skip comments
        if trimmed.starts_with("//") {
            continue;
        }
        // Match: post!("/broadcasting/auth", ...) or similar route with broadcasting_auth
        if trimmed.contains("broadcasting_auth") {
            // Try to extract the path from the route macro
            for macro_name in &["post!", "get!", "put!"] {
                if let Some(pos) = trimmed.find(macro_name) {
                    let after = &trimmed[pos + macro_name.len()..];
                    if let Some(q1) = after.find('"') {
                        let rest = &after[q1 + 1..];
                        if let Some(q2) = rest.find('"') {
                            *auth_route = Some(format!("{} ({})", &rest[..q2], path));
                            return;
                        }
                    }
                }
            }
            // Fallback: just note the file
            if auth_route.is_none() {
                *auth_route = Some(format!("(found in {path})"));
            }
        }
    }
}

/// Extract channel names from Broadcast/Broadcaster usage patterns.
///
/// Looks for:
/// - `.channel("channel-name")` - fluent builder API
/// - `Broadcaster::send("channel-name", ...)` or `broadcaster.send("channel-name", ...)`
/// - `broadcaster.broadcast("channel-name", ...)`
fn extract_channel_usage(content: &str, path: &str, channels: &mut Vec<ChannelInfo>) {
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") {
            continue;
        }

        // Match: .channel("name")
        if let Some(pos) = trimmed.find(".channel(") {
            let after = &trimmed[pos + ".channel(".len()..];
            if let Some(name) = extract_quoted_string(after) {
                let channel_type = classify_channel(&name);
                channels.push(ChannelInfo {
                    name,
                    source_file: path.to_string(),
                    line: line_idx + 1,
                    channel_type,
                });
            }
        }

        // Match: broadcaster.broadcast("channel", ...) or Broadcaster::broadcast(
        for pattern in &[
            "broadcaster.broadcast(",
            "Broadcaster::broadcast(",
            "broadcaster.send(",
            "Broadcaster::send(",
        ] {
            if let Some(pos) = trimmed.find(pattern) {
                let after = &trimmed[pos + pattern.len()..];
                // Skip &self references - look for the channel name arg
                let search = if after.starts_with('&') {
                    // Method call: first arg after &self is channel
                    after.find(',').map(|c| &after[c + 1..])
                } else {
                    Some(after)
                };

                if let Some(search) = search {
                    if let Some(name) = extract_quoted_string(search) {
                        let channel_type = classify_channel(&name);
                        channels.push(ChannelInfo {
                            name,
                            source_file: path.to_string(),
                            line: line_idx + 1,
                            channel_type,
                        });
                    }
                }
            }
        }
    }
}

/// Classify a channel by its name prefix.
fn classify_channel(name: &str) -> String {
    if name.starts_with("private-") {
        "private".to_string()
    } else if name.starts_with("presence-") {
        "presence".to_string()
    } else {
        "public".to_string()
    }
}

/// Extract the first double-quoted string from text.
fn extract_quoted_string(text: &str) -> Option<String> {
    let q1 = text.find('"')?;
    let rest = &text[q1 + 1..];
    let q2 = rest.find('"')?;
    let value = &rest[..q2];
    if value.is_empty() {
        return None;
    }
    Some(value.to_string())
}
