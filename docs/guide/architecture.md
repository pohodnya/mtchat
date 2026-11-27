# Architecture

MTChat is a microservice-based chat system designed to be embedded into existing business applications. Your application manages users and business logic; MTChat handles conversations.

## System Overview

```
┌──────────────────────────────────────────────────────────┐
│                  Host Application                        │
│  ┌────────────┐              ┌──────────────────────┐    │
│  │  Frontend  │              │  Backend             │    │
│  │ ┌────────┐ │              │  - Creates dialogs   │    │
│  │ │ MTChat │ │              │  - Manages members   │    │
│  │ │Vue SDK │ │              │  - Handles webhooks  │    │
│  │ └───┬────┘ │              └──────────┬───────────┘    │
│  └─────┼──────┘                         │                │
└────────┼────────────────────────────────┼────────────────┘
         │                                │
         │ Chat API                       │ Management API
         │ (user context)                 │ (Admin Token)
         ▼                                ▼
┌──────────────────────────────────────────────────────────┐
│                    MTChat Backend                        │
│  ┌──────────────┐  ┌────────────────┐  ┌────────────┐    │
│  │   Chat API   │  │ Management API │  │  Webhooks  │    │
│  │  (end users) │  │   (system)     │  │ (outgoing) │    │
│  └──────────────┘  └────────────────┘  └────────────┘    │
│                                                          │
│  ┌──────────┐  ┌────────────┐  ┌────────────────────┐    │
│  │PostgreSQL│  │   Redis    │  │  S3 / MinIO        │    │
│  │  (data)  │  │(presence,  │  │  (file storage)    │    │
│  │          │  │ jobs)      │  │                    │    │
│  └──────────┘  └────────────┘  └────────────────────┘    │
└──────────────────────────────────────────────────────────┘
```

## Two APIs

MTChat exposes two separate APIs:

### Management API

Called by **your backend** using an admin token. Used to create dialogs, add/remove participants, and manage access scopes. This is the server-to-server integration point.

```
POST /api/v1/management/dialogs
POST /api/v1/management/dialogs/{id}/participants
DELETE /api/v1/management/dialogs/{id}
```

### Chat API

Called by the **Vue SDK** (or any frontend client) in the context of an authenticated user. Provides endpoints for listing dialogs, sending messages, joining/leaving chats, and real-time communication via WebSocket.

```
GET  /api/v1/dialogs?type=participating
POST /api/v1/dialogs/{id}/messages
WS   /api/v1/ws
```

## Backend Internals

The Rust backend is structured in layers:

```
src/
├── api/           # HTTP handlers (REST endpoints)
├── domain/        # Business entities and value objects
├── repositories/  # Database access (sqlx + PostgreSQL)
├── services/      # External integrations (S3, Redis presence)
├── middleware/     # Authentication and request extractors
├── webhooks/      # Outgoing event notifications
├── jobs/          # Background task processing (apalis)
└── ws/            # WebSocket handling
```

### Request Flow

1. HTTP request arrives at an **axum** handler
2. **Middleware** extracts user identity and validates admin tokens
3. Handler calls **repository** methods for database operations
4. Side effects trigger **WebSocket broadcasts**, **webhook dispatches**, and **background jobs**

### Infrastructure Services

| Service | Purpose | Required |
|---------|---------|----------|
| **PostgreSQL** | Dialogs, messages, participants, scopes | Yes |
| **Redis** | Online presence (TTL-based), background job queue (apalis), PubSub | No |
| **S3 / MinIO** | File attachment storage with presigned URLs | No |

All optional services degrade gracefully -- if Redis is not configured, online status and background jobs are disabled. If S3 is not configured, file upload endpoints return errors.

## Real-Time Communication

MTChat uses WebSocket for real-time updates:

- **message.new** -- new message in a dialog
- **message.edited** / **message.deleted** -- message modifications
- **message.read** -- read receipt updates
- **participant.joined** / **participant.left** -- membership changes
- **dialog.archived** / **dialog.unarchived** -- archive state changes
- **presence.update** -- user online/offline status

The Vue SDK maintains a persistent WebSocket connection with automatic reconnection and heartbeat (30-second ping interval).

With several API replicas, a user's tabs may be connected to different replicas. Each replica delivers an event to its own connections directly and publishes it to the Redis channel `mtchat:ws:events`; the other replicas deliver it to their connections of the listed recipients. A user goes offline only when no connection is left on any replica. A replica that stops gracefully (SIGTERM/SIGINT) removes its connections from the shared registry but sends no offline updates, since clients reconnect to another replica within seconds and an offline update followed by an online one would make every user of the replica blink on each deploy. A user left with no connection keeps the online flag for 30 more seconds: a reconnect extends it, otherwise it expires and the REST presence shows the user offline, while connected clients get no `presence.update` with `is_online: false` for that user (a known limitation). Connections of a crashed replica expire after 120 seconds. Running more than one replica therefore requires a standalone Redis (Redis Cluster is not supported).

Delivery between replicas is at-most-once: events published while a replica's subscription is reconnecting, or dropped on a buffer overflow, are lost, and clients restore the state on reconnect or page reload. When Redis fails while a connection closes, the offline update is sent only to the connections of that replica, as the user may still have a tab on another one.

## Background Jobs

MTChat uses **apalis** with a Redis backend for background task processing:

- **Smart notifications** -- notification jobs wait briefly, then send a webhook only if the message is still unread and notifications are enabled for the recipient.
- **Auto-archive** -- periodic job (default: every 5 minutes) archives dialogs inactive for a configurable period (default: 3 days).

## Outgoing Webhooks

When configured, MTChat sends HTTP POST requests to your backend for key events:

- `message.new` -- a new message was sent
- `participant.joined` / `participant.left` -- someone joined or left a dialog
- `notification.pending` -- an unread message notification is ready after the read-state check

Webhooks are signed with HMAC-SHA256 for verification. See [Webhooks](../api/webhooks.md) for details.
