# 009: Potential Chat Access Control

## Overview

Исправить уязвимость безопасности: потенциальные участники (не присоединившиеся к чату) не должны видеть сообщения и не должны иметь возможность отправлять сообщения.

## Problem

### Текущее поведение (НЕВЕРНОЕ)

1. Потенциальный участник может читать все сообщения через `GET /dialogs/{id}/messages`
2. Потенциальный участник может отправлять сообщения через `POST /dialogs/{id}/messages`
3. В inline режиме UI позволяет отправку сообщений без присоединения
4. Бывший участник (вышедший из чата) сохраняет доступ к сообщениям

### Ожидаемое поведение

1. До присоединения пользователь видит: **"Необходимо присоединиться чтобы увидеть сообщения"**
2. Кнопка "Присоединиться" доступна
3. После присоединения — полный доступ к чату
4. После выхода — снова требуется присоединение

## Security Analysis

| Endpoint | Текущая проверка | Требуется |
|----------|-----------------|-----------|
| `GET /dialogs/{id}/messages` | ❌ Нет | ✅ Проверка участия |
| `POST /dialogs/{id}/messages` | ❌ Нет | ✅ Проверка участия |
| `GET /dialogs/{id}` | ❌ Нет | ✅ Проверка scope или участия |
| `GET /dialogs/{id}/participants` | ❌ Нет | ✅ Проверка участия |

## Implementation Plan

### Phase 1: Backend Security

#### 1.1 Добавить helper для проверки доступа

**Файл:** `mtchat-rust/src/main.rs`

```rust
/// Check if user is a participant of the dialog
async fn require_participant(
    state: &AppState,
    dialog_id: Uuid,
    user_id: &str,
) -> Result<(), ApiError> {
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden("Not a participant. Join the dialog first.".into()));
    }
    Ok(())
}
```

#### 1.2 Защитить endpoint сообщений

**Файл:** `mtchat-rust/src/main.rs` — функция `list_messages` (~строка 571)

```rust
async fn list_messages(...) -> Result<...> {
    // Проверить что диалог существует
    let dialog = state.dialogs.find_by_id(dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    // NEW: Проверить что пользователь - участник
    require_participant(&state, dialog_id, &user_id).await?;

    // ... rest of code
}
```

#### 1.3 Защитить отправку сообщений

**Файл:** `mtchat-rust/src/main.rs` — функция `send_message` (~строка 663)

```rust
async fn send_message(...) -> Result<...> {
    let dialog = state.dialogs.find_by_id(dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    // NEW: Проверить что пользователь - участник
    require_participant(&state, dialog_id, &sender_id).await?;

    // ... rest of code
}
```

#### 1.4 Защитить список участников

**Файл:** `mtchat-rust/src/main.rs` — функция `list_participants` (~строка 876)

```rust
async fn list_participants(...) -> Result<...> {
    // NEW: Проверить что пользователь - участник
    require_participant(&state, dialog_id, &user_id).await?;

    // ... rest of code
}
```

#### 1.5 Защитить получение диалога

**Файл:** `mtchat-rust/src/main.rs` — функция `get_dialog` (~строка 559)

Здесь логика сложнее: нужно разрешить доступ если:
- Пользователь участник, ИЛИ
- Пользователь потенциальный участник (по scope)

```rust
async fn get_dialog(...) -> Result<...> {
    let dialog = state.dialogs.find_by_id(dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    // NEW: Проверить доступ (участник ИЛИ потенциальный)
    let is_participant = state.participants.exists(dialog_id, &user_id).await?;
    let has_scope_access = state.scopes.check_access(dialog_id, &scope_config).await?;

    if !is_participant && !has_scope_access {
        return Err(ApiError::Forbidden("No access to this dialog".into()));
    }

    // ... rest of code
}
```

### Phase 2: Frontend UI

#### 2.1 Исправить canSendMessage

**Файл:** `mtchat-vue/src/components/MTChat.vue`

```typescript
// БЫЛО (НЕВЕРНО):
const canSendMessage = computed(() =>
  hasDialog.value &&
  (chat.currentDialog.value?.i_am_participant || isInlineMode.value)
)

// СТАЛО (ВЕРНО):
const canSendMessage = computed(() =>
  hasDialog.value && chat.currentDialog.value?.i_am_participant
)
```

#### 2.2 Добавить состояние "Требуется присоединение"

**Файл:** `mtchat-vue/src/components/MTChat.vue`

В секции messages area добавить проверку:

```vue
<!-- Messages Area -->
<div class="mtchat-messages-area">
  <!-- NEW: Требуется присоединение -->
  <div v-if="hasDialog && !currentDialog?.i_am_participant" class="mtchat-join-required">
    <div class="mtchat-join-required-content">
      <i class="pi pi-lock" />
      <p>{{ t.chat.joinRequired }}</p>
      <button
        v-if="canJoin"
        class="mtchat-join-btn"
        @click="joinCurrentDialog"
      >
        {{ t.buttons.join }}
      </button>
    </div>
  </div>

  <!-- Existing messages list (только для участников) -->
  <template v-else>
    <!-- ... existing messages code ... -->
  </template>
</div>
```

#### 2.3 Скрыть поле ввода для не-участников

```vue
<!-- Message Input (только для участников) -->
<div v-if="canSendMessage" class="mtchat-input-area">
  <!-- ... existing input ... -->
</div>
```

#### 2.4 Стили для "Требуется присоединение"

```css
.mtchat-join-required {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.mtchat-join-required-content {
  text-align: center;
  color: var(--mtchat-text-secondary);
}

.mtchat-join-required-content i {
  font-size: 48px;
  color: var(--mtchat-text-muted);
  margin-bottom: 16px;
  display: block;
}

.mtchat-join-required-content p {
  margin-bottom: 16px;
  font-size: 16px;
}

.mtchat-join-btn {
  padding: 10px 24px;
  background: var(--mtchat-primary);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
}

.mtchat-join-btn:hover {
  opacity: 0.9;
}
```

### Phase 3: Translations

**Файл:** `mtchat-vue/src/i18n/translations.ts`

```typescript
// ru
chat: {
  // ... existing
  joinRequired: 'Необходимо присоединиться чтобы увидеть сообщения',
}

// en
chat: {
  joinRequired: 'You need to join to see messages',
}

// zh
chat: {
  joinRequired: '您需要加入才能查看消息',
}
```

### Phase 4: Handle 403 Errors

#### 4.1 Обработка ошибки в useChat

**Файл:** `mtchat-vue/src/composables/useChat.ts`

При загрузке сообщений для inline режима может вернуться 403:

```typescript
async function loadMessages() {
  if (!currentDialog.value) return

  try {
    // Если не участник - не загружать сообщения
    if (!currentDialog.value.i_am_participant) {
      messages.value = []
      return
    }

    const result = await client.api.getMessages(currentDialog.value.id, {
      limit: 50,
    })
    messages.value = result.messages
    // ...
  } catch (err) {
    // 403 = не участник, это ожидаемо для потенциальных чатов
    if (err instanceof ApiError && err.status === 403) {
      messages.value = []
      return
    }
    throw err
  }
}
```

## Files to Modify

| File | Changes |
|------|---------|
| `mtchat-rust/src/main.rs` | Add participant checks to 4 endpoints |
| `mtchat-vue/src/components/MTChat.vue` | Fix canSendMessage, add join-required UI |
| `mtchat-vue/src/composables/useChat.ts` | Handle 403, skip loading for non-participants |
| `mtchat-vue/src/i18n/translations.ts` | Add joinRequired translation |

## Test Scenarios

### Backend Tests

1. **Потенциальный участник не может читать сообщения:**
   ```bash
   # User with scope access but NOT a participant
   GET /api/v1/dialogs/{id}/messages
   # Expected: 403 Forbidden
   ```

2. **Потенциальный участник не может отправлять сообщения:**
   ```bash
   POST /api/v1/dialogs/{id}/messages
   Body: { "content": "test" }
   # Expected: 403 Forbidden
   ```

3. **После присоединения - полный доступ:**
   ```bash
   POST /api/v1/dialogs/{id}/join
   # Expected: 200 OK

   GET /api/v1/dialogs/{id}/messages
   # Expected: 200 OK with messages
   ```

4. **После выхода - снова нет доступа:**
   ```bash
   POST /api/v1/dialogs/{id}/leave
   # Expected: 200 OK

   GET /api/v1/dialogs/{id}/messages
   # Expected: 403 Forbidden
   ```

### Frontend Tests

1. Открыть потенциальный чат в inline режиме → видеть "Необходимо присоединиться"
2. Кнопка "Присоединиться" видна и работает
3. После присоединения → сообщения загружаются, можно отправлять
4. После выхода → снова "Необходимо присоединиться"

## Verification

1. Запустить backend: `cd mtchat-rust && cargo run`
2. Запустить demo: `cd mtchat-example && npm run dev`
3. Создать тестовые данные:
   - Tenant с 2 пользователями
   - Диалог с scope доступом для обоих
   - Один пользователь - участник, другой - нет
4. Проверить:
   - Под не-участником открыть чат → видеть заглушку
   - Присоединиться → видеть сообщения
   - Выйти → снова заглушка
   - Проверить что нельзя отправить сообщение через DevTools (curl)
