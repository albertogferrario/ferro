# count_static.py
import json, sys, subprocess

def summarize_tokei(raw: str) -> dict:
    d = json.loads(raw)
    files, code = 0, 0
    for lang, v in d.items():
        if lang == "Total":
            continue
        code += v.get("code", 0)
        files += len(v.get("reports", []))
    return {"code_lines": code, "files": files}

def count_tokens(paths: list[str]) -> int:
    total = 0
    for p in paths:
        with open(p, encoding="utf-8", errors="replace") as fh:
            total += len(fh.read().split())
    return total

def run(app_dir: str) -> dict:
    """tokei over app_dir (excluding vendored deps) + a whitespace token count of source."""
    out = subprocess.run(
        ["tokei", "--output", "json", "--exclude", "vendor", "--exclude", "target", app_dir],
        capture_output=True, text=True, check=True,
    ).stdout
    summary = summarize_tokei(out)
    listing = subprocess.run(
        ["tokei", "--files", "--output", "json", "--exclude", "vendor", "--exclude", "target", app_dir],
        capture_output=True, text=True, check=True,
    ).stdout
    paths = [r["name"] for lang, v in json.loads(listing).items()
             if lang != "Total" for r in v.get("reports", [])]
    summary["source_tokens"] = count_tokens(paths)
    return summary

if __name__ == "__main__":
    print(json.dumps(run(sys.argv[1])))
