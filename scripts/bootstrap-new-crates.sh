#!/usr/bin/env bash
#
# Bootstrap-publish the four ferro crates that have never been on crates.io.
# After this runs successfully, the GitHub Actions publish workflow's
# CARGO_REGISTRY_TOKEN should be able to update them in future releases.
#
# Order:
#   1. ferro-theme    — pure leaf, no internal deps
#   2. ferro-ai       — depends on ferro-events (already on crates.io @ 0.2.0)
#   3. ferro-stripe   — depends on ferro-events, ferro-queue (both on crates.io @ 0.2.0)
#   4. ferro-whatsapp — depends on ferro-events, ferro-queue (both on crates.io @ 0.2.0)
#
# Wait 30 seconds between each publish to let the crates.io sparse index update.
#
# Usage: ./scripts/bootstrap-new-crates.sh
#
# Requires: cargo authenticated via `cargo login` or CARGO_REGISTRY_TOKEN env var
#           with `publish-new` endpoint scope.

set -uo pipefail

CRATES=(ferro-theme ferro-ai ferro-stripe ferro-whatsapp)

# Sanity checks
if [ ! -f Cargo.toml ]; then
    echo "ERROR: must be run from the repo root (Cargo.toml not found in cwd)" >&2
    exit 1
fi

VERSION=$(grep -m1 '^version = "' Cargo.toml | sed 's/.*version = "\(.*\)"/\1/')
if [ -z "$VERSION" ]; then
    echo "ERROR: could not read workspace version from Cargo.toml" >&2
    exit 1
fi

if [ "$VERSION" != "0.2.0" ]; then
    echo "WARNING: workspace version is $VERSION, expected 0.2.0"
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    [[ ! $REPLY =~ ^[Yy]$ ]] && exit 1
fi

echo "About to publish the following crates to crates.io as version $VERSION:"
for crate in "${CRATES[@]}"; do
    echo "  - $crate"
done
echo
echo "This action is NOT REVERSIBLE. Once a crate version is published, it cannot be deleted."
echo
read -p "Proceed? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

FAILED=()
SKIPPED=()
PUBLISHED=()

for crate in "${CRATES[@]}"; do
    echo
    echo "===== Publishing $crate ====="

    OUTPUT=$(cargo publish -p "$crate" --no-verify 2>&1)
    STATUS=$?

    if [ $STATUS -eq 0 ]; then
        echo "$crate published successfully"
        PUBLISHED+=("$crate")
    elif echo "$OUTPUT" | grep -qE "already exists|already uploaded"; then
        echo "$crate already on crates.io, skipping"
        SKIPPED+=("$crate")
    else
        echo "FAILED to publish $crate"
        echo "$OUTPUT"
        FAILED+=("$crate")
        # Stop on first real failure to avoid cascading issues
        break
    fi

    if [ "$crate" != "${CRATES[-1]}" ]; then
        echo "Sleeping 30s for crates.io sparse index propagation..."
        sleep 30
    fi
done

echo
echo "===== Summary ====="
echo "Published: ${#PUBLISHED[@]}"
for c in "${PUBLISHED[@]}"; do echo "  ✓ $c"; done

if [ ${#SKIPPED[@]} -gt 0 ]; then
    echo "Skipped (already exists): ${#SKIPPED[@]}"
    for c in "${SKIPPED[@]}"; do echo "  ⊙ $c"; done
fi

if [ ${#FAILED[@]} -gt 0 ]; then
    echo "Failed: ${#FAILED[@]}"
    for c in "${FAILED[@]}"; do echo "  ✗ $c"; done
    exit 1
fi

echo
echo "All four crates are now on crates.io. Re-run the GitHub Actions"
echo "publish workflow — the 'already exists' check will skip these and"
echo "continue to ferro-json-ui, ferro-inertia, framework, mcp, cli."
