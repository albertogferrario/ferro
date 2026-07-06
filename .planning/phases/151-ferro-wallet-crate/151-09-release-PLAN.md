---
phase: 151
plan: 151-09
slug: release
wave: 5
depends_on: [151-05, 151-06, 151-07, 151-08]
files_modified:
  - Cargo.toml
  - CHANGELOG.md
autonomous: false
requirements: [ACC-2, ACC-3, ACC-4]
must_haves:
  truths:
    - "Workspace version bumps to `0.2.24` (patch) per Risk 1 and D-10"
    - "CHANGELOG.md gains a Phase 151 entry describing the new `ferro-wallet` crate"
    - "Full workspace test suite is green (`cargo test --all-features`)"
    - "`cargo doc --no-deps -p ferro-wallet` produces clean output (no warnings)"
    - "First publish of `ferro-wallet` is bootstrapped from a local terminal with a personal publish token (Risk 1)"
  artifacts:
    - path: "Cargo.toml"
      provides: "Workspace version bumped"
      contains: "version = \"0.2.24\""
    - path: "CHANGELOG.md"
      provides: "Phase 151 entry under 0.2.24"
      contains: "ferro-wallet"
  key_links:
    - from: "Cargo.toml [workspace.package].version"
      to: "GH Actions publish.yml check-version job"
      via: "post-merge auto-publish for subsequent versions"
      pattern: "version = \"0.2.24\""
    - from: "manual cargo publish -p ferro-wallet"
      to: "crates.io ferro-wallet first version"
      via: "personal token bootstrap (Risk 1)"
      pattern: "cargo publish -p ferro-wallet"
---

<objective>
Release Phase 151. Bump workspace version to `0.2.24`, add CHANGELOG entry, run the full test + doc gate, then perform the first-publish bootstrap of `ferro-wallet` from a local terminal (CI publish token has `publish-update` only — Risk 1).
</objective>

<context>
@.planning/phases/151-ferro-wallet-crate/151-CONTEXT.md
@.planning/phases/151-ferro-wallet-crate/151-RESEARCH.md
@.planning/phases/151-ferro-wallet-crate/151-VALIDATION.md
@CLAUDE.md
@Cargo.toml
@CHANGELOG.md
@.github/workflows/publish.yml
</context>

<must_haves>
- `Cargo.toml` `[workspace.package] version = "0.2.24"`.
- `CHANGELOG.md` opens with a new top-level entry for `0.2.24` referencing Phase 151 and the new `ferro-wallet` crate.
- `cargo test --all-features` exits 0 (workspace-wide).
- `cargo doc --no-deps -p ferro-wallet` exits 0 with no warnings.
- `cargo build --workspace` exits 0.
- First-publish bootstrap of `ferro-wallet` succeeds and the version appears on crates.io.
- All four ACC-* IDs (ACC-2, ACC-3, ACC-4 explicitly; ACC-1a..ACC-1k implicitly via the workspace suite) are green.
</must_haves>

<tasks>

<task type="auto">
  <name>Task 1: Run the full pre-release gate</name>
  <files>(no edits; verification only)</files>
  <read_first>
    - CLAUDE.md §"Testing & Linting" (exact commands)
    - 151-VALIDATION.md §"Validation Sign-Off"
    - 151-RESEARCH.md §"Validation Architecture" → Nyquist Dimensions
  </read_first>
  <action>
    Execute the workspace-wide gate. Each command must exit 0 with no warnings. If any command fails, abort the plan and surface the failure to the user before proceeding.

    ```bash
    cargo fmt --all -- --check
    cargo clippy --all --all-targets -- -D warnings
    cargo build --workspace
    cargo test --all-features
    cargo doc --no-deps -p ferro-wallet
    ```
  </action>
  <verify>
    <automated>cargo fmt --all -- --check &amp;&amp; cargo clippy --all --all-targets -- -D warnings &amp;&amp; cargo build --workspace &amp;&amp; cargo test --all-features &amp;&amp; cargo doc --no-deps -p ferro-wallet</automated>
  </verify>
  <done>All five commands exit 0. Output of `cargo test --all-features` shows the `ferro-wallet` unit tests (≥ 17 — error 8 + subject 5 + config 7 + manifest 3 + package 2 + qr 3 + images 6 + google::jwt 3) plus both integration tests (`apple_integration`, `google_jwt`). ACC-2 + ACC-3 confirmed green.</done>
</task>

<task type="auto">
  <name>Task 2: Bump workspace version + add CHANGELOG entry</name>
  <files>Cargo.toml, CHANGELOG.md</files>
  <read_first>
    - Cargo.toml lines 26–27 (current `[workspace.package] version = "0.2.23"`)
    - CHANGELOG.md (top of file — observe existing entry format and style)
    - 151-CONTEXT.md D-10 (release flow)
    - 151-RESEARCH.md §"Risks & Open Questions" item 2 (version bump timing — CI auto-bumps if tag already exists)
  </read_first>
  <action>
    Edit `/Users/alberto/repositories/albertogferrario/ferro/Cargo.toml`. Change:

    ```toml
    [workspace.package]
    version = "0.2.23"
    ```

    to:

    ```toml
    [workspace.package]
    version = "0.2.24"
    ```

    Edit `/Users/alberto/repositories/albertogferrario/ferro/CHANGELOG.md`. Read the top of the file first to match the existing heading + bullet style. Prepend a new entry for `0.2.24` containing:

    - One-line summary: "Phase 151 — `ferro-wallet` crate (Apple `.pkpass` + Google Wallet save-link issuance)."
    - Bullet points (mirror the bullet style of the most recent prior entry):
      - "New crate `ferro-wallet` exposing the `WalletSubject` trait, `ApplePassBuilder` (PKCS#7-signed `.pkpass`), and `GoogleWalletBuilder` (RS256 save-link JWT)."
      - "`WalletConfig::from_env` reads `APP_NAME` / `APP_URL` and optional Apple / Google clusters; missing wallet env vars never error (D-02)."
      - "Image normalisation and QR generation helpers bundled."
      - "End-to-end integration tests mint crypto material at runtime — no real Apple/Google credentials in CI (D-09)."
      - "Workspace member registered; auto-publish Wave 1a slot reserved."

    Tone: neutral, scientific, no marketing language (per CLAUDE.md "scientific and minimalistic, no marketing").

    Note (per RESEARCH.md Risk 2): If the GH Actions `check-version` job auto-bumped `0.2.23 → 0.2.24` in a prior phase commit, this manual bump is redundant but harmless. If `0.2.23` was already tagged, CI may auto-bump again to `0.2.25` when this lands — that is acceptable; both versions will exist on crates.io.
  </action>
  <verify>
    <automated>grep -F 'version = "0.2.24"' Cargo.toml &amp;&amp; head -1 CHANGELOG.md &amp;&amp; grep -F 'ferro-wallet' CHANGELOG.md &amp;&amp; cargo build --workspace &amp;&amp; cargo test --all-features</automated>
  </verify>
  <done>`Cargo.toml` shows `version = "0.2.24"`. `CHANGELOG.md` opens with the new entry. Workspace still builds + tests pass.</done>
</task>

<task type="checkpoint:human-action" gate="blocking">
  <name>Task 3: First-publish bootstrap from local terminal (Risk 1)</name>
  <files>(no file edits — manual `cargo publish` invocation)</files>
  <what-built>
    Workspace version bumped, CHANGELOG entry added, full gate green. The crate is now ready for its FIRST publish to crates.io.
  </what-built>
  <how-to-verify>
    Per user memory `project_ferro_publish_token_scoping.md`: the CI publish token has `publish-update` only, NOT `publish-new`. New crates need a one-time bootstrap from a local terminal with a personal token. Subsequent versions will auto-publish via GH Actions.

    Required manual steps (run from a local terminal at the repo root, NOT inside Claude Code):

    1. Confirm `0.2.24` is the workspace version: `grep 'version' Cargo.toml | head -1`.
    2. Run the first publish with a personal publish token that has `publish-new` scope:
       ```bash
       cargo publish -p ferro-wallet --token <PERSONAL_PUBLISH_TOKEN>
       ```
    3. Verify the version landed on crates.io: open `https://crates.io/crates/ferro-wallet` in a browser, confirm `0.2.24` appears in the version list (or whatever patch the CI auto-bump produced if it raced ahead — `0.2.25` is also acceptable per Risk 2).
    4. After successful first-publish, push the commit (workspace version bump + CHANGELOG entry) to `master`. CI will see the crate already published at this version and skip with the "already published" path (publish.yml lines 207–213).
    5. Confirm GH Actions `publish.yml` run is green on master after the push.

    If the personal token is not available, abort: do NOT push the version bump until the bootstrap publish succeeds, otherwise CI will fail with "publish-new scope required" and the workspace will sit in a half-published state.
  </how-to-verify>
  <resume-signal>Type "published" once `https://crates.io/crates/ferro-wallet` shows the new version. Provide the version string (`0.2.24` or whatever CI auto-bumped to) for the SUMMARY.</resume-signal>
</task>

</tasks>

<threat_model>
This plan touches release infrastructure but introduces no new application code. The first-publish bootstrap is the only security-sensitive moment.

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-151-DEFAULT-CRED | I | `cargo publish` token | mitigate | Bootstrap uses a personal `publish-new`-scoped token, run from the user's local terminal — never persisted in the repo, never piped through CI. Subsequent versions auto-publish via the CI token which has only `publish-update` scope (cannot create new crates). Token rotation is a separate maintenance task. |

No STRIDE entries for the Cargo.toml + CHANGELOG.md edits themselves — pure configuration changes.
</threat_model>

<verification>
- `grep -F 'version = "0.2.24"' Cargo.toml` returns one match (or `0.2.25` if CI raced — Risk 2).
- `head -20 CHANGELOG.md | grep -F 'ferro-wallet'` returns at least one match.
- `cargo build --workspace` exits 0.
- `cargo test --all-features` exits 0.
- `cargo doc --no-deps -p ferro-wallet` exits 0 with no warnings.
- `https://crates.io/crates/ferro-wallet` shows the published version (verified manually post-bootstrap).
- ACC-2, ACC-3, ACC-4 all green.
</verification>

<success_criteria>
Phase 151 ships. `ferro-wallet` is published on crates.io. Downstream consumers (specifically gestiscilo-it) can add `ferro-wallet = "0.2.X"` to their `Cargo.toml` and begin the wallet-passes integration. All ACC-* requirements from VALIDATION.md are verified green.
</success_criteria>

<output>
After completion, create `.planning/phases/151-ferro-wallet-crate/151-09-SUMMARY.md` documenting:
- Final published version string (e.g., `0.2.24` or `0.2.25` per Risk 2 outcome).
- The crates.io URL.
- Confirmation that GH Actions `publish.yml` ran green after the push.
- A one-line note on whether the manual bootstrap was required or whether the workflow had already auto-bumped to a tagged-but-unpublished version.

Also update `.planning/STATE.md`:
- Increment `completed_phases`.
- Set `Workspace version` to the published value.
- Bump `Current focus` to the next phase per ROADMAP.
- Optional: patch the milestone field from `v11.0` to `v11.10` per RESEARCH.md Open Question 4 (out-of-scope cleanup but minor).
</output>

## PLANNING COMPLETE
