---
phase: 202
slug: login-resume-contract-magic-link-sample-app
status: secured
threats_open: 0
threats_closed: 12
asvs_level: 1
created: 2026-06-11
---

# SECURITY.md — Phase 202 Audit

**Phase:** 202 — Login-Resume Contract / Magic-Link Sample App
**Audit date:** 2026-06-11
**ASVS Level:** 1
**Auditor:** gsd-security-auditor (claude-sonnet-4-6)
**block_on:** high

---

## Result: SECURED

**Threats Closed:** 12/12
**Threats Open:** 0/12

---

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-199-04 | Tampering / open redirect | mitigate | CLOSED | `resume.rs:25-29,72-77` — doc comment and `oauth_resume_redirect` implementation confirm: stored value written only by `authorize.rs` from a self-constructed internal URL; default is a static caller-supplied path; no user input ever enters the redirect target |
| T-202-KEY | Tampering / session key ownership | mitigate | CLOSED | `resume.rs:38` — `const OAUTH_RETURN_TO_KEY: &str = "oauth_return_to"` is the single owner. Grep over `ferro-mcp-oauth/src/` and `app/src/` finds the literal only at `resume.rs:37-38` (doc comment + const definition). No inline `"oauth_return_to"` string in `authorize.rs`, `consent.rs`, or `app/src/` |
| T-202-01 | Elevation / replay | mitigate | CLOSED | `auth_controller.rs:194-195` — `Cache::get` then `Cache::forget` unconditionally before `Auth::login` (line 208). Order: get → forget → match → login. `magic_link_single_use` test (magic_link.rs:20-46) and `oauth_magic_link_resume_flow` SC-3 test (oauth_magic_link_resume_flow.rs:65-79) both confirm second get returns None |
| T-202-02 | DoS / stale credential | mitigate | CLOSED | `auth_controller.rs:141` — `Cache::put(&key, &user_id, Some(Duration::from_secs(15 * 60)))`. Absent/expired key maps to error re-render (lines 199-205). `magic_link_expired` test (magic_link.rs:56-78) proves absent key → None → error path |
| T-202-03 | Spoofing / token guessing | mitigate | CLOSED | `auth_controller.rs:242-243` — `rand::thread_rng().gen::<[u8; 32]>()` (256 bits from ChaCha12 CSPRNG) encoded with `URL_SAFE_NO_PAD`. REVIEW.md confirms `rand::thread_rng` is seeded from `OsRng` and is a CSPRNG |
| T-202-04 | Info disclosure / email enumeration | accept | CLOSED | Acceptance documented. `auth_controller.rs:115-116` code comment explicitly flags as "T-202-04 accepted flag — reveals account existence; acceptable for the sample exemplar". Behaviour: "No account found for this email." returned on unknown email (line 129) |
| T-202-MAIL | DoS / CI dependency | accept | CLOSED | Acceptance guard present. `auth_controller.rs:147` — entire mail path guarded by `env.is_development()` check; else branch at line 161 calls `send_magic_link_mail_best_effort`. That function (line 304) wraps dispatch in `if let Err(e)` with `tracing::warn!` — never a hard failure (`?` not used). CI runs `APP_ENV=local` which is development, so the mail branch is never reached |
| T-202-DEVLEAK | Info disclosure / dev link | mitigate | CLOSED | `login_confirm.json:32-35` — `dev_link` element has `"visible": {"path": "/dev_mode", "operator": "is_true"}`. Non-dev path in `auth_controller.rs:166-170` passes `"dev_mode": false, "dev_link": "", "dev_link_label": ""` — element is hidden and link is empty string, not a real URL |
| T-202-XSS | Tampering / injection | mitigate | CLOSED | `login.json:33-34` — error binding via `{ "$data": "/error" }`. `login_confirm.json:25,29` — label and handler via `{ "$data": "/dev_link_label" }` and `{ "$data": "/dev_link" }`. All dynamic content uses ferro-json-ui `$data` bindings (renderer-escaped); no raw HTML interpolation in either view file |
| T-202-CI | DoS / non-determinism | mitigate | CLOSED | Both test files use only `bootstrap_test_cache()` (in-memory) and `with_test_session()`. Grep over `magic_link.rs` and `oauth_magic_link_resume_flow.rs` finds no `reqwest`, `TcpListener`, `render_file`, `SmtpTransport`, or `lettre` — fully offline |
| T-202-GATE | Repudiation / quality | mitigate | CLOSED | `202-GATE.md` records all four gates green: `cargo fmt --all -- --check` PASS, `cargo clippy --all --all-targets --all-features -- -D warnings` PASS (zero warnings), `cargo test --all-features` PASS (16+55+1 tests), `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` PASS |
| T-202-BOOT | DoS / availability | mitigate | CLOSED | `bootstrap.rs:75` — `ThemeMiddleware::new().default_theme(Theme::default_theme())` uses embedded theme (no filesystem read). `grep -n "from_path(" app/src/bootstrap.rs` returns no matches. GATE.md SC-5 section records live boot from repo root succeeding with "Ferro server running" output |

---

## Accepted Risks Log

### T-202-04 — Email Enumeration

**Category:** Information disclosure  
**Component:** `POST /auth/login` (request-link handler)  
**Description:** The handler returns "No account found for this email." when the submitted address is not registered. This reveals whether an email address has a registered account to any unauthenticated caller.  
**Rationale:** Accepted for the sample exemplar context. The magic-link flow is demonstrating the OAuth resume contract, not production hardening. A production app should return a uniform "If an account exists, a link has been sent." response regardless of lookup result.  
**Code reference:** `app/src/controllers/auth_controller.rs:115-133`  
**Review note (IN-01):** The `docs/src/features/authentication.md` does not currently document this accepted risk. Adding a production-caveat note to the docs would prevent adopters from unintentionally shipping enumerable registration state.

### T-202-MAIL — Non-Dev Mail Dispatch

**Category:** DoS / CI non-determinism  
**Component:** `send_magic_link_mail_best_effort` in `auth_controller.rs`  
**Description:** SMTP dispatch occurs only when `APP_ENV != local`. Dispatch errors are logged as `tracing::warn!` and never propagate as hard failures. CI always runs with `APP_ENV=local`, so this branch is never exercised by automated tests.  
**Rationale:** Best-effort mail is appropriate for a sample app demonstrating magic-link architecture. SMTP is an external dependency; making it a hard failure would break apps without mail configured.  
**Code reference:** `app/src/controllers/auth_controller.rs:147-170, 250-307`

---

## Unregistered Threat Flags

No unregistered flags from SUMMARY.md were identified that lack a threat register mapping.

---

## Review Findings Cross-Check (from 202-REVIEW.md)

The independent code reviewer identified four warnings (WR-01 through WR-04) and three info items (IN-01 through IN-03). None of these map to mitigate-disposition threats in the Phase 202 threat register and none are security-critical per the reviewer's own assessment. They are noted here for completeness:

| Finding | Severity | Security relevance | Threat mapping |
|---------|----------|--------------------|----------------|
| WR-01: `state` not percent-encoded in redirect Location | Warning | Low — malformed URL, not a redirect to attacker-controlled target; redirect_uri was pre-validated | Not in threat register; out of scope for this audit |
| WR-02: `/auth/verify` under GuestMiddleware blocks authenticated resume | Warning | UX/correctness; not a security bypass | Not in threat register |
| WR-03: `POST /auth/register` does not call `oauth_resume_redirect` | Warning | OAuth flow completeness, not a security vulnerability | Not in threat register |
| WR-04: `scope` dropped from reconstructed `oauth_return_to` URL | Warning | Correctness regression risk in future multi-scope phases | Not in threat register |
| IN-01: Email enumeration not documented in auth.md | Info | Documentation gap for T-202-04 (accepted) | Maps to T-202-04 (accepted) |
| IN-02: `set_var` without cleanup in flow_integration.rs | Info | Test isolation concern, not a production security issue | Not in threat register |
| IN-03: `#[allow(dead_code)]` on `RegisterInput` misleading | Info | Code hygiene only | Not in threat register |

WR-01 through WR-04 are correctness and UX gaps that fall outside the Phase 202 threat register. They do not constitute open mitigate-disposition threats but are recommended for resolution in a follow-up phase.
