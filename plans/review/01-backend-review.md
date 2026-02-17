# Backend Code Review: mtchat-rust

**Reviewer:** backend-dev-1
**Date:** 2026-02-17
**Scope:** Full code review of `mtchat-rust/` (~7800 LOC across 30 .rs files)

---

## 1. Architecture & Structure

### Overall Assessment: Good

The project follows a clear layered architecture:
- `domain/` - Business entities and value objects
- `repositories/` - Database access (sqlx)
- `services/` - External integrations (S3, Redis presence)
- `middleware/` - Auth and request extractors
- `webhooks/` - Outgoing event notifications
- `jobs/` - Background task processing (apalis)
- `ws.rs` - WebSocket handling
- `main.rs` - Router setup, DI, handler functions

### Issues

**[HIGH] Monolithic main.rs (1516 lines)**
All HTTP handler functions (~30 handlers), all DTOs, and the entire router setup live in `main.rs`. This is the single biggest structural problem. Handlers should be extracted to `src/api/` modules (e.g., `api/dialogs.rs`, `api/messages.rs`, `api/upload.rs`, `api/management.rs`).

**[MEDIUM] No lib.rs**
The project is structured as a pure binary crate with no `lib.rs`. This makes integration testing harder because test code cannot import internal types. The existing `tests/domain_test.rs` works around this by re-implementing scope matching logic locally rather than testing the actual domain code.

**[LOW] Unused config module**
`config/default.toml` defines a comprehensive configuration schema (rate limiting, pool sizes, JWT, websocket params, telemetry) but none of it is actually used. The application reads all config directly from environment variables via `env::var()`. Either remove the config file or integrate the `config` crate properly.

**[LOW] Unused dependencies**
Several Cargo.toml dependencies appear to be unused or partially used:
- `argon2` - no password hashing anywhere in the code
- `jsonwebtoken` - no JWT validation (user_id extracted from query params)
- `metrics`, `metrics-exporter-prometheus` - no metrics recorded
- `sentry` - imported but never initialized
- `config` - config file exists but not loaded
- `validator` - never used for request validation
- `dashmap` - not used anywhere
- `regex` - not used anywhere
- `lazy_static` - not used (uses `once_cell` instead)
- `rand` - not used

---

## 2. Security Analysis

### Critical Issues

**[CRITICAL] No authentication on Chat API**
The Chat API endpoints have **zero authentication**. User identity is determined solely by the `user_id` query parameter, which can be trivially spoofed:

```
GET /api/v1/dialogs?type=participating&user_id=<any-uuid>
```

Any client can impersonate any user by simply changing the query parameter. The `jsonwebtoken` dependency exists in `Cargo.toml` but is never used. The `JWT_SECRET` environment variable is set but never read by the application.

**Impact:** Complete identity spoofing. Any user can read any other user's messages, send messages as them, join/leave dialogs, etc.

**Recommendation:** Implement JWT-based authentication middleware for the Chat API. Extract `user_id` from a validated JWT token in the `Authorization` header, not from query parameters.

**[CRITICAL] WebSocket authentication bypass**
WebSocket connections authenticate via query parameter only:
```rust
// ws_handler in main.rs:1322-1324
let user_id = params.get("user_id")
    .and_then(|s| s.parse().ok())
    .unwrap_or_else(Uuid::new_v4);  // Falls back to random UUID!
```

If no `user_id` is provided, a random UUID is generated and the connection proceeds. No token validation whatsoever.

**[HIGH] Admin token comparison is not constant-time**
In `middleware/admin_auth.rs:90`:
```rust
if token != expected_token {
```
This uses standard string comparison which is vulnerable to timing attacks. The webhook module already has a `constant_time_eq` function that should be reused here.

**[HIGH] Admin token read from env on every request**
`admin_auth.rs:55` calls `env::var("ADMIN_API_TOKEN")` on every request instead of reading it once at startup. This is both a performance issue and a potential TOCTOU race if the environment changes.

**[MEDIUM] CORS is fully open**
```rust
let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);
```
`Allow-Origin: *` with `Allow-Headers: *` means any website can make authenticated requests to the API. In production, this should be restricted to known origins.

**[MEDIUM] No rate limiting**
Despite `config/default.toml` defining rate limit settings, no rate limiting is implemented. The API is vulnerable to abuse (message flooding, brute-force, etc.).

**[LOW] S3 key path traversal**
In `presign_upload` handler, the file extension is extracted from user input:
```rust
let ext = req.filename.rsplit('.').next().unwrap_or("bin");
let s3_key = format!("dialogs/{}/pending/{}.{}", req.dialog_id, file_uuid, ext);
```
While the UUID prevents path traversal in the key itself, the extension is not sanitized. A filename like `file.../../etc/passwd` would produce `file_uuid.../../etc/passwd`. The generated UUID prefix likely prevents exploitation, but the extension should still be sanitized.

**[LOW] No input validation on text fields**
`display_name`, `company`, `object_type`, `title` etc. have no length limits. A user could submit megabytes of data in these fields.

---

## 3. Error Handling

### Assessment: Adequate with some issues

**[MEDIUM] sqlx errors leak database details**
```rust
impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError::Internal(e.to_string())
    }
}
```
The `ApiError::Internal` variant logs the full error message via `tracing::error!` (good) and returns "Internal server error" to the client (good). However, some handlers convert sqlx errors via `?` which triggers this impl, while the `Internal` variant's message field contains the full sqlx error. The `IntoResponse` impl correctly replaces it with a generic message, so this is safe but worth noting.

**[LOW] Silent failures in broadcast functions**
All `ws::broadcast_*` functions silently ignore send failures:
```rust
let _ = tx.send(json.clone()).await;
```
This is intentional (disconnected clients), but there's no logging or metrics to track broadcast failures.

**[LOW] No structured error codes for specific failures**
All errors use generic codes like `NOT_FOUND`, `BAD_REQUEST`, `FORBIDDEN`. More specific error codes (e.g., `ALREADY_PARTICIPANT`, `SCOPE_MISMATCH`, `FILE_TOO_LARGE`) would help API consumers handle errors programmatically.

---

## 4. Async/Await Patterns

### Assessment: Good

**[MEDIUM] N+1 query in list_dialogs**
The `list_dialogs` handler has an N+1 query problem:
```rust
for dialog in dialogs {
    let participants_count = state.dialogs.count_participants(dialog.id).await?;
    let (unread_count, is_archived, ...) = if dialog_type == "participating" {
        let participant = state.participants.find(dialog.id, user_id).await?;
        // ...
    };
}
```
For each dialog in the list, two additional queries are executed (participant count + participant lookup). With 50 dialogs, this means 100+ additional queries. This should use batch queries.

**[MEDIUM] N+1 in list_messages for presigned URLs**
```rust
for message in messages {
    for att in &attachments {
        let url = state.s3.generate_download_url(&att.s3_key).await...
        let thumbnail_url = state.s3.generate_download_url(thumb_key).await...
    }
}
```
Each attachment requires 1-2 S3 presigning calls. With 50 messages and 2 attachments each, that's 100-200 S3 API calls. Presigned URLs can be generated without network calls in newer AWS SDK versions (client-side computation).

**[MEDIUM] send_message handler is heavy**
The `send_message` handler does many sequential operations:
1. Find dialog (DB)
2. Check participant exists (DB)
3. Validate each attachment type/size
4. Check each attachment exists in S3 (N S3 calls)
5. Sanitize HTML
6. Create message (DB)
7. Create attachments (DB transaction)
8. Increment unread counts (DB)
9. Unarchive all participants (DB)
10. Mark sender's message as read (DB)
11. Broadcast WebSocket
12. Send webhook
13. Enqueue notification jobs for each participant (N Redis calls)

Some of these (steps 11-13) could be made concurrent since they don't depend on each other.

**[LOW] Global broadcast to all connected users**
`broadcast_message` sends the event to ALL connected WebSocket clients:
```rust
let conns = connections.read().await;
for (_, tx) in conns.iter() {
    let _ = tx.send(json.clone()).await;
}
```
There's no subscription/filtering mechanism. Every connected user receives every message event, even for dialogs they don't participate in. The `Subscribe`/`Unsubscribe` WebSocket messages are received but not acted upon (just logged as debug).

---

## 5. Database & SQL

### Assessment: Good query patterns

**[MEDIUM] No database transactions for multi-step operations**
Several handlers perform multiple database writes without transactions:
- `management_create_dialog`: Creates dialog, adds participants, creates scopes, creates system message - each as separate queries. If any step fails, the database is left in an inconsistent state.
- `send_message`: Creates message, creates attachments, updates unread counts - partial failures leave inconsistent state.
- `join_dialog`: Adds participant, sets unread count, creates system message.

**[LOW] Scope replacement not atomic**
`AccessScopeRepository::replace_for_dialog` does delete + insert without a transaction:
```rust
pub async fn replace_for_dialog(...) -> ... {
    self.delete_by_dialog(dialog_id).await?;
    for scope in scopes {
        let created = self.create(&scope).await?;
        // If this fails mid-loop, some scopes are deleted but not all new ones created
    }
}
```

**[LOW] ILIKE search without index**
`find_participating` and `find_available` use `ILIKE '%' || $2 || '%'` for search, which cannot use standard B-tree indexes. For large datasets, this will be slow. Consider `pg_trgm` extension with GIN indexes for text search.

**[LOW] No pagination on dialog list**
`find_participating` and `find_available` return all matching dialogs without pagination. For users with many chats, this could be slow.

---

## 6. Code Quality & Rust Best Practices

### Assessment: Good overall, some improvements needed

**[MEDIUM] Repository trait defined but never used**
```rust
pub trait Repository {
    fn pool(&self) -> &PgPool;
}
```
This trait is defined in `repositories/mod.rs` but no repository implements it. Either implement it or remove it.

**[MEDIUM] Inconsistent UUID versions**
- `Dialog::new` uses `Uuid::new_v4()` (random)
- `Message::new` uses `Uuid::now_v7()` (time-ordered)
- `Attachment::new` uses `Uuid::now_v7()` (time-ordered)
- `DialogAccessScope::new` uses `Uuid::new_v4()` (random)

Messages and attachments correctly use v7 for ordering. Dialogs and access scopes should also use v7 for consistency and better index performance.

**[LOW] JoinedAs enum stored as String**
`DialogParticipant.joined_as` is stored as `String` instead of using the `JoinedAs` enum directly. There's a `from_str` conversion and an `as_str` method, but sqlx custom encode/decode is not implemented (unlike `MessageType` which has proper sqlx type support).

**[LOW] MessageType::from_str shadows std trait**
```rust
impl MessageType {
    pub fn from_str(s: &str) -> Self {
```
This inherent method has the same name as `std::str::FromStr::from_str`. It should either implement the standard trait or use a different name like `parse_str`.

**[LOW] Presence TTL comment inconsistency**
```rust
const ONLINE_TTL: i64 = 20;  // Comment says 20s
// But CLAUDE.md says:
// "60s TTL" and "30s ping refreshes 60s TTL"
```
The actual TTL (20s) doesn't match the documentation (60s).

**[LOW] Duplicate code in broadcast functions**
`ws.rs` has 8 broadcast functions that all follow the same pattern: serialize event -> read connections -> iterate & send. This could be refactored to a generic `broadcast_to_all` and `broadcast_to_users` helper.

---

## 7. Test Coverage

### Assessment: Minimal

**Unit Tests (run without DB):**
- `html_sanitize.rs` - 12 tests (good coverage for sanitization)
- `system_messages.rs` - 5 tests
- `scope_config.rs` - 2 tests
- `s3.rs` - 1 test (URL rewriting)
- `webhooks/events.rs` - 2 tests
- `webhooks/sender.rs` - 3 tests
- `jobs/types.rs` - 3 tests
- `jobs/worker.rs` - 2 tests
- `jobs/producer.rs` - 1 test

**Integration Tests (require DB):**
- `migrations_test.rs` - 13 tests (schema validation, well-written)
- `management_api_test.rs` - 8 tests (all `#[ignore]`, require running server)
- `chat_api_test.rs` - 11 tests (all `#[ignore]`, require running server)
- `domain_test.rs` - 7 trivial tests (test basic string equality, not actual domain logic)

**Gaps:**
- No tests for repository layer (could use test DB)
- No tests for handler logic in `main.rs`
- `domain_test.rs` tests are trivial and don't test actual domain models
- `management_api_test.rs` and `chat_api_test.rs` require a running server, not suitable for CI
- No tests for WebSocket functionality
- No tests for error paths (invalid input, auth failures)
- No tests for edge cases (empty arrays, boundary values, concurrent access)

---

## 8. Performance Considerations

**[MEDIUM] Single HashMap for WebSocket connections**
```rust
connections: Arc<RwLock<HashMap<Uuid, ConnectionTx>>>
```
A single `RwLock` means all broadcasts take a read lock and all connect/disconnect take a write lock. Under high load with many users connecting/disconnecting, this becomes a contention point. `DashMap` (already in dependencies but unused) would be better here.

**[MEDIUM] No connection pooling configuration**
The database pool is hardcoded to 20 connections:
```rust
let db = PgPoolOptions::new()
    .max_connections(20)
    .connect(&database_url)
    .await
```
The `config/default.toml` defines `pool_min`, `pool_max`, `acquire_timeout` etc. but none are used.

**[LOW] Cloning in loops**
Several places clone strings unnecessarily inside loops (e.g., `json.clone()` in broadcast functions). For WebSocket broadcasts, each message is cloned N times. An `Arc<String>` would be more efficient for large payloads.

---

## 9. Docker & Deployment

**[LOW] Dockerfile does not use layer caching**
The Dockerfile copies everything and builds:

```dockerfile
COPY .. .
RUN cargo build --release
```
Any change to any file invalidates the entire build cache. Standard practice is to copy and build `Cargo.toml`/`Cargo.lock` first (to cache dependencies), then copy source code.

**[LOW] No .dockerignore for target/**
The `.dockerignore` only contains one entry. The `target/` directory (which can be gigabytes) might be sent to the Docker context. Verify `.dockerignore` includes `target/`.

---

## 10. Summary

### Severity Counts

| Severity | Count | Key Issues |
|----------|-------|------------|
| CRITICAL | 2 | No Chat API authentication, WebSocket auth bypass |
| HIGH | 3 | Admin token timing attack, admin token per-request env read, monolithic main.rs |
| MEDIUM | 10 | N+1 queries, no DB transactions, CORS open, no rate limiting, unused trait, inconsistent UUIDs, global WS broadcast, no lib.rs, connection pooling, WS HashMap contention |
| LOW | 14 | Various code quality, testing, Docker issues |

### Top Priority Recommendations

1. **Implement JWT authentication** for Chat API (CRITICAL security fix)
2. **Extract handlers from main.rs** to `src/api/` modules (major DX improvement)
3. **Add lib.rs** to enable proper integration testing
4. **Fix N+1 queries** in `list_dialogs` with batch queries
5. **Add database transactions** to multi-step write operations
6. **Remove unused dependencies** from `Cargo.toml`
7. **Add proper unit tests** for repository and handler layers
8. **Fix admin auth** to use constant-time comparison and read token once
9. **Implement WebSocket subscription filtering** (don't broadcast everything to everyone)
10. **Improve Docker build** with layer caching

### Positive Highlights

- Clean domain model with good separation of concerns
- Effective HTML sanitization with ammonia
- HMAC-SHA256 webhook signing with constant-time verification
- Graceful degradation (S3, Redis, job queue all optional)
- Good migration test coverage
- Proper use of UUIDv7 for time-ordered records
- Well-documented CLAUDE.md with comprehensive feature list
- Background job queue with smart notification debouncing
- Auto-archive with WebSocket event broadcasting
