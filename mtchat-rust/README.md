# MultitenancyChat API

B2B/B2C Chat Platform Backend built with Rust.

## Requirements

- Rust 1.75+
- PostgreSQL 17
- Redis 7+
- Docker (optional, for local development)

## Quick Start

### 1. Start Dependencies

```bash
# Using Docker Compose (recommended)
cd docker
docker-compose -f docker-compose.dev.yml up -d
```

### 2. Configure Environment

```bash
cp .env.example .env
# Edit .env with your settings
```

### 3. Run Migrations

```bash
# Install sqlx-cli if not installed
cargo install sqlx-cli

# Run migrations
sqlx migrate run
```

### 4. Start the Server

```bash
cargo run
```

The server will start at `http://localhost:8080`.

## API Documentation

### Authentication

All protected endpoints require a JWT token in the `Authorization` header:

```
Authorization: Bearer <token>
```

Get a token by calling:
```
POST /api/v1/auth/token
{ "external_id": "<employee-external-id>" }
```

### Endpoints

#### Health
- `GET /health` - Health check
- `GET /health/ready` - Readiness probe
- `GET /health/live` - Liveness probe
- `GET /metrics` - Prometheus metrics

#### Tenants
- `GET /api/v1/tenants/:id` - Get tenant
- `PUT /api/v1/tenants/:id` - Update tenant

#### Employees
- `GET /api/v1/employees` - List employees
- `POST /api/v1/employees` - Create employee
- `GET /api/v1/employees/me` - Current employee
- `GET /api/v1/employees/:id` - Get employee
- `PUT /api/v1/employees/:id` - Update employee
- `DELETE /api/v1/employees/:id` - Delete employee

#### Dialogs
- `GET /api/v1/dialogs` - List dialogs
- `POST /api/v1/dialogs` - Create dialog
- `GET /api/v1/dialogs/:id` - Get dialog
- `POST /api/v1/dialogs/:id/participants` - Add participant
- `DELETE /api/v1/dialogs/:id/participants/:eid` - Remove participant
- `POST /api/v1/dialogs/:id/archive` - Archive dialog
- `DELETE /api/v1/dialogs/:id/archive` - Unarchive dialog
- `PUT /api/v1/dialogs/:id/notifications` - Update notifications

#### Messages
- `GET /api/v1/dialogs/:id/messages` - List messages
- `POST /api/v1/dialogs/:id/messages` - Send message
- `GET /api/v1/dialogs/:id/messages/:mid` - Get message
- `PUT /api/v1/dialogs/:id/messages/:mid` - Edit message
- `DELETE /api/v1/dialogs/:id/messages/:mid` - Delete message
- `GET /api/v1/dialogs/:id/messages/:mid/history` - Edit history
- `POST /api/v1/dialogs/:id/messages/:mid/read` - Mark read

#### Attachments
- `POST /api/v1/uploads/presign` - Get presigned upload URL
- `GET /api/v1/attachments/:id` - Get attachment info
- `DELETE /api/v1/attachments/:id` - Delete attachment

#### Presence
- `GET /api/v1/presence/dialog/:id` - Dialog online status
- `GET /api/v1/presence/tenant/:id` - Tenant online status

### WebSocket

Connect to `ws://localhost:8080/api/v1/ws?token=<JWT>`.

#### Client → Server Events
- `subscribe` - Subscribe to dialog
- `unsubscribe` - Unsubscribe from dialog
- `message.send` - Send message
- `typing.start` / `typing.stop` - Typing indicator
- `message.read` - Mark message as read
- `ping` - Keep-alive

#### Server → Client Events
- `connected` - Connection established
- `message.new` - New message
- `message.edited` - Message edited
- `message.deleted` - Message deleted
- `typing` - Someone is typing
- `message.read_receipt` - Read receipt
- `participant.joined` / `participant.left` - Participant changes
- `presence.changed` - Online status change
- `notification.mention` - @mention notification
- `pong` - Keep-alive response
- `error` - Error message

## Development

```bash
# Check code
cargo check

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## Docker

```bash
# Build image
docker build -f docker/Dockerfile -t multitenancy-chat-api .

# Run with docker-compose
cd docker
docker-compose up
```

## License

Proprietary
