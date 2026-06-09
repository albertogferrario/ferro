//! Projection checkpoint: walks the intent-slice spine, owns the field→column seam,
//! aggregates a single verdict. Read-only and introspective — no cargo/compile,
//! no code mutation.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::inspect_projection::InspectResult;
use super::list_models;
use super::render_projection::reconstruct_service_def;

// ---------------------------------------------------------------------------
// Public output-contract types (D-07 locked shape, reused verbatim by Phase 195)
// ---------------------------------------------------------------------------

/// A single actionable finding from a seam check.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Finding {
    /// The field, entity, or structural element the finding is about.
    pub subject: String,
    /// Human-readable description of the problem.
    pub detail: String,
    /// Concrete remediation step an agent can act on without a second call.
    pub fix: String,
}

/// Per-seam status value.
///
/// `NotChecked` is a distinct variant — it must never be coerced to `Pass`
/// (CHK-03 coverage-honesty invariant). Prerequisite-absent paths must return
/// `NotChecked`, not `Pass`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SeamStatus {
    Pass,
    Warn,
    Fail,
    NotChecked,
}

/// Result for a single seam in the checkpoint run.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SeamResult {
    /// Seam identifier (e.g. "field_to_column").
    pub seam: String,
    pub status: SeamStatus,
    /// Provenance tag (e.g. "checkpoint").
    pub source: String,
    pub findings: Vec<Finding>,
    /// Populated for `not_checked` or `warn` outcomes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Aggregated checkpoint output for one projection.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Verdict {
    /// Aggregate status: `fail` if any seam fails, `warn` if any warns, `pass` otherwise.
    /// `not_checked` seams are listed but do not raise the aggregate to `fail`.
    pub status: SeamStatus,
    /// Projection name as supplied to the tool.
    pub projection: String,
    pub seams: Vec<SeamResult>,
    /// Ranked, deduplicated actionable strings (failures before warnings; cap 10).
    pub next_steps: Vec<String>,
}

// ---------------------------------------------------------------------------
// Name validation (T-194-01: path traversal guard, used by Plan 03 cache write)
// ---------------------------------------------------------------------------

/// Validate that `name` is safe to use as a filename stem in
/// `.ferro/checkpoints/{name}.json`. Rejects path separators, parent refs,
/// and null bytes — only `[a-zA-Z0-9_-]` is permitted.
pub(crate) fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("projection name must not be empty".to_string());
    }
    if name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        Ok(())
    } else {
        Err(format!(
            "invalid projection name `{name}`: only alphanumerics, underscore, and hyphen are allowed"
        ))
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the projection checkpoint against `name` in `project_root`.
///
/// Calls `run_for` with the current UTC timestamp.
pub fn execute(project_root: &Path, name: &str) -> Result<Verdict, String> {
    run_for(project_root, name, chrono::Utc::now())
}

/// Testable inner implementation accepting an injected timestamp (D-11: do not
/// read wall-clock inside pure logic).
pub(crate) fn run_for(
    project_root: &Path,
    name: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<Verdict, String> {
    validate_name(name)?;

    // 1. Locate the projection source file via inspect_projection.
    let inspect = super::inspect_projection::execute(project_root, name);
    let detail = match inspect {
        InspectResult::Found(d) => d,
        InspectResult::NotFound(nf) => {
            return Err(format!(
                "projection '{}' not found. Available: {:?}",
                nf.name, nf.available
            ))
        }
    };

    // 2. Read source.
    let file_path = project_root.join(&detail.file);
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("failed to read {}: {e}", detail.file))?;

    // 3. Run seam 2: field→column presence check.
    let seam2 = field_to_column_seam(project_root, &detail.service_name, &detail.display_name, &content);

    // 4. Stubs for seams 1/3/4/5 (Phase 195 fills these).
    //    Seam cascade rule (locked in STATE.md):
    //    - seam 1 fail → seams 4 and 5 become not_checked("seam_1_failed")
    //    - seam 4 fail → seam 5 becomes not_checked("seam_4_failed")
    //    - seams 2 and 3 run independently of seam 1.
    let seam1 = SeamResult {
        seam: "schema_load".to_string(),
        status: SeamStatus::NotChecked,
        source: "checkpoint".to_string(),
        findings: vec![],
        reason: Some("not_implemented_phase_195".to_string()),
    };
    let seam3 = SeamResult {
        seam: "field_type_compat".to_string(),
        status: SeamStatus::NotChecked,
        source: "checkpoint".to_string(),
        findings: vec![],
        reason: Some("not_implemented_phase_195".to_string()),
    };
    let seam4 = SeamResult {
        seam: "action_binding".to_string(),
        status: SeamStatus::NotChecked,
        source: "checkpoint".to_string(),
        findings: vec![],
        reason: Some("not_implemented_phase_195".to_string()),
    };
    let seam5 = SeamResult {
        seam: "render_target".to_string(),
        status: SeamStatus::NotChecked,
        source: "checkpoint".to_string(),
        findings: vec![],
        reason: Some("not_implemented_phase_195".to_string()),
    };

    // 5. Aggregate verdict (D-09).
    let seams = vec![seam1, seam2, seam3, seam4, seam5];
    let next_steps = aggregate_next_steps(&seams);
    let status = aggregate_status(&seams);

    let verdict = Verdict {
        status,
        projection: name.to_string(),
        seams,
        next_steps,
    };

    // 6. Write status cache (D-11).
    write_cache(project_root, name, &verdict, now)?;

    Ok(verdict)
}

// ---------------------------------------------------------------------------
// Seam 2: field→column presence check
// ---------------------------------------------------------------------------

fn field_to_column_seam(
    project_root: &Path,
    service_name: &str,
    display_name: &Option<String>,
    content: &str,
) -> SeamResult {
    // Reconstruct ServiceDef — not_checked on failure (D-03, Pitfall 1).
    let service = match reconstruct_service_def(service_name, display_name, content) {
        Ok(s) => s,
        Err(e) => {
            return SeamResult {
                seam: "field_to_column".to_string(),
                status: SeamStatus::NotChecked,
                source: "checkpoint".to_string(),
                findings: vec![],
                reason: Some(format!("reconstruction_failed: {e}")),
            }
        }
    };

    // D-06: completeness check — if source has more builder calls than parsed fields, warn.
    let invocation_count = count_column_backed_builders(content);
    if invocation_count > service.fields.len() {
        return SeamResult {
            seam: "field_to_column".to_string(),
            status: SeamStatus::Warn,
            source: "checkpoint".to_string(),
            findings: vec![Finding {
                subject: service_name.to_string(),
                detail: format!(
                    "reconstruction may be incomplete: {} builder calls in source, {} fields parsed",
                    invocation_count,
                    service.fields.len()
                ),
                fix: "check for unsupported builder patterns in the projection source".to_string(),
            }],
            reason: Some("reconstruction_incomplete".to_string()),
        };
    }

    // D-01: resolve model by name-match.
    let models = match list_models::execute(project_root) {
        Ok(m) => m,
        Err(_) => {
            return SeamResult {
                seam: "field_to_column".to_string(),
                status: SeamStatus::NotChecked,
                source: "checkpoint".to_string(),
                findings: vec![],
                reason: Some("source_model_unresolved".to_string()),
            }
        }
    };

    let model = match models
        .iter()
        .find(|m| m.name.to_lowercase() == service_name.to_lowercase())
    {
        Some(m) => m,
        None => {
            return SeamResult {
                seam: "field_to_column".to_string(),
                status: SeamStatus::NotChecked,
                source: "checkpoint".to_string(),
                findings: vec![],
                reason: Some("source_model_unresolved".to_string()),
            }
        }
    };

    // Build column set from model's FieldInfo (D-02: case-sensitive snake_case match).
    let column_names: std::collections::HashSet<&str> =
        model.fields.iter().map(|f| f.name.as_str()).collect();

    // D-04: service.fields excludes relationships (they are in service.relationships).
    let mut findings = Vec::new();
    for field in &service.fields {
        if !column_names.contains(field.name.as_str()) {
            findings.push(Finding {
                subject: field.name.clone(),
                detail: format!(
                    "no column `{}` on entity `{}`",
                    field.name,
                    service_name.to_lowercase()
                ),
                fix: format!(
                    "add column `{}` to `{}` migration, or remove the field from the projection",
                    field.name,
                    service_name.to_lowercase()
                ),
            });
        }
    }

    SeamResult {
        seam: "field_to_column".to_string(),
        status: if findings.is_empty() {
            SeamStatus::Pass
        } else {
            SeamStatus::Fail
        },
        source: "checkpoint".to_string(),
        findings,
        reason: None,
    }
}

// ---------------------------------------------------------------------------
// Builder invocation count (D-06 completeness check)
// ---------------------------------------------------------------------------

/// Count column-backed builder invocations in `content`, stripping `//` line
/// comments first to avoid matching commented-out calls (RESEARCH.md Pitfall 2).
///
/// Four builders counted (D-05 vocabulary — all are column-backed):
/// `.field(`, `.optional_field(`, `.read_only_field(`, `.write_only_field(`
fn count_column_backed_builders(content: &str) -> usize {
    // Strip // line comments before counting.
    let no_comments: String = content
        .lines()
        .map(|line| {
            if let Some(pos) = line.find("//") {
                &line[..pos]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let patterns = [
        r"\.field\(",
        r"\.optional_field\(",
        r"\.read_only_field\(",
        r"\.write_only_field\(",
    ];
    patterns
        .iter()
        .map(|p| regex::Regex::new(p).unwrap().find_iter(&no_comments).count())
        .sum()
}

// ---------------------------------------------------------------------------
// Verdict aggregation (D-09/D-10)
// ---------------------------------------------------------------------------

/// Aggregate overall status: fail if any seam fails, warn if any warns, pass otherwise.
/// `not_checked` seams never raise the overall status to fail (CHK-03).
fn aggregate_status(seams: &[SeamResult]) -> SeamStatus {
    let mut has_warn = false;
    for seam in seams {
        match seam.status {
            SeamStatus::Fail => return SeamStatus::Fail,
            SeamStatus::Warn => has_warn = true,
            SeamStatus::Pass | SeamStatus::NotChecked => {}
        }
    }
    if has_warn {
        SeamStatus::Warn
    } else {
        SeamStatus::Pass
    }
}

/// Build ranked, deduplicated, capped next_steps list (D-10).
///
/// Failures before warnings; within a rank, earlier seam first.
/// Dedup by `(subject, fix)`. Cap at 10.
fn aggregate_next_steps(seams: &[SeamResult]) -> Vec<String> {
    let mut items: Vec<(u8, usize, String, String, String)> = Vec::new();
    for (idx, seam) in seams.iter().enumerate() {
        let rank: u8 = match seam.status {
            SeamStatus::Fail => 0,
            SeamStatus::Warn => 1,
            _ => continue,
        };
        for finding in &seam.findings {
            let entry = format!("{} (seam: {})", finding.fix, seam.seam);
            items.push((rank, idx, finding.subject.clone(), finding.fix.clone(), entry));
        }
    }
    items.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for (_, _, subject, fix, entry) in items {
        if seen.insert((subject, fix)) {
            result.push(entry);
            if result.len() == 10 {
                break;
            }
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Status cache write (D-11)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct CacheEntry<'a> {
    verdict: &'a Verdict,
    ambient_status: &'static str,
    checked_at: chrono::DateTime<chrono::Utc>,
}

fn write_cache(
    project_root: &Path,
    name: &str,
    verdict: &Verdict,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(), String> {
    let ambient_status = match verdict.status {
        SeamStatus::Pass => "clean",
        _ => "failing",
    };
    let entry = CacheEntry {
        verdict,
        ambient_status,
        checked_at: now,
    };
    let cache_dir = project_root.join(".ferro").join("checkpoints");
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("failed to create cache dir: {e}"))?;
    let path = cache_dir.join(format!("{name}.json"));
    let json =
        serde_json::to_string_pretty(&entry).map_err(|e| format!("failed to serialize cache: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("failed to write cache: {e}"))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a temp project root with a projection source under src/projections/.
    /// Returns the tempdir (keep it alive for the test duration).
    fn project_with_projection(name: &str, projection_src: &str) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let proj_dir = tmp.path().join("src/projections");
        std::fs::create_dir_all(&proj_dir).unwrap();
        std::fs::write(proj_dir.join(format!("{name}.rs")), projection_src).unwrap();
        tmp
    }

    /// Add a SeaORM-style model source under src/models/ to an existing temp root.
    fn add_model(tmp: &tempfile::TempDir, name: &str, model_src: &str) {
        let models_dir = tmp.path().join("src/models");
        std::fs::create_dir_all(&models_dir).unwrap();
        std::fs::write(models_dir.join(format!("{name}.rs")), model_src).unwrap();
    }

    #[test]
    fn verdict_shape() {
        let v = Verdict {
            status: SeamStatus::Pass,
            projection: "booking".to_string(),
            seams: vec![],
            next_steps: vec![],
        };
        let val = serde_json::to_value(&v).unwrap();
        assert_eq!(val["status"], "pass");
        assert!(val.get("projection").is_some());
        assert!(val.get("seams").is_some());
        assert!(val.get("next_steps").is_some());
    }

    #[test]
    fn seamstatus_wire() {
        assert_eq!(serde_json::to_string(&SeamStatus::Pass).unwrap(), "\"pass\"");
        assert_eq!(serde_json::to_string(&SeamStatus::Warn).unwrap(), "\"warn\"");
        assert_eq!(serde_json::to_string(&SeamStatus::Fail).unwrap(), "\"fail\"");
        assert_eq!(
            serde_json::to_string(&SeamStatus::NotChecked).unwrap(),
            "\"not_checked\""
        );
    }

    #[test]
    fn name_validation() {
        assert!(validate_name("../../etc/passwd").is_err());
        assert!(validate_name("foo/bar").is_err());
        assert!(validate_name("foo\0bar").is_err());
        assert!(validate_name("").is_err());
        assert!(validate_name("Booking").is_ok());
        assert!(validate_name("user_service-1").is_ok());
    }
}
