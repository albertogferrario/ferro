#!/usr/bin/env bash
# Regenerate ferro-json-ui/assets/ferro-base.css using the pinned Tailwind v4 CLI.
# Run from anywhere; auto-installs the pinned binary into .tooling/bin/ on first use.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "${REPO_ROOT}"

bash scripts/install-tailwind.sh

.tooling/bin/tailwindcss \
  -i ferro-json-ui/assets/input.css \
  -o ferro-json-ui/assets/ferro-base.css \
  --minify

echo "regenerated: ferro-json-ui/assets/ferro-base.css ($(wc -c < ferro-json-ui/assets/ferro-base.css) bytes)"
