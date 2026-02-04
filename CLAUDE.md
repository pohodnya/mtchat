# MTChat - Project Documentation

## Overview

**MTChat** — встраиваемый чат-сервис для B2B/B2C платформ.

Разработан как микросервис для **TRUCKER TMS**, спроектирован как универсальное open-source решение.

### Ключевая концепция

- Чат **обязательно привязан к объекту** (тендер, заказ, рейс и т.д.)
- **Прямые участники** — получают уведомления, видят чат в "Участвую"
- **Потенциальные участники** — могут присоединиться, видят в "Доступные"
- Бизнес-логика формирования чатов остаётся в вашем приложении

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Host Application (TMS)                        │
│  ┌─────────────┐                 ┌─────────────────────┐        │
│  │  Frontend   │                 │  Backend            │        │
│  │ ┌─────────┐ │                 │  - Создание чатов   │        │
│  │ │ MTChat  │ │                 │  - Участники        │        │
│  │ │ Vue SDK │ │                 │  - Webhooks handler │        │
│  │ └────┬────┘ │                 └──────────┬──────────┘        │
└────────┼───────┘                            │                   │
         │                                    │
         │ Chat API                           │ Management API
         │ (User Token)                       │ (Admin Token)
         ▼                                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                      MTChat Backend                              │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────┐       │
│  │   Chat API     │  │ Management API │  │  Webhooks    │       │
│  │ (users)        │  │ (system)       │  │  (outgoing)  │       │
│  └────────────────┘  └────────────────┘  └──────────────┘       │
│                              │                                   │
│                    PostgreSQL + Redis                            │
└─────────────────────────────────────────────────────────────────┘
```

## Data Model

```
┌─────────────────────────────────────────────────────────────────┐
│                           Dialog                                 │
├─────────────────────────────────────────────────────────────────┤
│  id              UUID                                            │
│  object_id       UUID        ← привязка к объекту (required)    │
│  object_type     STRING      "tender", "order", "route"         │
│  title           STRING                                          │
│  created_by      UUID                                            │
│  created_at      TIMESTAMP                                       │
└─────────────────────────────────────────────────────────────────┘
         │
         ├──────────────────────────────────┐
         │                                  │
         ▼                                  ▼
┌─────────────────────┐        ┌─────────────────────────────────┐
│    Participants     │        │       Access Scopes             │
│  (прямые участники) │        │   (потенциальные участники)     │
├─────────────────────┤        ├─────────────────────────────────┤
│  dialog_id          │        │  dialog_id                      │
│  user_id            │        │  tenant_uid                     │
│  joined_at          │        │  scope_level1[]  (departments)  │
│  notifications      │        │  scope_level2[]  (permissions)  │
│  last_read_msg      │        │                                 │
└─────────────────────┘        └─────────────────────────────────┘
```

## Scope Matching Logic

```
Dialog scope:                    User scope:
{                                {
  tenant_uid: "X",                 tenant_uid: "X",
  scope_level1: ["A", "B"],        scope_level1: ["A"],
  scope_level2: ["mgr", "admin"]   scope_level2: ["mgr", "viewer"]
}                                }

Match:
  ✓ tenant_uid == tenant_uid
  ✓ scope_level1 ∩ ["A", "B"] ≠ ∅  →  "A" matches
  ✓ scope_level2 ∩ ["mgr", "admin"] ≠ ∅  →  "mgr" matches

Result: User is POTENTIAL participant (can join)
```

**Logic**: `tenant AND (ANY scope1) AND (ANY scope2)`

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust (axum 0.8, sqlx 0.8, tokio) |
| SDK | TypeScript + Vue.js 3 |
| Database | PostgreSQL 17 |
| Cache/PubSub | Redis 7 |
| Storage | MinIO (S3) |

## Project Structure

```
mtchat/
├── docker-compose.yml
├── README.md              # User documentation
├── CLAUDE.md              # Dev documentation (this file)
├── plans/                 # Implementation plans
│
├── mtchat-rust/           # Backend API
│   ├── src/
│   │   ├── main.rs
│   │   ├── api/           # REST handlers
│   │   ├── ws/            # WebSocket
│   │   └── webhooks/      # Outgoing webhooks
│   └── migrations/
│
├── mtchat-vue/            # SDK Library
│   └── src/
│       ├── sdk/
│       ├── composables/
│       └── components/
│
└── mtchat-example/        # Demo app
```

## API Overview

### Management API (Admin Token)

```
POST /api/v1/management/dialogs           # Create dialog
POST /api/v1/management/dialogs/{id}/participants  # Add participant
DELETE /api/v1/management/dialogs/{id}    # Delete dialog
```

### Chat API (User Token)

```
GET  /api/v1/dialogs?type=participating   # My chats
GET  /api/v1/dialogs?type=available       # Can join
GET  /api/v1/dialogs/by-object/{type}/{id}  # Inline mode
POST /api/v1/dialogs/{id}/join            # Join chat
POST /api/v1/dialogs/{id}/leave           # Leave chat
GET  /api/v1/dialogs/{id}/messages        # Get messages
POST /api/v1/dialogs/{id}/messages        # Send message
WS   /api/v1/ws                           # Real-time
```

### Outgoing Webhooks

```
POST {configured_url}
Events: message.new, participant.joined, participant.left
```

## Vue Component

### Full Mode (chat list)

```vue
<MTChat :config="config" mode="full" />
```

### Inline Mode (single chat)

```vue
<MTChat
  :config="config"
  mode="inline"
  :object-id="tender.id"
  object-type="tender"
/>
```

### Config

```typescript
const config = {
  baseUrl: 'https://chat.example.com',
  token: userToken,
  scopeConfig: {
    tenant_uid: user.tenant_id,
    scope_level1: user.departments,
    scope_level2: user.permissions,
  }
}
```

## Quick Start

```bash
docker-compose up -d

# App: http://localhost
# API: http://localhost:8080
```

## Database Tables

| Table | Description |
|-------|-------------|
| dialogs | Чаты, привязанные к объектам (object_id, object_type) |
| dialog_participants | Прямые участники (user_id — внешний идентификатор) |
| dialog_access_scopes | Правила для потенциальных участников (scope matching) |
| messages | Сообщения с поддержкой reply_to_id |
| attachments | Вложения к сообщениям |
| message_edit_history | История редактирования сообщений |

**Удалённые таблицы (v3):** `tenants`, `employees` — идентификация пользователей теперь через внешние ID (JWT).

## Feature Status

| Feature | Status |
|---------|--------|
| Object-bound dialogs | ✅ |
| Direct participants | ✅ |
| Potential participants (scopes) | ✅ |
| Join/Leave chat | ✅ |
| Two chat lists (My/Available) | ✅ |
| Inline mode (backend) | ✅ |
| Legacy tables removed | ✅ |
| Migration tests (13) | ✅ |
| Outgoing webhooks | ✅ |
| Vue SDK updates | ✅ |
| Message sending | ✅ |
| WebSocket real-time | ✅ |
| Basic UI component | ✅ |
| Demo App (Dev Playground) | ✅ |
| File attachments | 🔲 |

## Changelog

### 2025-02-04 (v3.1) - Demo App Complete
- Dev Playground (mtchat-example) fully implemented
- Admin panel: Tenants, Users, Objects, Dialogs, Settings tabs
- Full mode: chat list with "My Chats" / "Available" tabs
- Inline mode: TMS-style layout with data table + embedded chat
- TMS-style dark theme with sidebar navigation
- localStorage-based demo data management

### 2025-02-03 (v3) - Architecture Update
- New data model: object-bound dialogs
- Direct participants + potential participants (scopes)
- Two-level scope matching (departments + permissions)
- Join/leave flow
- Outgoing webhooks for notifications
- Inline mode for embedding

### 2025-02-02 (v2)
- Project restructure (mtchat-rust, mtchat-vue, mtchat-example)
- TypeScript SDK

### 2025-02-02 (v1)
- Initial implementation
