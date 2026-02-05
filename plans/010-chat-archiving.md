# 010: Chat Archiving

## Overview

Добавить возможность архивирования чатов для отдельного участника. Архивные чаты отображаются в свёрнутом аккордеоне под списком активных чатов.

## User Story

Как участник чата, я хочу:
- Архивировать чат, чтобы он не мешал в основном списке
- Видеть архивные чаты в отдельном аккордеоне "Архивные"
- Разархивировать чат при необходимости
- Искать по всем чатам (активным и архивным)

## UX Details

### Sidebar Layout

```
┌─────────────────────────┐
│ [Поиск по чатам...]     │
├─────────────────────────┤
│ Мои чаты │ Доступные    │
├─────────────────────────┤
│ ▪ Активный чат 1        │
│ ▪ Активный чат 2        │
│ ▪ Активный чат 3        │
│                         │
│ ▶ Архивные (2)          │  ← Свёрнуто по умолчанию
└─────────────────────────┘

При раскрытии:
┌─────────────────────────┐
│ ▶ Архивные (2)          │
│   ▪ Архивный чат 1      │
│   ▪ Архивный чат 2      │
└─────────────────────────┘
```

### Меню действий в Header

```
┌──────────────────┐
│ ℹ Информация     │
│ 📥 Архивировать  │  ← NEW (или "Разархивировать" для архивных)
│ 🚪 Покинуть чат  │
└──────────────────┘
```

### Поведение

- Аккордеон "Архивные" всегда внизу списка диалогов
- По умолчанию свёрнут
- При поиске результаты показывают и активные и архивные, но архивные остаются в своей секции
- Архивация/разархивация работает только для текущего пользователя
- После архивации чат исчезает из активного списка, появляется в архивных
- После разархивации — наоборот

## Implementation Plan

### Phase 1: Database Migration

**Файл:** `mtchat-rust/migrations/20250206000001_add_is_archived.sql`

```sql
-- Add is_archived column to dialog_participants
ALTER TABLE dialog_participants
ADD COLUMN is_archived BOOLEAN NOT NULL DEFAULT FALSE;

-- Index for filtering archived dialogs
CREATE INDEX idx_participants_archived
ON dialog_participants(user_id, is_archived);
```

### Phase 2: Backend Domain & Repository

#### 2.1 Обновить domain struct

**Файл:** `mtchat-rust/src/domain/participant.rs`

```rust
#[derive(sqlx::FromRow)]
pub struct DialogParticipant {
    // ... existing fields
    pub is_archived: bool,
}
```

#### 2.2 Добавить методы в repository

**Файл:** `mtchat-rust/src/repositories/participant_repo.rs`

```rust
/// Archive a dialog for user
pub async fn set_archived(
    &self,
    dialog_id: Uuid,
    user_id: Uuid,
    archived: bool,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE dialog_participants
         SET is_archived = $3
         WHERE dialog_id = $1 AND user_id = $2"
    )
    .bind(dialog_id)
    .bind(user_id)
    .bind(archived)
    .execute(&self.pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
```

#### 2.3 Обновить запросы в dialog_repo

**Файл:** `mtchat-rust/src/repositories/dialog_repo.rs`

Метод `find_participating` — добавить параметр `archived: Option<bool>`:

```rust
pub async fn find_participating(
    &self,
    user_id: Uuid,
    search: Option<&str>,
    archived: Option<bool>,  // NEW: None = all, Some(true) = only archived, Some(false) = only active
) -> Result<Vec<Dialog>, sqlx::Error> {
    sqlx::query_as::<_, Dialog>(
        r#"SELECT d.* FROM dialogs d
           INNER JOIN dialog_participants dp ON dp.dialog_id = d.id
           WHERE dp.user_id = $1
             AND ($2::text IS NULL OR d.title ILIKE '%' || $2 || '%')
             AND ($3::boolean IS NULL OR dp.is_archived = $3)
           ORDER BY d.created_at DESC"#,
    )
    .bind(user_id)
    .bind(search)
    .bind(archived)
    .fetch_all(&self.pool)
    .await
}
```

### Phase 3: Backend API

#### 3.1 Обновить DialogsQuery

**Файл:** `mtchat-rust/src/main.rs`

```rust
#[derive(Deserialize)]
struct DialogsQuery {
    r#type: Option<String>,
    search: Option<String>,
    archived: Option<bool>,  // NEW
}
```

#### 3.2 Добавить endpoint для архивации

```rust
// POST /api/v1/dialogs/{id}/archive
async fn archive_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check user is participant
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden("Not a participant".into()));
    }

    state.participants.set_archived(dialog_id, user_id, true).await?;

    Ok(Json(serde_json::json!({ "status": "archived" })))
}

// POST /api/v1/dialogs/{id}/unarchive
async fn unarchive_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden("Not a participant".into()));
    }

    state.participants.set_archived(dialog_id, user_id, false).await?;

    Ok(Json(serde_json::json!({ "status": "unarchived" })))
}
```

#### 3.3 Добавить routes

```rust
.route("/api/v1/dialogs/:dialog_id/archive", post(archive_dialog))
.route("/api/v1/dialogs/:dialog_id/unarchive", post(unarchive_dialog))
```

#### 3.4 Обновить list_dialogs

Передавать `archived` параметр в repository:

```rust
async fn list_dialogs(...) -> ... {
    let archived = params.archived;

    let dialogs = match dialog_type {
        "participating" => {
            state.dialogs.find_participating(user_id, search, archived).await?
        }
        // available не фильтруется по archived (там нет участия)
        ...
    };
}
```

#### 3.5 Добавить is_archived в DialogResponse

Обновить структуру ответа чтобы включить флаг `is_archived`.

### Phase 4: TypeScript SDK

#### 4.1 Обновить типы

**Файл:** `mtchat-vue/src/types/index.ts`

```typescript
export interface DialogListItem extends Dialog {
  participants_count: number
  i_am_participant?: boolean
  can_join?: boolean
  unread_count?: number
  is_archived?: boolean  // NEW
}
```

#### 4.2 Добавить методы в API

**Файл:** `mtchat-vue/src/sdk/api.ts`

```typescript
/**
 * Get participating dialogs with optional archived filter
 */
async getParticipatingDialogs(search?: string, archived?: boolean): Promise<DialogListItem[]> {
  const params: Record<string, string> = { type: 'participating' }
  if (search) params.search = search
  if (archived !== undefined) params.archived = String(archived)

  const response = await this.request<ApiResponse<DialogListItem[]>>(
    'GET', '/api/v1/dialogs', { params }
  )
  return response.data
}

/**
 * Archive a dialog
 */
async archiveDialog(dialogId: string): Promise<void> {
  await this.request<void>('POST', `/api/v1/dialogs/${dialogId}/archive`)
}

/**
 * Unarchive a dialog
 */
async unarchiveDialog(dialogId: string): Promise<void> {
  await this.request<void>('POST', `/api/v1/dialogs/${dialogId}/unarchive`)
}
```

### Phase 5: Vue Composable

**Файл:** `mtchat-vue/src/composables/useChat.ts`

```typescript
// Добавить ref для архивных диалогов
const archivedDialogs: Ref<DialogListItem[]> = ref([])

// Обновить loadParticipatingDialogs
async function loadParticipatingDialogs(): Promise<void> {
  try {
    isLoading.value = true
    error.value = null
    const search = searchQuery.value || undefined

    // Load both active and archived in parallel
    const [active, archived] = await Promise.all([
      client.api.getParticipatingDialogs(search, false),
      client.api.getParticipatingDialogs(search, true),
    ])

    participatingDialogs.value = active
    archivedDialogs.value = archived
  } catch (e) {
    // ...
  }
}

// Добавить методы архивации
async function archiveDialog(dialogId: string): Promise<void> {
  try {
    isLoading.value = true
    error.value = null
    await client.api.archiveDialog(dialogId)

    // Move from active to archived
    const idx = participatingDialogs.value.findIndex(d => d.id === dialogId)
    if (idx !== -1) {
      const dialog = participatingDialogs.value[idx]
      dialog.is_archived = true
      participatingDialogs.value.splice(idx, 1)
      archivedDialogs.value.push(dialog)
    }

    // Clear current dialog if it was archived
    if (currentDialog.value?.id === dialogId) {
      currentDialog.value = null
      messages.value = []
    }
  } catch (e) {
    error.value = e instanceof Error ? e : new Error(String(e))
    throw e
  } finally {
    isLoading.value = false
  }
}

async function unarchiveDialog(dialogId: string): Promise<void> {
  // Similar logic, move from archived to active
}

// Export
return {
  // ... existing
  archivedDialogs,
  archiveDialog,
  unarchiveDialog,
}
```

### Phase 6: Vue UI Component

**Файл:** `mtchat-vue/src/components/MTChat.vue`

#### 6.1 Добавить state для аккордеона

```typescript
const showArchivedAccordion = ref(false)
```

#### 6.2 Обновить template — добавить аккордеон после списка диалогов

```vue
<!-- Dialog List -->
<div class="mtchat__dialog-list">
  <!-- Active dialogs -->
  <div v-for="dialog in currentDialogsList" ...>
    ...
  </div>

  <div v-if="currentDialogsList.length === 0 && !hasArchivedDialogs" class="mtchat__empty">
    ...
  </div>

  <!-- Archived Accordion (only in participating tab) -->
  <div
    v-if="activeTab === 'participating' && hasArchivedDialogs"
    class="mtchat__archived-section"
  >
    <button
      class="mtchat__archived-toggle"
      @click="showArchivedAccordion = !showArchivedAccordion"
    >
      <svg ...><!-- chevron icon --></svg>
      {{ t.chat.archived }} ({{ chat.archivedDialogs.value.length }})
    </button>

    <div v-if="showArchivedAccordion" class="mtchat__archived-list">
      <div
        v-for="dialog in chat.archivedDialogs.value"
        :key="dialog.id"
        :class="['mtchat__dialog-item', 'mtchat__dialog-item--archived', ...]"
        @click="handleSelectDialog(dialog)"
      >
        <!-- Same content as regular dialog item -->
      </div>
    </div>
  </div>
</div>
```

#### 6.3 Добавить кнопку в меню

```vue
<div v-if="showHeaderMenu" class="mtchat__menu-dropdown">
  <button class="mtchat__menu-item" @click="...">
    {{ t.buttons.info }}
  </button>

  <!-- Archive/Unarchive button -->
  <button
    class="mtchat__menu-item"
    @click="handleToggleArchive(); showHeaderMenu = false"
  >
    <svg ...><!-- archive icon --></svg>
    {{ currentDialog?.is_archived ? t.buttons.unarchive : t.buttons.archive }}
  </button>

  <button class="mtchat__menu-item mtchat__menu-item--danger" @click="...">
    {{ t.buttons.leaveChat }}
  </button>
</div>
```

#### 6.4 Добавить обработчик

```typescript
async function handleToggleArchive() {
  if (!chat.currentDialog.value) return

  const dialogId = chat.currentDialog.value.id

  try {
    if (chat.currentDialog.value.is_archived) {
      await chat.unarchiveDialog(dialogId)
    } else {
      await chat.archiveDialog(dialogId)
    }
  } catch (e) {
    // Error handled in composable
  }
}

const hasArchivedDialogs = computed(() =>
  chat.archivedDialogs.value.length > 0
)
```

#### 6.5 Стили

```css
.mtchat__archived-section {
  border-top: 1px solid var(--mtchat-border);
  margin-top: auto;
}

.mtchat__archived-toggle {
  width: 100%;
  padding: 12px 16px;
  display: flex;
  align-items: center;
  gap: 8px;
  background: none;
  border: none;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
}

.mtchat__archived-toggle:hover {
  background: var(--mtchat-bg-hover);
}

.mtchat__archived-toggle svg {
  transition: transform 0.2s;
}

.mtchat__archived-section--open .mtchat__archived-toggle svg {
  transform: rotate(90deg);
}

.mtchat__dialog-item--archived {
  opacity: 0.7;
}
```

### Phase 7: Translations

**Файл:** `mtchat-vue/src/i18n/translations.ts`

```typescript
// Interface
buttons: {
  // ... existing
  archive: string
  unarchive: string
}
chat: {
  // ... existing
  archived: string
}

// ru
buttons: {
  archive: 'Архивировать',
  unarchive: 'Разархивировать',
}
chat: {
  archived: 'Архивные',
}

// en
buttons: {
  archive: 'Archive',
  unarchive: 'Unarchive',
}
chat: {
  archived: 'Archived',
}

// zh
buttons: {
  archive: '归档',
  unarchive: '取消归档',
}
chat: {
  archived: '已归档',
}
```

## Files to Modify

| File | Changes |
|------|---------|
| `mtchat-rust/migrations/20250206000001_add_is_archived.sql` | NEW: Add is_archived column |
| `mtchat-rust/src/domain/participant.rs` | Add is_archived field |
| `mtchat-rust/src/repositories/participant_repo.rs` | Add set_archived method |
| `mtchat-rust/src/repositories/dialog_repo.rs` | Add archived filter to find_participating |
| `mtchat-rust/src/main.rs` | Add archive/unarchive endpoints, update list_dialogs |
| `mtchat-vue/src/types/index.ts` | Add is_archived to DialogListItem |
| `mtchat-vue/src/sdk/api.ts` | Add archive/unarchive methods, update getParticipatingDialogs |
| `mtchat-vue/src/composables/useChat.ts` | Add archivedDialogs, archive/unarchive methods |
| `mtchat-vue/src/components/MTChat.vue` | Add accordion UI, menu button, handlers |
| `mtchat-vue/src/i18n/translations.ts` | Add translations |

## Verification

1. Запустить миграцию БД
2. Запустить backend: `cd mtchat-rust && cargo run`
3. Запустить demo: `cd mtchat-example && npm run dev`
4. Проверить:
   - [ ] Архивация из меню хедера
   - [ ] Чат исчезает из активных, появляется в архивных
   - [ ] Аккордеон "Архивные" виден только когда есть архивные чаты
   - [ ] Разархивация работает
   - [ ] Поиск находит и активные и архивные чаты
   - [ ] Архивация работает индивидуально для каждого участника
