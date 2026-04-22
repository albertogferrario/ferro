//! Runtime module: KeyValueEditor.
//!
//! Wires add-row, delete-row, and input-synced hidden-field serialization for
//! `[data-kv-editor]` wrappers rendered by `render_key_value_editor`.
//!
//! - Add-row: clones `[data-kv-row-template].content` and appends to
//!   `[data-kv-rows]`.
//! - Delete-row: delegated `click` on `[data-kv-rows]`, matches
//!   `[data-kv-delete]`, walks up to the nearest `[data-kv-row]` and removes it.
//! - Input sync: delegated `input` event on `[data-kv-rows]`, reads every
//!   `[data-kv-row]`'s `[data-kv-key]` and `[data-kv-value]`, serializes as
//!   a JSON object, writes to the hidden input whose `name` matches
//!   `data-kv-field`.
//!
//! Empty rows (blank key) are excluded from the serialized object.
//! Duplicate keys take last-write-wins. Vanilla ES5: `var` only, named
//! function declarations, no arrow functions, no closures capturing `this`.

pub(super) const SOURCE: &str = r#"
    // ── Key-value editor ──────────────────────────────────────────────────

    function setupKeyValueEditor() {
        var editors = document.querySelectorAll('[data-kv-editor]');
        for (var i = 0; i < editors.length; i++) {
            initKeyValueEditor(editors[i]);
        }
    }

    function initKeyValueEditor(editor) {
        var rowsContainer = editor.querySelector('[data-kv-rows]');
        var addBtn = editor.querySelector('[data-kv-add]');
        var tmpl = editor.querySelector('[data-kv-row-template]');

        if (!rowsContainer || !addBtn || !tmpl) {
            return;
        }

        addBtn.addEventListener('click', function() {
            var clone = tmpl.content.cloneNode(true);
            rowsContainer.appendChild(clone);
            syncHiddenField(editor);
        });

        rowsContainer.addEventListener('click', function(e) {
            var target = e.target;
            // Walk up from target to nearest [data-kv-delete] (if any).
            var delBtn = target.closest ? target.closest('[data-kv-delete]') : null;
            if (!delBtn) {
                return;
            }
            var row = delBtn.closest('[data-kv-row]');
            if (row && row.parentNode) {
                row.parentNode.removeChild(row);
                syncHiddenField(editor);
            }
        });

        rowsContainer.addEventListener('input', function(e) {
            var target = e.target;
            if (!target || !target.hasAttribute) {
                return;
            }
            if (target.hasAttribute('data-kv-key') || target.hasAttribute('data-kv-value')) {
                syncHiddenField(editor);
            }
        });
    }

    function syncHiddenField(editor) {
        var fieldName = editor.getAttribute('data-kv-field');
        if (!fieldName) {
            return;
        }
        var hiddenInput = editor.querySelector(
            'input[name="' + fieldName + '"][type="hidden"]'
        );
        if (!hiddenInput) {
            return;
        }
        var rows = editor.querySelectorAll('[data-kv-row]');
        var obj = {};
        for (var i = 0; i < rows.length; i++) {
            var keyEl = rows[i].querySelector('[data-kv-key]');
            var valEl = rows[i].querySelector('[data-kv-value]');
            var k = keyEl ? String(keyEl.value).replace(/^\s+|\s+$/g, '') : '';
            var v = valEl ? valEl.value : '';
            if (k !== '') {
                // Last-write-wins on duplicate keys (D-10).
                obj[k] = v;
            }
        }
        hiddenInput.value = JSON.stringify(obj);
    }
"#;
