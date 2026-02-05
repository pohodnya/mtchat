# 006: Информация об участниках чата (Participant Info)

## Цели

1. Хранить и отображать информацию об участниках: имя (обязательно), компания, email, телефон (опционально)
2. Отображать полное имя участника в сообщениях, для своих — добавлять "(Вы)"
3. Клик по названию чата открывает информационную панель с участниками
4. Меню с тремя точками: Информация, Архивировать (заглушка), Выйти
5. В inline mode информационная панель перекрывает чат целиком

---

## Текущее состояние

### Структура данных

| Таблица | Поля |
|---------|------|
| `dialog_participants` | `dialog_id`, `user_id`, `joined_at`, `joined_as`, `notifications_enabled`, `last_read_message_id`, `unread_count` |

**Проблема**: Нет полей для хранения имени, компании, email, телефона участника.

### Текущий UI

- Имя отправителя: первые 8 символов `user_id` (UUID)
- Хедер: название + кол-во участников + статус + кнопка Leave
- Нет правой панели с информацией

---

## Архитектура решения

### Вариант: Расширение таблицы `dialog_participants`

Добавляем поля профиля непосредственно в таблицу участников. Это позволяет:
- Иметь разное представление пользователя в разных чатах
- Не создавать отдельную таблицу user_profiles
- Информация передаётся при создании чата через Management API

```sql
ALTER TABLE dialog_participants ADD COLUMN display_name VARCHAR(255);
ALTER TABLE dialog_participants ADD COLUMN company VARCHAR(255);
ALTER TABLE dialog_participants ADD COLUMN email VARCHAR(255);
ALTER TABLE dialog_participants ADD COLUMN phone VARCHAR(50);
```

---

## Этап 1: Backend — Миграция БД

**Файл**: `mtchat-rust/migrations/20250205000001_add_participant_profile.sql`

```sql
-- Add participant profile fields
ALTER TABLE dialog_participants
  ADD COLUMN display_name VARCHAR(255),
  ADD COLUMN company VARCHAR(255),
  ADD COLUMN email VARCHAR(255),
  ADD COLUMN phone VARCHAR(50);

-- display_name is required for new participants (handled in app logic)
COMMENT ON COLUMN dialog_participants.display_name IS 'Display name (ФИО, инициалы, псевдоним)';
COMMENT ON COLUMN dialog_participants.company IS 'Company/organization name';
COMMENT ON COLUMN dialog_participants.email IS 'Contact email';
COMMENT ON COLUMN dialog_participants.phone IS 'Contact phone';
```

---

## Этап 2: Backend — Обновление Rust Domain

**Файл**: `mtchat-rust/src/domain/participant.rs`

```rust
pub struct DialogParticipant {
    pub dialog_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub joined_as: JoinedAs,
    pub notifications_enabled: bool,
    pub last_read_message_id: Option<Uuid>,
    pub unread_count: i32,
    // New fields
    pub display_name: Option<String>,
    pub company: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}
```

---

## Этап 3: Backend — Management API

**Файл**: `mtchat-rust/src/main.rs`

### Обновление CreateDialog request

```rust
#[derive(Deserialize)]
struct ParticipantInput {
    user_id: Uuid,
    display_name: String,  // Required
    company: Option<String>,
    email: Option<String>,
    phone: Option<String>,
}

#[derive(Deserialize)]
struct CreateDialogRequest {
    object_id: Uuid,
    object_type: String,
    title: Option<String>,
    participants: Vec<ParticipantInput>,  // Changed from Vec<Uuid>
    access_scopes: Option<Vec<AccessScopeInput>>,
}
```

### Обновление add_participant endpoint

```rust
#[derive(Deserialize)]
struct AddParticipantRequest {
    user_id: Uuid,
    display_name: String,
    company: Option<String>,
    email: Option<String>,
    phone: Option<String>,
}
```

---

## Этап 4: Backend — Chat API

**Файл**: `mtchat-rust/src/main.rs`

### GET /dialogs/{id}/participants

Возвращает полную информацию об участниках включая новые поля.

### Обработка join

При самостоятельном присоединении (`POST /dialogs/{id}/join`) нужно передавать display_name:

```rust
#[derive(Deserialize)]
struct JoinDialogRequest {
    display_name: String,
    company: Option<String>,
    email: Option<String>,
    phone: Option<String>,
}
```

---

## Этап 5: Frontend — Типы

**Файл**: `mtchat-vue/src/types/index.ts`

```typescript
export interface DialogParticipant {
  dialog_id: string
  user_id: string
  joined_at: string
  joined_as: 'creator' | 'participant' | 'joined'
  notifications_enabled: boolean
  last_read_message_id?: string
  // New fields
  display_name: string
  company?: string
  email?: string
  phone?: string
}
```

---

## Этап 6: Frontend — Отображение имени в сообщениях

**Файл**: `mtchat-vue/src/components/MTChat.vue`

### Helper function

```typescript
function getSenderName(senderId: string): string {
  if (senderId === props.config.userId) {
    // Find my display_name from participants
    const me = chat.participants.value.find(p => p.user_id === senderId)
    return me?.display_name ? `${me.display_name} (Вы)` : 'Вы'
  }
  const participant = chat.participants.value.find(p => p.user_id === senderId)
  return participant?.display_name || senderId.slice(0, 8)
}
```

### Template update

```vue
<span class="mtchat__message-sender">
  {{ getSenderName(message.sender_id) }}
</span>
```

---

## Этап 7: Frontend — Редизайн хедера

**Файл**: `mtchat-vue/src/components/MTChat.vue`

### Новый хедер

```vue
<header v-if="showHeader && hasDialog" class="mtchat__header">
  <!-- Clickable title area -->
  <button class="mtchat__header-info" @click="toggleInfoPanel">
    <h2 class="mtchat__header-title">{{ dialogTitle }}</h2>
    <span class="mtchat__header-participants">
      {{ chat.currentDialog.value?.participants_count || 0 }} участников
    </span>
  </button>

  <!-- Actions menu (3 dots) -->
  <div class="mtchat__header-actions">
    <button v-if="canJoin" class="mtchat__btn mtchat__btn--primary" @click="handleJoinDialog">
      Присоединиться
    </button>
    <div v-else class="mtchat__menu" ref="menuRef">
      <button class="mtchat__menu-trigger" @click="toggleMenu">
        <svg><!-- 3 dots icon --></svg>
      </button>
      <div v-if="menuOpen" class="mtchat__menu-dropdown">
        <button @click="openInfoPanel">
          <i class="pi pi-info-circle" /> Информация
        </button>
        <button @click="handleArchive" disabled>
          <i class="pi pi-inbox" /> Архивировать
        </button>
        <button @click="handleLeaveDialog" class="danger">
          <i class="pi pi-sign-out" /> Выйти
        </button>
      </div>
    </div>
  </div>
</header>
```

---

## Этап 8: Frontend — Информационная панель

### State

```typescript
const showInfoPanel = ref(false)

function toggleInfoPanel() {
  showInfoPanel.value = !showInfoPanel.value
}

function openInfoPanel() {
  showInfoPanel.value = true
  menuOpen.value = false
}

function closeInfoPanel() {
  showInfoPanel.value = false
}
```

### Full Mode — Right Panel

```vue
<!-- Add after mtchat__main -->
<aside v-if="showInfoPanel && !isInlineMode" class="mtchat__info-panel">
  <ChatInfoPanel
    :dialog="chat.currentDialog.value"
    :participants="chat.participants.value"
    :current-user-id="config.userId"
    @close="closeInfoPanel"
  />
</aside>
```

### Inline Mode — Overlay

```vue
<!-- Inside mtchat__main, after messages -->
<div v-if="showInfoPanel && isInlineMode" class="mtchat__info-overlay">
  <ChatInfoPanel
    :dialog="chat.currentDialog.value"
    :participants="chat.participants.value"
    :current-user-id="config.userId"
    @close="closeInfoPanel"
  />
</div>
```

---

## Этап 9: Компонент ChatInfoPanel

**Файл**: `mtchat-vue/src/components/chat/ChatInfoPanel.vue`

```vue
<template>
  <div class="chat-info-panel">
    <!-- Header -->
    <div class="chat-info-panel__header">
      <h3>Информация</h3>
      <button class="chat-info-panel__close" @click="$emit('close')">
        <svg><!-- X icon --></svg>
      </button>
    </div>

    <!-- Dialog Info -->
    <div class="chat-info-panel__dialog">
      <div class="chat-info-panel__avatar">
        {{ dialogInitials }}
      </div>
      <h2 class="chat-info-panel__title">{{ dialog?.title || 'Чат' }}</h2>
      <p class="chat-info-panel__meta">
        {{ participants.length }} участников
      </p>
    </div>

    <!-- Participants List -->
    <div class="chat-info-panel__section">
      <h4>Участники</h4>
      <div class="chat-info-panel__participants">
        <div
          v-for="p in participants"
          :key="p.user_id"
          class="participant-item"
        >
          <div class="participant-item__avatar">
            {{ getInitials(p.display_name) }}
          </div>
          <div class="participant-item__info">
            <div class="participant-item__name">
              {{ p.display_name }}
              <span v-if="p.user_id === currentUserId" class="participant-item__you">(Вы)</span>
              <span v-if="p.joined_as === 'creator'" class="participant-item__role">создатель</span>
            </div>
            <div v-if="p.company" class="participant-item__company">
              {{ p.company }}
            </div>
            <div class="participant-item__contacts">
              <a v-if="p.email" :href="`mailto:${p.email}`" class="participant-item__contact">
                {{ p.email }}
              </a>
              <a v-if="p.phone" :href="`tel:${p.phone}`" class="participant-item__contact">
                {{ p.phone }}
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
```

### Стили участника

```css
.participant-item {
  display: flex;
  gap: 12px;
  padding: 12px 0;
  border-bottom: 1px solid var(--mtchat-border);
}

.participant-item:last-child {
  border-bottom: none;
}

.participant-item__avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--mtchat-primary);
  color: var(--mtchat-primary-text);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 14px;
  flex-shrink: 0;
}

.participant-item__info {
  flex: 1;
  min-width: 0;
}

.participant-item__name {
  font-weight: 500;
  color: var(--mtchat-text);
  display: flex;
  align-items: center;
  gap: 6px;
}

.participant-item__you {
  color: var(--mtchat-text-secondary);
  font-weight: 400;
}

.participant-item__role {
  font-size: 11px;
  color: var(--mtchat-text-secondary);
  background: var(--mtchat-bg-secondary);
  padding: 2px 6px;
  border-radius: 4px;
}

.participant-item__company {
  font-size: 13px;
  color: var(--mtchat-text-secondary);
  margin-top: 2px;
}

.participant-item__contacts {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 4px;
}

.participant-item__contact {
  font-size: 12px;
  color: var(--mtchat-primary);
  text-decoration: none;
}

.participant-item__contact:hover {
  text-decoration: underline;
}
```

---

## Этап 10: Конфиг SDK с профилем пользователя

**Файл**: `mtchat-vue/src/types/index.ts`

### Расширение MTChatConfig

```typescript
export interface UserProfile {
  displayName: string      // Имя пользователя по умолчанию
  company: string          // Компания (обязательно, нельзя скрыть)
  email?: string           // Email (можно скрыть при join)
  phone?: string           // Телефон (можно скрыть при join)
}

export interface MTChatConfig {
  baseUrl: string
  userId: string
  scopeConfig: ScopeConfig
  userProfile: UserProfile  // NEW: профиль текущего пользователя
  // ... остальные поля
}
```

### Использование в компоненте

```vue
<MTChat
  :config="{
    baseUrl: 'https://chat.example.com',
    userId: user.id,
    scopeConfig: { ... },
    userProfile: {
      displayName: user.name,
      company: user.company.name,
      email: user.email,
      phone: user.phone
    }
  }"
/>
```

---

## Этап 11: Модальное окно Join

**Файл**: `mtchat-vue/src/components/chat/JoinDialog.vue`

При нажатии "Присоединиться" открывается модальное окно:

```vue
<template>
  <div class="join-dialog-overlay" @click.self="$emit('cancel')">
    <div class="join-dialog">
      <h3>Присоединиться к чату</h3>

      <div class="join-dialog__form">
        <!-- Имя: можно изменить или выбрать анонимное -->
        <div class="form-field">
          <label>Имя</label>
          <div class="name-options">
            <label class="radio-option">
              <input type="radio" v-model="nameOption" value="profile" />
              <span>{{ userProfile.displayName }}</span>
            </label>
            <label class="radio-option">
              <input type="radio" v-model="nameOption" value="anonymous" />
              <span>Сотрудник компании {{ userProfile.company }}</span>
            </label>
          </div>
        </div>

        <!-- Компания: только отображение, нельзя изменить -->
        <div class="form-field">
          <label>Компания</label>
          <div class="company-display">{{ userProfile.company }}</div>
        </div>

        <!-- Email: можно скрыть -->
        <div class="form-field" v-if="userProfile.email">
          <label>
            <input type="checkbox" v-model="shareEmail" />
            Показывать email ({{ userProfile.email }})
          </label>
        </div>

        <!-- Телефон: можно скрыть -->
        <div class="form-field" v-if="userProfile.phone">
          <label>
            <input type="checkbox" v-model="sharePhone" />
            Показывать телефон ({{ userProfile.phone }})
          </label>
        </div>
      </div>

      <div class="join-dialog__actions">
        <button class="btn btn--secondary" @click="$emit('cancel')">Отмена</button>
        <button class="btn btn--primary" @click="handleJoin">Присоединиться</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { UserProfile } from '../../types'

const props = defineProps<{
  userProfile: UserProfile
}>()

const emit = defineEmits<{
  (e: 'join', profile: { displayName: string; company: string; email?: string; phone?: string }): void
  (e: 'cancel'): void
}>()

const nameOption = ref<'profile' | 'anonymous'>('profile')
const shareEmail = ref(true)
const sharePhone = ref(true)

const displayName = computed(() => {
  return nameOption.value === 'profile'
    ? props.userProfile.displayName
    : `Сотрудник компании ${props.userProfile.company}`
})

function handleJoin() {
  emit('join', {
    displayName: displayName.value,
    company: props.userProfile.company,
    email: shareEmail.value ? props.userProfile.email : undefined,
    phone: sharePhone.value ? props.userProfile.phone : undefined,
  })
}
</script>
```

### Интеграция в MTChat.vue

```typescript
const showJoinDialog = ref(false)

function handleJoinClick() {
  showJoinDialog.value = true
}

async function handleJoinConfirm(profile: JoinProfile) {
  showJoinDialog.value = false
  await chat.joinDialog(chat.currentDialog.value!.id, profile)
}
```

```vue
<button v-if="canJoin" @click="handleJoinClick">
  Присоединиться
</button>

<JoinDialog
  v-if="showJoinDialog"
  :user-profile="config.userProfile"
  @join="handleJoinConfirm"
  @cancel="showJoinDialog = false"
/>
```

---

## Этап 12: Обновление SDK/API

**Файл**: `mtchat-vue/src/sdk/api.ts`

### joinDialog с профилем

```typescript
async joinDialog(
  dialogId: string,
  profile: {
    displayName: string
    company: string
    email?: string
    phone?: string
  }
): Promise<void> {
  await this.request('POST', `/api/v1/dialogs/${dialogId}/join`, {
    display_name: profile.displayName,
    company: profile.company,
    email: profile.email,
    phone: profile.phone,
  })
}
```

---

## Этап 13: Хедер для Inline Mode

Сейчас в inline mode хедер не показывается (`showHeader` часто false). Нужно:
1. Добавить упрощённый хедер в inline mode
2. Информационная панель в inline mode перекрывает чат целиком

```vue
<!-- Inline mode header -->
<header v-if="isInlineMode && hasDialog" class="mtchat__header mtchat__header--inline">
  <button class="mtchat__header-info" @click="toggleInfoPanel">
    <h2 class="mtchat__header-title">{{ dialogTitle }}</h2>
    <span class="mtchat__header-participants">
      {{ chat.currentDialog.value?.participants_count || 0 }} участников
    </span>
  </button>
  <div class="mtchat__header-actions">
    <!-- Same menu as full mode -->
  </div>
</header>
```

---

## Порядок реализации

| # | Задача | Приоритет |
|---|--------|-----------|
| 1 | Миграция БД: добавить поля профиля | High |
| 2 | Backend: обновить domain/participant.rs | High |
| 3 | Backend: обновить repository | High |
| 4 | Backend: обновить Management API | High |
| 5 | Backend: обновить Chat API (join) | High |
| 6 | Frontend: обновить типы (DialogParticipant, UserProfile) | High |
| 7 | Frontend: расширить MTChatConfig с userProfile | High |
| 8 | Frontend: отображение имени в сообщениях | High |
| 9 | Frontend: редизайн хедера с меню | High |
| 10 | Frontend: компонент ChatInfoPanel | High |
| 11 | Frontend: модальное окно JoinDialog | High |
| 12 | Frontend: интеграция панели (full mode) | High |
| 13 | Frontend: overlay для inline mode | High |
| 14 | Frontend: хедер для inline mode | Medium |
| 15 | Demo App: обновить создание участников | Medium |

---

## Ключевые файлы для изменения

### Backend (mtchat-rust)
- `migrations/20250205000001_add_participant_profile.sql` — новая миграция
- `src/domain/participant.rs` — расширить структуру
- `src/repositories/participant_repo.rs` — обновить SQL запросы
- `src/main.rs` — обновить API handlers

### Frontend (mtchat-vue)
- `src/types/index.ts` — обновить DialogParticipant, добавить UserProfile, расширить MTChatConfig
- `src/sdk/api.ts` — обновить joinDialog
- `src/composables/useChat.ts` — обновить joinDialog с профилем
- `src/components/MTChat.vue` — хедер, панель информации, отображение имён
- `src/components/chat/ChatInfoPanel.vue` — новый компонент (список участников)
- `src/components/chat/JoinDialog.vue` — новый компонент (модальное окно join)

### Demo App (mtchat-example)
- `src/components/admin/DialogsPanel.vue` — форма создания с профилями участников
- `src/pages/ChatPage.vue` — передача userProfile в config
- `src/pages/InlinePage.vue` — передача userProfile в config

---

## Верификация

1. **Миграция**: `cargo sqlx migrate run` без ошибок
2. **Management API**: создать диалог с участниками через curl/Postman
3. **Chat API**: `GET /dialogs/{id}/participants` возвращает профили
4. **UI сообщения**: имена участников отображаются полностью, свои с "(Вы)"
5. **UI хедер**: клик открывает панель информации
6. **UI меню**: 3 точки → dropdown с действиями
7. **UI Join**: модальное окно с выбором имени и настройкой видимости контактов
8. **Info панель**: список участников с именем, компанией, email, телефоном
9. **Inline mode**: панель перекрывает чат целиком
10. **Темы**: проверить light/dark отображение панели и модалки

---

## Дополнительные улучшения (Future)

- Поиск по участникам в панели информации
- Аватары участников (URL изображения)
- Онлайн-статус участников
- Возможность редактировать свой профиль в чате
