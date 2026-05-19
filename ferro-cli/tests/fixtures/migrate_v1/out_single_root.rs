// Fixture: v1 controller with a single top-level node — successfully migratable.
// Phase 163.1 test fixture: exercises the single-root branch of try_migrate_handler.

pub async fn dashboard(req: Request) -> Response {
    JsonUi::render_file("src/views/in_single_root/dashboard.json", serde_json::json!({}))
}
