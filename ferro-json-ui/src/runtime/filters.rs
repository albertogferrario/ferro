// Token normalization contract: tab token values use the same space->hyphen
// normalization applied at render time in render_tile (data-filter-tokens).
// The runtime normalizes the active tab token before comparing so that a tab
// with value "Bevande calde" matches tokens emitted as "Bevande-calde".

pub(super) const SOURCE: &str = r#"
    // ── Tile filtering — token/text client-side visibility ───────────────────
    //
    // Attribute contract:
    //   [data-filter-scope]              — scope container (one per filter group)
    //   [data-filter-tab="<token>"]      — filter tab; empty value = "All"
    //   [data-filter-search]             — optional search input
    //   [data-filter-text="<name>"]      — tile marker + search source (always present)
    //   [data-filter-tokens="t1 t2 ..."] — space-separated token list (conditional)
    //
    // Matching semantics (AND intersection, D-09):
    //   A tile is visible iff tokenMatch AND searchMatch.
    //   tokenMatch: empty active token (All) -> every tile matches;
    //               specific token -> tile must have data-filter-tokens containing it.
    //   Untokened tiles (no data-filter-tokens) are visible under All only (D-10).
    //   searchMatch: case-insensitive substring of data-filter-text; empty -> all.
    //
    // Hide/show: el.style.display = 'none' / '' (NOT hidden attribute, NOT Tailwind, D-11).
    // Active-tab classes: semantic tokens only — border-primary/text-primary/font-semibold
    //   vs border-transparent/text-text-muted/hover:text-text (mirrors tabs.rs, D-12).
    //
    // Space->hyphen tab-token normalization: the active tab's value is normalized
    // (spaces replaced with hyphens) before comparing against data-filter-tokens,
    // matching the normalization render_tile applies at render time (Pitfall 7).

    function setupFilters() {
        var scopes = document.querySelectorAll('[data-filter-scope]');
        if (scopes.length === 0) return;
        for (var i = 0; i < scopes.length; i++) {
            initFilterScope(scopes[i]);
        }
    }

    function initFilterScope(scope) {
        var tabs = scope.querySelectorAll('[data-filter-tab]');
        var search = scope.querySelector('[data-filter-search]');
        var activeToken = '';
        var searchText = '';

        for (var t = 0; t < tabs.length; t++) {
            initFilterTab(tabs, tabs[t], scope);
        }

        if (search) {
            search.addEventListener('input', function() {
                searchText = search.value || '';
                applyFilter(scope, activeToken, searchText);
            });
        }

        // Apply initial state
        applyFilter(scope, activeToken, searchText);

        // Shared mutable state capture: use a closure variable updated on each tab click.
        function initFilterTab(allTabs, tab, filterScope) {
            tab.addEventListener('click', function() {
                var rawToken = tab.getAttribute('data-filter-tab') || '';
                // Normalize space->hyphen to match render_tile token emission (Pitfall 7).
                activeToken = rawToken.replace(/ /g, '-');
                updateFilterTabClasses(allTabs, tab);
                applyFilter(filterScope, activeToken, searchText);
            });
        }
    }

    function updateFilterTabClasses(allTabs, activeTab) {
        for (var i = 0; i < allTabs.length; i++) {
            var tab = allTabs[i];
            if (tab === activeTab) {
                tab.classList.remove('border-transparent', 'text-text-muted', 'hover:text-text');
                tab.classList.add('border-primary', 'text-primary', 'font-semibold');
                tab.setAttribute('aria-selected', 'true');
            } else {
                tab.classList.remove('border-primary', 'text-primary', 'font-semibold');
                tab.classList.add('border-transparent', 'text-text-muted', 'hover:text-text');
                tab.setAttribute('aria-selected', 'false');
            }
        }
    }

    function applyFilter(scope, activeToken, searchText) {
        var tiles = scope.querySelectorAll('[data-filter-text]');
        var lowerSearch = searchText.toLowerCase();
        for (var i = 0; i < tiles.length; i++) {
            var tile = tiles[i];
            var tokenMatch = filterTokenMatch(tile, activeToken);
            var searchMatch = filterSearchMatch(tile, lowerSearch);
            tile.style.display = (tokenMatch && searchMatch) ? '' : 'none';
        }
    }

    // D-09 + D-10: token matching.
    // Empty activeToken (All) -> every tile visible.
    // Specific token: tile must have data-filter-tokens AND contain the token
    // (untokened tiles match All only).
    function filterTokenMatch(tile, activeToken) {
        if (activeToken === '') return true;
        var tokensAttr = tile.getAttribute('data-filter-tokens');
        if (!tokensAttr) return false;
        var lowerToken = activeToken.toLowerCase();
        var tokens = tokensAttr.split(' ');
        for (var i = 0; i < tokens.length; i++) {
            if (tokens[i].toLowerCase() === lowerToken) return true;
        }
        return false;
    }

    // Search matching: case-insensitive substring of data-filter-text.
    function filterSearchMatch(tile, lowerSearch) {
        if (lowerSearch === '') return true;
        var text = (tile.getAttribute('data-filter-text') || '').toLowerCase();
        return text.indexOf(lowerSearch) !== -1;
    }
"#;
