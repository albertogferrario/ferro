# test_parse_perf.py
import json
from parse_perf import parse_oha

SAMPLE = json.dumps({
    "summary": {"requestsPerSec": 12345.6, "total": 60.0, "successRate": 1.0},
    "latencyPercentiles": {"p50": 0.0012, "p90": 0.004, "p99": 0.009},
})

def test_parse_oha_extracts_core_metrics():
    m = parse_oha(SAMPLE)
    assert m["rps"] == 12345.6
    assert m["p50_ms"] == 1.2
    assert m["p99_ms"] == 9.0
    assert m["success_rate"] == 1.0

def test_parse_oha_rejects_low_success_rate():
    bad = json.dumps({"summary": {"requestsPerSec": 1.0, "successRate": 0.5},
                      "latencyPercentiles": {"p50": 0.1, "p99": 0.2}})
    try:
        parse_oha(bad)
        assert False, "expected ValueError"
    except ValueError:
        pass
