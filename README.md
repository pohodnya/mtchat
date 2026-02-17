<div align="center">

# MTChat

**Embeddable chat service for B2B/B2C platforms**

Add real-time chat to your application in minutes. MTChat handles messaging, delivery, notifications, and UI -- your app controls the business logic.

<table>
  <tr>
    <th>CI</th>
    <th>Packages</th>
    <th>Project</th>
  </tr>
  <tr>
    <td>
      <a href="https://github.com/pohodnya/mtchat/actions/workflows/ci.yml"><img src="https://github.com/pohodnya/mtchat/actions/workflows/ci.yml/badge.svg" alt="CI"></a><br>
      <a href="https://github.com/pohodnya/mtchat/actions/workflows/release.yml"><img src="https://github.com/pohodnya/mtchat/actions/workflows/release.yml/badge.svg" alt="Release"></a>
    </td>
    <td>
      <a href="https://www.npmjs.com/package/@mtchat/vue"><img src="https://img.shields.io/npm/v/@mtchat/vue?label=%40mtchat%2Fvue&color=blue" alt="npm @mtchat/vue"></a><br>
      <a href="https://www.npmjs.com/package/@mtchat/vue-primevue"><img src="https://img.shields.io/npm/v/@mtchat/vue-primevue?label=%40mtchat%2Fvue-primevue&color=blue" alt="npm @mtchat/vue-primevue"></a><br>
      <a href="https://hub.docker.com/r/pohodnya/mtchat"><img src="https://img.shields.io/docker/v/mtchat/backend?label=docker&sort=semver&color=blue" alt="Docker"></a>
    </td>
    <td>
      <img src="https://img.shields.io/badge/rust-1.75+-orange?logo=rust" alt="Rust 1.75+"><br>
      <img src="https://img.shields.io/badge/vue-3.4+-green?logo=vue.js" alt="Vue 3.4+"><br>
      <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT License"></a>
    </td>
  </tr>
</table>

</div>

---

## Highlights

- **Object-bound chats** -- each chat is linked to a business entity (order, tender, route, ticket)
- **Two participant types** -- direct participants (added by system) and potential participants (join via scope rules)
- **Vue.js SDK** -- drop-in `<MTChat>` component with full and inline modes
- **PrimeVue integration** -- optional `@mtchat/vue-primevue` wrapper for themed UI
- **Real-time** -- WebSocket-based messaging with presence tracking
- **Rich text** -- Tiptap editor with formatting, @mentions, and link support
- **File attachments** -- S3-compatible storage with presigned uploads (images, documents, archives)
- **i18n** -- Russian, English, and Chinese out of the box
- **Smart notifications** -- background job queue with debounced webhook delivery
- **Self-hosted** -- deploy with Docker Compose or Helm chart, no external dependencies

## Architecture

```
Your Application                          MTChat
┌──────────────┐                         ┌─────────────────────────┐
│   Frontend   │  Chat API (WebSocket)   │                         │
│  ┌────────┐  │ ──────────────────────> │   REST API + WebSocket  │
│  │ MTChat │  │                         │                         │
│  │Vue SDK │  │                         │   Background Jobs       │
│  └────────┘  │                         │                         │
│              │                         │   PostgreSQL + Redis    │
│   Backend    │  Management API         │   + S3 (MinIO)          │
│              │ ──────────────────────> │                         │
└──────────────┘                         └─────────────────────────┘
                  <── Outgoing Webhooks ──
```

**Your application** decides when to create chats, who participates, and how to handle events.
**MTChat** handles storage, delivery, real-time sync, file uploads, and the chat UI.

## Quick Start

### Docker Compose

```bash
git clone https://github.com/pohodnya/mtchat.git
cd mtchat

# Start all services (PostgreSQL, Redis, MinIO, API)
docker compose up -d

# Demo app: http://localhost
# API:      http://localhost:8080
# MinIO:    http://localhost:9001
```

### Production Deployment

```bash
cd deploy
cp .env.example .env    # Edit all values!
docker compose up -d
```

See [deploy/README.md](deploy/README.md) for Helm chart and advanced configuration.

## Usage

### 1. Create a Chat (Management API)

Your backend creates chats and manages participants via the Management API:

```bash
curl -X POST http://localhost:8080/api/v1/management/dialogs \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "object_id": "order-uuid-123",
    "object_type": "order",
    "title": "Order #1234 Discussion",
    "participants": ["user-1", "user-2"],
    "access_scopes": [{
      "tenant_uid": "tenant-abc",
      "scope_level1": ["logistics"],
      "scope_level2": ["manager", "admin"]
    }]
  }'
```

### 2. Embed the Chat (Vue.js)

Install the SDK:

```bash
npm install @mtchat/vue
```

Add the component:

```vue
<template>
  <MTChat :config="chatConfig" mode="full" />
</template>

<script setup>
import { MTChat } from '@mtchat/vue'
import '@mtchat/vue/style.css'

const chatConfig = {
  baseUrl: 'https://chat.example.com',
  token: userToken,
  userId: currentUser.id,
  scopeConfig: {
    tenant_uid: currentUser.tenantId,
    scope_level1: currentUser.departments,
    scope_level2: currentUser.permissions,
  },
  userProfile: {
    displayName: currentUser.name,
    company: currentUser.company,
  },
  locale: 'en', // 'ru' | 'en' | 'zh'
}
</script>
```

#### Inline Mode

Embed a chat directly inside a page (e.g., an order detail card):

```vue
<MTChat
  :config="chatConfig"
  mode="inline"
  :object-id="order.id"
  object-type="order"
/>
```

#### PrimeVue Integration

For projects using PrimeVue:

```bash
npm install @mtchat/vue-primevue
```

```vue
<template>
  <MTChatPrime :config="chatConfig" mode="full" theme="dark" />
</template>

<script setup>
import { MTChatPrime } from '@mtchat/vue-primevue'
</script>
```

### 3. Handle Webhooks

MTChat sends outgoing webhooks for chat events:

```javascript
// POST /webhooks/mtchat
app.post('/webhooks/mtchat', (req, res) => {
  const { event, data } = req.body

  switch (event) {
    case 'message.new':
      // New message sent
      break
    case 'notification.pending':
      // Unread message after delay -- send push notification
      sendPushNotification(data.user_id, data.dialog_title, data.message)
      break
    case 'participant.joined':
    case 'participant.left':
      // Participant change
      break
  }

  res.sendStatus(200)
})
```

## Features

| Feature | Status |
|---------|--------|
| Object-bound dialogs | Done |
| Direct + potential participants | Done |
| Scope-based access control | Done |
| Join/leave chat flow | Done |
| Two chat lists (My Chats / Available) | Done |
| Inline mode | Done |
| Real-time messaging (WebSocket) | Done |
| Rich text editor (Tiptap) | Done |
| Message replies | Done |
| Message editing & deletion | Done |
| File attachments (S3) | Done |
| Unread message tracking | Done |
| Read receipts | Done |
| User online status | Done |
| System messages | Done |
| Chat search | Done |
| Chat archiving (per-user) | Done |
| Chat pinning | Done |
| Per-chat notification toggle | Done |
| Smart notifications (debounced webhooks) | Done |
| Auto-archive inactive chats | Done |
| Infinite scroll + jump to message | Done |
| i18n (Russian, English, Chinese) | Done |
| Light & dark themes | Done |
| Outgoing webhooks | Done |
| Participant profiles | Done |
| Multiple dialogs per object | Done |

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust (axum 0.8, sqlx 0.8, tokio) |
| Frontend SDK | Vue.js 3, TypeScript |
| Database | PostgreSQL 17 |
| Cache / PubSub | Redis 7 |
| File Storage | S3-compatible (MinIO) |
| Job Queue | apalis 0.6 (Redis) |
| Rich Text | Tiptap (ProseMirror) |

## Project Structure

```
mtchat/
├── mtchat-rust/           # Backend API (Rust)
│   ├── src/
│   │   ├── api/           # REST handlers
│   │   ├── domain/        # Data models
│   │   ├── ws/            # WebSocket
│   │   ├── webhooks/      # Outgoing webhooks
│   │   ├── jobs/          # Background jobs (apalis)
│   │   └── services/      # S3, presence
│   └── migrations/        # SQL migrations
├── mtchat-vue/            # Vue.js SDK (@mtchat/vue)
├── mtchat-vue-primevue/   # PrimeVue wrapper (@mtchat/vue-primevue)
├── mtchat-example/        # Demo application
├── deploy/                # Docker Compose + Helm chart
└── .github/workflows/     # CI/CD
```

## API Reference

### Management API

Authenticated with `Authorization: Bearer <admin_token>`. Used by your backend.

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/management/dialogs` | Create dialog |
| GET | `/api/v1/management/dialogs/{id}` | Get dialog |
| DELETE | `/api/v1/management/dialogs/{id}` | Delete dialog |
| POST | `/api/v1/management/dialogs/{id}/participants` | Add participant |
| DELETE | `/api/v1/management/dialogs/{id}/participants/{user_id}` | Remove participant |
| PUT | `/api/v1/management/dialogs/{id}/access-scopes` | Update access scopes |

### Chat API

Authenticated with user token (passed via header). Used by the Vue SDK.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/dialogs?type=participating` | List user's chats |
| GET | `/api/v1/dialogs?type=available` | List joinable chats |
| GET | `/api/v1/dialogs/{id}` | Get dialog details |
| GET | `/api/v1/dialogs/by-object/{type}/{id}` | Get dialog by object |
| POST | `/api/v1/dialogs/{id}/join` | Join dialog |
| POST | `/api/v1/dialogs/{id}/leave` | Leave dialog |
| POST | `/api/v1/dialogs/{id}/archive` | Archive dialog |
| POST | `/api/v1/dialogs/{id}/unarchive` | Unarchive dialog |
| POST | `/api/v1/dialogs/{id}/pin` | Pin dialog |
| POST | `/api/v1/dialogs/{id}/unpin` | Unpin dialog |
| POST | `/api/v1/dialogs/{id}/notifications` | Toggle notifications |
| POST | `/api/v1/dialogs/{id}/read` | Mark as read |
| GET | `/api/v1/dialogs/{id}/participants` | List participants |
| GET | `/api/v1/dialogs/{id}/messages` | List messages |
| POST | `/api/v1/dialogs/{id}/messages` | Send message |
| PUT | `/api/v1/dialogs/{id}/messages/{msg_id}` | Edit message |
| DELETE | `/api/v1/dialogs/{id}/messages/{msg_id}` | Delete message |
| WS | `/api/v1/ws` | WebSocket connection |

### File Upload API

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/upload/presign` | Get presigned upload URL |
| GET | `/api/v1/attachments/{id}/url` | Get download URL |

### Webhook Events

| Event | Description |
|-------|-------------|
| `message.new` | New message sent |
| `participant.joined` | User joined dialog |
| `participant.left` | User left dialog |
| `notification.pending` | Unread message after delay (for push notifications) |

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | -- | PostgreSQL connection string |
| `REDIS_URL` | No | -- | Redis URL (enables presence, jobs) |
| `ADMIN_API_TOKEN` | No | -- | Management API auth token |
| `WEBHOOK_URL` | No | -- | Outgoing webhook endpoint |
| `WEBHOOK_SECRET` | No | -- | Webhook HMAC signing secret |
| `S3_ENDPOINT` | No | -- | S3-compatible endpoint URL |
| `S3_BUCKET` | No | -- | S3 bucket name |
| `S3_ACCESS_KEY` | No | -- | S3 access key |
| `S3_SECRET_KEY` | No | -- | S3 secret key |
| `S3_REGION` | No | `us-east-1` | S3 region |
| `PORT` | No | `8080` | Server listen port |
| `RUST_LOG` | No | `info` | Log level |
| `NOTIFICATION_DELAY_SECS` | No | `30` | Delay before sending notification webhook |
| `ARCHIVE_AFTER_SECS` | No | `259200` | Auto-archive inactive chats (default: 3 days) |

## Scope Matching

MTChat uses a two-level scope system to determine which users can see and join chats:

```
Dialog scope:                    User scope:
{                                {
  tenant_uid: "X",                 tenant_uid: "X",
  scope_level1: ["A", "B"],       scope_level1: ["A"],
  scope_level2: ["mgr", "admin"]  scope_level2: ["mgr", "viewer"]
}                                }

Match: tenant_uid matches
       scope_level1 intersection: ["A"] (not empty)
       scope_level2 intersection: ["mgr"] (not empty)

Result: User CAN JOIN this dialog
```

**Logic:** `same tenant AND (any scope_level1 overlap) AND (any scope_level2 overlap)`

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## Security

See [SECURITY.md](SECURITY.md) for reporting vulnerabilities.

## License

[MIT](LICENSE)
