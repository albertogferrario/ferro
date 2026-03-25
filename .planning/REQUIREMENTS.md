# Requirements: Ferro v10.0 JSON-UI Visual Overhaul

**Defined:** 2026-03-24
**Core Value:** Agents can go from "I want an app that does X" to a working, deployed application with minimal friction.

## v10.0 Requirements

Requirements for JSON-UI visual quality uplift to professional grade. Each maps to roadmap phases.

### Foundation

- [x] **FND-01**: Font token namespace uses correct Tailwind v4 names (`--font-sans`, `--font-mono` not `--font-family-sans`)
- [x] **FND-02**: Inter Variable font loads via Bunny Fonts CDN in base document `<head>`
- [x] **FND-03**: Body and all text elements render in Inter (or system fallback)
- [x] **FND-04**: Test suite separates structural assertions from cosmetic class assertions to prevent cascade failures

### Surface & Elevation

- [x] **SRF-01**: Card component uses `bg-card` (visually distinct from page `bg-background`)
- [x] **SRF-02**: Modal panel uses `bg-card` for elevated surface appearance
- [x] **SRF-03**: StatCard uses `bg-card` for elevated surface appearance
- [x] **SRF-04**: NotificationDropdown panel uses `bg-card` for elevated surface appearance
- [x] **SRF-05**: Three-tier surface hierarchy enforced: `background` (page) → `surface` (sidebar, panels) → `card` (cards, modals, dropdowns)
- [ ] **SRF-06**: All 8 critical dark mode token pairs pass WCAG 4.5:1 contrast ratio
- [x] **SRF-07**: Runtime JS (toast, tab switcher) uses semantic token classes instead of hardcoded colors (`bg-primary` not `bg-blue-500`)

### Typography

- [ ] **TYP-01**: H1 renders with `leading-tight tracking-tight`
- [ ] **TYP-02**: H2 renders with `leading-tight tracking-tight`
- [ ] **TYP-03**: H3 renders with `leading-snug`
- [ ] **TYP-04**: Body text (P, Div, Section) renders with `leading-relaxed`
- [ ] **TYP-05**: Muted text uses consistent `text-text-muted` across all components

### Form Polish

- [ ] **FRM-01**: Select element displays a custom SVG dropdown arrow (CSS-only, no JS)
- [ ] **FRM-02**: Input in error state shows `focus-visible:ring-destructive` (not primary)
- [ ] **FRM-03**: All form elements have `transition-colors duration-150 motion-reduce:transition-none`
- [ ] **FRM-04**: All form elements have `disabled:opacity-50 disabled:cursor-not-allowed`
- [ ] **FRM-05**: Select in error state shows `focus-visible:ring-destructive`
- [ ] **FRM-06**: Textarea in error state shows `focus-visible:ring-destructive`
- [ ] **FRM-07**: Form field order is consistent: label → input → description → error message

### Interactive States

- [ ] **INT-01**: All buttons have `focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2`
- [ ] **INT-02**: Tab buttons have focus-visible ring
- [ ] **INT-03**: Pagination links have focus-visible ring
- [ ] **INT-04**: Breadcrumb links have focus-visible ring
- [ ] **INT-05**: Sidebar nav items have focus-visible ring
- [ ] **INT-06**: Table rows have `hover:bg-surface` for row highlighting
- [ ] **INT-07**: All interactive elements have `transition-colors duration-150 motion-reduce:transition-none`

### Component Details

- [ ] **CMP-01**: Alert renders an inline SVG icon per variant (info, success, warning, error)
- [ ] **CMP-02**: Skeleton uses shimmer animation (CSS keyframe) instead of `animate-pulse`
- [ ] **CMP-03**: Breadcrumb uses SVG chevron separator instead of `/` text
- [ ] **CMP-04**: Active tab has `font-semibold` weight
- [ ] **CMP-05**: NotificationDropdown bell renders as SVG icon (not emoji)
- [ ] **CMP-06**: Collapsible renders a rotating SVG chevron indicator

## Future Requirements

Deferred to a later milestone. Tracked but not in current roadmap.

### Advanced Rendering

- **ADV-01**: Tooltip-style help text on form inputs
- **ADV-02**: Sticky table headers for data-heavy views
- **ADV-03**: Responsive table collapse to card layout on mobile
- **ADV-04**: Dark mode toggle with localStorage persistence (requires JS)
- **ADV-05**: JavaScript-powered custom select dropdown with search

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| New components | This is a polish milestone, not a feature milestone |
| JavaScript interactivity | JSON-UI is CSS-only; JS features belong in a separate milestone |
| Custom icon library | Inline SVG strings in Rust are sufficient for ~10 icons needed |
| Production CSS build | CDN is the development story; production build is a deployment concern |
| `<dialog>` modal migration | Architectural change; current `<details>` works; flag for future |
| Tailwind arbitrary-value tokens (`rounded-[--radius-md]`) | Unverified in CDN mode; standard Tailwind scale classes match token defaults |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| FND-01 | Phase 102 | Complete |
| FND-02 | Phase 102 | Complete |
| FND-03 | Phase 102 | Complete |
| FND-04 | Phase 102 | Complete |
| SRF-01 | Phase 103 | Complete |
| SRF-02 | Phase 103 | Complete |
| SRF-03 | Phase 103 | Complete |
| SRF-04 | Phase 103 | Complete |
| SRF-05 | Phase 103 | Complete |
| SRF-06 | Phase 103 | Pending |
| SRF-07 | Phase 103 | Complete |
| TYP-01 | Phase 104 | Pending |
| TYP-02 | Phase 104 | Pending |
| TYP-03 | Phase 104 | Pending |
| TYP-04 | Phase 104 | Pending |
| TYP-05 | Phase 104 | Pending |
| FRM-01 | Phase 105 | Pending |
| FRM-02 | Phase 105 | Pending |
| FRM-03 | Phase 105 | Pending |
| FRM-04 | Phase 105 | Pending |
| FRM-05 | Phase 105 | Pending |
| FRM-06 | Phase 105 | Pending |
| FRM-07 | Phase 105 | Pending |
| INT-01 | Phase 106 | Pending |
| INT-02 | Phase 106 | Pending |
| INT-03 | Phase 106 | Pending |
| INT-04 | Phase 106 | Pending |
| INT-05 | Phase 106 | Pending |
| INT-06 | Phase 106 | Pending |
| INT-07 | Phase 106 | Pending |
| CMP-01 | Phase 107 | Pending |
| CMP-02 | Phase 107 | Pending |
| CMP-03 | Phase 107 | Pending |
| CMP-04 | Phase 107 | Pending |
| CMP-05 | Phase 107 | Pending |
| CMP-06 | Phase 107 | Pending |

**Coverage:**
- v10.0 requirements: 36 total
- Mapped to phases: 36
- Unmapped: 0

---
*Requirements defined: 2026-03-24*
*Last updated: 2026-03-24 after roadmap creation — all 36 requirements mapped to Phases 102-107*
