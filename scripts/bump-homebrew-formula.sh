#!/usr/bin/env bash
# Render Formula/ferro.rb from template with computed SHA256s and push to tap repo.
# Usage (from CI): VERSION=v0.2.59 bash scripts/bump-homebrew-formula.sh
#          (local): VERSION=v0.2.59 HOMEBREW_TAP_TOKEN=<pat> bash scripts/bump-homebrew-formula.sh
set -euo pipefail

VERSION="${1:-${VERSION:?VERSION env var required}}"
TAG="$VERSION"
VER="${TAG#v}"  # strip leading 'v'

BASE_URL="https://github.com/albertogferrario/ferro/releases/download/${TAG}"
TEMPLATE="homebrew/Formula/ferro.rb.tpl"
TAP_REPO="albertogferrario/homebrew-ferro"

compute_sha256() {
  # Works on both macOS (shasum) and Linux (sha256sum / shasum from homebrew)
  curl -fsSL "$1" | shasum -a 256 | awk '{print $1}'
}

echo "Computing SHA256 for ferro ${VER} tarballs..."
SHA256_MACOS_ARM64=$(compute_sha256    "${BASE_URL}/ferro-${TAG}-aarch64-apple-darwin.tar.gz")
SHA256_MACOS_X86_64=$(compute_sha256   "${BASE_URL}/ferro-${TAG}-x86_64-apple-darwin.tar.gz")
SHA256_LINUX_AARCH64=$(compute_sha256  "${BASE_URL}/ferro-${TAG}-aarch64-unknown-linux-gnu.tar.gz")
SHA256_LINUX_X86_64=$(compute_sha256   "${BASE_URL}/ferro-${TAG}-x86_64-unknown-linux-gnu.tar.gz")

echo "Rendering formula template..."
FORMULA=$(sed \
  -e "s/VERSION_PLACEHOLDER/${VER}/g" \
  -e "s/SHA256_MACOS_ARM64/${SHA256_MACOS_ARM64}/g" \
  -e "s/SHA256_MACOS_X86_64/${SHA256_MACOS_X86_64}/g" \
  -e "s/SHA256_LINUX_AARCH64/${SHA256_LINUX_AARCH64}/g" \
  -e "s/SHA256_LINUX_X86_64/${SHA256_LINUX_X86_64}/g" \
  "${TEMPLATE}")

echo "Cloning tap repo..."
git clone "https://x-access-token:${HOMEBREW_TAP_TOKEN}@github.com/${TAP_REPO}.git" _tap_clone
printf '%s\n' "$FORMULA" > _tap_clone/Formula/ferro.rb

cd _tap_clone
git config user.name  "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"
git add Formula/ferro.rb
if git diff --staged --quiet; then
  echo "Formula already at ${VER}, nothing to push."
  exit 0
fi
git commit -m "chore: bump ferro to ${VER}"
git push
echo "Tap updated to ${VER}."
