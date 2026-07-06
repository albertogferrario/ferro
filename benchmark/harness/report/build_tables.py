# build_tables.py
import json, sys, glob, os

def render_markdown(data: dict) -> str:
    fw = list(data.keys())
    endpoints = sorted({ep for f in fw for ep in data[f]["perf"]})
    lines = ["## Raw performance (requests/sec)", "",
             "| Endpoint | " + " | ".join(fw) + " | ratio |", "|" + "---|" * (len(fw) + 2)]
    for ep in endpoints:
        rps = [data[f]["perf"][ep]["rps"] for f in fw]
        hi, lo = max(rps), min(rps)
        ratio = f"{hi/lo:.2f}x" if lo else "n/a"
        lines.append("| " + ep + " | " + " | ".join(f"{r:,.0f}" for r in rps) + f" | {ratio} |")
    lines += ["", "## Static compression", "",
              "| Metric | " + " | ".join(fw) + " |", "|" + "---|" * (len(fw) + 1)]
    for metric in ("code_lines", "source_tokens"):
        vals = [str(data[f]["static"][metric]) for f in fw]
        lines.append(f"| {metric} | " + " | ".join(vals) + " |")
    return "\n".join(lines) + "\n"

def load_results(date_dir: str) -> dict:
    data = {}
    for path in glob.glob(os.path.join(date_dir, "*.json")):
        name = os.path.basename(path)[:-5]  # strip .json
        # Only process perf-<fw>.json and static-<fw>.json; skip meta.json etc.
        if not name.startswith(("perf-", "static-")):
            continue
        kind, fw = name.split("-", 1)
        with open(path) as fh:
            payload = json.load(fh)
        data.setdefault(fw, {})["perf" if kind == "perf" else "static"] = payload
    return data

if __name__ == "__main__":
    date_dir = sys.argv[1]
    md = render_markdown(load_results(date_dir))
    with open(os.path.join(date_dir, "internal.md"), "w") as fh:
        fh.write(md)
    print(md)
