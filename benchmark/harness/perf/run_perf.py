# run_perf.py — drive oha against a target URL, write normalized metrics JSON.
import subprocess, sys, json, os
sys.path.insert(0, os.path.dirname(__file__))
from parse_perf import parse_oha

def run(base_url: str, framework: str, out_dir: str,
        endpoints=("/json", "/db", "/queries?n=20", "/updates?n=20"),
        duration="30s", concurrency="256", warmup="5s"):
    perf = {}
    for ep in endpoints:
        url = base_url.rstrip("/") + ep
        # warm-up (discarded)
        subprocess.run(["oha", "-z", warmup, "-c", concurrency, "--no-tui", url],
                       capture_output=True, text=True)
        raw = subprocess.run(
            ["oha", "-z", duration, "-c", concurrency, "--no-tui",
             "--output-format", "json", url],
            capture_output=True, text=True, check=True).stdout
        key = ep.split("?")[0]
        perf[key] = parse_oha(raw)
    os.makedirs(out_dir, exist_ok=True)
    path = os.path.join(out_dir, f"perf-{framework}.json")
    with open(path, "w") as fh:
        json.dump(perf, fh, indent=2)
    print(f"wrote {path}")

if __name__ == "__main__":
    # argv: base_url framework out_dir
    run(sys.argv[1], sys.argv[2], sys.argv[3])
