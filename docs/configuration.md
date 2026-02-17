# Configuration

MTChat is configured via environment variables. All settings have sensible defaults -- only `DATABASE_URL` and `ADMIN_API_TOKEN` are required.

## Required Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://user:pass@localhost:5432/mtchat` |
| `ADMIN_API_TOKEN` | Token for Management API authentication | `your-secure-admin-token` |

## Server

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8080` | HTTP server port |
| `RUST_LOG` | `info` | Log level (e.g., `multitenancy_chat_api=debug,tower_http=info`) |

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
| `S3_PUBLIC_ENDPOINT` | -- | Public-facing S3 URL for presigned download links |
| `S3_BUCKET` | -- | S3 bucket name for attachments |
| `S3_ACCESS_KEY_ID` | -- | S3 access key |
| `S3_SECRET_ACCESS_KEY` | -- | S3 secret key |
| `S3_REGION` | `us-east-1` | S3 region |

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
| `NOTIFICATION_DELAY_SECS` | `30` | Seconds to wait before sending a notification webhook |
| `NOTIFICATION_CONCURRENCY` | `10` | Number of concurrent notification workers |
| `ARCHIVE_CRON` | `0 */5 * * * *` | Cron schedule for auto-archive check |
| `ARCHIVE_AFTER_SECS` | `259200` | Seconds of inactivity before auto-archiving (default: 3 days) |

## Authentication

| Variable | Default | Description |
|----------|---------|-------------|
| `JWT_SECRET` | -- | Secret for JWT token signing (reserved for future use) |
| `ADMIN_API_TOKEN` | required | Bearer token for Management API |

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
    image: ghcr.io/nicenemo/mtchat:latest
    environment:
      DATABASE_URL: postgres://postgres:postgres@postgres:5432/multitenancy_chat
      ADMIN_API_TOKEN: ${ADMIN_API_TOKEN:-demo-admin-token}
      REDIS_URL: redis://redis:6379
      S3_ENDPOINT: http://minio:9000
      S3_PUBLIC_ENDPOINT: http://localhost:9000
      S3_BUCKET: mtchat-attachments
      S3_ACCESS_KEY_ID: minioadmin
      S3_SECRET_ACCESS_KEY: minioadmin
      S3_REGION: us-east-1
      WEBHOOK_URL: https://your-app.com/webhooks/mtchat
      WEBHOOK_SECRET: your-webhook-secret
      NOTIFICATION_DELAY_SECS: 30
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
