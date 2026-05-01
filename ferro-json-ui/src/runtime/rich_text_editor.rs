//! Runtime module: RichTextEditor.
//!
//! Mounts Quill 2.0.3 (Snow theme) on every `[data-rich-text-editor]` wrapper
//! rendered by `render_rich_text_editor`. On form submit, serializes the
//! editor state to two hidden inputs:
//!
//! - `{name}_delta` — `JSON.stringify(quill.getContents())` (Delta JSON,
//!   canonical, lossless)
//! - `{name}_html` — `sanitizeHtmlByFormats(quill.root.innerHTML, formats)`
//!   (sanitized HTML, rendering input)
//!
//! The `formats` allowlist is the single source of truth (D-15): it drives
//! both the Quill toolbar config (init time) AND the HTML post-process
//! (submit time). Image / video / HTML-paste paths are not reachable
//! through the prop surface — Quill drops disallowed paste content because
//! `formats` constrains its clipboard module, and the post-process strips
//! anything that survived.
//!
//! No external HTML sanitizer (no DOMPurify, no `ammonia`) — the allowlist
//! post-process is a hand-rolled DOM walker (D-17). DOMParser is preferred
//! over regex (regex sanitizers are bug factories — D-17).
//!
//! Multiple instances on the same page (D-14): each wrapper installs its
//! own submit listener. Listeners write disjoint hidden inputs, so no
//! ordering or coordination is required.
//!
//! Vanilla ES5: `var` only, named function declarations, no arrow functions,
//! no closures capturing `this`, no `innerHTML =` assignment from user
//! input. (Reading `quill.root.innerHTML` is safe — it is Quill's
//! controlled output and is sanitized before reaching `{name}_html`.)

pub(super) const SOURCE: &str = r#"
    // ── Rich-text editor (Quill 2.0.3, Snow theme) ───────────────────────

    function setupRichTextEditor() {
        if (typeof window === 'undefined' || typeof window.Quill === 'undefined') {
            return;
        }
        var wrappers = document.querySelectorAll('[data-rich-text-editor]');
        for (var i = 0; i < wrappers.length; i++) {
            initRichTextEditor(wrappers[i]);
        }
    }

    function initRichTextEditor(wrapper) {
        var name = wrapper.getAttribute('data-rte-name') || '';
        var theme = wrapper.getAttribute('data-rte-theme') || 'snow';
        var placeholder = wrapper.getAttribute('data-rte-placeholder') || '';
        var formatsRaw = wrapper.getAttribute('data-rte-formats') || '[]';
        var formats;
        try {
            formats = JSON.parse(formatsRaw);
        } catch (e) {
            formats = [];
        }
        if (!Array.isArray(formats)) {
            formats = [];
        }

        var host = wrapper.querySelector('[data-rte-host]');
        if (!host) {
            return;
        }

        // Auto-detect initial Delta vs HTML (D-12). The renderer puts the
        // initial value (escaped) into the host's text content. If it
        // parses as a JSON object with an `ops` array, treat as Delta;
        // otherwise leave the host content for Quill's clipboard pipeline.
        var hostText = host.textContent || '';
        var initialDelta = null;
        if (hostText.length > 0) {
            try {
                var parsed = JSON.parse(hostText);
                if (parsed && typeof parsed === 'object' && Array.isArray(parsed.ops)) {
                    initialDelta = parsed;
                    // Clear the host so Quill does not double-render the
                    // raw JSON string under the rendered Delta.
                    host.textContent = '';
                }
            } catch (err) {
                initialDelta = null;
            }
        }

        var toolbarConfig = formatsToToolbarConfig(formats);

        var quill = new window.Quill(host, {
            theme: theme,
            placeholder: placeholder,
            modules: { toolbar: toolbarConfig },
            formats: formats
        });

        if (initialDelta !== null) {
            quill.setContents(initialDelta);
        }

        // Submit interception — capture phase so we run before any other
        // handler that might cancel propagation (D-13).
        var form = wrapper.closest ? wrapper.closest('form') : null;
        if (!form) {
            return;
        }

        form.addEventListener('submit', function(event) {
            // D-29 required-content guard.
            var requiredMarker = wrapper.querySelector('[data-rte-required]');
            if (requiredMarker) {
                var plain = quill.getText() || '';
                var trimmed = plain.replace(/^\s+|\s+$/g, '');
                if (trimmed.length === 0) {
                    event.preventDefault();
                    showOrCreateRteError(wrapper, name, 'Required');
                    return;
                }
            }
            // Serialize Delta and sanitized HTML into the hidden inputs.
            var deltaInput = wrapper.querySelector('[data-rte-hidden="delta"]');
            var htmlInput = wrapper.querySelector('[data-rte-hidden="html"]');
            if (deltaInput) {
                deltaInput.value = JSON.stringify(quill.getContents());
            }
            if (htmlInput) {
                htmlInput.value = sanitizeHtmlByFormats(
                    quill.root.innerHTML,
                    formats
                );
            }
        }, true);
    }

    function showOrCreateRteError(wrapper, name, message) {
        var errId = 'err-' + name;
        var existing = wrapper.querySelector('#' + cssEscapeId(errId));
        if (existing) {
            existing.textContent = message;
            return;
        }
        var p = document.createElement('p');
        p.id = errId;
        p.className = 'text-sm text-destructive';
        p.textContent = message;
        wrapper.appendChild(p);
    }

    // Minimal CSS.escape polyfill scoped to id strings (ES5-safe).
    function cssEscapeId(id) {
        return String(id).replace(/(["\\#.()\[\]:>+~*=^$|])/g, '\\$1');
    }

    function formatsToToolbarConfig(formats) {
        // D-19 deterministic mapping. Unknown formats are silently dropped.
        var inlineGroup = [];
        var blockGroup = [];
        var listGroup = null;
        var headerGroup = null;
        for (var i = 0; i < formats.length; i++) {
            var f = formats[i];
            if (f === 'bold' || f === 'italic' || f === 'underline' || f === 'strike') {
                inlineGroup.push(f);
            } else if (f === 'link') {
                inlineGroup.push('link');
            } else if (f === 'list') {
                listGroup = [{ 'list': 'ordered' }, { 'list': 'bullet' }];
            } else if (f === 'header') {
                headerGroup = [{ 'header': [1, 2, 3, false] }];
            } else if (f === 'blockquote') {
                blockGroup.push('blockquote');
            } else if (f === 'code-block') {
                blockGroup.push('code-block');
            }
            // Unknown -> dropped silently.
        }
        var groups = [];
        if (headerGroup) { groups.push(headerGroup); }
        if (inlineGroup.length > 0) { groups.push(inlineGroup); }
        if (listGroup) { groups.push(listGroup); }
        if (blockGroup.length > 0) { groups.push(blockGroup); }
        return groups;
    }

    function sanitizeHtmlByFormats(html, formats) {
        // D-16 tag -> format mapping table. Unknown tags fall through to
        // "always-allowed" or "always-stripped" via the helpers below.
        var tagToFormat = {
            'B': 'bold', 'STRONG': 'bold',
            'I': 'italic', 'EM': 'italic',
            'U': 'underline',
            'S': 'strike',
            'UL': 'list', 'OL': 'list', 'LI': 'list',
            'H1': 'header', 'H2': 'header', 'H3': 'header',
            'H4': 'header', 'H5': 'header', 'H6': 'header',
            'A': 'link',
            'BLOCKQUOTE': 'blockquote',
            'PRE': 'code-block', 'CODE': 'code-block'
        };
        // Always-allowed structural tags (paragraph flow, line break,
        // and the Quill-internal span used for inline format wrappers).
        var alwaysAllowed = { 'P': 1, 'BR': 1, 'SPAN': 1, 'DIV': 1 };
        // Always-stripped tags — image / video / HTML-paste paths.
        var alwaysStripped = {
            'IMG': 1, 'VIDEO': 1, 'AUDIO': 1, 'IFRAME': 1,
            'SCRIPT': 1, 'STYLE': 1, 'LINK': 1,
            'EMBED': 1, 'OBJECT': 1, 'FORM': 1, 'INPUT': 1,
            'BUTTON': 1, 'TEXTAREA': 1, 'SELECT': 1, 'OPTION': 1
        };

        // Build a formats lookup set for O(1) membership tests.
        var allowedFormats = {};
        for (var fi = 0; fi < formats.length; fi++) {
            allowedFormats[formats[fi]] = 1;
        }

        // Parse the HTML inside a synthetic root so we can extract the
        // sanitized children without dragging the synthetic element into
        // the output.
        var doc = new DOMParser().parseFromString(
            '<!DOCTYPE html><html><body><div id="ferro-rte-sanitize-root">' +
            html + '</div></body></html>',
            'text/html'
        );
        var root = doc.getElementById('ferro-rte-sanitize-root');
        if (!root) {
            return '';
        }

        walkSanitize(root, tagToFormat, allowedFormats, alwaysAllowed, alwaysStripped);

        return root.innerHTML;
    }

    function walkSanitize(node, tagToFormat, allowedFormats, alwaysAllowed, alwaysStripped) {
        // Iterate children in reverse so structural mutations (replace /
        // remove) do not invalidate the index of nodes we have not visited.
        var children = node.childNodes;
        for (var i = children.length - 1; i >= 0; i--) {
            var child = children[i];
            if (child.nodeType === 1) {
                walkSanitize(child, tagToFormat, allowedFormats, alwaysAllowed, alwaysStripped);
                var tag = child.tagName;
                if (alwaysStripped[tag]) {
                    child.parentNode.removeChild(child);
                    continue;
                }
                if (alwaysAllowed[tag]) {
                    stripDisallowedAttributes(child);
                    continue;
                }
                var fmt = tagToFormat[tag];
                if (fmt && allowedFormats[fmt]) {
                    stripDisallowedAttributes(child);
                    continue;
                }
                // Unknown tag or disallowed format: replace with text content.
                var text = child.textContent || '';
                var textNode = node.ownerDocument.createTextNode(text);
                child.parentNode.replaceChild(textNode, child);
            }
            // Text nodes (nodeType 3) and other types are kept as-is.
        }
    }

    function stripDisallowedAttributes(el) {
        // Defensive: collect attribute names first so we can mutate
        // without invalidating live NamedNodeMap iteration.
        var names = [];
        for (var i = 0; i < el.attributes.length; i++) {
            names.push(el.attributes[i].name);
        }
        var tag = el.tagName;
        for (var j = 0; j < names.length; j++) {
            var attr = names[j];
            var lower = attr.toLowerCase();
            // Strip event handlers always.
            if (lower.indexOf('on') === 0) {
                el.removeAttribute(attr);
                continue;
            }
            // Strip style and class (except ql-*).
            if (lower === 'style') {
                el.removeAttribute(attr);
                continue;
            }
            if (lower === 'class') {
                var cls = el.getAttribute(attr) || '';
                var keep = [];
                var parts = cls.split(/\s+/);
                for (var k = 0; k < parts.length; k++) {
                    if (parts[k].indexOf('ql-') === 0) {
                        keep.push(parts[k]);
                    }
                }
                if (keep.length === 0) {
                    el.removeAttribute(attr);
                } else {
                    el.setAttribute(attr, keep.join(' '));
                }
                continue;
            }
            // For <a>: keep only href and target.
            if (tag === 'A') {
                if (lower !== 'href' && lower !== 'target') {
                    el.removeAttribute(attr);
                }
                continue;
            }
            // For everything else: strip src and any data-* / aria-* not
            // explicitly required. Quill's own formatting may add
            // data-list, data-indent — keep those.
            if (lower === 'src') {
                el.removeAttribute(attr);
                continue;
            }
            if (lower.indexOf('data-list') === 0 || lower.indexOf('data-indent') === 0) {
                continue;
            }
            if (lower.indexOf('data-') === 0 || lower.indexOf('aria-') === 0) {
                el.removeAttribute(attr);
            }
        }
    }
"#;
