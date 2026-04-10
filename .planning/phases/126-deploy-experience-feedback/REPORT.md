# Deploy Experience Report — gestiscilo end-to-end (2026-04-08)

Captured during the first real end-to-end deploy of a Ferro app
(`gestiscilo-it/app`) against the Phase 122.2 deploy scaffold. The session
went from "I want to deploy this" to a successful local `docker build` of a
multi-bin image with chromium runtime, postgres-ready Cargo features, and
themes bundling. Two bugs were fixed inline (commit `70ad9ed4`, ferro 0.2.1);
the rest is unfiltered field notes for an agent to triage into phases.

The point of this report is **not** to prescribe phases. It is to give the
next agent a faithful, dated snapshot of where the scaffold helped, where it
hurt, and what surprised the user. The agent should read it, group items,
prioritize, and propose a phase plan back to the user.

---

## Bugs found and fixed today (already shipped in 0.2.1)

### 1. `rewrite_cargo_docker_toml` dropped dep metadata
`ferro-cli/src/deploy/rewrite_ferro_version.rs` was building a fresh `Map`
containing only `version`, dropping `package = "..."`, `features = [...]`,
`default-features`, and `optional`. Any project using a renamed crate or
feature flags would emit a phantom dep that fails to resolve on crates.io.

gestiscilo declares
```toml
ferro = { path = "...", package = "ferro-rs", features = ["json-ui", "theme"] }
```
The rewrite produced `[dependencies.ferro] version = "0.2.0"` — phantom crate,
no features, instant build failure.

**Fix:** preserve `package`, `features`, `default-features`, `optional`,
`registry`, `rename` from the original path-dep table. Regression test
`preserves_package_rename_and_features` added.

### 2. `rust:stable-slim-bookworm` is not a real Docker tag
`templates/files/docker/Dockerfile.tpl` interpolated `{{RUST_CHANNEL}}` into
`rust:{channel}-slim-bookworm`. When `rust-toolchain.toml` is absent,
`read_rust_channel` defaults to `"stable"`, producing `rust:stable-slim-bookworm`
— a tag Docker Hub does not publish. Build failed at the very first FROM with
`docker.io/library/rust:stable-slim-bookworm: not found`.

**Fix:** template now uses `{{RUST_IMAGE_TAG}}`. Renderer special-cases the
generic "stable" channel to drop the prefix, producing `rust:slim-bookworm`
(which tracks stable on Docker Hub). Regression test added.

---

## Bugs / sharp edges still present

### 3. Silent conflict between `copy_dirs` metadata and `.dockerignore`
I listed `data` in `[package.metadata.ferro.deploy].copy_dirs`, `docker:init`
happily emitted `COPY data data`, and `docker build` failed at runtime because
`.dockerignore` excludes `data/` (added in Phase 122.2 §8 — correct, since
`data/` is runtime artifacts).

The CLI knows both pieces and could fail loudly at `docker:init` time:
"copy_dirs entry `data` is excluded by .dockerignore". Cheap check, would have
saved a build cycle.

### 4. No version-skew detection between local ferro path deps and registry version
gestiscilo had source compiling against newer `ferro-json-ui` APIs than were
on crates.io (e.g. `CalendarCellProps.dot_colors`, `ImageProps.placeholder_label`,
`Action.target`). The Docker build was the first place this surfaced, after
several minutes of compiling deps.

`docker:init` should run a quick `cargo check --offline` against the rewritten
`Cargo.docker.toml` (or at minimum a `cargo metadata` resolve) and bail if it
can't resolve, with a hint to bump `ferro_version` or update consumer code.

### 5. `docker:init` reorders dep tables when re-serializing `Cargo.docker.toml`
Output is alphabetized — fine functionally, but produces a noisy diff and
obscures intent in code review. `toml_edit` would preserve order; the current
`toml` crate re-emits sorted. Worth swapping if `Cargo.docker.toml` is meant
to be human-reviewed.

### 6. Generated Dockerfile runs `cargo build --release` three times
The plain `cargo build --release` already builds every `[[bin]]`. The
per-bin runs (`cargo build --release --bin gestiscilo`,
`cargo build --release --bin screenshot-worker`) are no-ops in cache but
they're in the file and confusing on review. Drop them.

### 7. `docker:init` and `do:init` print success messages but don't tell the user what to do next
`do:init` in particular leaves an `app.yaml` with envs only as comments and no
hint about how to populate them — `doctl apps update` or dashboard? Users have
to grep the source. A two-line "Next steps" footer would help.

### 8. Publish workflow auto-bumps patch when current version is already tagged
Clever but means a push touching only `ferro-cli/` (which does not consume
from itself) still triggers a full crates.io release of every workspace
member. Today that worked in our favor (we needed 0.2.1 anyway), but it'll
churn versions on docs-only or CI-only commits. Worth gating on whether any
*library* crate actually changed.

### 9. No `--dry-run` on `docker:init` / `do:init`
I'd like to inspect the rendered output before it touches my project. Trivial
to add; useful in CI for "would this regenerate?" checks.

### 10. `.dockerignore` excludes `*.md` and `LICENSE`
Image builds fine, but cargo prints a warning per workspace member because
some crates declare `readme = "README.md"`. Not blocking, just noise. Either
include `README.md` only (whitelist) or document the warning.

### 18. Generated Dockerfile has no `ENTRYPOINT` or `CMD`
The Phase 122.2 `Dockerfile.tpl` ends after `EXPOSE 8080` with no
`ENTRYPOINT ["/usr/local/bin/<bin>"]` line. As a result the built image runs
debian's default `bash`, exits 0 immediately, and is unusable without the
caller passing an explicit command on every `docker run`.

Discovered during smoke test:

```
$ docker run --rm -p 8080:8080 --env-file .env.production gestiscilo:test
$ docker ps -a --filter name=gestiscilo-smoke
Exited (0) 3 seconds ago     # silent, no logs
```

The binary itself works fine when invoked explicitly:

```
$ docker run --rm gestiscilo:test /usr/local/bin/gestiscilo serve
thread 'main' panicked at src/main.rs:125:10:
Failed to connect to database: ...
```

Worse, this will break DigitalOcean App Platform too. The current `do:init`
template emits `run_command` only for the **worker** services, not for the
**web** service:

```yaml
services:
  - name: web
    dockerfile_path: Dockerfile
    # no run_command — relies on Dockerfile ENTRYPOINT
```

With no ENTRYPOINT in the Dockerfile, DO will run `bash` and the deployment
will silently exit 0 forever.

**Fix options:**
- Single-bin projects: emit `ENTRYPOINT ["/usr/local/bin/<package_name>"]`
  with the package name detected from `Cargo.toml`.
- Multi-bin projects: pick the bin matching the package name as the default
  service entrypoint, fall back to the first bin, or require the user to
  declare it in `[package.metadata.ferro.deploy].web_bin`. The current
  `do:init` already heuristically picks `web_bin = pkg` — apply the same
  detection to the Dockerfile ENTRYPOINT so they stay in sync.
- Either way, also emit `CMD ["serve"]` so the default subcommand runs without
  arguments. Today gestiscilo's main is a clap dispatcher with `serve` as the
  default-listed subcommand but it's not actually the clap default — the
  binary prints help and exits when invoked with no args.

**Severity:** blocker for any actual deploy. Item 1 and item 2 only affected
build; item 18 means even a successful build produces a non-functional image.
This is the first thing any user trying to deploy will hit.

### 11. Two unrelated bugs from the planning session — cross-referenced
Already logged in STATE.md "Roadmap Evolution" but worth carrying here:
- `gsd-tools phase add` assigned `115` four times in a row in one batch —
  doesn't see its own previous insertions when computing the next integer.
- The CLI also collided with an unrelated active milestone's phase numbers
  (JSON-UI v2 was already at 115-121); needed manual renumber to 122-125.

These belong to gsd-tools, not ferro, but they bit me in the same workflow.

---

## DX improvements that would have changed the experience

### 12. `ferro deploy:check` as a real command, not just a planned MCP tool
Phase 123 already plans `deploy_check` as an MCP tool. Promote it to a CLI
command too, runnable as the first step *before* any Docker work:
- `cargo metadata` against `Cargo.docker.toml` resolves cleanly (item 4)
- `.env.production` exists and contains every key in `.env.example`
- `copy_dirs` entries exist and are not dockerignored (item 3)
- Local ferro source compiles against the rewritten `Cargo.docker.toml`
  (catches API skew)
- Git tree clean and pushed
- Dockerfile + .do/app.yaml present and regenerated since last source change

Today I discovered each of these one at a time, with a 1–10 minute Docker
round-trip per discovery.

### 13. Better feedback when ferro is being pulled from crates.io vs path
Right now nothing tells you "your local working tree has commits that are not
in the version this Dockerfile will use." A `docker:init` warning saying
"`ferro_version = "0.2.0"` is N commits behind local path dep `framework/`"
would have caught the missing fields immediately.

### 14. `ferro_version` is one global field in metadata, but ferro is many crates
If `ferro-json-ui` lands a feature in 0.2.1 and `ferro-whatsapp` is still at
0.2.0, you can't express that. Probably fine for now (lockstep release), but
worth a note in PUBLISHING.md and a future per-crate override if crates ever
desync.

### 15. `[package.metadata.ferro.deploy]` is a mouthful and easy to typo
A `ferro deploy:init` interactive command that writes this block correctly
(asking: "Which dirs should be bundled? Which apt packages? What's your ferro
version?") would be friendlier than reading the docs.

### 16. Generated `.do/app.yaml` `envs:` section is comments-only
This means after `do:init` you still have manual work via the dashboard.
Generating real `envs:` entries with `value: ""` and `type: SECRET` for
secret-shaped keys would let `doctl apps update` work end-to-end without UI
clicks. (This was actually in the Phase 122/123 SCOPE I wrote — confirming
it's still worth doing.)

### 17. `Cargo.docker.toml` can drift from `Cargo.toml`
`docker:init` re-renders `Cargo.docker.toml` from scratch but doesn't touch
`Cargo.toml`. So if a user adds a new ferro dep with a path, runs
`cargo build` locally (succeeds), then `docker build` (fails because
`Cargo.docker.toml` is stale), there's no warning.

Either regenerate `Cargo.docker.toml` automatically as part of `docker build`
(via a Docker stage that runs ferro CLI), or have a `ferro docker:check` that
compares timestamps and exits non-zero.

---

## What worked well

- Multi-bin handling (`gestiscilo` + `screenshot-worker`) Just Worked once
  the rewriter was fixed.
- `runtime_apt = ["chromium", "fonts-liberation"]` is a clean knob — simple,
  declarative, solved the chrome dependency in one line.
- `themes/` auto-detection from on-disk presence is exactly the right
  behavior.
- The `Cargo.docker.toml` indirection (rather than mutating `Cargo.toml`) is
  a great pattern — local dev untouched.
- Tests in `templates/docker.rs` and `deploy/rewrite_ferro_version.rs` were
  easy to extend with a regression case. Good test discoverability and
  isolation.
- The CLI output is concise and cargo-style (not chatty, not silent).

---

## Suggested clusters (for the analyzing agent — non-binding)

Most items naturally cluster:

- **Deploy preflight**: items 3, 4, 12, 13, 17 — single command, catches
  issues that today only surface during `docker build`.
- **Generated artifact polish**: items 5, 6, 7, 9, 16 — small quality-of-life
  fixes to the templates and command output.
- **Publish workflow refinement**: items 8 and 14 — version bump gating and
  per-crate overrides if needed.
- **Interactive scaffolder**: item 15 — could fold into preflight or stand
  alone.

These are suggestions, not phase boundaries. The analyzing agent should
weigh:
- Which items block real users today (gestiscilo, mkmenu) vs which are
  speculative.
- Whether any of these belong to existing phases 123/124/125 instead of new
  ones.
- Whether item 11 (gsd-tools collision bugs) deserves an issue against
  gsd-tools rather than a ferro phase.
- Sequencing constraints (e.g. preflight depends on `runtime_requirements`
  scanner from Phase 123).

## Provenance

- Session date: 2026-04-08
- App deployed: `/Users/alberto/repositories/gestiscilo-it/app`
- Ferro version at session end: 0.2.1 (published mid-session)
- Fix commit: `70ad9ed4`
- Reporter: Claude (interactive Code session with Alberto)
