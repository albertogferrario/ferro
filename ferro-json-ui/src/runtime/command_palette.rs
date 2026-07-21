pub(super) const SOURCE: &str = r#"
    // ── Command palette (Plan 02 fills this) ─────────────────────────────────
    //
    // Intercepts fjui:open-command-palette and ⌘K / Ctrl+K keydown.
    // Renders grouped search results via GET /dashboard/search?q=.
    // Recents stored in localStorage; quick actions from static config.
    // ARIA: role="combobox" on input, role="listbox" on results, stable
    // option ids ("fjui-palette-opt-{n}"), aria-activedescendant updated
    // on every arrow-key press.

    function setupCommandPalette() {}
"#;
