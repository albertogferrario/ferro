#!/usr/bin/env bash
# Idempotent installer for the Tailwind v4 standalone CLI.
# Pins a single version used by both local dev and CI.
# Installs to .tooling/bin/tailwindcss (gitignored); safe to run repeatedly.
set -euo pipefail

TAILWIND_VERSION="v4.2.3"

REPO_ROOT="$(git rev-parse --show-toplevel)"
INSTALL_DIR="${REPO_ROOT}/.tooling/bin"
BINARY="${INSTALL_DIR}/tailwindcss"

if [[ -x "${BINARY}" ]] && "${BINARY}" --help 2>&1 | grep -q "tailwindcss v${TAILWIND_VERSION#v}"; then
  exit 0
fi

OS_RAW=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH_RAW=$(uname -m)

case "${OS_RAW}" in
  darwin) OS=macos ;;
  linux)  OS=linux ;;
  *) echo "error: unsupported OS '${OS_RAW}' (need darwin or linux)" >&2; exit 1 ;;
esac

case "${ARCH_RAW}" in
  x86_64|amd64)      ARCH=x64 ;;
  aarch64|arm64)     ARCH=arm64 ;;
  *) echo "error: unsupported arch '${ARCH_RAW}' (need x86_64 or arm64)" >&2; exit 1 ;;
esac

ASSET="tailwindcss-${OS}-${ARCH}"
BASE_URL="https://github.com/tailwindlabs/tailwindcss/releases/download/${TAILWIND_VERSION}"
BINARY_URL="${BASE_URL}/${ASSET}"
CHECKSUM_URL="${BASE_URL}/sha256sums.txt"

mkdir -p "${INSTALL_DIR}"

TMPDIR=$(mktemp -d)
trap 'rm -rf "${TMPDIR}"' EXIT

echo "downloading tailwindcss ${TAILWIND_VERSION} (${ASSET})..."
curl -fsSL -o "${TMPDIR}/${ASSET}"     "${BINARY_URL}"
curl -fsSL -o "${TMPDIR}/sha256sums.txt" "${CHECKSUM_URL}"

EXPECTED=$(awk -v asset="${ASSET}" '$2 == "./" asset || $2 == asset {print $1; exit}' "${TMPDIR}/sha256sums.txt")
if [[ -z "${EXPECTED}" ]]; then
  echo "error: could not find checksum for ${ASSET} in sha256sums.txt" >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  ACTUAL=$(sha256sum "${TMPDIR}/${ASSET}" | awk '{print $1}')
else
  ACTUAL=$(shasum -a 256 "${TMPDIR}/${ASSET}" | awk '{print $1}')
fi

if [[ "${EXPECTED}" != "${ACTUAL}" ]]; then
  echo "error: checksum mismatch for ${ASSET}" >&2
  echo "  expected: ${EXPECTED}" >&2
  echo "  actual:   ${ACTUAL}" >&2
  exit 1
fi

mv "${TMPDIR}/${ASSET}" "${BINARY}"
chmod +x "${BINARY}"
echo "installed: ${BINARY}"
