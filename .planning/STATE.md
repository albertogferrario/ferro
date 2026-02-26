# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-13)

**Core value:** Agents can go from "I want an app that does X" to a working, deployed application with minimal friction.
**Current focus:** Security Hardening — Phase 74 complete

## Current Position

Phase: 74 (Session Absolute Expiry)
Plan: 2 of 2
Status: Phase 74 complete — Auth facade, session invalidation API, and documentation shipped
Last activity: 2026-02-26 — Phase 74 plan 02 executed

## Milestone Summary

| Milestone | Phases | Plans | Status | Shipped |
|-----------|--------|-------|--------|---------|
| v1.0 DX Overhaul | 1-12 | 18 | Complete | 2026-01-16 |
| v2.0 Rebrand | 13-22 | 13 | Complete | 2026-01-16 |
| v2.0.1 Macro Fix | 22.1-22.3 | 6 | Complete | 2026-01-17 |
| v2.0.2 Type Generator Fixes | 22.4-22.9 | 6 | Complete | 2026-01-17 |
| v2.0.3 DO Apps Deploy | 22.10 | 1 | Complete | 2026-01-17 |
| v2.1 Inertia DX & Fixes | 33-34 | 4 | Complete | 2026-01-17 |
| v2.2 CLI Improvements | 35-37 | 5 | Complete | 2026-02-09 |
| v3.0 JSON-UI | 23-32 | 24 | Complete | 2026-02-09 |
| v4.0 Production Readiness | 38-46 | 24 | Complete | 2026-02-10 |
| v5.0 Proximity — JSON-UI Field Test | 47-53 | 20 | Complete | 2026-02-10 |
| v5.1 Housekeeping | 54-57 | 5 | Complete | 2026-02-13 |
| v6.0 ferro-lang — Localization | 58-66 | 11 | Complete | 2026-02-13 |
| v6.1 Fix Known Issues | 67 | 1 | Complete | 2026-02-24 |
| v7.0 Resend Integration | 68 | 3 | Complete | 2026-02-25 |
| v7.1 Static File Serving | 69 | 1 | Complete | 2026-02-25 |
| Security Hardening | 72-74 | 5 | Complete | 2026-02-26 |

## Accumulated Context

### Key Decisions

Archived to PROJECT.md and milestone archive files.

- Static file responses use hyper::Response<Full<Bytes>> directly (not HttpResponse) to preserve binary file integrity
- Static files checked before fallback handler in handle_request() to prevent SPA catch-all from intercepting asset requests
- HttpResponse body changed from String to Bytes; body() returns &str with from_utf8 fallback for backward compatibility
- bytes() constructor sets no default Content-Type; download() auto-detects from filename extension
- SecurityHeaders HSTS off by default to avoid breaking localhost over HTTP
- X-XSS-Protection set to 0 per OWASP (XSS Auditor can create vulnerabilities)
- CSP includes unsafe-inline/unsafe-eval for Inertia.js and Vite SPA compatibility
- SecurityHeaders placed after CSRF middleware in bootstrap so headers apply to all responses including CSRF rejections
- Nullable created_at for backward compat with existing sessions tables (NULL skips absolute check)
- destroy_for_user default trait impl returns error rather than panic
- Cookie max_age uses max(idle, absolute) so cookie outlives both server-side checks
- DatabaseSessionDriver with zero-duration lifetimes in logout_other_devices (destroy_for_user never reads them)
- DatabaseSessionDriver and SessionStore re-exported from framework root for admin flows

### Roadmap Evolution

- All planned milestones v1.0–v7.1 complete (15 milestones, 150 plans shipped)
- Security hardening phases 72-74 added to roadmap
- Phase 72 plan 01 complete: binary-safe HttpResponse with bytes()/download() constructors
- Phase 73 plan 01 complete: SecurityHeaders middleware with OWASP defaults and builder API
- Phase 73 plan 02 complete: SecurityHeaders in CLI bootstrap template and middleware documentation
- Phase 74 plan 01 complete: dual timeout session enforcement with created_at tracking and destroy_for_user
- Phase 74 plan 02 complete: Auth::logout_other_devices(), invalidate_all_for_user(), and dual-timeout documentation

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-26
Stopped at: Phase 74 complete — all session absolute expiry features shipped (dual timeout, invalidation API, docs)
Resume file: None
