# test_conformance.py — BASE_URL env points at a running app; asserts the shared contract.
import os, requests

BASE = os.environ["BASE_URL"].rstrip("/")

def test_json():
    r = requests.get(f"{BASE}/json")
    assert r.headers["content-type"].startswith("application/json")
    assert r.json() == {"message": "Hello, World!"}

def test_db():
    o = requests.get(f"{BASE}/db").json()
    assert set(o) == {"id", "randomNumber"} and 1 <= o["id"] <= 10000

def test_queries_clamps():
    assert len(requests.get(f"{BASE}/queries?n=600").json()) == 500
    assert len(requests.get(f"{BASE}/queries?n=0").json()) == 1
    assert len(requests.get(f"{BASE}/queries?n=5").json()) == 5

def test_updates_returns_k():
    assert len(requests.get(f"{BASE}/updates?n=7").json()) == 7
