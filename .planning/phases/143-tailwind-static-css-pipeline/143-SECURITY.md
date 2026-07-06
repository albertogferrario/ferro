---
phase: 143
slug: tailwind-static-css-pipeline
status: verified
threats_open: 0
asvs_level: 1
created: 2026-04-21
---

# Phase 143 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| contributor → repo | Contributors run scripts/gen-ferro-base-css.sh locally; output committed into the repo. | Build artifact (CSS) — no secrets |
| CI → build artifacts | CI downloads Tailwind CLI from GitHub Releases and regenerates CSS in ephemeral /tmp for comparison. | Binary download over HTTPS |
| client → GET /_ferro/ferro-base.css | Public HTTP endpoint; no authentication. Served to every visitor. | Static CSS bytes — design tokens only |
| app config → build_response head | `config.stylesheet_urls` values emitted into HTML `href` attributes. Sourced from app developer code (trusted). | URL strings into HTML attributes |
| theme.css → style injection | theme.css content injected verbatim into a `<style>` tag. Framework-embedded or app-provided tokens.css (trusted). | CSS text — design tokens only |
| developer → `ferro make:theme` | Scaffolder writes files into `themes/<name>/` in developer's working directory. | Template string → local filesystem |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-143-01 | Tampering | ferro-base.css committed file | mitigate | CI drift job (`ferro-base-css-drift`) regenerates from input.css and diffs; any hand-edit fails CI. Verified: `.github/workflows/ci.yml:93` | closed |
| T-143-02 | Tampering | Tailwind CLI binary downloaded in CI | accept | Binary fetched from GitHub Releases via HTTPS (TLS integrity). ASVS L1 does not require SRI/checksum for build tools. | closed |
| T-143-03 | Information Disclosure | input.css `@source` globs | accept | Scans `ferro-json-ui/src` and `framework/src` only — no env vars, secrets, or generated artifacts reachable. Tailwind extracts plain-text tokens only. | closed |
| T-143-04 | Denial of Service | scripts/gen-ferro-base-css.sh | accept | Script runs locally/CI only; not invoked from network-facing paths. Resource bounded by Tailwind CLI. | closed |
| T-143-05 | Tampering | ferro-theme/assets/default.css | accept | Committed to framework repo and embedded at compile time. No runtime user-controlled path reaches this asset. Same surface as any compiled Rust source. | closed |
| T-143-06 | Information Disclosure | CSS content served to clients | accept | CSS variables are design tokens (colors, radii, fonts). No secrets. Served to every visitor by design. | closed |
| T-143-07 | Tampering | `/_ferro/ferro-base.css` route | mitigate | Exact string comparison (`path.as_str() == "/_ferro/ferro-base.css"`) — no path parsing, no segment matching. Path traversal structurally impossible. Verified: `framework/src/server.rs:223` | closed |
| T-143-08 | Information Disclosure | CSS body served to every client | accept | Framework design tokens compiled from ferro-json-ui sources — no secrets. Intended to be public. | closed |
| T-143-09 | Tampering (XSS) | `stylesheet_urls` emitted into `href` attributes | mitigate | Values passed through `html_escape()` before href interpolation. Test `stylesheet_urls_are_html_escaped_in_href_attribute` locks the contract. ASVS L1 V5.3 satisfied. Verified: `framework/src/json_ui/mod.rs:108`, test at `:474` | closed |
| T-143-10 | Tampering (XSS) | `theme.css` injected into `<style>` | accept | Source is framework-embedded `default.css` or `Theme::from_path()` reading from app filesystem (trusted operator input). Not user-reachable. Same trust model as `custom_head`. | closed |
| T-143-11 | Denial of Service | `/_ferro/ferro-base.css` response size | accept | Body is `&'static [u8]` embedded at compile time. `Bytes::from_static` is zero-copy — no per-request allocation. Cache-Control reduces repeat traffic. | closed |
| T-143-12 | Repudiation | Route access logging | accept | No per-endpoint audit trail required for static assets under ASVS L1 V7. Framework-level request logs capture path like any other endpoint. | closed |
| T-143-13 | Tampering | `themes/<name>/` directory creation | mitigate | Existing guard `if theme_dir.exists() { return Err(...) }` prevents overwriting an existing theme. Verified: `ferro-cli/src/commands/make_theme.rs:25-26` | closed |
| T-143-14 | Tampering | Scaffolded tokens.css content | accept | Content is a compile-time string constant in the CLI binary. No user input reaches the template. No injection vector. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-143-01 | T-143-02 | Tailwind CLI binary TLS-only verification is ASVS L1 compliant for build tools; SRI/checksum pinning not required at this level. | Alberto Ferrario | 2026-04-21 |
| AR-143-03 | T-143-03 | Source glob scanning of Rust src directories cannot expose secrets; Tailwind extracts only plain-text utility class tokens. | Alberto Ferrario | 2026-04-21 |
| AR-143-04 | T-143-04 | Shell script is local/CI-only; no network-facing execution path. Resource use is bounded by the Tailwind CLI process. | Alberto Ferrario | 2026-04-21 |
| AR-143-05 | T-143-05 | Compile-time embedded asset has identical trust surface to any Rust source file. No runtime user-controlled path exists. | Alberto Ferrario | 2026-04-21 |
| AR-143-06 | T-143-06 | CSS design tokens are non-sensitive by definition; public disclosure is intended behavior. | Alberto Ferrario | 2026-04-21 |
| AR-143-08 | T-143-08 | Static byte embedding — no secrets, intended public access. | Alberto Ferrario | 2026-04-21 |
| AR-143-10 | T-143-10 | theme.css injection is operator-controlled, not user-controlled. Trust model equivalent to any framework template. | Alberto Ferrario | 2026-04-21 |
| AR-143-11 | T-143-11 | Zero-copy static response with Cache-Control; bounded by compile-time file size. No per-request allocation risk. | Alberto Ferrario | 2026-04-21 |
| AR-143-12 | T-143-12 | Static asset routes do not require per-endpoint audit trails under ASVS L1 V7. | Alberto Ferrario | 2026-04-21 |
| AR-143-14 | T-143-14 | Compile-time constant template; zero user-input surface. No injection vector possible. | Alberto Ferrario | 2026-04-21 |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-04-21 | 14 | 14 | 0 | gsd-security-auditor (automated) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-04-21
