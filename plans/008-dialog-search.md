# 008: Dialog Search

## Overview

Добавить поиск по чатам в боковую панель MTChat. Поиск осуществляется по названию диалога (title) на стороне бэкенда.

## User Story

Как пользователь, я хочу:
- Видеть поле поиска над списком чатов
- Вводить текст для фильтрации чатов по названию
- Очищать поле поиска нажатием Esc
- Видеть результаты поиска в реальном времени (с debounce)

## Implementation Plan

### Phase 1: Backend API

#### 1.1 Обновить SQL-запрос в `dialog_repo.rs`

**Файл:** `mtchat-rust/src/repos/dialog_repo.rs`

Добавить опциональный параметр `search` в методы `get_participating_dialogs` и `get_available_dialogs`:

```rust
pub async fn get_participating_dialogs(
    pool: &PgPool,
    user_id: &str,
    search: Option<&str>,  // NEW
) -> Result<Vec<DialogWithMeta>, Error> {
    // Добавить в WHERE:
    // AND ($3::text IS NULL OR d.title ILIKE '%' || $3 || '%')
}

pub async fn get_available_dialogs(
    pool: &PgPool,
    user_id: &str,
    scope_config: &ScopeConfig,
    search: Option<&str>,  // NEW
) -> Result<Vec<DialogWithMeta>, Error> {
    // Аналогично
}
```

#### 1.2 Обновить API handler

**Файл:** `mtchat-rust/src/api/handlers/dialogs.rs`

Добавить query parameter `search` в `list_dialogs`:

```rust
#[derive(Deserialize)]
pub struct ListDialogsQuery {
    #[serde(rename = "type")]
    pub dialog_type: Option<String>,
    pub search: Option<String>,  // NEW
}
```

### Phase 2: TypeScript SDK

#### 2.1 Обновить API клиент

**Файл:** `mtchat-vue/src/sdk/api.ts`

```typescript
interface GetDialogsOptions {
  type?: 'participating' | 'available'
  search?: string  // NEW
}

async getDialogs(options: GetDialogsOptions = {}): Promise<Dialog[]> {
  const params = new URLSearchParams()
  if (options.type) params.append('type', options.type)
  if (options.search) params.append('search', options.search)
  // ...
}
```

#### 2.2 Обновить composable

**Файл:** `mtchat-vue/src/composables/useChat.ts`

```typescript
// Добавить reactive search state
const searchQuery = ref('')

async function loadParticipatingDialogs() {
  const dialogs = await api.getDialogs({
    type: 'participating',
    search: searchQuery.value || undefined
  })
  // ...
}

async function loadAvailableDialogs() {
  const dialogs = await api.getDialogs({
    type: 'available',
    search: searchQuery.value || undefined
  })
  // ...
}

// Добавить функцию поиска с debounce
function setSearchQuery(query: string) {
  searchQuery.value = query
}

// Watch с debounce для перезагрузки при изменении поиска
```

### Phase 3: UI Component

#### 3.1 Добавить поле поиска в sidebar

**Файл:** `mtchat-vue/src/components/MTChat.vue`

В секции sidebar, перед табами добавить:

```vue
<div class="mtchat-search">
  <input
    v-model="searchInput"
    type="text"
    :placeholder="t.search.placeholder"
    class="mtchat-search-input"
    @keydown.esc="clearSearch"
  />
  <button
    v-if="searchInput"
    class="mtchat-search-clear"
    @click="clearSearch"
  >
    <i class="pi pi-times" />
  </button>
</div>
```

#### 3.2 Логика поиска с debounce

```typescript
const searchInput = ref('')

// Debounced search
const debouncedSearch = useDebounceFn(() => {
  setSearchQuery(searchInput.value)
}, 300)

watch(searchInput, () => {
  debouncedSearch()
})

function clearSearch() {
  searchInput.value = ''
  setSearchQuery('')
}
```

#### 3.3 Стили

```css
.mtchat-search {
  padding: 12px;
  border-bottom: 1px solid var(--mtchat-border);
  display: flex;
  gap: 8px;
}

.mtchat-search-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--mtchat-border);
  border-radius: 6px;
  background: var(--mtchat-input-bg);
  color: var(--mtchat-text);
  outline: none;
}

.mtchat-search-input:focus {
  border-color: var(--mtchat-primary);
}

.mtchat-search-clear {
  background: none;
  border: none;
  color: var(--mtchat-text-secondary);
  cursor: pointer;
  padding: 4px;
}
```

### Phase 4: Translations

**Файл:** `mtchat-vue/src/i18n/translations.ts`

```typescript
// Добавить в каждую локаль
search: {
  placeholder: 'Поиск по чатам...',  // ru
  // placeholder: 'Search chats...',  // en
  // placeholder: '搜索聊天...',  // zh
  noResults: 'Ничего не найдено',
}
```

## Files to Modify

| File | Changes |
|------|---------|
| `mtchat-rust/src/repos/dialog_repo.rs` | Add `search` param to queries |
| `mtchat-rust/src/api/handlers/dialogs.rs` | Add `search` query param |
| `mtchat-vue/src/sdk/api.ts` | Add `search` to getDialogs |
| `mtchat-vue/src/composables/useChat.ts` | Add search state + debounce |
| `mtchat-vue/src/components/MTChat.vue` | Add search input UI |
| `mtchat-vue/src/i18n/translations.ts` | Add search translations |

## UX Details

- **Debounce:** 300ms delay before sending search request
- **Esc key:** Clears search input and shows all chats
- **Clear button:** Appears only when input has text
- **Loading state:** Show spinner while searching
- **Empty state:** Show "No results" when search returns empty

## Verification

1. Запустить backend: `cd mtchat-rust && cargo run`
2. Запустить demo app: `cd mtchat-example && npm run dev`
3. Проверить:
   - Ввод текста в поиск фильтрует список чатов
   - Работает в обоих табах (Участвую / Доступные)
   - Esc очищает поиск
   - Кнопка X очищает поиск
   - Debounce работает (не спамит запросы)
   - Пустой результат показывает сообщение
