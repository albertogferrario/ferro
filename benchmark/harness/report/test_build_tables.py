# test_build_tables.py
from build_tables import render_markdown

DATA = {
  "ferro":   {"perf": {"/json": {"rps": 200000.0, "p99_ms": 1.2}},
              "static": {"code_lines": 40, "files": 3, "source_tokens": 180}},
  "laravel": {"perf": {"/json": {"rps": 9000.0, "p99_ms": 30.0}},
              "static": {"code_lines": 70, "files": 5, "source_tokens": 360}},
}

def test_render_includes_both_frameworks_and_endpoints():
    md = render_markdown(DATA)
    assert "ferro" in md and "laravel" in md
    assert "/json" in md
    assert "200000" in md or "200,000" in md

def test_render_marks_winner_per_metric():
    md = render_markdown(DATA)
    # ferro wins rps; the table notes the ratio honestly
    assert "22.2x" in md or "22.22x" in md

def test_static_table_omits_files():
    md = render_markdown(DATA)
    lines = [l for l in md.splitlines() if "files" in l.lower()]
    # the word "files" must not appear as a metric row in the static table
    assert not any(l.startswith("| files") for l in lines)
