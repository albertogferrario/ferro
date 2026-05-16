//! Integration tests for the `ferro json-ui:migrate-v1` codemod (Plan 163-07).
//!
//! These tests mutate `std::env::current_dir` (the codemod writes spec files
//! under the cwd). cwd is process-global, so a shared Mutex serializes the
//! tests. The Mutex pattern matches `docker_init_dry_run.rs`. The integration
//! tests run in the same binary, so the Mutex is sufficient — no `--test-threads=1`
//! requirement.

use std::path::PathBuf;
use std::sync::Mutex;

use tempfile::TempDir;

use ferro_cli::commands::json_ui_migrate_v1;

// chdir is process-global; serialize tests that touch it.
static CHDIR_LOCK: Mutex<()> = Mutex::new(());

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("migrate_v1")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing fixture: {}", path.display()))
}

fn write_fixture(dir: &TempDir, source_name: &str, dest_name: &str) -> PathBuf {
    let content = fixture(source_name);
    let dest = dir.path().join(dest_name);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&dest, content).unwrap();
    dest
}

/// Best-effort cwd guard: restores cwd on drop. Acquires CHDIR_LOCK on
/// construction so concurrent tests do not race on cwd.
struct ChangeCwd<'a> {
    original: PathBuf,
    _guard: std::sync::MutexGuard<'a, ()>,
}

impl<'a> ChangeCwd<'a> {
    fn new(new_dir: &std::path::Path) -> Self {
        let guard = CHDIR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(new_dir).unwrap();
        Self {
            original,
            _guard: guard,
        }
    }
}

impl Drop for ChangeCwd<'_> {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}

#[test]
fn codemod_one_handler_emits_spec_and_rewrites_controller() {
    // Phase 163.1 (WR-01): multi-root v1 handlers are rejected as Unsupported.
    // in_auth.rs has TWO top-level nodes (page-title + login-form), so the
    // codemod must emit the TODO marker on the controller and produce NO
    // JSON spec file.
    let dir = TempDir::new().unwrap();
    let src_path = write_fixture(&dir, "in_auth.rs", "src/controllers/in_auth.rs");
    let _cwd_guard = ChangeCwd::new(dir.path());

    json_ui_migrate_v1::run(src_path.to_string_lossy().to_string(), false).expect("codemod runs");

    // No spec file emitted for the multi-root case.
    let spec_path = dir.path().join("src/views/in_auth/login_form.json");
    assert!(
        !spec_path.exists(),
        "no spec written for multi-root handler; unexpected file at {}",
        spec_path.display()
    );

    // Controller body retains the TODO marker above the handler.
    let rewritten = std::fs::read_to_string(&src_path).unwrap();
    assert!(
        rewritten.contains("// TODO: ferro json-ui:migrate-v1 could not auto-translate this handler"),
        "TODO marker present on multi-root handler; got:\n{rewritten}"
    );
    // Controller body for the handler is left intact (no render_file rewrite).
    assert!(
        !rewritten.contains("JsonUi::render_file"),
        "multi-root handler must not be rewritten to render_file; got:\n{rewritten}"
    );
}

#[test]
fn codemod_single_root_emits_spec_and_rewrites_controller() {
    // Phase 163.1: a v1 handler with exactly ONE top-level node is still
    // successfully migrated — produces a JSON spec under src/views/ and
    // rewrites the controller to call JsonUi::render_file.
    let dir = TempDir::new().unwrap();
    let src_path =
        write_fixture(&dir, "in_single_root.rs", "src/controllers/in_single_root.rs");
    let _cwd_guard = ChangeCwd::new(dir.path());

    json_ui_migrate_v1::run(src_path.to_string_lossy().to_string(), false).expect("codemod runs");

    let spec_path = dir.path().join("src/views/in_single_root/dashboard.json");
    assert!(
        spec_path.exists(),
        "spec written to {}",
        spec_path.display()
    );

    let actual_spec: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&spec_path).unwrap()).unwrap();
    let expected_spec: serde_json::Value =
        serde_json::from_str(&fixture("out_single_root_dashboard.json")).unwrap();
    assert_eq!(actual_spec, expected_spec, "spec content matches fixture");

    let actual_controller = std::fs::read_to_string(&src_path).unwrap();
    let expected_controller = fixture("out_single_root.rs");
    assert_eq!(actual_controller.trim(), expected_controller.trim());
}

#[test]
fn codemod_dry_run_does_not_write() {
    let dir = TempDir::new().unwrap();
    let src_path = write_fixture(&dir, "in_auth.rs", "src/controllers/in_auth.rs");
    let before = std::fs::read_to_string(&src_path).unwrap();
    let _cwd_guard = ChangeCwd::new(dir.path());

    json_ui_migrate_v1::run(src_path.to_string_lossy().to_string(), true)
        .expect("codemod runs in dry-run mode");

    let after = std::fs::read_to_string(&src_path).unwrap();
    assert_eq!(before, after, "controller unchanged in dry-run");
    let spec_path = dir.path().join("src/views/in_auth/login_form.json");
    assert!(!spec_path.exists(), "no spec written in dry-run");
}

#[test]
fn codemod_idempotent_on_migrated_file() {
    let dir = TempDir::new().unwrap();
    let src_path = write_fixture(&dir, "in_auth.rs", "src/controllers/in_auth.rs");
    let _cwd_guard = ChangeCwd::new(dir.path());

    json_ui_migrate_v1::run(src_path.to_string_lossy().to_string(), false).unwrap();
    let after_first = std::fs::read_to_string(&src_path).unwrap();

    // Second run — must be a no-op.
    json_ui_migrate_v1::run(src_path.to_string_lossy().to_string(), false).unwrap();
    let after_second = std::fs::read_to_string(&src_path).unwrap();
    assert_eq!(
        after_first, after_second,
        "idempotent re-run produces identical output"
    );
}

#[test]
fn codemod_runtime_branch_emits_todo_marker() {
    let dir = TempDir::new().unwrap();
    let src_path = write_fixture(
        &dir,
        "in_with_runtime_branch.rs",
        "src/controllers/branchy.rs",
    );
    let _cwd_guard = ChangeCwd::new(dir.path());

    json_ui_migrate_v1::run(src_path.to_string_lossy().to_string(), false).expect("codemod runs");

    let after = std::fs::read_to_string(&src_path).unwrap();
    assert!(
        after.contains("// TODO: ferro json-ui:migrate-v1 could not auto-translate this handler"),
        "TODO marker present; got:\n{after}"
    );
}

#[test]
fn codemod_writes_no_spec_for_unsupported_handler() {
    let dir = TempDir::new().unwrap();
    let src_path = write_fixture(
        &dir,
        "in_with_runtime_branch.rs",
        "src/controllers/branchy.rs",
    );
    let _cwd_guard = ChangeCwd::new(dir.path());

    json_ui_migrate_v1::run(src_path.to_string_lossy().to_string(), false).unwrap();

    let spec_root = dir.path().join("src/views");
    if spec_root.exists() {
        for entry in std::fs::read_dir(&spec_root).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                let inner: Vec<_> = std::fs::read_dir(&p).unwrap().collect();
                assert!(
                    inner.is_empty(),
                    "no spec files emitted for unsupported handler; found: {inner:?}"
                );
            } else {
                panic!("unexpected file under src/views: {}", p.display());
            }
        }
    }
}
