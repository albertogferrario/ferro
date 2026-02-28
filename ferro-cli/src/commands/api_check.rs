use console::Style;
use reqwest::blocking::Client;
use serde_json::Value;
use std::time::Duration;

/// Validate OpenAPI spec structure and extract metadata.
pub fn validate_openapi_json(json: &Value) -> Result<SpecInfo, String> {
    let version = json
        .get("openapi")
        .and_then(|v| v.as_str())
        .ok_or("OpenAPI spec is malformed: missing `openapi` field")?;

    if !version.starts_with("3.") {
        return Err(format!(
            "OpenAPI spec version `{version}` is not supported (expected 3.x)"
        ));
    }

    let paths = json
        .get("paths")
        .and_then(|v| v.as_object())
        .ok_or("OpenAPI spec is malformed: missing `paths` field")?;

    if paths.is_empty() {
        return Err("OpenAPI spec has no API paths defined".to_string());
    }

    let http_methods = [
        "get", "post", "put", "patch", "delete", "head", "options", "trace",
    ];
    let mut operation_count = 0;
    for (_path, methods) in paths {
        if let Some(obj) = methods.as_object() {
            for key in obj.keys() {
                if http_methods.contains(&key.as_str()) {
                    operation_count += 1;
                }
            }
        }
    }

    if operation_count == 0 {
        return Err("OpenAPI spec has paths but no operations defined".to_string());
    }

    Ok(SpecInfo {
        version: version.to_string(),
        operation_count,
        path_count: paths.len(),
    })
}

/// Metadata extracted from a valid OpenAPI spec.
#[derive(Debug)]
pub struct SpecInfo {
    pub version: String,
    pub operation_count: usize,
    pub path_count: usize,
}

/// Find the first GET endpoint path in an OpenAPI spec, falling back to any method.
fn find_first_endpoint(json: &Value) -> Option<(String, String)> {
    let paths = json.get("paths")?.as_object()?;
    let http_methods = [
        "get", "post", "put", "patch", "delete", "head", "options", "trace",
    ];

    // Prefer GET endpoints
    for (path, methods) in paths {
        if let Some(obj) = methods.as_object() {
            if obj.contains_key("get") {
                return Some((path.clone(), "GET".to_string()));
            }
        }
    }

    // Fall back to any method
    for (path, methods) in paths {
        if let Some(obj) = methods.as_object() {
            for key in obj.keys() {
                if http_methods.contains(&key.as_str()) {
                    return Some((path.clone(), key.to_uppercase()));
                }
            }
        }
    }

    None
}

/// Check local API readiness for MCP integration.
pub fn run(url: String, api_key: Option<String>, spec_path: String) {
    let green = Style::new().green();
    let red = Style::new().red();
    let dim = Style::new().dim();

    let base_url = url.trim_end_matches('/');
    let spec_url = format!("{base_url}{spec_path}");

    println!("Checking API at {base_url}...\n");

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create HTTP client");

    // Check 1: Server connectivity
    match client.get(&spec_url).send() {
        Ok(_) => {
            println!("  {} Server is running", green.apply_to("\u{2713}"));
        }
        Err(_) => {
            println!("  {} Server not reachable", red.apply_to("\u{2717}"));
            println!(
                "    {} Start your server with: cargo run",
                dim.apply_to("\u{2192}")
            );
            println!("\n  Stopped \u{2014} fix the above issues and try again.");
            return;
        }
    }

    // Check 2: OpenAPI spec available
    let spec_response = match client.get(&spec_url).send() {
        Ok(resp) => resp,
        Err(_) => {
            println!(
                "  {} Could not fetch OpenAPI spec",
                red.apply_to("\u{2717}")
            );
            println!("\n  Stopped \u{2014} fix the above issues and try again.");
            return;
        }
    };

    let status = spec_response.status();
    if status.as_u16() == 404 {
        println!(
            "  {} OpenAPI spec not found at {spec_path}",
            red.apply_to("\u{2717}")
        );
        println!(
            "    {} Did you register docs_routes()?",
            dim.apply_to("\u{2192}")
        );
        println!("\n  Stopped \u{2014} fix the above issues and try again.");
        return;
    }

    if !status.is_success() {
        println!(
            "  {} OpenAPI spec returned HTTP {status}",
            red.apply_to("\u{2717}")
        );
        println!("\n  Stopped \u{2014} fix the above issues and try again.");
        return;
    }

    let spec_json: Value = match spec_response.json() {
        Ok(v) => v,
        Err(_) => {
            println!(
                "  {} OpenAPI spec endpoint returned non-JSON response",
                red.apply_to("\u{2717}")
            );
            println!("\n  Stopped \u{2014} fix the above issues and try again.");
            return;
        }
    };

    println!(
        "  {} OpenAPI spec available at {spec_path}",
        green.apply_to("\u{2713}")
    );

    // Check 3: OpenAPI spec valid
    let spec_info = match validate_openapi_json(&spec_json) {
        Ok(info) => info,
        Err(msg) => {
            println!("  {} {msg}", red.apply_to("\u{2717}"));
            println!("\n  Stopped \u{2014} fix the above issues and try again.");
            return;
        }
    };

    println!(
        "  {} Valid OpenAPI {} spec \u{2014} {} operations across {} paths",
        green.apply_to("\u{2713}"),
        spec_info.version,
        spec_info.operation_count,
        spec_info.path_count,
    );

    // Check 4: API key authentication
    if let Some(ref key) = api_key {
        if let Some((endpoint_path, method)) = find_first_endpoint(&spec_json) {
            let endpoint_url = format!("{base_url}{endpoint_path}");
            let request = match method.as_str() {
                "GET" => client.get(&endpoint_url),
                "POST" => client.post(&endpoint_url),
                "PUT" => client.put(&endpoint_url),
                "PATCH" => client.patch(&endpoint_url),
                "DELETE" => client.delete(&endpoint_url),
                "HEAD" => client.head(&endpoint_url),
                _ => client.get(&endpoint_url),
            };

            match request.header("X-API-Key", key).send() {
                Ok(resp) if resp.status().as_u16() == 401 => {
                    println!("  {} API key rejected", red.apply_to("\u{2717}"));
                    println!(
                        "    {} Check if the key is valid and not expired.",
                        dim.apply_to("\u{2192}")
                    );
                    println!("\n  Stopped \u{2014} fix the above issues and try again.");
                    return;
                }
                Ok(_) => {
                    println!(
                        "  {} API key authentication working",
                        green.apply_to("\u{2713}")
                    );
                }
                Err(e) => {
                    println!("  {} Could not test API key: {e}", red.apply_to("\u{2717}"));
                    println!("\n  Stopped \u{2014} fix the above issues and try again.");
                    return;
                }
            }
        } else {
            println!(
                "  {} No API endpoints found to test authentication",
                red.apply_to("\u{2717}")
            );
            println!("\n  Stopped \u{2014} fix the above issues and try again.");
            return;
        }
    } else {
        println!("  {} API key authentication", dim.apply_to("-"));
        println!(
            "    {} Skipped \u{2014} provide --api-key to test authentication",
            dim.apply_to("\u{2192}")
        );
    }

    // Summary
    println!();
    println!("  Ready for MCP! Configure ferro-api-mcp:");
    if let Some(ref key) = api_key {
        println!("    ferro-api-mcp --spec-url {spec_url} --api-key {key}");
    } else {
        println!("    ferro-api-mcp --spec-url {spec_url} --api-key <your-key>");
    }
}
