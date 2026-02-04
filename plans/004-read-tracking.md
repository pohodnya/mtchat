# 004: Отслеживание прочтения сообщений

## Цели

1. Пользователь видит количество непрочитанных сообщений в каждом чате (где он участник)
2. При входе в чат — разделитель "Новые сообщения ↓" между прочитанными и новыми
3. Счетчик работает очень быстро

---

## Архитектура

### Хранение в `dialog_participants`

```sql
ALTER TABLE dialog_participants
ADD COLUMN unread_count INTEGER NOT NULL DEFAULT 0;
```

**Почему не Redis/вычисление:**
- SELECT без JOIN — максимальная скорость (~1ms)
- Не нужна внешняя зависимость
- Атомарные операции в PostgreSQL

### Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     Unread Count Flow                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  New Message                                                     │
│       │                                                          │
│       ▼                                                          │
│  UPDATE dialog_participants                                      │
│  SET unread_count = unread_count + 1                            │
│  WHERE dialog_id = $1 AND user_id != $author_id                 │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Mark as Read                                                    │
│       │                                                          │
│       ▼                                                          │
│  UPDATE dialog_participants                                      │
│  SET unread_count = 0, last_read_message_id = $msg_id           │
│  WHERE dialog_id = $1 AND user_id = $2                          │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  GET /dialogs (list)                                            │
│       │                                                          │
│       ▼                                                          │
│  SELECT d.*, dp.unread_count FROM dialogs d                     │
│  JOIN dialog_participants dp ON ...                              │
│  → Мгновенный результат, без COUNT                              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Текущее состояние

### Уже реализовано

| Компонент | Статус | Файл |
|-----------|--------|------|
| `dialog_participants.last_read_message_id` | ✅ | migrations/20240101000004 |
| `participant_repo.update_last_read()` | ✅ | repositories/participant_repo.rs |
| TypeScript типы | ✅ | types/index.ts |

### Не реализовано

| Компонент | Статус |
|-----------|--------|
| Колонка `unread_count` в participants | ❌ |
| Инкремент при новом сообщении | ❌ |
| API endpoint для mark as read | ❌ |
| Популяция unread_count в list_dialogs | ❌ |
| WebSocket событие `message.read` | ❌ |
| Фронтенд: UI счетчик | ❌ |
| Фронтенд: UI разделитель | ❌ |

---

## Этап 1: Миграция БД

**Файл:** `mtchat-rust/migrations/20250204000001_add_unread_count.sql`

```sql
-- Add unread_count column
ALTER TABLE dialog_participants
ADD COLUMN unread_count INTEGER NOT NULL DEFAULT 0;

-- Index for fast lookups
CREATE INDEX idx_participants_unread
ON dialog_participants(user_id, dialog_id)
WHERE unread_count > 0;
```

---

## Этап 2: Backend - Repository

### 2.1 Инкремент при новом сообщении

**Файл:** `mtchat-rust/src/repositories/participant_repo.rs`

```rust
/// Increment unread_count for all participants except the author
pub async fn increment_unread(
    &self,
    dialog_id: Uuid,
    exclude_user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE dialog_participants
        SET unread_count = unread_count + 1
        WHERE dialog_id = $1 AND user_id != $2
        "#,
        dialog_id,
        exclude_user_id
    )
    .execute(&*self.pool)
    .await?;
    Ok(())
}
```

### 2.2 Сброс при прочтении

**Файл:** `mtchat-rust/src/repositories/participant_repo.rs`

```rust
/// Reset unread_count and update last_read_message_id
pub async fn mark_as_read(
    &self,
    dialog_id: Uuid,
    user_id: Uuid,
    last_read_message_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE dialog_participants
        SET unread_count = 0, last_read_message_id = $3
        WHERE dialog_id = $1 AND user_id = $2
        "#,
        dialog_id,
        user_id,
        last_read_message_id
    )
    .execute(&*self.pool)
    .await?;
    Ok(())
}
```

---

## Этап 3: Backend API

### 3.1 Новый endpoint: Mark as read

**Файл:** `mtchat-rust/src/main.rs`

```
POST /api/v1/dialogs/{dialog_id}/read
Body: { "last_read_message_id": "uuid" }
Response: { "success": true }
```

**Логика:**
1. Проверить что пользователь — участник диалога
2. Вызвать `participant_repo.mark_as_read()`
3. Опционально: broadcast WebSocket событие

### 3.2 Модификация send_message

**Файл:** `mtchat-rust/src/main.rs` (функция `send_message`)

**Добавить после сохранения сообщения:**
```rust
// Increment unread for all participants except author
participant_repo.increment_unread(dialog_id, author_user_id).await?;
```

### 3.3 Популяция unread_count в list_dialogs

**Файл:** `mtchat-rust/src/main.rs` (функция `list_dialogs`)

Изменить SQL запрос чтобы включал `dp.unread_count`:
```sql
SELECT
    d.*,
    dp.unread_count,
    dp.last_read_message_id
FROM dialogs d
JOIN dialog_participants dp ON dp.dialog_id = d.id
WHERE dp.user_id = $1
```

### 3.4 Добавить first_unread_message_id в get_messages

**Файл:** `mtchat-rust/src/main.rs` (функция `get_messages`)

```rust
// После получения сообщений
let participant = participant_repo.get(dialog_id, user_id).await?;
let first_unread_message_id = if let Some(last_read_id) = participant.last_read_message_id {
    // Найти первое сообщение после last_read_id
    messages.iter()
        .find(|m| m.id > last_read_id)
        .map(|m| m.id)
} else if !messages.is_empty() {
    // Если никогда не читал — первое сообщение
    Some(messages[0].id)
} else {
    None
};
```

---

## Этап 4: WebSocket событие (опционально)

**Файл:** `mtchat-rust/src/ws.rs`

```rust
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum WsEvent {
    // ... existing
    MessageRead {
        dialog_id: Uuid,
        user_id: Uuid,
        last_read_message_id: Uuid,
    },
}
```

**Использование:** Real-time обновление счетчиков без refresh.

---

## Этап 5: Frontend SDK

### 5.1 API метод

**Файл:** `mtchat-vue/src/sdk/api.ts`

```typescript
async markAsRead(dialogId: string, lastReadMessageId: string): Promise<void> {
  await this.request(`/dialogs/${dialogId}/read`, {
    method: 'POST',
    body: JSON.stringify({ last_read_message_id: lastReadMessageId })
  })
}
```

### 5.2 Типы

**Файл:** `mtchat-vue/src/types/index.ts`

```typescript
export interface DialogListItem {
  // ... existing fields
  unread_count: number
}

export interface MessagesResponse {
  messages: Message[]
  first_unread_message_id?: string
}
```

### 5.3 Composable

**Файл:** `mtchat-vue/src/composables/useChat.ts`

```typescript
const firstUnreadMessageId = ref<string | null>(null)

// При загрузке сообщений
async function loadMessages(dialogId: string) {
  const response = await api.getMessages(dialogId)
  messages.value = response.messages
  firstUnreadMessageId.value = response.first_unread_message_id ?? null
}

// Отметка прочтения
async function markAsRead(dialogId: string) {
  if (messages.value.length === 0) return

  const lastMessage = messages.value[messages.value.length - 1]
  await api.markAsRead(dialogId, lastMessage.id)

  firstUnreadMessageId.value = null

  // Обновить счетчик в списке
  const dialog = dialogs.value.find(d => d.id === dialogId)
  if (dialog) dialog.unread_count = 0
}
```

---

## Этап 6: Frontend UI

### 6.1 Счетчик в списке чатов

**Файл:** `mtchat-vue/src/components/DialogList.vue`

```vue
<template>
  <div class="dialog-item">
    <span class="title">{{ dialog.title }}</span>
    <span v-if="dialog.unread_count > 0" class="unread-badge">
      {{ dialog.unread_count > 99 ? '99+' : dialog.unread_count }}
    </span>
  </div>
</template>

<style scoped>
.unread-badge {
  background: #007AFF;
  color: white;
  border-radius: 10px;
  padding: 2px 8px;
  font-size: 12px;
  font-weight: 600;
  min-width: 20px;
  text-align: center;
}
</style>
```

### 6.2 Разделитель "Новые сообщения"

**Файл:** `mtchat-vue/src/components/MessageList.vue`

```vue
<template>
  <div class="messages">
    <template v-for="message in messages" :key="message.id">
      <!-- Разделитель перед первым непрочитанным -->
      <div
        v-if="message.id === firstUnreadMessageId"
        class="unread-divider"
      >
        <span>Новые сообщения ↓</span>
      </div>

      <MessageBubble :message="message" />
    </template>
  </div>
</template>

<style scoped>
.unread-divider {
  display: flex;
  align-items: center;
  margin: 16px 0;
  gap: 12px;
}

.unread-divider::before,
.unread-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: rgba(0, 122, 255, 0.3);
}

.unread-divider span {
  color: #007AFF;
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
}
</style>
```

### 6.3 Автоматическая отметка прочтения

**Логика:**
1. При открытии чата — НЕ отмечать сразу
2. IntersectionObserver на последнем сообщении
3. Когда последнее сообщение видно 1 секунду — отметить
4. При получении нового сообщения через WS — инкремент счетчика

```typescript
// В компоненте ChatView
const lastMessageRef = ref<HTMLElement | null>(null)
let readTimeout: ReturnType<typeof setTimeout> | null = null

function setupReadObserver() {
  const observer = new IntersectionObserver((entries) => {
    const [entry] = entries
    if (entry.isIntersecting && firstUnreadMessageId.value) {
      readTimeout = setTimeout(() => {
        markAsRead(currentDialogId.value)
      }, 1000)
    } else if (readTimeout) {
      clearTimeout(readTimeout)
      readTimeout = null
    }
  }, { threshold: 0.5 })

  watch(lastMessageRef, (el) => {
    if (el) observer.observe(el)
  }, { immediate: true })

  onUnmounted(() => observer.disconnect())
}
```

---

## Порядок реализации

| # | Задача | Приоритет |
|---|--------|-----------|
| 1 | Миграция: добавить `unread_count` | High |
| 2 | Backend: `increment_unread()` в repo | High |
| 3 | Backend: `mark_as_read()` в repo | High |
| 4 | Backend: инкремент в `send_message` | High |
| 5 | Backend: endpoint `POST /dialogs/{id}/read` | High |
| 6 | Backend: популяция `unread_count` в `list_dialogs` | High |
| 7 | Backend: `first_unread_message_id` в `get_messages` | High |
| 8 | Frontend: API метод `markAsRead()` | High |
| 9 | Frontend: UI счетчик в списке | High |
| 10 | Frontend: UI разделитель | High |
| 11 | Frontend: авто-отметка (IntersectionObserver) | High |
| 12 | Backend: WebSocket событие `message.read` | Medium |

---

## Производительность

| Операция | Время |
|----------|-------|
| GET /dialogs (с unread_count) | ~2ms |
| POST /dialogs/{id}/read | ~3ms |
| Инкремент при send_message | ~2ms |

**Сравнение с Redis-подходом:**
- PostgreSQL atomic increment так же быстр как Redis INCR
- Нет сетевого оверхеда на Redis
- Проще отладка и мониторинг

---

## Edge Cases

### Пользователь присоединяется к существующему чату

При `POST /dialogs/{id}/join`:
```sql
INSERT INTO dialog_participants (dialog_id, user_id, unread_count)
VALUES ($1, $2, (SELECT COUNT(*) FROM messages WHERE dialog_id = $1))
```

### Удаление сообщения

При удалении нужно декрементить счетчики. Но:
- Soft delete не требует изменений (сообщение остается)
- Hard delete редок — можно игнорировать или пересчитывать

### Race conditions

PostgreSQL гарантирует атомарность `unread_count = unread_count + 1`.

---

## API Changes Summary

### New Endpoint

```
POST /api/v1/dialogs/{dialog_id}/read
  Body: { "last_read_message_id": "uuid" }
  Response: { "success": true }
```

### Modified Responses

```typescript
// GET /api/v1/dialogs
interface DialogListItem {
  // ... existing fields
  unread_count: number  // NEW
}

// GET /api/v1/dialogs/{id}/messages
interface MessagesResponse {
  messages: Message[]
  first_unread_message_id?: string  // NEW
}
```

### New WebSocket Event

```typescript
{
  type: "message.read",
  dialog_id: "uuid",
  user_id: "uuid"
}
```
