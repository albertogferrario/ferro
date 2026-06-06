# Runtime Primitives

ferro-json-ui ships a small JavaScript runtime that is inlined into every page rendered by `DefaultLayout` and `DashboardLayout`. Most of the runtime is internal: it powers behaviors emitted by components (popover menus, tabs, dismissible toasts, sidebar, modals) and the attributes those components use are implementation details.

A small subset of the runtime is a public contract: DOM attributes the runtime recognizes on hand-authored or component-output HTML. This page documents that subset.

## `data-lazy-hero`

Opts a `<video>` element into intersection-driven `preload` promotion. When the element approaches the viewport, the runtime sets `preload="auto"` and calls `<video>.load()` so the first frame is ready by the time the user reaches the element.

### Contract

| Attribute | Required | Default | Description |
|-----------|----------|---------|-------------|
| `data-lazy-hero` | yes | — | Opt-in marker. The element must also have `preload="none"`. |
| `data-lazy-hero-margin` | no | `200px 0px` | Per-element `rootMargin` for the IntersectionObserver. Any CSS-margin shorthand the IntersectionObserver constructor accepts. |
| `data-lazy-hero-promoted` | no (runtime sets it) | absent | Idempotency marker. The runtime sets this to `"1"` after promotion; re-running the primitive on the same element is a no-op. |

### Selector

The runtime matches `video[preload="none"][data-lazy-hero]:not([data-lazy-hero-promoted])`. Three consequences:

- `<video>` without `preload="none"` is ignored. The runtime does not override an author-tuned `preload` value.
- Non-`<video>` elements with `data-lazy-hero` are ignored. The promote action (flip `preload`, call `.load()`) is video-specific.
- Already-promoted elements are excluded by the `:not(...)` clause.

### Usage

```html
<!-- Above-the-fold: load eagerly -->
<video preload="auto" poster="/posters/hero-0.jpg" muted playsinline>
  <source src="/assets/hero-0.mp4" type="video/mp4">
</video>

<!-- Below-the-fold: lazy-promote on viewport approach (default 200px lead) -->
<video preload="none" data-lazy-hero poster="/posters/hero-1.jpg" muted playsinline>
  <source src="/assets/hero-1.mp4" type="video/mp4">
</video>

<!-- Below-the-fold with a larger lead time (slower-loading hero) -->
<video preload="none" data-lazy-hero data-lazy-hero-margin="400px 0px"
       poster="/posters/hero-2.jpg" muted playsinline>
  <source src="/assets/hero-2.mp4" type="video/mp4">
</video>
```

### Observer cardinality

Elements are grouped by their resolved `data-lazy-hero-margin` value. The runtime constructs one IntersectionObserver per distinct margin value, with all elements sharing that value fanned out to it. A page where every hero uses the default has exactly one observer; a page mixing defaults with one override has two observers.

### Browser support

The primitive depends on the `IntersectionObserver` API. On environments without it (rare; some legacy embedded WebViews, certain test harnesses with minimal DOM polyfills), the runtime silently no-ops and the videos behave exactly as authored.

### Lifecycle

The runtime scans the DOM once, when `DOMContentLoaded` fires. Elements inserted into the DOM after that point are not observed. Pages that render heroes server-side as part of the initial HTML — the intended use case — are unaffected.

### Performance, not access control

`data-lazy-hero` defers a fetch the browser would otherwise issue at page load. It does not prevent a fetch. Once the element approaches the viewport, the runtime initiates the fetch. Do not use this attribute to gate paid content or otherwise restrict resource access — that is an access-control concern, not a performance concern.
