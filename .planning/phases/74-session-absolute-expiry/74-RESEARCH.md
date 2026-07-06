# Phase 74: Session Absolute Expiry — Research

**Researched:** 2026-02-26
**Domain:** Session management — absolute expiry, idle timeout separation, bulk invalidation
**Confidence:** HIGH

<research_summary>
## Summary

Researched OWASP session management requirements and Laravel's session implementation to inform Ferro's absolute session expiry feature. The domain is well-established with clear standards.

Ferro currently has a single `lifetime` config (idle timeout, default 2h) and no `created_at` tracking. OWASP mandates both idle and absolute timeouts as independent server-side controls. Laravel lacks native absolute expiry entirely — this is a known gap that Ferro should fix as a first-class feature.

The implementation is straightforward: add `created_at` column to the sessions table, add `absolute_lifetime` to `SessionConfig`, enforce both timeouts in `DatabaseSessionDriver::read()` and `gc()`, and add `destroy_for_user()` to the `SessionStore` trait for password-change invalidation flows.

**Primary recommendation:** Add `created_at` column + `absolute_lifetime` config. Direct DB deletion for `invalidate_all_for_user()` (not Laravel's indirect password-rehashing trick). Keep 30-day default for absolute timeout per roadmap spec, with OWASP-recommended values documented for high-security apps.
</research_summary>

<standard_stack>
## Standard Stack

No new dependencies required. This feature is built entirely within Ferro's existing session module using:

### Core (Already Present)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| sea-orm | current | ORM queries for session operations | Already used by DatabaseSessionDriver |
| chrono | current | Timestamp handling | Already used for last_activity |
| tokio | current | Async runtime | Already used throughout |

### No New Libraries Needed

This is internal session management logic. The entire feature is:
- Schema change (add column)
- Config change (add field)
- Driver logic (two timestamp checks instead of one)
- Trait extension (add method)
- Bulk deletion query (DELETE WHERE user_id = ?)
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Pattern 1: Dual Timeout — Idle + Absolute

**What:** Two independent server-side timeout checks on every session read.
**When to use:** Always — OWASP mandates both.

```
Session valid = (now - last_activity < idle_timeout) AND (now - created_at < absolute_timeout)
```

The idle timeout (currently `lifetime`) catches abandoned sessions. The absolute timeout catches actively-used stolen sessions that an attacker keeps alive by generating requests.

**Enforcement points:**
1. `DatabaseSessionDriver::read()` — reject expired sessions on load
2. `DatabaseSessionDriver::gc()` — clean up expired rows: `DELETE WHERE last_activity < idle_threshold OR created_at < absolute_threshold`

### Pattern 2: Direct DB Deletion for Session Invalidation

**What:** `DELETE FROM sessions WHERE user_id = ? AND id != ?` for "logout other devices".
**When to use:** Password change, explicit "logout everywhere", account compromise response.

This is immediate and reliable — no middleware dependency, no lazy invalidation.

Laravel uses an indirect approach (rehash password, detect mismatch on next request) that is fragile and only works with password-based auth. Direct deletion is simpler, more reliable, and auth-method-agnostic.

**API surface:**
```rust
// On SessionStore trait
async fn destroy_for_user(&self, user_id: i64, except_session_id: Option<&str>) -> Result<u64, FrameworkError>;

// Convenience on Auth facade
Auth::logout_other_devices();  // Destroys all sessions for current user except current

// Or standalone for admin/security flows
Session::invalidate_all_for_user(user_id);  // Destroys ALL sessions for a user
```

### Pattern 3: Config Structure — Two Durations

**What:** Separate `idle_lifetime` and `absolute_lifetime` in SessionConfig.
**When to use:** Always — they serve different security purposes with different defaults.

```rust
pub struct SessionConfig {
    pub idle_lifetime: Duration,      // Default: 2 hours (existing behavior)
    pub absolute_lifetime: Duration,  // Default: 30 days (new)
    // ... existing fields
}
```

Env vars: `SESSION_LIFETIME` (existing, minutes) + `SESSION_ABSOLUTE_LIFETIME` (new, minutes or days).

### Anti-Patterns to Avoid
- **Storing created_at in session payload instead of DB column:** Can't query/GC efficiently, requires deserializing every session
- **Password-rehashing trick for invalidation:** Fragile, requires middleware everywhere, doesn't work for passwordless auth
- **Client-side timeout enforcement only:** OWASP explicitly says server-side enforcement is mandatory
- **Single timeout for both concerns:** Idle and absolute serve different threat models
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Session expiry checking | Complex middleware chain | Check in `DatabaseSessionDriver::read()` | Single enforcement point, can't be bypassed |
| Bulk session deletion | Iterate-and-delete pattern | Single DELETE query with WHERE clause | SeaORM handles this efficiently |
| Timestamp arithmetic | Manual epoch math | `chrono::Duration` comparisons | Already used, handles edge cases |
| Config parsing | Custom duration parser | Reuse existing `from_env()` pattern with new env var | Consistency with existing config |

**Key insight:** This is standard session management. The entire feature is ~100 lines of changes across existing files. No new abstractions, no new crates, no new patterns.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Breaking Existing Apps on Upgrade
**What goes wrong:** Existing sessions table lacks `created_at` column, driver crashes on read
**Why it happens:** SeaORM entity expects column that doesn't exist in old schema
**How to avoid:** Make `created_at` optional in the entity model (`Option<DateTime>`). If NULL, treat session as having no absolute expiry (backward compatible). Only new sessions get `created_at` set.
**Warning signs:** "column not found" errors after framework upgrade

### Pitfall 2: GC Query Missing Absolute Timeout
**What goes wrong:** Expired absolute sessions accumulate in DB, never cleaned up
**Why it happens:** GC only checks `last_activity` (idle), not `created_at` (absolute)
**How to avoid:** Update GC query: `WHERE last_activity < idle_threshold OR (created_at IS NOT NULL AND created_at < absolute_threshold)`
**Warning signs:** Growing sessions table despite low traffic

### Pitfall 3: Renaming `lifetime` Breaks Existing Config
**What goes wrong:** Existing `SESSION_LIFETIME` env var stops working
**Why it happens:** Renaming the field or env var for clarity
**How to avoid:** Keep `SESSION_LIFETIME` as the idle timeout env var (backward compatible). Add `SESSION_ABSOLUTE_LIFETIME` as new env var. Rename the struct field from `lifetime` to `idle_lifetime` but keep the env var name.
**Warning signs:** Session timeout changes unexpectedly after upgrade

### Pitfall 4: invalidate_all_for_user Destroys Current Session
**What goes wrong:** User changes password and gets logged out themselves
**Why it happens:** DELETE WHERE user_id = ? without excluding current session
**How to avoid:** Always accept an optional `except_session_id` parameter. The "logout other devices" flow passes the current session ID to exclude.
**Warning signs:** Password change redirects to login page

### Pitfall 5: Cookie Max-Age vs Server-Side Expiry Mismatch
**What goes wrong:** Cookie outlives server session or vice versa
**Why it happens:** Cookie `max_age` set to idle timeout but session has longer absolute timeout
**How to avoid:** Set cookie `max_age` to the longer of the two timeouts (absolute_lifetime). The server enforces the real expiry; the cookie just needs to survive long enough to present the session ID.
**Warning signs:** Sessions that should still be valid get new IDs because cookie expired
</common_pitfalls>

<code_examples>
## Code Examples

### SessionConfig with Dual Timeouts
```rust
// Source: Ferro convention, informed by OWASP
pub struct SessionConfig {
    pub idle_lifetime: Duration,      // SESSION_LIFETIME env (default: 120 min)
    pub absolute_lifetime: Duration,  // SESSION_ABSOLUTE_LIFETIME env (default: 43200 min = 30 days)
    // ... existing fields unchanged
}

impl SessionConfig {
    pub fn from_env() -> Self {
        let idle_minutes: u64 = env_optional("SESSION_LIFETIME")
            .and_then(|s: String| s.parse().ok())
            .unwrap_or(120);

        let absolute_minutes: u64 = env_optional("SESSION_ABSOLUTE_LIFETIME")
            .and_then(|s: String| s.parse().ok())
            .unwrap_or(43200); // 30 days

        Self {
            idle_lifetime: Duration::from_secs(idle_minutes * 60),
            absolute_lifetime: Duration::from_secs(absolute_minutes * 60),
            // ...
        }
    }
}
```

### Dual Timeout Check in read()
```rust
// Source: OWASP pattern applied to Ferro's DatabaseSessionDriver
async fn read(&self, id: &str) -> Result<Option<SessionData>, FrameworkError> {
    // ... load session from DB ...

    let now = chrono::Utc::now();

    // Check idle timeout (existing)
    let idle_expiry = session.last_activity
        + chrono::Duration::seconds(self.idle_lifetime.as_secs() as i64);
    if now > idle_expiry {
        let _ = self.destroy(id).await;
        return Ok(None);
    }

    // Check absolute timeout (new)
    if let Some(created_at) = session.created_at {
        let absolute_expiry = created_at
            + chrono::Duration::seconds(self.absolute_lifetime.as_secs() as i64);
        if now > absolute_expiry {
            let _ = self.destroy(id).await;
            return Ok(None);
        }
    }

    // ... build SessionData ...
}
```

### Bulk Session Invalidation
```rust
// Source: Direct DB approach (better than Laravel's password-rehash trick)
async fn destroy_for_user(
    &self,
    user_id: i64,
    except_session_id: Option<&str>,
) -> Result<u64, FrameworkError> {
    let db = DB::connection()?;

    let mut query = sessions::Entity::delete_many()
        .filter(sessions::Column::UserId.eq(user_id));

    if let Some(except_id) = except_session_id {
        query = query.filter(sessions::Column::Id.ne(except_id));
    }

    let result = query.exec(db.inner()).await
        .map_err(|e| FrameworkError::database(e.to_string()))?;

    Ok(result.rows_affected)
}
```

### Updated Sessions Entity with created_at
```rust
// Source: Ferro convention
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub user_id: Option<i64>,
    #[sea_orm(column_type = "Text")]
    pub payload: String,
    pub csrf_token: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,  // NEW — nullable for backward compat
    pub last_activity: chrono::DateTime<chrono::Utc>,
}
```
</code_examples>

<sota_updates>
## State of the Art (2025)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single idle timeout | Dual idle + absolute | OWASP long-standing | Both are mandatory per OWASP |
| Password rehash for invalidation (Laravel) | Direct DB deletion | N/A (Ferro design choice) | Simpler, more reliable, auth-agnostic |
| No created_at tracking | First-class created_at column | N/A (Ferro design choice) | Enables SQL-level GC for absolute expiry |

**OWASP 2025 Top 10 A07 (Authentication Failures):**
- Explicitly requires session invalidation after re-authentication
- Requires token rotation
- Both idle and absolute timeouts mandatory

**No new tools/patterns needed** — this is stable, well-understood session management.

**Default values per OWASP:**
- Idle: 2-5 min (high-security) to 30-60 min (low-risk). Ferro default: 120 min (reasonable for web apps)
- Absolute: 4-8 hours (standard). Ferro default: 30 days (per roadmap spec — appropriate for a framework where apps configure their own, documented that high-security apps should lower this)
</sota_updates>

<open_questions>
## Open Questions

1. **Should `invalidate_all_for_user` live on the `SessionStore` trait or as a separate concern?**
   - What we know: It needs DB access. The `SessionStore` trait already has `destroy(id)`.
   - What's unclear: Whether adding to the trait burdens custom store implementors
   - Recommendation: Add to `SessionStore` trait with a default implementation that returns an error ("not supported"). `DatabaseSessionDriver` overrides with the real implementation.

2. **Should the `absolute_lifetime` default be 30 days or something shorter?**
   - What we know: OWASP recommends 4-8 hours for standard apps. Roadmap says 30 days.
   - What's unclear: Whether 30 days is too permissive for a framework default
   - Recommendation: Follow roadmap (30 days). Document OWASP recommendations. Framework users can configure shorter values. A web framework default should be permissive; apps tighten as needed.

3. **Backward compatibility for existing apps without `created_at` column?**
   - What we know: Making the column `Option<DateTime>` handles NULL gracefully
   - What's unclear: Whether we need a framework-level migration helper
   - Recommendation: Make `created_at` nullable in entity. If NULL, skip absolute expiry check. Update CLI migration template for new projects. Document manual migration for existing apps.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- OWASP Session Management Cheat Sheet — idle/absolute timeout requirements, enforcement rules
- OWASP Authentication Cheat Sheet — session invalidation on password change
- OWASP Top 10 2025 A07 — authentication failure patterns including session management
- Laravel 12.x Session docs — session table schema, lifetime configuration
- Laravel 12.x Authentication docs — `logoutOtherDevices` API

### Secondary (MEDIUM confidence)
- Laravel `AuthenticateSession` middleware source — password hash comparison mechanism
- Laravel `DatabaseSessionHandler` source — write/GC implementation details
- Laravel Enlightn security analyzer — absolute timeout as a known gap

### Tertiary (LOW confidence - needs validation)
- None — all findings verified against official sources
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Ferro session module (framework/src/session/)
- Ecosystem: No new dependencies — internal feature
- Patterns: Dual timeout, direct DB invalidation
- Pitfalls: Backward compatibility, GC coverage, config naming

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, well-understood domain
- Architecture: HIGH — OWASP patterns are clear and prescriptive
- Pitfalls: HIGH — common session management issues are well-documented
- Code examples: HIGH — based on existing Ferro code patterns

**Research date:** 2026-02-26
**Valid until:** 2026-06-26 (120 days — session management is a stable domain)
</metadata>

---

*Phase: 74-session-absolute-expiry*
*Research completed: 2026-02-26*
*Ready for planning: yes*
