pub(super) const SOURCE: &str = r#"
    // ── Filter bar (Plan 05 fills this) ──────────────────────────────────────
    //
    // List-page toolbar row: active-filter chips + add-filter affordance +
    // saved-view switcher. State via query params; GET navigation intercepted
    // by nav runtime. Tracks "modificata" state by comparing current URL
    // params against the saved view's canonical query-string. Idempotent via
    // data-filter-bar-init guard.

    function setupFilterBar() {}
"#;
