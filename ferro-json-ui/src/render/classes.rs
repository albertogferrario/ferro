//! Shared interactive-base class fragments. Single source of truth for the
//! focus-visible ring, disabled treatment, and frequency-tiered motion, composed
//! into each component's class string so the interactive feel cannot drift.
//!
//! Every constant is a complete class literal in crate source so the Tailwind
//! `@source` scanner picks the utilities up; compose via `concat!`/`format!`
//! of these fragments — never build class names dynamically.

/// focus-visible ring from the `--color-ring` token (D-14). ring-2 + offset-2 baseline.
pub(crate) const FOCUS_RING: &str =
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2";

/// Fast tier (D-03): hover, toggles, controls, nav — high-frequency interactions.
pub(crate) const MOTION_FAST: &str = "transition-colors duration-fast ease-base";

/// Base tier (D-03): dropdowns, modals, toasts.
pub(crate) const MOTION_BASE: &str = "transition-opacity duration-base ease-base";

/// Uniform disabled treatment for native controls (D-16).
pub(crate) const DISABLED_BASE: &str = "disabled:opacity-50 disabled:pointer-events-none";

/// Fast-tier interactive base = fast motion + focus ring, for buttons/links/nav.
pub(crate) const INTERACTIVE_BASE: &str = concat!(
    "transition-colors duration-fast ease-base ",
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
);

/// Touch-manipulation scroll optimisation for tap targets. Full literal.
pub const TOUCH_ACTION: &str = "touch-manipulation";

/// Minimum 44px hit target for interactive elements (WCAG 2.5.5). Full literal.
pub const HIT_TARGET_MIN: &str = "min-h-[44px] min-w-[44px]";

/// Enlarged 56px hit target for Numpad keys (Phase 256 consumer). Full literal.
pub const HIT_TARGET_NUMPAD: &str = "min-h-[56px] min-w-[56px]";

/// Press-state feedback for tap surfaces: scale-down + border tint.
/// Token-compliant (`active:bg-border` = `--color-border`); no raw palette class.
pub const PRESS_ACTIVE: &str = "active:scale-95 active:bg-border";

/// Prevents scroll bounce from escaping a pane boundary. Full literal.
pub const OVERSCROLL_CONTAIN: &str = "overscroll-contain";

/// Removes the default iOS tap-highlight rectangle on touch targets.
/// Backed by `@utility pos-tap-highlight` in input.css (Path B — guaranteed CSS).
pub const TAP_HIGHLIGHT: &str = "pos-tap-highlight";

#[cfg(test)]
mod tests {
    use super::*;

    /// `INTERACTIVE_BASE` must be exactly the fast-tier motion + focus ring —
    /// editing one fragment without the other is drift.
    #[test]
    fn interactive_base_is_motion_fast_plus_focus_ring() {
        assert_eq!(INTERACTIVE_BASE, format!("{MOTION_FAST} {FOCUS_RING}"));
    }

    /// Every fragment sources the Phase 250 token vocabulary, never raw values.
    #[test]
    fn fragments_use_token_utilities() {
        assert!(FOCUS_RING.contains("focus-visible:ring-ring"));
        assert!(MOTION_FAST.contains("duration-fast") && MOTION_FAST.contains("ease-base"));
        assert!(MOTION_BASE.contains("duration-base") && MOTION_BASE.contains("ease-base"));
        assert!(DISABLED_BASE.contains("disabled:pointer-events-none"));
    }

    /// D-07: no render file (outside classes.rs) may inline the touch literals.
    /// Auto-covers Phase 256 render files via read_dir — no manual re-enrollment.
    #[test]
    fn render_functions_use_constants_not_literals() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
        let render_dir = std::path::Path::new(&manifest_dir).join("src/render");
        let guarded = [
            "touch-manipulation",
            "min-h-[44px] min-w-[44px]",
            "min-h-[56px] min-w-[56px]",
        ];
        for entry in std::fs::read_dir(&render_dir).expect("src/render readable") {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            if filename == "classes.rs" {
                continue;
            }
            let source = std::fs::read_to_string(&path).unwrap();
            for lit in &guarded {
                assert!(
                    !source.contains(lit),
                    "{filename}: raw POS literal {lit:?} — import from render::classes instead"
                );
            }
        }
    }

    /// D-07 extension (Plan 06): render functions migrated to fjui-* classes must not
    /// re-introduce the specific appearance utility strings that moved to the skin layer.
    ///
    /// Scope: render functions migrated in Plans 05 and 06 (Button, Badge, Alert,
    /// StatCard, EmptyState, Card, Tabs, Text, Separator, Progress, Avatar, Image,
    /// Skeleton, Breadcrumb, Pagination, DescriptionList, Toast, Sidebar, Header,
    /// CalendarCell, ActionCard, Tile).
    ///
    /// Deferred (NOT banned here — migrate in a later plan):
    /// - QuantityStepper / Numpad button classes (plan-scope boundary)
    /// - Checklist checkbox native-control classes (form-control migration deferred)
    /// - StatCard icon tint classes (icon system migration deferred)
    /// - Header sub-element avatar/notification badge micro-classes (CHR-01 deferred to Phase 247)
    /// - render_sidebar border-t border-border on fixed_bottom nav separator (structural inline)
    #[test]
    fn atoms_render_fns_contain_no_migrated_appearance_utilities() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
        let atoms_path = std::path::Path::new(&manifest_dir).join("src/render/atoms.rs");
        let source = std::fs::read_to_string(&atoms_path).expect("atoms.rs readable");

        // Scan only the production source (before #[cfg(test)]).
        let test_start = source.find("#[cfg(test)]").unwrap_or(source.len());
        let production_source = &source[..test_start];

        // These are top-level container appearance utilities on the migrated component
        // elements themselves (not on inner sub-elements still pending migration).
        // Each is a string that must NOT appear in a class= attribute emitted by
        // the migrated render functions listed in the doc comment above.
        let banned: &[(&str, &str)] = &[
            // Toast: moved to fjui-toast / fjui-toast--{tone}
            ("bg-primary/70", "toast tone bg — use fjui-toast--neutral skin rule"),
            ("bg-success/70", "toast tone bg — use fjui-toast--success skin rule"),
            ("bg-warning/70", "toast tone bg — use fjui-toast--warning skin rule"),
            ("bg-destructive/70", "toast tone bg — use fjui-toast--destructive skin rule"),
            // Action card: moved to fjui-action-card / fjui-action-card--{tone}
            ("fjui-action-card\" class=\"", "action card must not carry appearance utilities after fjui-action-card"),
            // Pagination: moved to fjui-pagination__btn / fjui-pagination__btn--active
            ("bg-primary text-primary-foreground\">", "pagination active page — use fjui-pagination__btn--active"),
        ];

        for (lit, reason) in banned {
            assert!(
                !production_source.contains(lit),
                "atoms.rs: appearance utility {lit:?} found — {reason}"
            );
        }
    }

    /// D-07 extension (Plan 07): container render functions migrated to fjui-* classes
    /// must not re-introduce the specific appearance utility strings that moved to the
    /// skin layer.
    ///
    /// Scope: render functions migrated in Plan 07 (Card, PageHeader, Modal, Tabs,
    /// KanbanBoard, KanbanCard, DataTable shell).
    ///
    /// Deferred (NOT banned here):
    /// - DataTable row/cell classes (Plan 08 scope)
    /// - render_action_group non-kebab button appearance (partial migration only)
    #[test]
    fn containers_render_fns_contain_no_migrated_appearance_utilities() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
        let containers_path =
            std::path::Path::new(&manifest_dir).join("src/render/containers.rs");
        let source = std::fs::read_to_string(&containers_path).expect("containers.rs readable");

        // Scan only the production source (before #[cfg(test)]).
        let test_start = source.find("#[cfg(test)]").unwrap_or(source.len());
        let production_source = &source[..test_start];

        // Appearance utilities that moved to the skin layer in Plan 07.
        // Each string must NOT appear in any class= attribute emitted by the
        // migrated render functions listed above.
        let banned: &[(&str, &str)] = &[
            // Card: moved to fjui-card
            ("rounded-lg border border-border bg-card", "card outer — use fjui-card skin rule"),
            ("shadow-sm rounded-lg", "card bordered shadow — use fjui-card skin rule"),
            ("shadow-md rounded-lg", "card elevated shadow — use fjui-card skin rule"),
            // Tabs: moved to fjui-tabs / fjui-tab / fjui-tab--active
            // Note: bare "border-b border-border" is NOT banned — it appears on structural
            // row separators (e.g. render_selection_panel line dividers) which are out of
            // scope. Only the tabs-specific wrapper div pattern is banned.
            ("<div class=\"border-b border-border\">", "tabs wrapper div — use fjui-tabs nav (no wrapper div needed)"),
            ("border-b-2 border-primary", "active tab underline — use fjui-tab--active skin rule"),
            ("text-text-muted hover:text-text", "tab inactive text — use fjui-tab skin rule"),
            // Modal: moved to fjui-modal
            ("bg-card rounded-lg shadow-lg", "modal dialog — use fjui-modal skin rule"),
            // Kanban: moved to fjui-kanban__* skin rules
            ("bg-secondary/10 rounded-lg", "kanban column bg — use fjui-kanban__column skin rule"),
            ("bg-primary text-primary-foreground", "kanban active count badge — use fjui-badge--neutral skin rule"),
            // PageHeader: moved to fjui-page-header
            ("border-b border-border bg-background", "page header bar — use fjui-page-header skin rule"),
        ];

        for (lit, reason) in banned {
            assert!(
                !production_source.contains(lit),
                "containers.rs: appearance utility {lit:?} found — {reason}"
            );
        }
    }

    /// D-07 extension (Plan 08): data.rs render functions (DataTable body/rows/cells)
    /// must not contain the appearance utilities removed by the Plan 08 migration.
    ///
    /// Scope: render functions migrated in Plan 08 (DataTable row/cell).
    ///
    /// NOT banned (out of scope):
    /// - Table (simple, not DataTable) — not migrated in Plan 08
    /// - MediaCardGrid card classes — separate surface, partial migration only
    #[test]
    fn data_render_fns_contain_no_migrated_appearance_utilities() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
        let data_path = std::path::Path::new(&manifest_dir).join("src/render/data.rs");
        let source = std::fs::read_to_string(&data_path).expect("data.rs readable");

        // Scan only the production source (before #[cfg(test)]).
        let test_start = source.find("#[cfg(test)]").unwrap_or(source.len());
        let production_source = &source[..test_start];

        let banned: &[(&str, &str)] = &[
            // DataTable row zebra striping: removed (RSK-01) — hover surface via skin rule.
            // Note: render_table (simple Table, non-DataTable) is out of Plan 08 scope.
            ("even:bg-surface", "DataTable row zebra — removed by RSK-01; skin hover replaces it"),
            ("odd:bg-surface", "DataTable row zebra — removed by RSK-01; skin hover replaces it"),
        ];

        for (lit, reason) in banned {
            assert!(
                !production_source.contains(lit),
                "data.rs: appearance utility {lit:?} found — {reason}"
            );
        }
    }

    /// D-07 extension (Plan 08): form.rs render functions (Input/Select/Textarea)
    /// must not contain the appearance utilities removed by the Plan 08 migration.
    ///
    /// Scope: render functions migrated in Plan 08 (render_input, render_select).
    ///
    /// NOT banned (out of scope, not migrated in Plan 08):
    /// - Checkbox/CheckboxList classes (native control micro-styling, deferred)
    /// - Switch pill classes (complex state machine, deferred)
    /// - File input classes (specialized input, deferred)
    #[test]
    fn form_render_fns_contain_no_migrated_appearance_utilities() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
        let form_path = std::path::Path::new(&manifest_dir).join("src/render/form.rs");
        let source = std::fs::read_to_string(&form_path).expect("form.rs readable");

        // Scan only the production source (before #[cfg(test)]).
        let test_start = source.find("#[cfg(test)]").unwrap_or(source.len());
        let production_source = &source[..test_start];

        let banned: &[(&str, &str)] = &[
            // Input/Textarea old appearance: moved to fjui-input/fjui-textarea skin rule
            ("rounded-md border border-border px-3 py-2", "Input/Textarea border+padding — use fjui-input skin rule"),
            ("rounded-md border border-destructive px-3 py-2", "Input error border — use fjui-input--error skin rule"),
            // Select old appearance: moved to fjui-select skin rule
            ("appearance-none bg-background rounded-md border", "Select appearance — use fjui-select skin rule"),
        ];

        for (lit, reason) in banned {
            assert!(
                !production_source.contains(lit),
                "form.rs: appearance utility {lit:?} found — {reason}"
            );
        }
    }

    #[test]
    fn touch_constants_are_full_literals_and_token_compliant() {
        assert_eq!(TOUCH_ACTION, "touch-manipulation");
        assert_eq!(HIT_TARGET_MIN, "min-h-[44px] min-w-[44px]");
        assert_eq!(HIT_TARGET_NUMPAD, "min-h-[56px] min-w-[56px]");
        assert_eq!(OVERSCROLL_CONTAIN, "overscroll-contain");
        assert_eq!(TAP_HIGHLIGHT, "pos-tap-highlight");
        assert!(PRESS_ACTIVE.contains("active:scale-95"));
        assert!(PRESS_ACTIVE.contains("active:bg-border"));
        for raw in ["red-", "blue-", "orange-", "zinc-", "gray-", "slate-"] {
            assert!(
                !PRESS_ACTIVE.contains(raw),
                "raw palette class in PRESS_ACTIVE"
            );
        }
    }
}
