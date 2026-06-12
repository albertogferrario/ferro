# Phase 206 — Verification

**Phase:** 206-ferro-storage-provider-agnostic-storage-env-vars
**Plan:** 206-01-PLAN.md
**Verified:** 2026-06-12 (static + runtime, both green)
**Method:** static (grep + file inspection) + runtime (`cargo check -p ferro-storage --features s3`, `cargo test -p ferro-storage --features s3 --lib config::tests`).

---

## Success Criteria

| SC | Description | Status | Evidence |
|---|---|---|---|
| SC-1 | `StorageConfig::from_env` reads STORAGE_* primary via `env_with_fallback` | PASS | `config.rs` lines 92-93, 110-114 — `env_with_fallback("STORAGE_BUCKET", &["AWS_BUCKET"])` etc. |
| SC-2 | `S3Driver::new` reads STORAGE_ACCESS_KEY_ID / STORAGE_SECRET_KEY primary | PASS | `drivers/s3.rs` lines 37-40 — `env_with_fallback("STORAGE_ACCESS_KEY_ID", &["AWS_ACCESS_KEY_ID"])` + secret-key equivalent |
| SC-3 | Facade reads STORAGE_ENDPOINT primary | PASS | `facade.rs:228` — `env_with_fallback("STORAGE_ENDPOINT", &["AWS_URL"])` |
| SC-4 | `env_with_fallback` hoisted to crate-level `env_helpers` | PASS | `src/env_helpers.rs` exists; `cdn::mod` uses `crate::env_helpers::env_with_fallback` import; no private copy remains |
| SC-5 | Legacy AWS_* parity tests pass byte-identical | PASS | `cargo test config::tests` → `from_env_cdn_url` ok, `cdn_url_parity_aws_fallback` ok |
| SC-6 | New primary-path test `from_env_storage_primary` PASS | PASS | `cargo test config::tests::from_env_storage_primary` → ok |
| SC-7 | `app/.env.example` declares STORAGE_* primary with deprecation note | PASS | Section "Object Storage Settings" lines 68-87 — 6 STORAGE_* vars + AWS_* deprecation alias note |
| SC-8 | Workspace 0.2.54; CHANGELOG `## [0.2.54]` entry | PASS | `Cargo.toml:38` → `version = "0.2.54"`; `ferro-storage/CHANGELOG.md` `## [0.2.54] - 2026-06-12` with rename + deprecation table |
| SC-9 | `cargo check -p ferro-storage --features s3` exits 0 | PASS | `Finished dev profile [unoptimized] target(s) in 30.18s` |
| SC-10 | `cargo test config::tests` → 7/7 PASS | PASS | `test result: ok. 7 passed; 0 failed; 0 ignored` |

---

## Static gate captures

### env_with_fallback hoist
```
$ grep -n 'env_with_fallback' ferro-storage/src/env_helpers.rs ferro-storage/src/cdn/mod.rs ferro-storage/src/config.rs ferro-storage/src/drivers/s3.rs ferro-storage/src/facade.rs
ferro-storage/src/env_helpers.rs:10:pub(crate) fn env_with_fallback(primary: &str, aliases: &[&str]) -> Option<String> {
ferro-storage/src/cdn/mod.rs:25:use crate::env_helpers::env_with_fallback;
ferro-storage/src/config.rs:9:use crate::env_helpers::env_with_fallback;
ferro-storage/src/drivers/s3.rs:5:use crate::env_helpers::env_with_fallback;
ferro-storage/src/facade.rs:5:use crate::env_helpers::env_with_fallback;
```

### Test results
```
running 7 tests
test config::tests::test_storage_config_builder ... ok
test config::tests::test_storage_config_defaults ... ok
test config::tests::test_storage_config_from_env ... ok
test config::tests::purge_parity_legacy_do ... ok
test config::tests::from_env_cdn_url ... ok
test config::tests::from_env_storage_primary ... ok
test config::tests::cdn_url_parity_aws_fallback ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 50 filtered out
```

---

## Operator follow-ups

1. **Push 5 phase commits** on `ferro` master (no auto-push):
   - `cb700f34 refactor(206): hoist env_with_fallback to crate-level env_helpers`
   - `8e658626 feat(206): STORAGE_BUCKET/REGION/ENDPOINT/PUBLIC_URL primary; AWS_* legacy fallback`
   - `70ea8fec feat(206): STORAGE_ACCESS_KEY_ID/SECRET_KEY/ENDPOINT in drivers + facade`
   - `9c961168 chore(206): tests + .env.example + CHANGELOG + workspace 0.2.53 → 0.2.54`
   - (plus this closeout commit)
2. Pushing master triggers the GH Actions publish wave for 0.2.54. Bundles with the unpushed 0.2.53 (Phase 204 CDN quartet) — both versions ship in one push.
3. Open consumer-side phase in `gestiscilo-it/app` to adopt the rename: bump ferro 0.2.53 → 0.2.54, rename `.env.example` + `app-env/production/.env` to STORAGE_* (preserve values verbatim), update ROADMAP Pending Operator Tasks. Mirrors Phase 205's shape.

---

## Deprecation cushion

`AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` / `AWS_DEFAULT_REGION` / `AWS_BUCKET` / `AWS_URL` / `AWS_PUBLIC_URL` continue to work for one release with a `tracing::warn!` on first read. Removal slated for the release following 0.2.54.
