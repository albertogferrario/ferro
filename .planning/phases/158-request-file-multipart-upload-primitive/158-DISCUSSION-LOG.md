# Phase 158: Request::file() multipart upload primitive - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-15
**Phase:** 158-request-file-multipart-upload-primitive
**Mode:** --auto (all decisions auto-selected)
**Areas discussed:** Parser Library, API Surface, UploadedFile Shape, Storage Integration, Size Limits, Validation, Module Location

---

## Parser Library

| Option | Description | Selected |
|--------|-------------|----------|
| `multer` | Async multipart crate compatible with hyper 1.x body streams | ✓ |
| `actix-multipart` | Not compatible — actix ecosystem only | |
| `multipart` crate | Older, sync-only | |

**Auto-selected:** `multer`
**Notes:** Only viable async choice for raw hyper 1.x. `mime_guess` already present for MIME assistance.

---

## API Surface

| Option | Description | Selected |
|--------|-------------|----------|
| `req.multipart() -> MultipartForm` | Mirrors existing req.form()/req.json() pattern | ✓ |
| `req.file("x")` as convenience | Wraps multipart(), single-file shorthand | ✓ |
| Raw `multer::Multipart` stream | Too low-level for framework handlers | |

**Auto-selected:** Both `req.multipart()` (primary) and `req.file()` (convenience wrapper)
**Notes:** MultipartForm bundles both file fields and text fields — single parse pass covers mixed forms.

---

## UploadedFile Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal: field_name + bytes | Too sparse — loses filename and MIME | |
| Full: field_name, file_name, content_type, bytes | Standard upload metadata | ✓ |
| Include `store()` method | Storage integration on the type itself | ✓ |

**Auto-selected:** Full struct with `store()` — this is the killer feature.

---

## Storage Integration

| Option | Description | Selected |
|--------|-------------|----------|
| `UploadedFile::store(storage, path)` | Direct bridge to ferro-storage | ✓ |
| Defer — caller manages storage | Requires more boilerplate | |

**Auto-selected:** Include `store()` in scope — it's the DX payoff of the whole primitive.

---

## Size Limits

| Option | Description | Selected |
|--------|-------------|----------|
| 10 MB default, env-configurable | Reasonable default, production-safe | ✓ |
| No limit | DoS risk | |
| Hardcoded limit | Inflexible | |

**Auto-selected:** 10 MB default, `UPLOAD_MAX_SIZE_MB` env override, typed error on exceed.

---

## Validation Helpers

| Option | Description | Selected |
|--------|-------------|----------|
| Standalone `validate_mime()` / `validate_size()` | Simple, no Validator coupling | ✓ |
| Full Validator builder integration | Complex, future phase | |

**Auto-selected:** Standalone helpers only.

---

## Claude's Discretion

- Whether `store()` takes `&Storage` or `&dyn DiskDriver`
- Internal representation of `MultipartForm::files_map`
- Test structure (in-memory body construction)
