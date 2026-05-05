# Configuration

MTChat is configured via environment variables. `DATABASE_URL` has a local-development default, but should be set explicitly in deployed environments. `ADMIN_API_TOKEN` should always be set outside local development; if it is omitted, the Management API starts unprotected.

## Core Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string; defaults to local PostgreSQL if omitted | `postgres://user:pass@localhost:5432/mtchat` |
| `ADMIN_API_TOKEN` | Token for Management API authentication; required for protected deployments | `your-secure-admin-token` |

## Server

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8080` | HTTP server port |
| `RUST_LOG` | `info` | Log level (e.g., `multitenancy_chat_api=debug,tower_http=info`) |

## Database Pool

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_MAX_CONNECTIONS` | `20` | Maximum PostgreSQL pool size |
| `DATABASE_MIN_CONNECTIONS` | `5` | Minimum PostgreSQL pool size |
| `DATABASE_ACQUIRE_TIMEOUT_SECS` | `30` | Connection acquire timeout |
| `DATABASE_IDLE_TIMEOUT_SECS` | `600` | Idle connection timeout |
| `DATABASE_MAX_LIFETIME_SECS` | `1800` | Maximum connection lifetime |

## Redis (Optional)

Enables online presence, background job queue, and smart notifications.

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIS_URL` | -- | Redis connection URL (e.g., `redis://localhost:6379`) |

If not set, online status, job queue, and auto-archive features are disabled.

## S3 / MinIO (Optional)

Enables file attachment uploads and downloads.

| Variable | Default | Description |
|----------|---------|-------------|
| `S3_ENDPOINT` | -- | S3/MinIO endpoint URL (e.g., `http://minio:9000`) |
| `S3_PUBLIC_ENDPOINT` | `S3_ENDPOINT` | Public-facing S3 URL for presigned links |
| `S3_BUCKET` | -- | S3 bucket name for attachments |
| `S3_ACCESS_KEY_ID` | -- | S3 access key |
| `S3_SECRET_ACCESS_KEY` | -- | S3 secret key |
| `S3_REGION` | `us-east-1` | S3 region |
| `S3_PRESIGN_UPLOAD_EXPIRY` | `300` | Upload URL lifetime in seconds |
| `S3_PRESIGN_DOWNLOAD_EXPIRY` | `3600` | Download URL lifetime in seconds |

If `S3_ENDPOINT` is not set, file upload endpoints return an error.

!!! tip
    `S3_PUBLIC_ENDPOINT` is the URL that browsers use to access S3. In local development with MinIO, this is typically `http://localhost:9000`, while `S3_ENDPOINT` is the internal Docker network URL `http://minio:9000`.

## Webhooks (Optional)

Enables outgoing event notifications to your backend.

| Variable | Default | Description |
|----------|---------|-------------|
| `WEBHOOK_URL` | -- | Your webhook endpoint URL |
| `WEBHOOK_SECRET` | -- | Secret for HMAC-SHA256 signing |

See [Webhooks](api/webhooks.md) for event types and signature verification.

## Background Jobs

Configure the apalis background job queue (requires Redis).

| Variable | Default | Description |
|----------|---------|-------------|
| `NOTIFICATION_CONCURRENCY` | `4` | Number of concurrent notification workers |
| `ARCHIVE_CRON` | `0 */5 * * * *` | Cron schedule for auto-archive check |
| `ARCHIVE_AFTER_SECS` | `259200` | Seconds of inactivity before auto-archiving (default: 3 days) |

Notification jobs currently use a short fixed delay before checking whether the message was read.

## Rate Limiting

Built-in request rate limiting is disabled by default.

| Variable | Default | Description |
|----------|---------|-------------|
| `RATE_LIMIT_ENABLED` | `false` | Enable the token-bucket rate limiter |
| `RATE_LIMIT_RPS` | `100` | Requests per second refill rate |
| `RATE_LIMIT_BURST` | `50` | Burst capacity |

## CORS

Configure cross-origin resource sharing for the API.

| Variable | Default | Description |
|----------|---------|-------------|
| `CORS_ALLOWED_ORIGINS` | `*` | Comma-separated origins or `*` for all |
| `CORS_ALLOWED_METHODS` | `GET,POST,PUT,DELETE,OPTIONS` | Allowed HTTP methods |
| `CORS_ALLOWED_HEADERS` | `*` | Allowed headers or `*` for all |
| `CORS_ALLOW_CREDENTIALS` | `false` | Allow credentials (`true`/`false`) |
| `CORS_MAX_AGE` | `3600` | Preflight cache duration (seconds) |

**Examples:**

```bash
# Development (default - fully open)
# No configuration needed

# Production (specific domains)
CORS_ALLOWED_ORIGINS="https://app.example.com,https://admin.example.com"
CORS_ALLOW_CREDENTIALS="true"
```

## Authentication

### Management API

| Variable | Default | Description |
|----------|---------|-------------|
| `ADMIN_API_TOKEN` | -- | Bearer token for Management API; omit only in local development |

### Chat API (JWT)

Optional JWT authentication for the Chat API. When enabled, validates token signature (HS256) without expiration check.

| Variable | Default | Description |
|----------|---------|-------------|
| `JWT_AUTH_ENABLED` | `false` | Enable JWT authentication for Chat API |
| `JWT_SECRET` | -- | Secret key for HS256 signature verification (required if JWT enabled) |
| `JWT_USER_ID_CLAIM` | `sub` | Claim name to read the user identifier from |

**How it works:**

- REST API: Token passed in `Authorization: Bearer <token>` header
- WebSocket: Token passed as `?token=<token>` query parameter
- User ID extracted from the claim configured via `JWT_USER_ID_CLAIM` (default: `sub`)
- Numeric claim values are stringified to match MTChat's `String` user identifiers
- When disabled, falls back to `?user_id=<id>` query parameter (legacy mode)

**Token format (HS256), default claim:**

```json
{
  "sub": "user-id-here",
  "iat": 1234567890
}
```

**Custom claim name** — when your host application encodes the user ID under a non-standard claim:

```bash
JWT_USER_ID_CLAIM=user_id
```

```json
{
  "user_id": "user-id-here",
  "iat": 1234567890
}
```

!!! tip "Token reuse"
    MTChat does not validate token expiration. The token is expected to be reused from your host application's authentication system.

## Monitoring (Optional)

| Variable | Default | Description |
|----------|---------|-------------|
| `SENTRY_DSN` | -- | Sentry error tracking DSN |

## Health Checks

| Endpoint | Description |
|----------|-------------|
| `GET /health` | Basic liveness check (returns `{"status":"ok"}`) |
| `GET /health/ready` | Readiness check (verifies database connection) |

## Docker Compose Example

```yaml
services:
  api:
    image: pohodnya/mtchat:latest
    environment:
      DATABASE_URL: postgres://postgres:postgres@postgres:5432/multitenancy_chat
      ADMIN_API_TOKEN: ${ADMIN_API_TOKEN}
      REDIS_URL: redis://redis:6379
      S3_ENDPOINT: http://minio:9000
      S3_PUBLIC_ENDPOINT: http://localhost:9000
      S3_BUCKET: mtchat-attachments
      S3_ACCESS_KEY_ID: minioadmin
      S3_SECRET_ACCESS_KEY: minioadmin
      S3_REGION: us-east-1
      WEBHOOK_URL: https://your-app.com/webhooks/mtchat
      WEBHOOK_SECRET: your-webhook-secret
      NOTIFICATION_CONCURRENCY: 4
      ARCHIVE_CRON: "0 */5 * * * *"
      ARCHIVE_AFTER_SECS: 259200
      RUST_LOG: multitenancy_chat_api=info,tower_http=info
    ports:
      - "8080:8080"
```

## Graceful Degradation

MTChat is designed to work with minimal infrastructure. Only PostgreSQL is required:

| Feature | Requires |
|---------|----------|
| Core messaging | PostgreSQL |
| Online status | PostgreSQL + Redis |
| File attachments | PostgreSQL + S3 |
| Smart notifications | PostgreSQL + Redis + Webhook URL |
| Auto-archive | PostgreSQL + Redis |

All optional features degrade gracefully when their dependencies are not configured.
