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
        ["tokei", "--output", "json",
         "--exclude", "vendor", "--exclude", "target", "--exclude", "docker",
         app_dir],
        capture_output=True, text=True, check=True,
    ).stdout
    summary = summarize_tokei(out)
    listing = subprocess.run(
        ["tokei", "--files", "--output", "json",
         "--exclude", "vendor", "--exclude", "target", "--exclude", "docker",
         app_dir],
        capture_output=True, text=True, check=True,
    ).stdout
    paths = [r["name"] for lang, v in json.loads(listing).items()
             if lang != "Total" for r in v.get("reports", [])]
    summary["source_tokens"] = count_tokens(paths)
    return summary

def count_code_lines(paths: list[str]) -> int:
    """tokei code-line count over an explicit file list (carveout)."""
    if not paths:
        return 0
    out = subprocess.run(
        ["tokei", "--output", "json", *paths],
        capture_output=True, text=True, check=True,
    ).stdout
    return summarize_tokei(out)["code_lines"]

def run_with_carveout(app_dir: str, carveout_paths: list[str], label: str) -> dict:
    """Total static count for `app_dir` plus a SEPARATE count for an explicit list of
    "not framework-provided" files (`carveout_paths`, relative to app_dir).

    The carved-out code is subtracted from a `framework_provided_code_lines` figure so
    the framework-provided line count is never overstated (D-10 honesty). Used for the
    hand-rolled JWT modules (src/jwt.rs + JWT middleware) that Ferro does not provide —
    Ferro auth is session-based."""
    import os
    summary = run(app_dir)
    abs_carveout = [os.path.join(app_dir, p) for p in carveout_paths]
    present = [p for p in abs_carveout if os.path.exists(p)]
    hand_rolled_lines = count_code_lines(present)
    summary["hand_rolled"] = {
        "files": carveout_paths,
        "code_lines": hand_rolled_lines,
        "label": label,
    }
    summary["framework_provided_code_lines"] = summary["code_lines"] - hand_rolled_lines
    return summary

if __name__ == "__main__":
    # Usage:
    #   count_static.py <app_dir>
    #   count_static.py <app_dir> --carveout <rel_path>[,<rel_path>...] [--label "..."]
    if "--carveout" in sys.argv:
        i = sys.argv.index("--carveout")
        app_dir = sys.argv[1]
        carveout = sys.argv[i + 1].split(",")
        label = "not framework-provided"
        if "--label" in sys.argv:
            label = sys.argv[sys.argv.index("--label") + 1]
        print(json.dumps(run_with_carveout(app_dir, carveout, label)))
    else:
        print(json.dumps(run(sys.argv[1])))
