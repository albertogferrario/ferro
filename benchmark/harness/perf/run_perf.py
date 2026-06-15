# run_perf.py — drive oha against a target URL, write normalized metrics JSON.
import subprocess, sys, json, os
sys.path.insert(0, os.path.dirname(__file__))
from parse_perf import parse_oha

# Default micro-benchmark endpoints (Phase 229 micro workload).
MICRO_ENDPOINTS = ("/json", "/db", "/queries?n=20", "/updates?n=20")

# Conduit (RealWorld) read-path endpoints — representative of the real-app workload.
# {slug} is substituted by the caller; the authed /api/user endpoint is driven via an
# auth header (see `auth_header`). Kept stable across Ferro/Laravel for comparability.
CONDUIT_ENDPOINTS = (
    "/api/tags",
    "/api/articles?limit=20",
    "/api/articles?limit=20&offset=0&tag=dragons",
    "/api/profiles/celeb",
)

def run(base_url: str, framework: str, out_dir: str,
        endpoints=MICRO_ENDPOINTS,
        duration="30s", concurrency="256", warmup="5s",
        auth_header: str | None = None, authed_endpoints=()):
    """Drive oha against each endpoint, write perf-{framework}.json.

    `endpoints` are unauthenticated GETs. `authed_endpoints` (optional) are driven with
    `auth_header` (e.g. "Authorization: Token <jwt>") via oha's -H flag — used for the
    Conduit authed read path (GET /api/user)."""
    perf = {}
    plan = [(ep, None) for ep in endpoints] + \
           [(ep, auth_header) for ep in authed_endpoints if auth_header]
    for ep, hdr in plan:
        url = base_url.rstrip("/") + ep
        hdr_args = ["-H", hdr] if hdr else []
        # warm-up (discarded)
        subprocess.run(["oha", "-z", warmup, "-c", concurrency, "--no-tui", *hdr_args, url],
                       capture_output=True, text=True)
        raw = subprocess.run(
            ["oha", "-z", duration, "-c", concurrency, "--no-tui",
             "--output-format", "json", *hdr_args, url],
            capture_output=True, text=True, check=True).stdout
        key = ep.split("?")[0]
        perf[key] = parse_oha(raw)
    os.makedirs(out_dir, exist_ok=True)
    path = os.path.join(out_dir, f"perf-{framework}.json")
    with open(path, "w") as fh:
        json.dump(perf, fh, indent=2)
    print(f"wrote {path}")

if __name__ == "__main__":
    # argv: base_url framework out_dir [preset] [auth_header]
    #   preset: "micro" (default) | "conduit"
    #   auth_header (conduit only): full header string for GET /api/user
    base_url, framework, out_dir = sys.argv[1], sys.argv[2], sys.argv[3]
    preset = sys.argv[4] if len(sys.argv) > 4 else "micro"
    auth = sys.argv[5] if len(sys.argv) > 5 else None
    if preset == "conduit":
        run(base_url, framework, out_dir,
            endpoints=CONDUIT_ENDPOINTS,
            auth_header=auth, authed_endpoints=("/api/user",) if auth else ())
    else:
        run(base_url, framework, out_dir)
