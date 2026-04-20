#!/usr/bin/env bash
# Regenerate ferro-json-ui/assets/ferro-base.css using the Tailwind v4 CLI.
# Run from repo root.
#
# Resolution order:
#   1. `tailwindcss` standalone binary on PATH
#   2. `npx @tailwindcss/cli` (npm must be available)
#
# Standalone binary: https://tailwindcss.com/blog/standalone-cli
# npm install:       npm install --global @tailwindcss/cli
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

_run_tailwind() {
  "$@" \
    -i ferro-json-ui/assets/input.css \
    -o ferro-json-ui/assets/ferro-base.css \
    --minify
}

_find_tailwind() {
  # standalone binary locations
  for candidate in tailwindcss "$HOME/.local/bin/tailwindcss" /usr/local/bin/tailwindcss; do
    if command -v "$candidate" >/dev/null 2>&1; then
      echo "$candidate"
      return 0
    fi
  done
  return 1
}

if TW=$(_find_tailwind); then
  _run_tailwind "$TW"
else
  echo "error: tailwindcss standalone CLI not found" >&2
  echo "" >&2
  echo "Install options:" >&2
  echo "  macOS arm64: python3 -c \"import urllib.request,os,stat; p=os.path.expanduser('~/.local/bin/tailwindcss'); os.makedirs(os.path.dirname(p),exist_ok=True); urllib.request.urlretrieve('https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-macos-arm64',p); os.chmod(p,os.stat(p).st_mode|0o111)\"" >&2
  echo "  macOS x64:   brew install tailwindcss  (standalone, not npm wrapper)" >&2
  echo "  Linux x64:   curl -sLo /usr/local/bin/tailwindcss https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64 && chmod +x /usr/local/bin/tailwindcss" >&2
  exit 1
fi

echo "regenerated: ferro-json-ui/assets/ferro-base.css ($(wc -c < ferro-json-ui/assets/ferro-base.css) bytes)"
