#!/usr/bin/env bash
# Seed the globals the vintage Conduit collection expects, then run the FULL
# collection against a live backend with Newman.
#
# This collection has no request that registers the primary user or creates the
# first article — it consumes pre-seeded globals (USERNAME, EMAIL, PASSWORD, token,
# slug). The canonical RealWorld flow folds those into environment setup; we
# reproduce them here so the like-for-like gate is identical for both backends.
#
# Usage: ./seed_and_run.sh APIURL OUT_JSON
#   APIURL    e.g. http://localhost:3002/api
#   OUT_JSON  where to copy the machine-readable newman report
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COLLECTION="$HERE/Conduit.postman_collection.json"
APIURL="${1:?APIURL required}"
OUT_JSON="${2:?OUT_JSON required}"

STAMP="$(date +%s)"
USERNAME="primary_${STAMP}"
EMAIL="primary_${STAMP}@bench.local"
PASSWORD="benchPassw0rd!"

echo "Seeding primary user $EMAIL at $APIURL ..."
REG=$(curl -s -X POST "$APIURL/users" \
  -H 'Content-Type: application/json' \
  -d "{\"user\":{\"username\":\"$USERNAME\",\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}}")
TOKEN=$(printf '%s' "$REG" | python3 -c "import sys,json;print(json.load(sys.stdin)['user']['token'])")
[ -n "$TOKEN" ] || { echo "registration failed: $REG"; exit 1; }

echo "Creating seed article (dragons tag) ..."
ART=$(curl -s -X POST "$APIURL/articles" \
  -H 'Content-Type: application/json' \
  -H "Authorization: Token $TOKEN" \
  -d '{"article":{"title":"How to train your dragon '"$STAMP"'","description":"Ever wonder how?","body":"You have to believe","tagList":["dragons","training"]}}')
SLUG=$(printf '%s' "$ART" | python3 -c "import sys,json;print(json.load(sys.stdin)['article']['slug'])")
[ -n "$SLUG" ] || { echo "article create failed: $ART"; exit 1; }

GLOBALS="$(mktemp)"
python3 - "$GLOBALS" "$APIURL" "$USERNAME" "$EMAIL" "$PASSWORD" "$TOKEN" "$SLUG" <<'PY'
import json, sys
path, apiurl, username, email, password, token, slug = sys.argv[1:8]
vals = {"APIURL": apiurl, "USERNAME": username, "EMAIL": email,
        "PASSWORD": password, "token": token, "slug": slug}
json.dump({"id": "conduit-seed", "name": "conduit-seed",
           "values": [{"key": k, "value": v, "enabled": True} for k, v in vals.items()]},
          open(path, "w"))
PY

NEWMAN=(newman)
command -v newman >/dev/null 2>&1 || NEWMAN=(npx --yes newman)

echo "Running FULL Newman collection against $APIURL ..."
set +e
"${NEWMAN[@]}" run "$COLLECTION" \
  --globals "$GLOBALS" \
  --env-var "APIURL=$APIURL" \
  --reporters cli,json \
  --reporter-json-export "$OUT_JSON"
RC=$?
set -e
rm -f "$GLOBALS"
exit $RC
