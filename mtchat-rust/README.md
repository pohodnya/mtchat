# MTChat Rust Backend

Rust backend for MTChat.

This crate provides:

- Chat API for end users
- Management API for your backend
- WebSocket real-time delivery
- PostgreSQL persistence
- optional Redis-backed presence and jobs
- optional S3-compatible file storage

## Requirements

- Rust `1.78+`
- PostgreSQL `17`
- Optional: Redis `7+` for presence and background jobs
- Optional: S3-compatible storage for attachments

## Local Development

From the repository root:

```bash
docker compose up -d postgres redis minio
```

Then run the backend:

```bash
cd mtchat-rust
cargo run
```

The server applies database migrations automatically on startup.

## Main Endpoints

### Health

- `GET /health`
- `GET /health/ready`

### Management API

- `POST /api/v1/management/dialogs`
- `GET /api/v1/management/dialogs/{id}`
- `DELETE /api/v1/management/dialogs/{id}`
- `POST /api/v1/management/dialogs/{id}/participants`
- `DELETE /api/v1/management/dialogs/{id}/participants/{user_id}`
- `PUT /api/v1/management/dialogs/{id}/access-scopes`

### Chat API

- `GET /api/v1/dialogs`
- `GET /api/v1/dialogs/{id}`
- `GET /api/v1/dialogs/by-object/{object_type}/{object_id}`
- `POST /api/v1/dialogs/{id}/join`
- `POST /api/v1/dialogs/{id}/leave`
- `POST /api/v1/dialogs/{id}/archive`
- `POST /api/v1/dialogs/{id}/unarchive`
- `POST /api/v1/dialogs/{id}/pin`
- `POST /api/v1/dialogs/{id}/unpin`
- `POST /api/v1/dialogs/{id}/notifications`
- `POST /api/v1/dialogs/{id}/read`
- `GET /api/v1/dialogs/{id}/participants`
- `GET /api/v1/dialogs/{dialog_id}/messages`
- `POST /api/v1/dialogs/{dialog_id}/messages`
- `GET /api/v1/dialogs/{dialog_id}/messages/{id}`
- `PUT /api/v1/dialogs/{dialog_id}/messages/{id}`
- `DELETE /api/v1/dialogs/{dialog_id}/messages/{id}`
- `POST /api/v1/upload/presign`
- `GET /api/v1/attachments/{id}/url`

### WebSocket

- `GET /api/v1/ws`

Authentication depends on server configuration:

- JWT mode: `?token=<jwt>`
- legacy mode: `?user_id=<id>`

## Documentation

- API reference: [`../docs/api/chat.md`](../docs/api/chat.md)
- Management API: [`../docs/api/management.md`](../docs/api/management.md)
- WebSocket: [`../docs/api/websocket.md`](../docs/api/websocket.md)
- File upload: [`../docs/api/file-upload.md`](../docs/api/file-upload.md)
- Security: [`../docs/security.md`](../docs/security.md)
- Deployment: [`../deploy/README.md`](../deploy/README.md)

## Verification

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --lib
cargo test --tests
```

## License

MIT
