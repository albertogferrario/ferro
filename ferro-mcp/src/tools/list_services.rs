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
pub struct ServiceItem {
    /// Service name (trait or concrete type)
    pub name: String,
    /// Type of binding (trait_binding or singleton)
    pub binding_type: String,
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
            })
            .collect(),
    )
}

pub async fn execute(project_root: &Path) -> Result<ServicesInfo> {
    // Try runtime endpoint first
    for base_url in ["http://localhost:8080", "http://127.0.0.1:8080"] {
        if let Some(services) = fetch_runtime_services(base_url).await {
            return Ok(ServicesInfo {
                services,
                source: ServiceSource::Runtime,
            });
        }
    }

    // Fall back to static analysis
    let services = scan_services_from_files(project_root);
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
            // Find #[service(ConcreteType)] on traits
            for line in content.lines() {
                let trimmed = line.trim();

                // Match #[service(SomeType)]
                if trimmed.starts_with("#[service(") {
                    if let Some(start) = trimmed.find('(') {
                        if let Some(end) = trimmed.find(')') {
                            let impl_name = &trimmed[start + 1..end];
                            services.push(ServiceItem {
                                name: impl_name.trim().to_string(),
                                binding_type: "trait_binding".to_string(),
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
                            });
                        }
                    }
                }
            }
        }
    }

    services
}
