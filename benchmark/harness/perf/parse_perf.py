# parse_perf.py
import json

def parse_oha(raw: str) -> dict:
    """Normalize `oha --output-format json` output into a flat metrics dict (latency in ms)."""
    d = json.loads(raw)
    s = d["summary"]
    p = d["latencyPercentiles"]
    success = float(s.get("successRate", 1.0))
    if success < 0.99:
        raise ValueError(f"success rate too low to trust results: {success}")
    return {
        "rps": float(s["requestsPerSec"]),
        "p50_ms": round(float(p["p50"]) * 1000, 3),
        "p90_ms": round(float(p.get("p90", 0)) * 1000, 3),
        "p99_ms": round(float(p["p99"]) * 1000, 3),
        "success_rate": success,
    }
