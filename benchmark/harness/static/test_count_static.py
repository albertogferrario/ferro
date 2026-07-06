# test_count_static.py
import json, subprocess
import count_static
from count_static import summarize_tokei, count_tokens, run_with_carveout

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

def test_run_with_carveout_subtracts_hand_rolled(monkeypatch, tmp_path):
    # The carved-out (hand-rolled JWT) file exists; total is 100, carveout is 30.
    jwt = tmp_path / "src" / "jwt.rs"
    jwt.parent.mkdir(parents=True)
    jwt.write_text("// jwt")

    monkeypatch.setattr(count_static, "run",
                        lambda app_dir: {"code_lines": 100, "files": 5, "source_tokens": 500})
    monkeypatch.setattr(count_static, "count_code_lines", lambda paths: 30)

    out = run_with_carveout(str(tmp_path), ["src/jwt.rs"],
                            "not framework-provided (JWT auth — Ferro is session-based)")
    assert out["hand_rolled"]["code_lines"] == 30
    assert out["hand_rolled"]["files"] == ["src/jwt.rs"]
    assert "not framework-provided" in out["hand_rolled"]["label"]
    # framework_provided == total - hand_rolled (the honesty invariant)
    assert out["framework_provided_code_lines"] == out["code_lines"] - out["hand_rolled"]["code_lines"]
    assert out["framework_provided_code_lines"] == 70
