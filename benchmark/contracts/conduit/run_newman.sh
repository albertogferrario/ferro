#!/usr/bin/env bash
# Run the vendored official RealWorld/Conduit Postman collection with Newman.
#
# Usage:
#   ./run_newman.sh [APIURL] [FOLDER]
#
#   APIURL   Base API URL (default: http://localhost:3000/api)
#   FOLDER   Optional Postman folder name to run a single group
#            (e.g. "Error Cases - Auth", "Articles, Favorite, Comments").
#
# Newman is invoked via `npx newman` so a global install is not required.
# Exits non-zero if any assertion fails (Newman's native behaviour).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COLLECTION="$HERE/Conduit.postman_collection.json"
APIURL="${1:-http://localhost:3000/api}"
FOLDER="${2:-}"

NEWMAN=(newman)
if ! command -v newman >/dev/null 2>&1; then
  NEWMAN=(npx --yes newman)
fi

ARGS=(run "$COLLECTION"
  --env-var "APIURL=$APIURL"
  --reporters cli,json
  --reporter-json-export "$HERE/newman-result.json")

if [[ -n "$FOLDER" ]]; then
  ARGS+=(--folder "$FOLDER")
fi

echo "Running Newman against APIURL=$APIURL${FOLDER:+ (folder: $FOLDER)}"
exec "${NEWMAN[@]}" "${ARGS[@]}"
