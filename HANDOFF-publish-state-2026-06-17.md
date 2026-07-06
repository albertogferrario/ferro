# Ferro handoff — publish-state divergence + 0.2.69 release (2026-06-17)

> ## ✅ CLOSED — superseded through 0.2.73 (2026-06-22)
> Publish state has since advanced cleanly via the canonical
> `push master → publish.yml` flow. Latest live: **`ferro-rs 0.2.73`** on
> crates.io (CSS regen + workspace bump 0.2.72 → 0.2.73), tag `v0.2.73` pushed
> with all 5 platform release binaries, GitHub Release created, install script /
> brew tap updated, and E2E-from-release-artifact green. The 0.2.69 divergence
> this doc was written for is fully resolved; the canonical flow has held for
> every release since (0.2.70 ferro-payments, 0.2.71/0.2.72 patches, 0.2.73). No
> open publish-state action remains. _Doc retained for historical context only._

> ## ✅ RESOLVED — 2026-06-17 (ferro side)
> **`ferro-rs 0.2.69` is published to crates.io** (whole workspace), and `master`
> is pushed and current. The gestiscilo SegmentedControl/SidebarLayout closeout is
> **unblocked** — run `app/tmp/ferro-0269-closeout.sh`.
>
> Point-by-point against the requests below:
> 1. **Publish flow** — canonical path is `push master → publish.yml` (auto
>    library-change gate → patch-bump → `cargo publish` waves → post-publish
>    scaffold smoke). It was used for 0.2.67/0.2.68/0.2.69 and works. The flow was
>    never broken; the alarming numbers in this report came from the **stale local
>    `origin/master` ref** (SSH fetch is denied in these sessions, so `origin/*`
>    refs lie — here it read 0.2.58 / "366 ahead"). Get true remote state via
>    `gh api repos/albertogferrario/ferro/commits/master` and HTTPS fetch via
>    `git -c credential.helper='!gh auth git-credential' fetch https://github.com/albertogferrario/ferro.git master`. The stale `origin/master` ref has now been corrected locally.
> 2. **Release-ready** — 0.2.69 published with the CI Test gate green (the first
>    0.2.69 attempt was correctly *blocked* by Test on two ferro-mcp builtin-count
>    mirrors; fixed, re-published clean). Full CI matrix (Docs, css-drift, Clippy,
>    Test) is green at HEAD.
> 3. **Publish 0.2.69** — DONE.
> 4. **Push master** — DONE. True remote = local HEAD; the only delta was 2
>    `.planning`-only payments-design commits (excluded by the publish gate, so no
>    spurious release), now pushed.
>
> Note on coupling (req. §"gestiscilo coupling"): 0.2.69 adds only the two new
> primitives + a `no_required`/count-guard test adjustment; no breaking public API
> change beyond SegmentedControl/SidebarLayout. The closeout `cargo check` should
> be clean.
>
> _Original report below, preserved for context (premise now superseded)._

---

Written for the ferro developer by the gestiscilo session that hit this while
trying to close out the SegmentedControl/SidebarLayout consumer work. Nothing in
ferro was pushed or published — this is a read-only report + a request.

## TL;DR
`master` is **366 commits ahead of `origin/master`** (4 days: 2026-06-13 → 06-17),
and the three version anchors disagree:

| where | version |
|---|---|
| `origin/master` | **0.2.58** |
| crates.io (what gestiscilo consumes today) | **0.2.66** |
| local `master` (`Cargo.toml`) | **0.2.69** |

crates.io is **8 versions ahead of `origin/master`** but behind local. So the
`publish.yml` "push to master → auto patch-bump + `cargo publish`" flow is **not**
the path that produced 0.2.59–0.2.66 on crates.io — those were published some
other way (local `cargo publish`?) without updating origin. The publish flow and
the git remote have drifted apart.

## Why this matters right now
gestiscilo has finished consuming the new **SegmentedControl** and **SidebarLayout**
primitives (Calendario nav + Settings sidebar), but that work is **blocked**: it
builds locally only via a `[patch.crates-io]` override against this ferro checkout
(0.2.69). It can't ship until **0.2.69 is on crates.io**. A ready-to-run closeout
script is staged on the gestiscilo side (`app/tmp/ferro-0269-closeout.sh`) — it
runs in ~5 min the moment 0.2.69 publishes.

## What's in the 366 unpushed commits
Healthy, accumulated work — no WIP/experimental/revert code markers (the only
"broken" hits are docs commits *fixing* broken links). Breakdown by type:
`212 docs · 63 feat · 28 chore · 27 fix · 13 ci · 11 test · 6 style · 2 perf`.

Notable deliverables in the range:
- **SegmentedControl + SidebarLayout** json-ui primitives (+ `ferro-base.css`
  regenerated, catalog/builtin-count guards updated) — the gestiscilo blocker.
- CRUD handler proc-macros (phases 209 / 212 / 214).
- Statemachine derived-executor work (231 / 232).
- ferro-payments crate design + roadmap (233–236) — appears design/docs stage.
- Homebrew formula template + multi-arch release (226-01).
- Version bumps 0.2.59 → 0.2.69 (11 bumps).

## Requests (in priority order)
1. **Reconcile the publish flow.** Decide and document the canonical release path:
   is it `push master → publish.yml`, or manual `cargo publish`? Right now origin
   is 8 versions stale, which makes `publish.yml` unsafe to rely on (a 366-commit
   push would fire the auto-publish against a registry that already has
   0.2.59–0.2.66 → `cargo publish` rejects existing versions → partial/failed run).
2. **Confirm 0.2.69 is release-ready** — that every crate compiles + tests green at
   this HEAD, and that all 366 commits are intended to ship (no half-done phase
   meant to stay local). I could not build-validate 366 commits from the consumer
   side; that judgment is yours.
3. **Publish 0.2.69** (whichever way is canonical). That unblocks the gestiscilo
   SegmentedControl/SidebarLayout closeout immediately.
4. **Push `master`** so origin stops being 8 versions behind crates.io — even if
   publishing stays manual, the git history should not diverge this far.

## Note on gestiscilo coupling
gestiscilo `Cargo.toml` pins all ferro crates at `0.2.66` today (ferro-cache at
0.2.59, ferro-stripe 0.8.0 — both fine as-is). The closeout bumps the 15 patched
crates to `0.2.69` only. If 0.2.69 changes any public API the consumer relies on
beyond the two new primitives, flag it so the closeout `cargo check` doesn't
surprise us.
