pub(super) const SOURCE: &str = r#"
    // ── Inline edit (Plan 04 fills this) ─────────────────────────────────────
    //
    // Double-click or pencil icon activates an in-place input overlay on
    // DescriptionList <dd> elements with data-inline-edit-field. Enter commits
    // via POST; Escape and blur cancel (D-09). One field in edit at a time.
    // Idempotent via data-inline-edit-init guard.

    function setupInlineEdit() {}
"#;
