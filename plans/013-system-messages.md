# Plan 013: System Messages

## Overview

Реализовать системные сообщения для событий чата:
1. **Чат создан** — "Чат создан с участниками: Имя1 (Компания1), Имя2 (Компания2)"
2. **Участник присоединился** — "Имя Участника присоединился к чату"
3. **Участник покинул** — "Имя Участника покинул чат"

**Архитектура:**
- Бэкенд хранит `message_type = 'system'`, `sender_id = NULL`, и структурированные данные в `content` (JSON)
- Фронтенд форматирует текст по локали пользователя (ru/en/zh)
- Системные сообщения **НЕ увеличивают** unread_count

## UI Reference

По скриншоту: системные сообщения отображаются как центрированный серый текст между обычными сообщениями, без аватара и bubble.

---

## Implementation (COMPLETED)

### 1. Database Migration

**File:** `mtchat-rust/migrations/20250206000002_add_system_messages.sql`

- Added `message_type VARCHAR(20) NOT NULL DEFAULT 'user'` column
- Made `sender_id` nullable
- Added check constraints for valid message types and sender_id consistency

### 2. Backend Changes (Rust)

#### Domain Model
- **`message.rs`**: Added `MessageType` enum (User, System), made `sender_id: Option<Uuid>`, added `Message::system()` constructor
- **`system_messages.rs`**: New module with `ParticipantInfo` struct and content generators for chat_created, participant_joined, participant_left
- **`mod.rs`**: Exported `MessageType` and `system_messages` module

#### Repository
- **`message_repo.rs`**: Updated `create()` to include `message_type` in INSERT

#### WebSocket
- **`ws.rs`**: Updated `WsEvent::MessageNew` to include `message_type` and handle nullable `sender_id`

#### API Handlers (main.rs)
- **`management_create_dialog`**: Creates "chat_created" system message with participant list
- **`join_dialog`**: Creates "participant_joined" system message, broadcasts via WebSocket
- **`leave_dialog`**: Creates "participant_left" system message, broadcasts via WebSocket

### 3. Frontend Changes (Vue SDK)

#### Types
- **`types/index.ts`**: Added `MessageType`, `SystemMessageEvent`, `SystemMessageContent`, updated `Message.sender_id` to `string | null`

#### i18n
- **`translations.ts`**: Added `system.chatCreated`, `system.participantJoined`, `system.participantLeft` for ru/en/zh

#### Component
- **`MTChat.vue`**:
  - Added `formatSystemMessage()` function to parse JSON and format based on locale
  - Conditional rendering: system messages use `.mtchat__system-message` class (centered, gray)
  - User messages wrapped in `v-else` to separate rendering
  - CSS styles for system messages

---

## Files Modified

### Backend (mtchat-rust)
1. `migrations/20250206000002_add_system_messages.sql` — NEW
2. `src/domain/message.rs` — MessageType enum, Option<Uuid> sender_id
3. `src/domain/mod.rs` — Export system_messages module
4. `src/domain/system_messages.rs` — NEW
5. `src/repositories/message_repo.rs` — Updated create()
6. `src/ws.rs` — Updated WsEvent::MessageNew
7. `src/main.rs` — System message creation in handlers

### Frontend (mtchat-vue)
1. `src/types/index.ts` — MessageType, SystemMessageContent
2. `src/i18n/translations.ts` — System message translations
3. `src/components/MTChat.vue` — formatSystemMessage, rendering, CSS

---

## Verification

1. **Create dialog via Management API** → Check system message "Чат создан с участниками..."
2. **Join dialog** → Check system message "Имя присоединился к чату"
3. **Leave dialog** → Check system message "Имя покинул чат"
4. **UI** → System messages centered, gray, no avatar
5. **WebSocket** → System messages broadcast to all participants in real-time
6. **i18n** → Switch locale, check translations update
7. **Unread** → System messages don't increment unread_count
8. **Reply** → Reply button hidden for system messages

---

## Notes

- Системные сообщения **НЕ увеличивают** unread_count
- Reply на системное сообщение **запрещён** (кнопка скрыта)
- Системные сообщения **не редактируются**
- Content хранится как JSON для поддержки i18n

---

*Plan implemented 2025-02-06*
