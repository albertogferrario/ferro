pub(super) const SOURCE: &str = r#"
    // ── Bulk action bar (Plan 06 fills this) ─────────────────────────────────
    //
    // Floating action bar at viewport bottom, visible when ≥1 DataTable rows
    // are checked via bulk-select checkboxes with data-row-key. Tracks
    // selection state; destructive actions require native <dialog> confirm.
    // Idempotent via data-bulk-bar-init guard.

    function setupBulkBar() {}
"#;
