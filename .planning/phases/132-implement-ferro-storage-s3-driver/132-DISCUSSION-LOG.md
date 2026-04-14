# Phase 132: Implement ferro-storage S3 Driver - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-14
**Phase:** 132-implement-ferro-storage-s3-driver
**Areas discussed:** S3 client initialization, Presigned URL strategy, Visibility/ACL mapping, Directory operations semantics
**Mode:** --auto (all decisions auto-selected as recommended defaults)

---

## S3 Client Initialization

| Option | Description | Selected |
|--------|-------------|----------|
| Default credential chain at creation | Build Client from DiskConfig in create_driver(), aws-config default chain | ✓ |
| Lazy initialization | Build client on first request | |
| Explicit credentials in DiskConfig | Add access_key/secret fields to DiskConfig | |

**User's choice:** [auto] Default credential chain at creation (recommended default)
**Notes:** S3Driver becomes struct with Client, bucket, url_base fields. Custom endpoint via AWS_URL with force_path_style.

---

## Presigned URL Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| url() = CDN/public, temporary_url() = presigned | url() uses url_base if set, else bucket URL; temporary_url() always presigned | ✓ |
| Both presigned | Both methods generate presigned URLs | |
| url() always bucket URL | Ignore url_base for url() | |

**User's choice:** [auto] url() = CDN/public, temporary_url() = presigned (recommended default)
**Notes:** Matches LocalDriver pattern where url() uses url_base.

---

## Visibility / ACL Mapping

| Option | Description | Selected |
|--------|-------------|----------|
| Best-effort ACL header | Set public-read on Public, omit on Private; providers that ignore ACLs use bucket policy | ✓ |
| Strict ACL enforcement | Error if provider doesn't support ACLs | |
| Skip ACLs entirely | Ignore visibility on S3 | |

**User's choice:** [auto] Best-effort ACL header (recommended default)
**Notes:** AWS itself deprecated per-object ACLs. DO Spaces and R2 use bucket-level access.

---

## Directory Operations Semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Prefix-based with marker objects | ListObjectsV2 with delimiter for listing; zero-byte .keep for make_directory; batch delete for delete_directory | ✓ |
| Prefix-based without markers | No marker objects for make_directory (no-op) | |
| Error on directory ops | S3 doesn't have directories, so error out | |

**User's choice:** [auto] Prefix-based with marker objects (recommended default)
**Notes:** Standard S3 convention. .keep marker ensures empty directories are visible in listings.

---

## Claude's Discretion

- Path normalization details
- Pagination batch size
- Content-Type detection strategy

## Deferred Ideas

None
