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
│              PostgreSQL + Redis + MinIO (S3)                     │
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
│  display_name       │        │  scope_level1[]  (departments)  │
│  company            │        │  scope_level2[]  (permissions)  │
│  email              │        │                                 │
│  phone              │        │                                 │
│  joined_at          │        │                                 │
│  joined_as          │        │                                 │
│  notifications      │        │                                 │
│  last_read_msg_id   │        │                                 │
│  unread_count       │        │                                 │
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
GET  /api/v1/dialogs?type=participating   # My chats (includes unread_count)
GET  /api/v1/dialogs?type=available       # Can join
GET  /api/v1/dialogs/by-object/{type}/{id}  # Inline mode
POST /api/v1/dialogs/{id}/join            # Join chat
POST /api/v1/dialogs/{id}/leave           # Leave chat
POST /api/v1/dialogs/{id}/read            # Mark messages as read
GET  /api/v1/dialogs/{id}/messages        # Get messages (includes first_unread_message_id)
POST /api/v1/dialogs/{id}/messages        # Send message
WS   /api/v1/ws                           # Real-time (message.new, message.read)
```

### File Upload API

```
POST /api/v1/upload/presign         # Get presigned S3 upload URL
GET  /api/v1/attachments/{id}/url   # Get presigned download URL
```

**Upload Flow:**
1. Client calls `/upload/presign` with `{ dialog_id, filename, content_type, size }`
2. Server returns `{ upload_url, s3_key, expires_in }`
3. Client uploads file directly to S3 via `PUT upload_url`
4. Client sends message with `attachments: [{ s3_key, filename, content_type, size }]`

**Supported File Types:**
- Images: jpeg, png, gif, webp, svg, bmp, tiff
- Documents: pdf, doc/docx, xls/xlsx, ppt/pptx, odt/ods/odp, rtf
- Text: txt, csv, markdown, html, xml, json
- Archives: zip, rar, 7z, gzip, tar
- Audio: mp3, wav, ogg, m4a
- Video: mp4, webm, ogg, mov

**Limits:**
- Max file size: 100MB
- Max attachments per message: 10

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

### Theme Support

```vue
<!-- Light theme -->
<MTChat :config="config" theme="light" />

<!-- Dark theme -->
<MTChat :config="config" theme="dark" />
```

**CSS Variables (MTChat):**
- `--mtchat-bg`, `--mtchat-bg-secondary`, `--mtchat-bg-hover`
- `--mtchat-text`, `--mtchat-text-secondary`
- `--mtchat-border`, `--mtchat-primary`

**CSS Variables (Demo Layout):**
- `--tms-bg`, `--tms-bg-sidebar`, `--tms-bg-panel`, `--tms-bg-hover`
- `--tms-text`, `--tms-text-secondary`, `--tms-text-muted`
- `--tms-border`, `--tms-primary`, `--tms-primary-bg`

### Config

```typescript
const config: MTChatConfig = {
  baseUrl: 'https://chat.example.com',
  token: userToken,
  userId: user.id,
  // Scope config for access matching
  scopeConfig: {
    tenant_uid: user.tenant_id,
    scope_level1: user.departments,
    scope_level2: user.permissions,
  },
  // User profile for join dialog
  userProfile: {
    displayName: user.name,
    company: user.company,
    email: user.email,      // optional
    phone: user.phone,      // optional
  }
}
```

### Participant Profiles

When joining a chat, users can customize their visible profile:

- **Display name**: Real name or anonymous ("Сотрудник компании X")
- **Company**: Always shown (from userProfile)
- **Contacts**: Email/phone can be toggled on/off

Profile data is stored per-participant and shown in:
- Chat info panel (participant list with contacts)
- Message headers (display_name)

```typescript
// Join dialog request
interface JoinDialogRequest {
  display_name: string
  company: string
  email?: string
  phone?: string
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
| File attachments | ✅ |
| Unified FileViewer (images/PDF) | ✅ |
| Message replies | ✅ |
| Unread message tracking | ✅ |
| Date dividers (sticky) | ✅ |
| List-style message layout | ✅ |
| PrimeVue theme support | ✅ |
| Participant profiles | ✅ |
| Join dialog with profile selection | ✅ |
| Chat info panel | ✅ |

## Changelog

### 2025-02-05 (v3.5) - Participant Profiles & Join UX
- Participant profile support (display_name, company, email, phone)
- Join dialog with name selection (real name or anonymous)
- Contact visibility toggles (email/phone) when joining
- Chat info panel showing all participants with contacts
- Auto-switch to "My Chats" tab after joining
- Auto-reload available dialogs after leaving (stay on "My Chats")
- Keyboard shortcuts: Esc to close info panel and join dialog
- Dark theme support for join dialog
- Compact, clean join dialog design
- Demo app: email/phone fields in user management
- Demo app: improved text contrast in light theme

### 2025-02-05 (v3.4) - UI Improvements & PrimeVue Themes
- List-style message layout (left-aligned, full width, no bubbles)
- Date dividers between messages with smart formatting (Сегодня/Вчера/дата)
- Sticky date headers that appear when scrolling past dividers
- Message hover highlighting on full row
- Full light/dark theme support based on PrimeVue Lara Blue scheme
- Theme toggle button in demo app sidebar
- CSS variables system for easy theming (--tms-*, --mtchat-*)
- Theme persistence via localStorage

### 2025-02-04 (v3.3) - Message Replies & Unread Tracking
- Message reply functionality with reply-to icon on hover (outside message bubble)
- Quote preview above input field with cancel (X/Esc)
- Quoted message display in sent messages with click-to-scroll
- Scroll-to-bottom button (fixed position, appears when scrolled up)
- Unread message counter per dialog (stored in dialog_participants.unread_count)
- "Новые сообщения" divider between read and unread messages
- Mark as read API endpoint: POST /dialogs/{id}/read
- Auto-mark-as-read when scrolled to bottom (1 second delay)
- WebSocket message.read event for real-time updates across devices
- Unread badge in dialog list UI (99+ cap)

### 2025-02-04 (v3.2) - File Attachments
- Full file attachment support with S3/MinIO storage
- Presigned URL upload flow (secure, direct-to-S3)
- Unified FileViewer component for images and PDFs
- PDF.js integration with zoom, pan, and multi-page support
- Image gallery with keyboard navigation (arrows, Esc)
- Native macOS trackpad gestures support (two-finger pan)
- Upload progress tracking with retry on failure
- Expanded file type support (40+ MIME types)
- Cross-origin file download support
- Inline mode layout fixes (proper 100vh containment)

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
