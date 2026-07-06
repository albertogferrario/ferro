# Plan 12-05 Summary: Fix Inertia.js Redirects

## Objective

Fix Inertia.js redirects after POST/PUT/PATCH/DELETE requests that were returning 500 errors instead of properly redirecting.

## Problem Statement

The `redirect!()` macro and `Redirect` struct don't handle Inertia XHR requests correctly:
- Standard 302 redirects with `Location` header don't work for Inertia
- Inertia expects 303 (See Other) status for POST/PUT/PATCH/DELETE to force GET on redirect
- Missing `X-Inertia` header in redirect responses

Impact: All form submissions that redirect after success fail - login, registration, logout, CRUD operations.

## Solution

Created Inertia-aware redirect handling that works transparently with existing code:

1. **InertiaHttpResponse::redirect()** - Low-level redirect builder in inertia-rs
2. **InertiaRedirect type** - Request-aware redirect builder with query param support
3. **Inertia::redirect()** - Clean API for redirects with request context
4. **Inertia::redirect_ctx()** - Redirect using saved context after consuming request

## Tasks Completed

| # | Task | Files Modified |
|---|------|----------------|
| 1 | Add InertiaHttpResponse::redirect() method | inertia-rs/src/response.rs |
| 2 | Create InertiaRedirect type | framework/src/http/response.rs |
| 3 | Add Inertia::redirect() and redirect_ctx() | framework/src/inertia/context.rs |
| 4 | Export InertiaRedirect from public API | framework/src/http/mod.rs, framework/src/lib.rs |
| 5 | Document Inertia redirect handling | docs/src/features/inertia.md |
| 6 | Update auth controller template | cancer-cli/src/templates/files/backend/controllers/auth.rs.tpl |
| 7 | Update profile controller template | cancer-cli/src/templates/files/backend/controllers/profile.rs.tpl |

## Commits

- `c1d5f3c`: feat(inertia): add redirect method to InertiaHttpResponse
- `57c02b9`: feat(http): add InertiaRedirect type for Inertia-aware redirects
- `ef1a25e`: feat(inertia): add Inertia::redirect() and redirect_ctx() methods
- `7361bbf`: feat(http): export InertiaRedirect from framework public API
- `1e08e27`: docs(inertia): add Redirects section explaining Inertia::redirect()
- `6f778f6`: feat(cli): update auth controller template to use Inertia::redirect()
- `0ec79e0`: feat(cli): update profile controller template to use Inertia::redirect()

## Usage

### With Request Reference

```rust
pub async fn logout(req: Request) -> Response {
    Auth::logout();
    Inertia::redirect(&req, "/")
}
```

### With Saved Context

```rust
pub async fn login(req: Request) -> Response {
    let ctx = SavedInertiaContext::from(&req);
    let form: LoginForm = req.input().await?;

    // ... validation and auth ...

    Inertia::redirect_ctx(&ctx, "/dashboard")
}
```

## Verification

- [x] `cargo build --all` passes
- [x] `Inertia::redirect(&req, path)` method exists
- [x] `Inertia::redirect_ctx(&ctx, path)` method exists
- [x] Documentation includes redirect examples
- [x] Controller templates updated to use new redirect method
- [x] No breaking changes to existing `redirect!()` macro usage

## Key Decisions

1. **Inertia::redirect() preferred over InertiaRedirect** - Cleaner API that doesn't require `.into()` conversion
2. **303 status for POST-like methods** - Follows Inertia protocol to force GET on redirect
3. **X-Inertia header in responses** - Required for Inertia client to recognize redirect responses
4. **Standard 302 for non-Inertia requests** - Maintains backward compatibility
