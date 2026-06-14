# Homebrew tap CI (for albertogferrario/homebrew-ferro)

This directory stages files the operator copies into the **external tap repo**
`albertogferrario/homebrew-ferro`. They are not used by this (ferro) repo's CI.

## Files

- `tests.yml` — place at `albertogferrario/homebrew-ferro/.github/workflows/tests.yml`.
  Runs `ruby -c` + `brew audit --strict` on every push to `main`, and adds `--online`
  on pull requests (the `--online` check needs the release tarball to be reachable; it is
  reliable on PRs but can race a just-pushed bump, so it is skipped on push).

## Seeding the tap

1. Create the public repo `albertogferrario/homebrew-ferro` (the `homebrew-` prefix is required
   for `brew tap albertogferrario/ferro` to resolve).
2. Copy this repo's `homebrew/Formula/ferro.rb` into `homebrew-ferro/Formula/ferro.rb` to bootstrap.
   (Its sha256 values are placeholders; the first real release's bump job overwrites them.)
3. Copy `tests.yml` into `homebrew-ferro/.github/workflows/tests.yml`.

See `226-04-PLAN.md` for the full operator runbook (PAT + secret + live verification).
