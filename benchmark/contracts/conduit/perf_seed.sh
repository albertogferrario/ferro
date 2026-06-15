#!/usr/bin/env bash
# Seed a stable, comparable dataset for the perf read-path workload:
#   - a `celeb` user (so GET /api/profiles/celeb is 200)
#   - N dragons-tagged articles (so /api/articles and the tag filter return data)
# Idempotent-ish: registers celeb if absent (ignores duplicate), then tops up articles.
#
# Usage: ./perf_seed.sh APIURL [N]
set -euo pipefail
APIURL="${1:?APIURL required}"
N="${2:-25}"

# Register or log in celeb.
REG=$(curl -s -X POST "$APIURL/users" -H 'Content-Type: application/json' \
  -d '{"user":{"username":"celeb","email":"celeb@bench.local","password":"benchPassw0rd!"}}' || true)
TOKEN=$(printf '%s' "$REG" | python3 -c "import sys,json
try: print(json.load(sys.stdin)['user']['token'])
except Exception: print('')")
if [ -z "$TOKEN" ]; then
  LOGIN=$(curl -s -X POST "$APIURL/users/login" -H 'Content-Type: application/json' \
    -d '{"user":{"email":"celeb@bench.local","password":"benchPassw0rd!"}}')
  TOKEN=$(printf '%s' "$LOGIN" | python3 -c "import sys,json;print(json.load(sys.stdin)['user']['token'])")
fi
[ -n "$TOKEN" ] || { echo "could not obtain celeb token"; exit 1; }

for i in $(seq 1 "$N"); do
  curl -s -o /dev/null -X POST "$APIURL/articles" \
    -H 'Content-Type: application/json' -H "Authorization: Token $TOKEN" \
    -d "{\"article\":{\"title\":\"Dragon perf article $i\",\"description\":\"perf seed $i\",\"body\":\"body $i\",\"tagList\":[\"dragons\",\"perf\"]}}" || true
done

COUNT=$(curl -s "$APIURL/articles?limit=1" | python3 -c "import sys,json;print(json.load(sys.stdin).get('articlesCount','?'))")
echo "seeded; articlesCount=$COUNT; celeb profile ready at $APIURL/profiles/celeb"
