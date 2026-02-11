# MTChat - Project Documentation

## Overview

**MTChat** — встраиваемый чат-сервис для B2B/B2C платформ.

Разработан как микросервис для **TRUCKER TMS**, спроектирован как универсальное open-source решение.

### Ключевая концепция

- Чат **привязан к объекту** (тендер, заказ, рейс и т.д.)
- **Несколько чатов на один объект** — можно создать сколько угодно чатов для одного объекта
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
│  is_archived        │        │                                 │
│  is_pinned          │        │                                 │
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
| Job Queue | apalis 0.6 (Redis backend) |

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
│   │   ├── webhooks/      # Outgoing webhooks
│   │   └── jobs/          # Background job queue (apalis)
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
GET  /api/v1/dialogs?type=participating&archived=true  # Archived chats
GET  /api/v1/dialogs?type=available       # Can join
GET  /api/v1/dialogs/by-object/{type}/{id}  # Inline mode
POST /api/v1/dialogs/{id}/join            # Join chat
POST /api/v1/dialogs/{id}/leave           # Leave chat
POST /api/v1/dialogs/{id}/read            # Mark messages as read
POST /api/v1/dialogs/{id}/archive         # Archive chat for current user
POST /api/v1/dialogs/{id}/unarchive       # Unarchive chat for current user
POST /api/v1/dialogs/{id}/pin             # Pin chat for current user
POST /api/v1/dialogs/{id}/unpin           # Unpin chat for current user
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
Events: message.new, participant.joined, participant.left, notification.pending
```

**notification.pending** - Sent after delay (default 30s) if message was not read by recipient. Supports debouncing: multiple messages to same recipient trigger only one notification.

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
  },
  // UI locale (optional, default: 'ru')
  locale: 'ru',  // 'ru' | 'en' | 'zh'
}
```

### Internationalization (i18n)

MTChat supports three languages:
- **Russian** (`ru`) - default
- **English** (`en`)
- **Chinese** (`zh`)

Set the locale via the config:

```typescript
// Russian (default)
<MTChat :config="{ ...config, locale: 'ru' }" />

// English
<MTChat :config="{ ...config, locale: 'en' }" />

// Chinese
<MTChat :config="{ ...config, locale: 'zh' }" />
```

All UI strings are translated including:
- Tab labels (My Chats, Available)
- Buttons (Join, Send, Cancel, Leave Chat)
- Status indicators (Connected, Disconnected)
- Empty states and placeholders
- Date formatting (Today, Yesterday, locale-aware dates)
- File viewer controls and file type labels
- Join dialog labels
- Chat info panel

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
| i18n (ru/en/zh) | ✅ |
| Potential chat access control | ✅ |
| Dialog search | ✅ |
| Chat archiving (per-user) | ✅ |
| System messages | ✅ |
| Message formatting (Tiptap) | ✅ |
| User online status | ✅ |
| Message editing & deletion | ✅ |
| Chat pinning | ✅ |
| Per-chat notification toggle | ✅ |
| Multiple dialogs per object | ✅ |
| Read receipts | ✅ |
| Background job queue (apalis) | ✅ |
| Smart notifications (debounce) | ✅ |
| Auto-archive inactive chats | ✅ |

## Changelog

### 2026-02-11 (v3.18) - Background Job Queue
- **apalis 0.6** integration for background task processing with Redis backend
- **Smart notifications** with 30s delay and debounce (configurable via env vars)
- Debounce logic: multiple messages to same recipient trigger only one notification
- Notification skipped if message already read before delay expires
- Notification skipped if user has notifications disabled for the chat
- **Auto-archive job** runs on cron schedule (default: every 5 mins)
- Archives all participants of dialogs inactive for N days (default: 7)
- `notification.pending` webhook event for unread message notifications
- New `jobs/` module: types, handlers, producer, worker, middleware
- `JobProducer` integrated into AppState for enqueueing from handlers
- `find_inactive_since()` in DialogRepository for auto-archive queries
- `archive_all_for_dialog()` in ParticipantRepository for batch archiving
- Environment variables: `NOTIFICATION_DELAY_SECS`, `ARCHIVE_CRON`, `ARCHIVE_AFTER_DAYS`, `NOTIFICATION_CONCURRENCY`
- Graceful degradation: job queue disabled if Redis not configured
- Unit tests for job types, handlers, worker config, debounce logic

### 2026-02-10 (v3.17) - Read Receipts & Reactivity Fixes
- Checkmark indicator on own messages when at least 1 participant has read
- PrimeVue tooltip on hover showing who read (max 2 names + "and X more")
- Click checkmark opens ReadersDialog modal with full list of readers
- Reader detection based on participant's last_read_message_id vs message sent_at
- Computed `messageReadersMap` for proper Vue reactivity tracking
- WebSocket message.read event updates participant's last_read_message_id in real-time
- ReadersDialog component with scrollable list (company — name format)
- i18n translations for read receipts (ru/en/zh)
- **WebSocket events for participant.joined/left** - enables real-time dialog list updates
- Backend broadcasts participant events on join/leave (Chat API and Management API)
- Frontend reloads dialog lists when current user joins/leaves a dialog
- Immutable state updates throughout useChat.ts for proper Vue reactivity
- Fixed: joinDialog, leaveDialog, archiveDialog, pinDialog, toggleNotifications

### 2026-02-10 - Header Menu Customization
- Custom action slot in header dropdown menu (before "Leave chat")
- `headerMenuAction` prop for action label
- `header-menu-action-icon` slot for custom icon
- `header-menu-action` emit event on click
- Removed delete message action from context menu (edit only)

### 2026-02-10 - PDF Viewer UX Improvements
- Fixed zoom/pan issues for multi-page PDFs
- Improved scroll and navigation behavior

### 2026-02-10 - Editor UX Improvements
- Swap Enter/Shift+Enter behavior: Enter sends, Shift+Enter adds newline
- Auto-focus link input when link dialog opens
- Responsive formatting toolbar with overflow dropdown
- Container queries for proper toolbar layout

### 2026-02-10 - Dialog Features
- Search dialogs by participant company names
- Add `object_url` field for dialog links to host system
- Create chat button with placeholder dialog slot

### 2026-02-10 (v3.16) - Multiple Dialogs Per Object
- Removed uniqueness constraint on (object_id, object_type)
- Multiple chats can now be created for the same business object
- API `by-object/{type}/{id}` returns the most recent dialog (backward compatible)
- Database migration: drops unique index, creates regular index for query performance

### 2026-02-07 (v3.15) - Per-Chat Notification Toggle
- Mute/unmute notifications for individual chats
- Toggle via header menu (⋮) or context menu (right-click)
- Bell-off icon displayed in chat list for muted chats
- API endpoint: POST /dialogs/{id}/notifications with { enabled: bool }
- `notifications_enabled` field in dialog list API response
- i18n translations for mute/unmute (ru/en/zh)

### 2026-02-07 (v3.14) - Chat Pinning & Sorting
- Pin/unpin chats via header menu (⋮) or context menu (right-click)
- Pinned chats displayed at top of list with pin icon
- Sorting: pinned first, then by last_message_at (newest first)
- Added `last_message_at` field to dialog list API response (batch query)
- Real-time update of dialog position when new message sent/received
- Context menu on dialog items in sidebar (right-click)
- Lazy loading of archived dialogs (only on accordion open)
- Removed archived count display from accordion
- Database migration: `is_pinned` column in dialog_participants
- API endpoints: POST /dialogs/{id}/pin, POST /dialogs/{id}/unpin
- i18n translations for pin/unpin (ru/en/zh)
- Esc closes context menu

### 2026-02-06 (v3.13) - Message Editing & Deletion
- Edit and delete messages via dropdown menu (⋮)
- Dropdown menu with Reply, Edit, Delete actions
- Edit/Delete only available for own messages
- "(ред.)" / "(edited)" badge on edited messages
- Arrow Up keyboard shortcut to edit last own message when editor is empty
- Edit mode preview above input (similar to reply preview)
- Edit history saved to message_edit_history table in DB
- WebSocket events: message.edited, message.deleted for real-time sync
- Backend validation: only author can edit/delete, system messages protected
- HTML sanitization preserved on edit
- i18n translations for ru/en/zh
- Esc key cancels edit mode
- Click outside closes dropdown menu

### 2026-02-06 (v3.12) - User Online Status
- Real-time online status tracking via Redis with 60s TTL
- Message avatars: circular avatar with initials (36x36px) next to each message
- Green online indicator on message avatars (bottom-right, 10x10px)
- Green indicator dot on participant avatars in chat info panel
- Presence service with set_online/refresh_online/set_offline operations
- WebSocket presence.update events for real-time status changes
- Batch MGET for efficient online status checking
- Heartbeat-based TTL refresh (30s ping refreshes 60s TTL)
- Graceful degradation when Redis is not configured
- is_online field in participants API response
- Frontend onlineUsers Set with reactive updates
- isUserOnline() helper function in useChat composable
- getInitials() helper for generating avatar letters

### 2025-02-06 (v3.11) - Message Formatting
- Rich text editor based on Tiptap (ProseMirror)
- Formatting toolbar: bold, italic, underline, strikethrough, link, lists, quote, code
- Active formatting buttons highlighted with light gray background
- Keyboard shortcuts: Cmd+B (bold), Cmd+I (italic), Cmd+U (underline), Cmd+K (link), etc.
- Cmd+Enter to send message (regular Enter for newlines/lists/quotes)
- Markdown shortcuts: `**bold**`, `*italic*`, `> quote`, `- list`, etc.
- @mentions with participant suggestions dropdown
- Link dialog (Cmd+K) for inserting/editing links
- Compact attach/send buttons in editor footer
- Send button: disabled state (gray icon), enabled state (blue button)
- Auto-resize editor (up to 25vh)
- Backend HTML sanitization with ammonia crate
- Allowed HTML tags: p, br, strong, em, u, s, a, ul, ol, li, blockquote, code, pre, span
- XSS prevention: removes scripts, event handlers, javascript: URLs
- HTML content rendering in messages with proper styling
- i18n support for formatting toolbar tooltips and placeholder (ru/en/zh)

### 2025-02-06 (v3.10) - System Messages
- System messages for chat events (created, joined, left)
- Message types: 'user' (default) and 'system'
- System messages have no sender_id (NULL in DB)
- JSON content format for i18n support on frontend
- Backend creates system messages on: dialog creation, join, leave
- Frontend formats system messages based on locale (ru/en/zh)
- System messages displayed centered with gray text
- System messages don't increment unread_count
- Reply button hidden for system messages
- Database migration: `message_type` column, nullable `sender_id`
- WebSocket broadcasts system messages with message_type field
- Translations for all system message types (ru/en/zh)

### 2025-02-06 (v3.9) - Chat Archiving
- Per-user chat archiving (each participant archives independently)
- Archived chats shown in collapsible accordion below active chats
- Accordion toggles between 50% height and collapsed states
- Archive/Unarchive action in chat header menu
- "Archived" badge in chat header for archived chats
- Separate scroll areas for active and archived chat lists
- Search works across both active and archived chats
- Unread counter tracking for archived chats
- Database migration: `is_archived` column in dialog_participants
- API endpoints: POST /dialogs/{id}/archive, POST /dialogs/{id}/unarchive
- Translations for archive UI elements (ru/en/zh)

### 2025-02-05 (v3.8) - Dialog Search
- Search input in sidebar above dialog tabs
- Backend search by dialog title using ILIKE (case-insensitive)
- 300ms debounce to prevent excessive API calls
- Cmd+K (Mac) / Ctrl+K (Windows/Linux) hotkey to focus search
- Esc key clears search input
- Clear button (X) appears when search has text
- "No results found" message when search returns empty
- Search works across both "My Chats" and "Available" tabs
- Translations for search placeholder and empty state (ru/en/zh)

### 2025-02-05 (v3.7) - Potential Chat Access Control
- Security fix: potential participants can no longer read messages before joining
- Potential participants see "Join to view messages" prompt with join button
- Backend validation on all message-related endpoints (403 Forbidden for non-participants)
- Protected endpoints: GET/POST messages, GET participants, GET dialog (with scope check)
- Frontend guards prevent message loading and sending for non-participants
- Proper handling of 403 errors in useChat composable
- Messages and participants load automatically after successful join
- Updated translations for join-required state (ru/en/zh)

### 2025-02-05 (v3.6) - Internationalization (i18n)
- Full i18n support with Russian (default), English, and Chinese translations
- ~65 strings translated across all components
- Lightweight implementation using Vue provide/inject (no external dependencies)
- Reactive language switching without component remount (chat state preserved)
- Template interpolation for dynamic values (e.g., "{count} participants")
- Locale-aware date formatting via Intl.DateTimeFormat
- Language selector in demo app sidebar
- Locale persistence via localStorage
- Removed redundant "can join" badge from available dialogs list
- Removed unused ImageGallery and PDFViewer components (FileViewer handles all)

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
