# test_count_static.py
import json, subprocess
from count_static import summarize_tokei, count_tokens

TOKEI_JSON = json.dumps({
    "Rust":  {"code": 120, "comments": 10, "blanks": 5, "reports": [{"name":"a.rs"},{"name":"b.rs"}]},
    "Total": {"code": 120, "comments": 10, "blanks": 5},
})

def test_summarize_tokei_totals_code_and_files():
    s = summarize_tokei(TOKEI_JSON)
    assert s["code_lines"] == 120
    assert s["files"] == 2

def test_count_tokens_counts_whitespace_separated(tmp_path):
    f = tmp_path / "x.txt"
    f.write_text("one two three\nfour")
    assert count_tokens([str(f)]) == 4
