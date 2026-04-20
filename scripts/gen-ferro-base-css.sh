#!/usr/bin/env bash
# Regenerate ferro-json-ui/assets/ferro-base.css using the Tailwind v4 standalone CLI.
# Run from repo root. Requires `tailwindcss` on PATH (https://tailwindcss.com/blog/standalone-cli).
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

if ! command -v tailwindcss >/dev/null 2>&1; then
  echo "error: tailwindcss CLI not found on PATH" >&2
  echo "install: curl -sLo /usr/local/bin/tailwindcss https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-macos-arm64 && chmod +x /usr/local/bin/tailwindcss" >&2
  exit 1
fi

tailwindcss \
  -i ferro-json-ui/assets/input.css \
  -o ferro-json-ui/assets/ferro-base.css \
  --minify

echo "regenerated: ferro-json-ui/assets/ferro-base.css ($(wc -c < ferro-json-ui/assets/ferro-base.css) bytes)"
