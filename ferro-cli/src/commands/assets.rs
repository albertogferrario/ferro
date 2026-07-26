use clap::Subcommand;
use reqwest::blocking::Client;
use reqwest::Url;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Manage project assets (Iconify icons, Fontsource fonts).
#[derive(Subcommand)]
pub enum AssetsCommand {
    /// Download Iconify icon sets or Fontsource font families into the asset directory
    Fetch {
        #[command(subcommand)]
        source: FetchSource,
    },
}

/// Source provider for `ferro assets fetch`.
#[derive(Subcommand)]
pub enum FetchSource {
    /// Fetch an Iconify set (e.g. `heroicons`) or a specific icon (`heroicons/check`)
    Iconify {
        /// Icon set prefix, optionally `prefix/icon`
        set: String,
        /// Output directory (default: assets/)
        #[arg(long, default_value = "assets")]
        out: String,
    },
    /// Fetch a Fontsource font family (e.g. `inter`, `open-sans`)
    Fontsource {
        /// Font family id (e.g. `inter`, `open-sans`)
        family: String,
        /// Comma-separated weights (default: 400)
        #[arg(long, default_value = "400", value_delimiter = ',')]
        weights: Vec<u32>,
        /// Comma-separated subsets (default: latin)
        #[arg(long, default_value = "latin", value_delimiter = ',')]
        subsets: Vec<String>,
        /// Output directory (default: assets/)
        #[arg(long, default_value = "assets")]
        out: String,
    },
}

/// Reject any name segment that could escape the fixed API host or the output
/// directory. Allowed: ASCII lowercase alphanumerics and `-` (matches Iconify
/// prefixes and Fontsource family ids). One `/` separator is allowed ONLY for
/// the `prefix/icon` Iconify form and is validated per-segment.
fn validate_segment(seg: &str) -> anyhow::Result<()> {
    if seg.is_empty() {
        anyhow::bail!("empty name segment");
    }
    if !seg
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!(
            "invalid name segment {seg:?}: only lowercase ascii letters, digits, and '-' are allowed"
        );
    }
    Ok(())
}

/// Write a single SVG file under `out_dir/{prefix}/{name}.svg`.
///
/// Both `prefix` and `name` are validated before being used as path components.
/// Returns the path written.
pub fn write_icon(out_dir: &Path, prefix: &str, name: &str, svg: &str) -> anyhow::Result<PathBuf> {
    validate_segment(prefix)?;
    validate_segment(name)?;
    let dest = out_dir.join(prefix).join(format!("{name}.svg"));
    std::fs::create_dir_all(dest.parent().expect("parent always exists for file path"))?;
    std::fs::write(&dest, svg)?;
    Ok(dest)
}

/// Validate a woff2 download URL returned by the Fontsource API.
///
/// Requires HTTPS and a host in the known Fontsource CDN allowlist.
/// Rejects non-HTTPS schemes, bare IP addresses, and any host not in the list,
/// preventing SSRF via a malicious or compromised API response.
fn validate_woff2_url(url: &str) -> anyhow::Result<()> {
    let parsed = Url::parse(url).map_err(|_| anyhow::anyhow!("invalid woff2 URL: {url:?}"))?;
    if parsed.scheme() != "https" {
        anyhow::bail!("woff2 URL must use HTTPS; got scheme {:?}", parsed.scheme());
    }
    let host = parsed.host_str().unwrap_or("");
    // Fontsource CDN hosts only. This allowlist is the SSRF control: any URL
    // returned by the API that points elsewhere is rejected.
    if !matches!(host, "cdn.fontsource.com" | "api.fontsource.org") {
        anyhow::bail!("woff2 URL host {host:?} is not an allowed Fontsource host");
    }
    Ok(())
}

/// Return the destination path for a woff2 file without writing it.
///
/// Shape: `{out_dir}/{family}/{subset}-{weight}-normal.woff2`
pub fn woff2_dest(out_dir: &Path, family: &str, subset: &str, weight: u32) -> PathBuf {
    out_dir
        .join(family)
        .join(format!("{subset}-{weight}-normal.woff2"))
}

/// Return `true` if the SVG body fragment from an Iconify set response is safe
/// to embed in a reconstructed `<svg>` document.
///
/// Rejects bodies that contain patterns that could become stored-XSS when the
/// generated SVG files are served to browsers: `<script`, `<foreignobject`,
/// `javascript:` URIs, and HTML event-handler attributes (`on*=`).
/// Matching is case-insensitive.
fn is_safe_svg_body(body: &str) -> bool {
    use regex::Regex;
    use std::sync::OnceLock;
    static EVENT_HANDLER_RE: OnceLock<Regex> = OnceLock::new();
    let re = EVENT_HANDLER_RE
        .get_or_init(|| Regex::new(r"(?i)\son[a-z]+=").expect("static regex is valid"));
    let lower = body.to_ascii_lowercase();
    !lower.contains("<script")
        && !lower.contains("<foreignobject")
        && !lower.contains("javascript:")
        && !re.is_match(body)
}

fn fetch_iconify(client: &Client, set: &str, out_dir: &Path) -> anyhow::Result<()> {
    // Split into at most 2 segments on '/'
    let parts: Vec<&str> = set.splitn(3, '/').collect();
    match parts.as_slice() {
        [prefix, icon] => {
            // Single icon: GET https://api.iconify.design/{prefix}/{icon}.svg
            validate_segment(prefix)?;
            validate_segment(icon)?;
            let url = format!("https://api.iconify.design/{prefix}/{icon}.svg");
            let svg = client.get(&url).send()?.error_for_status()?.text()?;
            let dest = write_icon(out_dir, prefix, icon, &svg)?;
            println!("wrote {}", dest.display());
        }
        [prefix] => {
            // Full set: GET https://api.iconify.design/{prefix}.json
            validate_segment(prefix)?;
            let url = format!("https://api.iconify.design/{prefix}.json");
            let body: Value = client.get(&url).send()?.error_for_status()?.json()?;
            let icons = body["icons"]
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("Iconify response missing 'icons' object"))?;
            let default_w = body["width"].as_u64().unwrap_or(24);
            let default_h = body["height"].as_u64().unwrap_or(24);
            let mut count = 0usize;
            for (name, def) in icons {
                // Defense-in-depth: validate API-returned keys before use in paths
                if validate_segment(name).is_err() {
                    eprintln!("warning: skipping icon with invalid name {name:?}");
                    continue;
                }
                let icon_body = def["body"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("icon {name:?} missing 'body' field"))?;
                if !is_safe_svg_body(icon_body) {
                    eprintln!(
                        "warning: skipping icon {name:?} — body contains potentially unsafe content"
                    );
                    continue;
                }
                let w = def["width"].as_u64().unwrap_or(default_w);
                let h = def["height"].as_u64().unwrap_or(default_h);
                let svg = format!(
                    "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\">{icon_body}</svg>"
                );
                write_icon(out_dir, prefix, name, &svg)?;
                count += 1;
            }
            println!("wrote {count} icons to {}/{prefix}/", out_dir.display());
        }
        _ => {
            anyhow::bail!(
                "invalid set format {set:?}: expected 'prefix' or 'prefix/icon' (at most one '/')"
            );
        }
    }
    Ok(())
}

fn fetch_fontsource(
    client: &Client,
    family: &str,
    weights: &[u32],
    subsets: &[&str],
    out_dir: &Path,
) -> anyhow::Result<()> {
    validate_segment(family)?;
    for subset in subsets {
        validate_segment(subset)?;
    }

    let meta_url = format!("https://api.fontsource.org/v1/fonts/{family}");
    let meta: Value = client.get(&meta_url).send()?.error_for_status()?.json()?;

    let variants = meta["variants"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Fontsource response for '{family}' missing 'variants'"))?;

    let mut count = 0usize;
    for weight in weights {
        let w_key = weight.to_string();
        let styles = match variants.get(&w_key).and_then(|v| v.as_object()) {
            Some(s) => s,
            None => {
                eprintln!("warning: weight {weight} not available for '{family}', skipping");
                continue;
            }
        };
        let normal = match styles.get("normal").and_then(|v| v.as_object()) {
            Some(n) => n,
            None => {
                eprintln!(
                    "warning: normal style for weight {weight} not available for '{family}', skipping"
                );
                continue;
            }
        };
        for subset in subsets {
            let url_obj = match normal
                .get(*subset)
                .and_then(|v| v.get("url"))
                .and_then(|v| v.as_object())
            {
                Some(u) => u,
                None => {
                    eprintln!(
                        "warning: subset '{subset}' weight {weight} not available for '{family}', skipping"
                    );
                    continue;
                }
            };
            let woff2_url = match url_obj.get("woff2").and_then(|v| v.as_str()) {
                Some(u) => u,
                None => {
                    eprintln!(
                        "warning: no woff2 URL for '{family}' subset '{subset}' weight {weight}, skipping"
                    );
                    continue;
                }
            };
            validate_woff2_url(woff2_url)?;
            let bytes = client.get(woff2_url).send()?.error_for_status()?.bytes()?;
            let dest = woff2_dest(out_dir, family, subset, *weight);
            std::fs::create_dir_all(dest.parent().expect("parent always exists"))?;
            std::fs::write(&dest, &bytes)?;
            println!("wrote {}", dest.display());
            count += 1;
        }
    }
    println!(
        "wrote {count} font file(s) to {}/{family}/",
        out_dir.display()
    );
    Ok(())
}

/// Entry point called from main.rs dispatch.
pub fn run(subcommand: AssetsCommand) {
    if let Err(e) = run_inner(subcommand) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run_inner(subcommand: AssetsCommand) -> anyhow::Result<()> {
    let client = Client::new();
    match subcommand {
        AssetsCommand::Fetch { source } => match source {
            FetchSource::Iconify { set, out } => fetch_iconify(&client, &set, Path::new(&out)),
            FetchSource::Fontsource {
                family,
                weights,
                subsets,
                out,
            } => {
                let subset_refs: Vec<&str> = subsets.iter().map(|s| s.as_str()).collect();
                fetch_fontsource(&client, &family, &weights, &subset_refs, Path::new(&out))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{is_safe_svg_body, validate_segment, validate_woff2_url, woff2_dest, write_icon};

    // ── validate_segment ────────────────────────────────────────────────────

    #[test]
    fn rejects_traversal_and_host_injection() {
        assert!(validate_segment("..").is_err());
        assert!(validate_segment("evil.com").is_err()); // '.' rejected
        assert!(validate_segment("a/b").is_err()); // '/' rejected at segment level
        assert!(validate_segment("A").is_err()); // uppercase rejected
        assert!(validate_segment("a%2e").is_err()); // '%' rejected
        assert!(validate_segment("").is_err());
    }

    #[test]
    fn accepts_valid_names() {
        assert!(validate_segment("heroicons").is_ok());
        assert!(validate_segment("open-sans").is_ok());
        assert!(validate_segment("check").is_ok());
        assert!(validate_segment("inter").is_ok());
        assert!(validate_segment("123abc").is_ok());
        assert!(validate_segment("a-b-c").is_ok());
    }

    // ── write_icon (tempdir) ─────────────────────────────────────────────────

    #[test]
    fn write_icon_lands_under_out_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dest = write_icon(tmp.path(), "heroicons", "check", "<svg/>").unwrap();
        assert!(dest.starts_with(tmp.path()), "dest {dest:?} not under tmp");
        assert_eq!(dest.extension().unwrap(), "svg");
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "<svg/>");
    }

    #[test]
    fn write_icon_rejects_invalid_prefix() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(write_icon(tmp.path(), "../evil", "check", "<svg/>").is_err());
    }

    #[test]
    fn write_icon_rejects_invalid_name() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(write_icon(tmp.path(), "heroicons", "../escape", "<svg/>").is_err());
    }

    // ── woff2_dest (tempdir) ─────────────────────────────────────────────────

    #[test]
    fn woff2_dest_is_expected_shape() {
        let tmp = tempfile::TempDir::new().unwrap();
        let d = woff2_dest(tmp.path(), "inter", "latin", 400);
        assert!(d.starts_with(tmp.path()), "dest {d:?} not under tmp");
        assert!(
            d.ends_with("inter/latin-400-normal.woff2"),
            "unexpected path: {d:?}"
        );
    }

    #[test]
    fn woff2_dest_varies_by_weight() {
        let tmp = tempfile::TempDir::new().unwrap();
        let d700 = woff2_dest(tmp.path(), "inter", "latin", 700);
        assert!(d700.ends_with("inter/latin-700-normal.woff2"));
    }

    // ── validate_woff2_url ──────────────────────────────────────────────────

    #[test]
    fn woff2_url_accepts_allowed_cdn_hosts() {
        assert!(validate_woff2_url(
            "https://cdn.fontsource.com/fonts/inter/latin-400-normal.woff2"
        )
        .is_ok());
        assert!(validate_woff2_url(
            "https://api.fontsource.org/v1/fonts/inter/latin-400-normal.woff2"
        )
        .is_ok());
    }

    #[test]
    fn woff2_url_rejects_non_allowlisted_host() {
        assert!(validate_woff2_url("https://evil.example/fonts/inter.woff2").is_err());
    }

    #[test]
    fn woff2_url_rejects_http_scheme() {
        assert!(
            validate_woff2_url("http://cdn.fontsource.com/fonts/inter/latin-400-normal.woff2")
                .is_err()
        );
    }

    #[test]
    fn woff2_url_rejects_file_scheme() {
        assert!(validate_woff2_url("file:///etc/passwd").is_err());
    }

    #[test]
    fn woff2_url_rejects_metadata_endpoint() {
        assert!(validate_woff2_url("http://169.254.169.254/latest/meta-data/").is_err());
    }

    // ── is_safe_svg_body ────────────────────────────────────────────────────

    #[test]
    fn safe_svg_body_accepts_normal_path_data() {
        assert!(is_safe_svg_body(
            r#"<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/>"#
        ));
    }

    #[test]
    fn safe_svg_body_rejects_script_tag() {
        assert!(!is_safe_svg_body("<script>alert(1)</script>"));
        // case-insensitive
        assert!(!is_safe_svg_body("<SCRIPT>alert(1)</SCRIPT>"));
    }

    #[test]
    fn safe_svg_body_rejects_foreign_object() {
        assert!(!is_safe_svg_body(
            r#"<foreignObject width="100"><div>hi</div></foreignObject>"#
        ));
        assert!(!is_safe_svg_body("<FOREIGNOBJECT/>"));
    }

    #[test]
    fn safe_svg_body_rejects_javascript_uri() {
        assert!(!is_safe_svg_body(r#"<a href="javascript:alert(1)"/>"#));
    }

    #[test]
    fn safe_svg_body_rejects_event_handler_attribute() {
        assert!(!is_safe_svg_body(r#"<circle onclick="evil()"/>"#));
        assert!(!is_safe_svg_body(r#"<image onload="x"/>"#));
    }
}
