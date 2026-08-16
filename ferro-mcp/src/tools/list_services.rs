//! List services tool - show registered DI container services
//!
//! This tool tries to fetch services from the running application first via
//! the `/_ferro/services` debug endpoint, falling back to static file parsing
//! when the app isn't running.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

/// Timeout for HTTP requests to the running application
const HTTP_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Serialize)]
pub struct ServicesInfo {
    pub services: Vec<ServiceItem>,
    /// Indicates whether services came from runtime or static analysis
    pub source: ServiceSource,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceSource {
    /// Services fetched from running application via HTTP endpoint
    Runtime,
    /// Services parsed from source files (fallback when app not running)
    StaticAnalysis,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OffloadParam {
    pub name: String,
    pub rust_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OffloadableMethod {
    pub name: String,
    /// Queue declared in `#[offload(queue = "...")]` or `"default"` when omitted.
    pub queue: String,
    /// Non-self parameters, types as Rust strings (owned equivalents of borrow types).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<OffloadParam>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceItem {
    /// Service name (trait or concrete type)
    pub name: String,
    /// Type of binding (trait_binding or singleton)
    pub binding_type: String,
    /// Offloadable methods declared on this service trait.
    /// Absent (not serialized) for services with no `#[offload]` methods.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<OffloadableMethod>,
}

/// Response format from the `/_ferro/services` endpoint
#[derive(Debug, Deserialize)]
struct DebugResponse {
    success: bool,
    data: Vec<RuntimeServiceInfo>,
}

/// Service info as returned by the runtime endpoint
#[derive(Debug, Deserialize)]
struct RuntimeServiceInfo {
    name: String,
    binding_type: String,
}

/// Try to fetch services from the running application
async fn fetch_runtime_services(base_url: &str) -> Option<Vec<ServiceItem>> {
    let url = format!("{base_url}/_ferro/services");

    let client = reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .ok()?;
    let response = client.get(&url).send().await.ok()?;

    if !response.status().is_success() {
        return None;
    }

    let debug_response: DebugResponse = response.json().await.ok()?;

    if !debug_response.success {
        return None;
    }

    Some(
        debug_response
            .data
            .into_iter()
            .map(|s| ServiceItem {
                name: s.name,
                binding_type: s.binding_type,
                methods: Vec::new(),
            })
            .collect(),
    )
}

pub async fn execute(project_root: &Path) -> Result<ServicesInfo> {
    // Try runtime endpoint first
    for base_url in ["http://localhost:8080", "http://127.0.0.1:8080"] {
        if let Some(mut services) = fetch_runtime_services(base_url).await {
            scan_offload_methods_from_files(project_root, &mut services);
            return Ok(ServicesInfo {
                services,
                source: ServiceSource::Runtime,
            });
        }
    }

    // Fall back to static analysis
    let mut services = scan_services_from_files(project_root);
    scan_offload_methods_from_files(project_root, &mut services);
    Ok(ServicesInfo {
        services,
        source: ServiceSource::StaticAnalysis,
    })
}

/// Scan source files for service definitions (static analysis fallback)
fn scan_services_from_files(project_root: &Path) -> Vec<ServiceItem> {
    use std::fs;
    use walkdir::WalkDir;

    let mut services = Vec::new();
    let src_dir = project_root.join("src");

    if !src_dir.exists() {
        return services;
    }

    // Look for #[service(...)] and #[injectable] attributes
    for entry in WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            let logical_lines = normalize_service_lines(&content);
            // Find #[service(ConcreteType)] on traits
            for line in logical_lines.iter().map(String::as_str) {
                let trimmed = line.trim();

                // Match #[service(SomeType)]
                if trimmed.starts_with("#[service(") {
                    if let Some(start) = trimmed.find('(') {
                        if let Some(end) = trimmed.find(')') {
                            let impl_name = extract_service_impl_name(&trimmed[start + 1..end]);
                            services.push(ServiceItem {
                                name: impl_name,
                                binding_type: "trait_binding".to_string(),
                                methods: Vec::new(),
                            });
                        }
                    }
                }

                // Match #[injectable]
                if trimmed == "#[injectable]" {
                    // Look for the next pub struct line
                    // This is a simplified approach - real parsing would use syn
                }
            }

            // Look for singleton! and bind! macro calls
            for line in content.lines() {
                let trimmed = line.trim();

                if trimmed.contains("singleton!(") {
                    // Extract type from singleton!(TypeName::new())
                    if let Some(start) = trimmed.find("singleton!(") {
                        let rest = &trimmed[start + 11..];
                        if let Some(type_end) = rest.find("::") {
                            let type_name = &rest[..type_end];
                            services.push(ServiceItem {
                                name: type_name.trim().to_string(),
                                binding_type: "singleton".to_string(),
                                methods: Vec::new(),
                            });
                        }
                    }
                }

                if trimmed.contains("bind!(") {
                    // Extract trait from bind!(dyn TraitName, ...)
                    if let Some(start) = trimmed.find("bind!(") {
                        let rest = &trimmed[start + 6..];
                        if let Some(comma) = rest.find(',') {
                            let trait_part = &rest[..comma];
                            services.push(ServiceItem {
                                name: trait_part.trim().to_string(),
                                binding_type: "trait_binding".to_string(),
                                methods: Vec::new(),
                            });
                        }
                    }
                }
            }
        }
    }

    services
}

/// Detect `#[offload]` or `#[offload(queue = "name")]` on a trimmed source line.
///
/// Returns `Some(queue_name)` for `#[offload]` (defaults to `"default"`) or
/// `#[offload(queue = "name")]`. Returns `None` for any other line.
fn detect_offload_attr(trimmed: &str) -> Option<String> {
    if trimmed == "#[offload]" {
        return Some("default".to_string());
    }
    if trimmed.starts_with("#[offload(") {
        // Look for queue = "..."
        if let Some(q_start) = trimmed.find("queue = \"") {
            let after = &trimmed[q_start + 9..];
            if let Some(q_end) = after.find('"') {
                return Some(after[..q_end].to_string());
            }
        }
        // Has args but no queue = "..." key — treat as bare (default queue)
        return Some("default".to_string());
    }
    None
}

/// Apply `owned_type` substitution rules mirroring `ferro-macros/src/offload.rs`.
///
/// - `&str` → `String`
/// - `&[T]` → `Vec<T>`
/// - `&T` → `T` (strip leading `&`, non-`&mut`)
/// - everything else → verbatim
fn owned_type(ty: &str) -> String {
    let ty = ty.trim();
    if ty == "&str" {
        return "String".to_string();
    }
    if let Some(inner) = ty.strip_prefix("&[").and_then(|s| s.strip_suffix(']')) {
        return format!("Vec<{inner}>");
    }
    if ty.starts_with('&') && !ty.starts_with("&mut ") {
        return ty[1..].to_string();
    }
    ty.to_string()
}

/// Extract the concrete impl type name from a `#[service(...)]` argument list.
///
/// Mirrors the two forms the `#[service]` macro accepts
/// (ferro-macros/src/service.rs): the positional form `#[service(ReportBuilder)]`
/// (returned verbatim) and the named form
/// `#[service(impl = ReportBuilder, fake = FakeBuilder)]` (the `impl =` value is
/// returned; `fake =` and any other keys are ignored). Without this
/// normalization the named form would surface as the malformed service name
/// `"impl = ReportBuilder"` and break concrete-name correlation of offload
/// methods.
fn extract_service_impl_name(inner: &str) -> String {
    let inner = inner.trim();
    for part in inner.split(',') {
        if let Some(rest) = part.trim().strip_prefix("impl") {
            if let Some(value) = rest.trim_start().strip_prefix('=') {
                return value.trim().to_string();
            }
        }
    }
    inner.to_string()
}

/// Join any multi-line `#[service(...)]` attribute into a single logical line.
///
/// Keys on the `#[service(` prefix only. Walks physical lines; when a `#[service(`
/// line's parens are not balanced on that line, accumulates subsequent trimmed lines
/// (joined by a single space) until paren depth returns to `<= 0`, then emits the
/// joined text as one logical line. All other lines — including `fn` signatures and
/// other attributes — pass through unchanged. Counts only `(` / `)` (never `[` / `]`),
/// so the `)]` close and `<...>` generics inside an impl type are handled correctly.
/// Mirrors the paren-depth balancing in `extract_inner_params`.
fn normalize_service_lines(content: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut accumulating: Option<String> = None;
    let mut paren_depth: i32 = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(ref mut acc) = accumulating {
            // Continuation of a multi-line #[service(...)]
            if !acc.is_empty() {
                acc.push(' ');
            }
            acc.push_str(trimmed);
            for ch in trimmed.chars() {
                match ch {
                    '(' => paren_depth += 1,
                    ')' => paren_depth -= 1,
                    _ => {}
                }
            }
            if paren_depth <= 0 {
                // Attribute closed — emit the joined line
                result.push(accumulating.take().unwrap());
                paren_depth = 0;
            }
        } else if trimmed.starts_with("#[service(") {
            // Compute paren depth contributed by this line alone
            let mut depth: i32 = 0;
            for ch in trimmed.chars() {
                match ch {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
            }
            if depth <= 0 {
                // Attribute closed on this line — pass through unchanged (preserve indentation)
                result.push(line.to_string());
            } else {
                // Attribute continues onto subsequent lines — start accumulation
                accumulating = Some(trimmed.to_string());
                paren_depth = depth;
            }
        } else {
            result.push(line.to_string());
        }
    }

    // If the file ends mid-attribute (malformed input), flush to avoid losing content
    if let Some(acc) = accumulating {
        result.push(acc);
    }

    result
}

/// Parse non-self parameters from the text between the outer `(` and matching `)`.
///
/// Performs a bracket-aware split on `,` so that generic types such as
/// `HashMap<K, V>` are not incorrectly split on their inner comma.
fn extract_method_params(inner: &str) -> Vec<OffloadParam> {
    // Bracket-aware split on ','
    let mut params = Vec::new();
    let mut depth: i32 = 0;
    let mut current = String::new();

    for ch in inner.chars() {
        match ch {
            '<' | '[' => {
                depth += 1;
                current.push(ch);
            }
            '>' | ']' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                let segment = current.trim().to_string();
                if !segment.is_empty() {
                    params.push(segment);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let segment = current.trim().to_string();
    if !segment.is_empty() {
        params.push(segment);
    }

    // Drop the receiver segment (&self / self / &mut self / mut self)
    let segments = params.into_iter().filter(|s| {
        let t = s.trim();
        t != "&self" && t != "self" && t != "&mut self" && t != "mut self"
    });

    let mut result = Vec::new();
    for seg in segments {
        // Split on the first ':' into (name, type)
        if let Some(colon) = seg.find(':') {
            let name = seg[..colon].trim().to_string();
            let ty = owned_type(seg[colon + 1..].trim());
            if !name.is_empty() {
                result.push(OffloadParam {
                    name,
                    rust_type: ty,
                });
            }
        }
    }
    result
}

/// Second-pass walker: augment `services` with `#[offload]`-annotated methods.
///
/// Walks `{project_root}/src/**/*.rs` using the same filter as
/// `scan_services_from_files`. For each file runs a three-state machine:
///
/// - **Idle**: scanning for `#[offload]` / `#[service(...)]` / `trait TraitName`.
/// - **OffloadPending(queue)**: saw `#[offload]`; waiting for `fn` line.
/// - **FnCollecting(name, queue, buf)**: accumulating param list until the
///   closing `)` is depth-0 balanced.
///
/// Discovered methods are correlated to `ServiceItem` entries by either the
/// concrete impl name or the trait name of the enclosing block.
fn scan_offload_methods_from_files(project_root: &Path, services: &mut [ServiceItem]) {
    use std::fs;
    use walkdir::WalkDir;

    let src_dir = project_root.join("src");
    if !src_dir.exists() {
        return;
    }

    for entry in WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        let Ok(content) = fs::read_to_string(entry.path()) else {
            continue;
        };
        let logical_lines = normalize_service_lines(&content);

        // Track current service block: (concrete_name, trait_name)
        let mut current_concrete: Option<String> = None;
        let mut current_trait: Option<String> = None;
        // Collected offload methods for the current block
        let mut block_methods: Vec<OffloadableMethod> = Vec::new();

        // Three-state machine
        enum State {
            Idle,
            OffloadPending(String), // queue
            FnCollecting {
                method_name: String,
                queue: String,
                buf: String,
                paren_depth: i32,
            },
        }

        let mut state = State::Idle;

        for line in logical_lines.iter().map(String::as_str) {
            let trimmed = line.trim();

            // Detect start of a #[service(...)] block (sets concrete name)
            if trimmed.starts_with("#[service(") {
                if let Some(start) = trimmed.find('(') {
                    if let Some(end) = trimmed.find(')') {
                        let impl_name = extract_service_impl_name(&trimmed[start + 1..end]);
                        // Flush any previous block
                        flush_block(
                            &current_concrete,
                            &current_trait,
                            &mut block_methods,
                            services,
                        );
                        current_concrete = Some(impl_name);
                        current_trait = None;
                        block_methods.clear();
                    }
                }
                continue;
            }

            // Detect `pub trait TraitName` or `trait TraitName` following a #[service(...)]
            if current_concrete.is_some() && current_trait.is_none() {
                let stripped = trimmed
                    .strip_prefix("pub trait ")
                    .or_else(|| trimmed.strip_prefix("trait "));
                if let Some(rest) = stripped {
                    let trait_name = rest
                        .split(|c: char| !c.is_alphanumeric() && c != '_')
                        .next()
                        .unwrap_or("")
                        .to_string();
                    if !trait_name.is_empty() {
                        current_trait = Some(trait_name);
                    }
                }
            }

            // Run state machine
            match state {
                State::Idle => {
                    if let Some(queue) = detect_offload_attr(trimmed) {
                        state = State::OffloadPending(queue);
                    }
                }
                State::OffloadPending(ref queue) => {
                    // Skip doc comments, other attributes, and blank lines
                    if trimmed.starts_with("///")
                        || trimmed.starts_with("//")
                        || trimmed.starts_with("#[")
                        || trimmed.is_empty()
                    {
                        continue;
                    }

                    // Look for `fn` or `async fn`
                    let fn_line = trimmed
                        .strip_prefix("async fn ")
                        .or_else(|| trimmed.strip_prefix("pub async fn "))
                        .or_else(|| trimmed.strip_prefix("fn "))
                        .or_else(|| trimmed.strip_prefix("pub fn "));

                    if let Some(rest) = fn_line {
                        // Extract method name (ident before '(')
                        let method_name = rest
                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                            .next()
                            .unwrap_or("")
                            .to_string();

                        if method_name.is_empty() {
                            state = State::Idle;
                            continue;
                        }

                        // Find the opening '(' and collect from there
                        let buf = if let Some(paren_pos) = rest.find('(') {
                            rest[paren_pos + 1..].to_string()
                        } else {
                            String::new()
                        };

                        // Count paren depth in buf so far
                        let mut depth: i32 = 0;
                        for ch in buf.chars() {
                            match ch {
                                '(' => depth += 1,
                                ')' => depth -= 1,
                                _ => {}
                            }
                        }

                        let queue_str = queue.clone();
                        if depth < 0 {
                            // The ')' is already in buf — param list closed on this line
                            // Extract up to the first ')' that closes depth
                            let inner = extract_inner_params(&buf);
                            let params = extract_method_params(&inner);
                            block_methods.push(OffloadableMethod {
                                name: method_name,
                                queue: queue_str,
                                params,
                            });
                            state = State::Idle;
                        } else {
                            state = State::FnCollecting {
                                method_name,
                                queue: queue_str,
                                buf,
                                paren_depth: depth,
                            };
                        }
                    } else {
                        // Unexpected line — reset
                        state = State::Idle;
                    }
                }
                State::FnCollecting {
                    ref method_name,
                    ref queue,
                    ref mut buf,
                    ref mut paren_depth,
                } => {
                    buf.push(' ');
                    buf.push_str(trimmed);

                    // Recount paren depth from the new portion
                    for ch in trimmed.chars() {
                        match ch {
                            '(' => *paren_depth += 1,
                            ')' => *paren_depth -= 1,
                            _ => {}
                        }
                    }

                    if *paren_depth < 0 {
                        // Param list closed
                        let inner = extract_inner_params(buf);
                        let params = extract_method_params(&inner);
                        block_methods.push(OffloadableMethod {
                            name: method_name.clone(),
                            queue: queue.clone(),
                            params,
                        });
                        state = State::Idle;
                    }
                }
            }
        }

        // Flush the last block in the file
        flush_block(
            &current_concrete,
            &current_trait,
            &mut block_methods,
            services,
        );
    }
}

/// Extract the text before the first unbalanced `)` in `buf`.
///
/// `buf` is the text that was accumulated starting after the opening `(`.
/// The first char whose paren depth goes negative is the closing `)`.
fn extract_inner_params(buf: &str) -> String {
    let mut depth: i32 = 0;
    let mut result = String::new();
    for ch in buf.chars() {
        match ch {
            '(' => {
                depth += 1;
                result.push(ch);
            }
            ')' => {
                if depth == 0 {
                    break; // This is the closing paren
                }
                depth -= 1;
                result.push(ch);
            }
            _ => result.push(ch),
        }
    }
    result
}

/// Attach collected `methods` to the matching `ServiceItem` (by concrete or trait name).
fn flush_block(
    concrete: &Option<String>,
    trait_name: &Option<String>,
    methods: &mut Vec<OffloadableMethod>,
    services: &mut [ServiceItem],
) {
    if methods.is_empty() {
        return;
    }
    for svc in services.iter_mut() {
        let matches = concrete.as_deref() == Some(svc.name.as_str())
            || trait_name.as_deref() == Some(svc.name.as_str());
        if matches {
            svc.methods.append(methods);
            return;
        }
    }
    // No matching ServiceItem — discard (unregistered service)
    methods.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detect_offload_attr_bare_returns_default() {
        assert_eq!(
            detect_offload_attr("#[offload]"),
            Some("default".to_string())
        );
    }

    #[test]
    fn detect_offload_attr_reads_declared_queue() {
        assert_eq!(
            detect_offload_attr("#[offload(queue = \"reports\")]"),
            Some("reports".to_string())
        );
        // Non-offload line returns None
        assert_eq!(detect_offload_attr("#[service(ReportBuilder)]"), None);
    }

    #[test]
    fn extract_service_impl_name_positional_and_named() {
        // Positional form returned verbatim
        assert_eq!(extract_service_impl_name("ReportBuilder"), "ReportBuilder");
        // Named form: the `impl =` value is returned, `fake =` ignored
        assert_eq!(
            extract_service_impl_name("impl = ReportBuilder"),
            "ReportBuilder"
        );
        assert_eq!(
            extract_service_impl_name("impl = ReportBuilder, fake = FakeBuilder"),
            "ReportBuilder"
        );
        // Key order does not matter
        assert_eq!(
            extract_service_impl_name("fake = FakeBuilder, impl = ReportBuilder"),
            "ReportBuilder"
        );
        // A positional type whose name merely starts with "impl" is not misread
        assert_eq!(extract_service_impl_name("ImplRegistry"), "ImplRegistry");
    }

    #[test]
    fn extract_method_params_bracket_aware() {
        // The inner comma of HashMap<K, V> must NOT split the parameter
        let params = extract_method_params("&self, id: i64, map: HashMap<K, V>");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "id");
        assert_eq!(params[0].rust_type, "i64");
        assert_eq!(params[1].name, "map");
        assert_eq!(params[1].rust_type, "HashMap<K, V>");
    }

    #[test]
    fn extract_method_params_owned_substitution() {
        let params = extract_method_params("&self, name: &str, tags: &[Tag]");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "name");
        assert_eq!(params[0].rust_type, "String");
        assert_eq!(params[1].name, "tags");
        assert_eq!(params[1].rust_type, "Vec<Tag>");
    }

    #[test]
    fn scan_offload_methods() {
        // Write a fixture .rs file to a temp directory and run the parser
        let tmp = std::env::temp_dir().join("ferro_mcp_offload_test_scan");
        let src_dir = tmp.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let fixture = r#"
#[service(ReportBuilder)]
pub trait ReportsService: Send + Sync {
    #[offload]
    async fn build_monthly(&self, tenant_id: i64, month: Month) -> Report;

    #[offload(queue = "reports")]
    async fn export_csv(&self, tenant_id: i64) -> CsvFile;

    async fn get_status(&self) -> Status;
}
"#;
        fs::write(src_dir.join("reports.rs"), fixture).unwrap();

        // Build the initial ServiceItem as scan_services_from_files would
        let mut services = vec![ServiceItem {
            name: "ReportBuilder".to_string(),
            binding_type: "trait_binding".to_string(),
            methods: Vec::new(),
        }];

        scan_offload_methods_from_files(&tmp, &mut services);

        // Clean up
        let _ = fs::remove_dir_all(&tmp);

        assert_eq!(services.len(), 1);
        let svc = &services[0];
        assert_eq!(
            svc.methods.len(),
            2,
            "should have exactly 2 offload methods"
        );

        let build_monthly = svc.methods.iter().find(|m| m.name == "build_monthly");
        assert!(build_monthly.is_some(), "build_monthly must be present");
        let bm = build_monthly.unwrap();
        assert_eq!(bm.queue, "default");
        assert_eq!(bm.params.len(), 2);

        let export_csv = svc.methods.iter().find(|m| m.name == "export_csv");
        assert!(export_csv.is_some(), "export_csv must be present");
        let ec = export_csv.unwrap();
        assert_eq!(ec.queue, "reports");

        // get_status is non-offload — must be absent
        let get_status = svc.methods.iter().find(|m| m.name == "get_status");
        assert!(get_status.is_none(), "non-offload method must not appear");
    }

    #[test]
    fn scan_offload_methods_multiline_service_attr() {
        let tmp = std::env::temp_dir().join("ferro_mcp_offload_test_multiline");
        let src_dir = tmp.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let fixture = r#"
#[service(
    impl = ReportBuilder,
    fake = FakeBuilder,
)]
pub trait ReportsService: Send + Sync {
    #[offload]
    async fn build_monthly(&self, tenant_id: i64, month: Month) -> Report;

    #[offload(queue = "reports")]
    async fn export_csv(&self, tenant_id: i64) -> CsvFile;

    async fn get_status(&self) -> Status;
}
"#;
        fs::write(src_dir.join("reports.rs"), fixture).unwrap();

        // SC#1: scan_services_from_files must surface the multi-line-attributed service.
        let mut services = scan_services_from_files(&tmp);
        assert!(
            services.iter().any(|s| s.name == "ReportBuilder"),
            "multi-line #[service(...)] must surface ReportBuilder; got: {:?}",
            services.iter().map(|s| &s.name).collect::<Vec<_>>()
        );

        // SC#2: its #[offload] methods must correlate to that service.
        scan_offload_methods_from_files(&tmp, &mut services);
        let _ = fs::remove_dir_all(&tmp);

        let svc = services.iter().find(|s| s.name == "ReportBuilder").unwrap();
        assert_eq!(svc.methods.len(), 2, "should correlate exactly 2 offload methods");
        assert!(svc.methods.iter().any(|m| m.name == "build_monthly"));
        let export = svc.methods.iter().find(|m| m.name == "export_csv").unwrap();
        assert_eq!(export.queue, "reports");
        assert!(
            svc.methods.iter().all(|m| m.name != "get_status"),
            "non-offload method must not appear"
        );
    }

    #[test]
    fn normalize_service_lines() {
        // Single-line positional passes through unchanged
        let out = super::normalize_service_lines("#[service(ReportBuilder)]");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].trim(), "#[service(ReportBuilder)]");

        // Single-line named passes through unchanged
        let out = super::normalize_service_lines("#[service(impl = X, fake = Y)]");
        assert_eq!(out.len(), 1);

        // Multi-line positional collapses to one logical line
        let src = "#[service(\n    ReportBuilder\n)]";
        let out = super::normalize_service_lines(src);
        assert_eq!(out.len(), 1);
        assert!(out[0].contains("#[service("));
        assert!(out[0].contains(')'));

        // Multi-line named with trailing comma collapses; inner extracts to ReportBuilder
        let src = "#[service(\n    impl = ReportBuilder,\n    fake = FakeBuilder,\n)]";
        let out = super::normalize_service_lines(src);
        assert_eq!(out.len(), 1);
        let t = out[0].trim();
        let start = t.find('(').unwrap();
        let end = t.find(')').unwrap();
        assert_eq!(extract_service_impl_name(&t[start + 1..end]), "ReportBuilder");

        // Ordinary lines and a comment containing the prefix pass through unchanged
        let src = "pub trait Foo {}\n// see #[service(Bar)]\nasync fn x() {}";
        let out = super::normalize_service_lines(src);
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn plain_service_unchanged() {
        let item = ServiceItem {
            name: "MailerService".to_string(),
            binding_type: "trait_binding".to_string(),
            methods: Vec::new(),
        };
        let json = serde_json::to_string(&item).unwrap();
        // Must NOT contain a "methods" key
        assert!(
            !json.contains("\"methods\""),
            "plain service must not serialize the methods field; got: {json}"
        );
        // Must contain the two base fields
        assert!(json.contains("\"name\":\"MailerService\""));
        assert!(json.contains("\"binding_type\":\"trait_binding\""));
    }
}
