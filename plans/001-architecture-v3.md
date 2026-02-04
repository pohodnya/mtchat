# План реализации: MTChat Architecture v3

**Дата**: 2025-02-03
**Статус**: ✅ Завершено

## Обзор изменений

Переход от простой модели "tenant-to-tenant" к object-bound модели с прямыми и потенциальными участниками.

### Ключевые изменения
- **Удаление таблиц `tenants` и `employees`** — в новой архитектуре чат-сервис не хранит пользователей/тенантов, а работает с внешними идентификаторами (user_id из JWT)
- **Object-bound dialogs** — каждый чат привязан к объекту (tender, order, route)
- **Scope-based access** — потенциальные участники определяются через scope matching

## Новая модель данных

### Основные сущности

```
┌─────────────────────────────────────────────────────────────────┐
│                           Dialog                                 │
├─────────────────────────────────────────────────────────────────┤
│  id              UUID PK                                         │
│  object_id       UUID NOT NULL    ← привязка к объекту          │
│  object_type     VARCHAR(100)     "tender", "order", "route"    │
│  title           VARCHAR(500)                                    │
│  created_by      UUID NOT NULL                                   │
│  created_at      TIMESTAMPTZ                                     │
└─────────────────────────────────────────────────────────────────┘
         │
         ├──────────────────────────────────┐
         │                                  │
         ▼                                  ▼
┌─────────────────────┐        ┌─────────────────────────────────┐
│ dialog_participants │        │     dialog_access_scopes        │
├─────────────────────┤        ├─────────────────────────────────┤
│  dialog_id    FK    │        │  id             UUID PK         │
│  user_id      UUID  │        │  dialog_id      FK              │
│  joined_at          │        │  tenant_uid     UUID            │
│  joined_as          │        │  scope_level1   TEXT[]          │
│  notifications      │        │  scope_level2   TEXT[]          │
│  last_read_msg_id   │        │  created_at                     │
│  PK(dialog,user)    │        └─────────────────────────────────┘
└─────────────────────┘
```

### Логика доступа

```
Прямой участник:
  user_id IN dialog_participants

Потенциальный участник:
  EXISTS (
    SELECT 1 FROM dialog_access_scopes s
    WHERE s.dialog_id = dialog.id
      AND s.tenant_uid = user.tenant_uid
      AND s.scope_level1 && user.scope_level1  -- array overlap (ANY)
      AND s.scope_level2 && user.scope_level2  -- array overlap (ANY)
  )
```

## Database Schema

### Миграции

#### 001_update_dialogs.sql

```sql
-- Удаляем старые поля
ALTER TABLE dialogs DROP COLUMN IF EXISTS chat_key;
ALTER TABLE dialogs DROP COLUMN IF EXISTS tenant_a_id;
ALTER TABLE dialogs DROP COLUMN IF EXISTS tenant_b_id;
ALTER TABLE dialogs DROP COLUMN IF EXISTS context_id;

-- Добавляем новые поля
ALTER TABLE dialogs ADD COLUMN object_id UUID NOT NULL;
ALTER TABLE dialogs ADD COLUMN object_type VARCHAR(100) NOT NULL;
ALTER TABLE dialogs ADD COLUMN title VARCHAR(500);
ALTER TABLE dialogs ADD COLUMN created_by UUID NOT NULL;

-- Уникальность: один чат на объект
CREATE UNIQUE INDEX idx_dialogs_object ON dialogs(object_id, object_type);
```

#### 002_create_dialog_participants.sql

```sql
CREATE TABLE dialog_participants (
    dialog_id UUID NOT NULL REFERENCES dialogs(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,

    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    joined_as VARCHAR(50) NOT NULL DEFAULT 'participant',
    -- 'creator' - создатель чата
    -- 'participant' - добавлен при создании
    -- 'joined' - присоединился сам

    notifications_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    last_read_message_id UUID REFERENCES messages(id),

    PRIMARY KEY (dialog_id, user_id)
);

CREATE INDEX idx_participants_user ON dialog_participants(user_id);
CREATE INDEX idx_participants_dialog ON dialog_participants(dialog_id);
```

#### 003_create_dialog_access_scopes.sql

```sql
CREATE TABLE dialog_access_scopes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dialog_id UUID NOT NULL REFERENCES dialogs(id) ON DELETE CASCADE,

    tenant_uid UUID NOT NULL,
    scope_level1 TEXT[] NOT NULL DEFAULT '{}',
    scope_level2 TEXT[] NOT NULL DEFAULT '{}',

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_access_scopes_dialog ON dialog_access_scopes(dialog_id);
CREATE INDEX idx_access_scopes_tenant ON dialog_access_scopes(tenant_uid);

-- GIN индексы для array overlap операций
CREATE INDEX idx_access_scopes_level1 ON dialog_access_scopes USING GIN(scope_level1);
CREATE INDEX idx_access_scopes_level2 ON dialog_access_scopes USING GIN(scope_level2);
```

#### 004_update_messages.sql

```sql
-- Добавляем поддержку reply
ALTER TABLE messages ADD COLUMN reply_to_id UUID REFERENCES messages(id);

-- Индекс для быстрого поиска replies
CREATE INDEX idx_messages_reply ON messages(reply_to_id) WHERE reply_to_id IS NOT NULL;
```

## API Endpoints

### Management API

```
POST   /api/v1/management/dialogs
       → CreateDialogRequest { object_id, object_type, title, participants[], access_scopes[] }
       ← Dialog { id, ... }

GET    /api/v1/management/dialogs/{id}
       ← Dialog with participants and scopes

POST   /api/v1/management/dialogs/{id}/participants
       → { user_id }
       ← 201 Created

DELETE /api/v1/management/dialogs/{id}/participants/{user_id}
       ← 204 No Content

PUT    /api/v1/management/dialogs/{id}/access-scopes
       → { access_scopes[] }
       ← 200 OK

DELETE /api/v1/management/dialogs/{id}
       ← 204 No Content
```

### Chat API

```
GET    /api/v1/dialogs?type=participating
       Header: X-Scope-Config: base64(json)
       ← { dialogs: Dialog[], total: number }

GET    /api/v1/dialogs?type=available
       Header: X-Scope-Config: base64(json)
       ← { dialogs: Dialog[], total: number }

GET    /api/v1/dialogs/by-object/{object_type}/{object_id}
       ← { dialog: Dialog | null, messages: Message[], can_join: boolean }

GET    /api/v1/dialogs/{id}
       ← Dialog with messages

POST   /api/v1/dialogs/{id}/join
       ← { status: "joined", dialog: Dialog }

POST   /api/v1/dialogs/{id}/leave
       ← { status: "left" }

GET    /api/v1/dialogs/{id}/messages?limit=50&before={msg_id}
       ← { messages: Message[], has_more: boolean }

POST   /api/v1/dialogs/{id}/messages
       → { content: string, reply_to?: string }
       ← Message

GET    /api/v1/dialogs/{id}/participants
       ← { participants: Participant[] }
```

### WebSocket Events

```typescript
// Client → Server
interface WsClientMessage {
  type: 'subscribe' | 'unsubscribe' | 'message.send' | 'typing' | 'ping';
  dialog_id?: string;
  content?: string;
  reply_to?: string;
}

// Server → Client
interface WsServerEvent {
  type: 'message.new' | 'message.edited' | 'participant.joined' |
        'participant.left' | 'typing' | 'pong' | 'error';
  dialog_id: string;
  data: any;
}
```

### Outgoing Webhooks

```typescript
interface WebhookPayload {
  event: 'message.new' | 'participant.joined' | 'participant.left';
  timestamp: string;
  data: {
    dialog_id: string;
    object_id: string;
    object_type: string;
    // event-specific data
    message?: Message;
    user_id?: string;
    participants?: string[];  // для нотификаций
  };
}

// Signature: HMAC-SHA256(payload, WEBHOOK_SECRET)
// Header: X-MTChat-Signature: sha256=...
```

## Rust Implementation

### Project Structure

```
mtchat-rust/src/
├── main.rs
├── config.rs
├── error.rs
│
├── api/
│   ├── mod.rs
│   ├── management/
│   │   ├── mod.rs
│   │   ├── dialogs.rs
│   │   └── participants.rs
│   └── chat/
│       ├── mod.rs
│       ├── dialogs.rs
│       ├── messages.rs
│       └── join_leave.rs
│
├── ws/
│   ├── mod.rs
│   ├── handler.rs
│   └── hub.rs
│
├── webhooks/
│   ├── mod.rs
│   └── sender.rs
│
├── domain/
│   ├── mod.rs
│   ├── dialog.rs
│   ├── participant.rs
│   ├── access_scope.rs
│   └── message.rs
│
├── repositories/
│   ├── mod.rs
│   ├── dialog_repo.rs
│   ├── participant_repo.rs
│   ├── scope_repo.rs
│   └── message_repo.rs
│
├── services/
│   ├── mod.rs
│   ├── dialog_service.rs
│   ├── access_service.rs
│   └── notification_service.rs
│
└── middleware/
    ├── mod.rs
    ├── auth.rs
    └── scope_config.rs
```

### Key Types

```rust
// domain/dialog.rs
pub struct Dialog {
    pub id: Uuid,
    pub object_id: Uuid,
    pub object_type: String,
    pub title: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

// domain/participant.rs
pub struct Participant {
    pub dialog_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub joined_as: JoinedAs,
    pub notifications_enabled: bool,
    pub last_read_message_id: Option<Uuid>,
}

pub enum JoinedAs {
    Creator,
    Participant,
    Joined,
}

// domain/access_scope.rs
pub struct AccessScope {
    pub id: Uuid,
    pub dialog_id: Uuid,
    pub tenant_uid: Uuid,
    pub scope_level1: Vec<String>,
    pub scope_level2: Vec<String>,
}

// middleware/scope_config.rs
pub struct UserScopeConfig {
    pub tenant_uid: Uuid,
    pub scope_level1: Vec<String>,
    pub scope_level2: Vec<String>,
}
```

### Access Check

```rust
// services/access_service.rs

impl AccessService {
    /// Проверяет, является ли пользователь прямым участником
    pub async fn is_direct_participant(
        &self,
        user_id: Uuid,
        dialog_id: Uuid,
    ) -> Result<bool, Error> {
        self.participant_repo.exists(dialog_id, user_id).await
    }

    /// Проверяет, может ли пользователь видеть чат (потенциальный участник)
    pub async fn can_access_dialog(
        &self,
        user_scope: &UserScopeConfig,
        dialog_id: Uuid,
    ) -> Result<bool, Error> {
        // Сначала проверяем прямое участие
        // (user_id должен быть в scope или отдельно)

        // Затем проверяем scope rules
        let scopes = self.scope_repo.find_by_dialog(dialog_id).await?;

        for scope in scopes {
            if scope.tenant_uid == user_scope.tenant_uid
                && has_overlap(&scope.scope_level1, &user_scope.scope_level1)
                && has_overlap(&scope.scope_level2, &user_scope.scope_level2)
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Проверяет, может ли пользователь присоединиться
    pub async fn can_join(
        &self,
        user_id: Uuid,
        user_scope: &UserScopeConfig,
        dialog_id: Uuid,
    ) -> Result<bool, Error> {
        // Уже участник?
        if self.is_direct_participant(user_id, dialog_id).await? {
            return Ok(false); // уже участник
        }

        // Проверяем потенциальный доступ
        self.can_access_dialog(user_scope, dialog_id).await
    }
}

fn has_overlap(a: &[String], b: &[String]) -> bool {
    a.iter().any(|x| b.contains(x))
}
```

## Vue SDK Updates

### Types

```typescript
// types/index.ts

export interface Dialog {
  id: string;
  object_id: string;
  object_type: string;
  title?: string;
  created_at: string;

  // Computed on response
  unread_count?: number;
  last_message?: Message;
  participants_count?: number;
  i_am_participant?: boolean;
  can_join?: boolean;
}

export interface ScopeConfig {
  tenant_uid: string;
  scope_level1: string[];
  scope_level2: string[];
}

export interface MTChatConfig {
  baseUrl: string;
  token: string;
  scopeConfig: ScopeConfig;

  // Callbacks
  onNotification?: (event: NotificationEvent) => void;
}
```

### Component Props

```typescript
// components/MTChat.vue

interface Props {
  config: MTChatConfig;

  // Mode
  mode: 'full' | 'inline';

  // For inline mode
  objectId?: string;
  objectType?: string;

  // UI options
  showParticipants?: boolean;
  showJoinButton?: boolean;
}
```

### API Client Updates

```typescript
// sdk/api.ts

export class MTChatApi {
  // Dialogs
  async getMyDialogs(): Promise<Dialog[]>;
  async getAvailableDialogs(): Promise<Dialog[]>;
  async getDialogByObject(type: string, id: string): Promise<DialogWithMessages | null>;

  // Join/Leave
  async joinDialog(dialogId: string): Promise<void>;
  async leaveDialog(dialogId: string): Promise<void>;

  // Messages (existing)
  async getMessages(dialogId: string, options?: PaginationOptions): Promise<Message[]>;
  async sendMessage(dialogId: string, content: string, replyTo?: string): Promise<Message>;
}
```

## Этапы реализации

### Этап 1: Database Migration ✅
- [x] Создать миграции для новой схемы dialogs
- [x] Создать таблицу dialog_participants
- [x] Создать таблицу dialog_access_scopes
- [x] Добавить reply_to_id в messages
- [x] Удалить FK на employees из messages и attachments
- [x] Удалить таблицы `employees` и `tenants`
- [x] Удалить legacy API endpoints (/tenants, /employees)
- [x] Обновить sqlx queries
- [x] Тесты миграций (13 тестов в tests/migrations_test.rs)

### Этап 2: Domain & Repository Layer ✅
- [x] Новые domain types (src/domain/)
  - Dialog, DialogParticipant, DialogAccessScope, Message, JoinedAs
- [x] DialogRepository (find, create, delete, find_participating, find_available)
- [x] ParticipantRepository (add, remove, exists, list_by_dialog)
- [x] AccessScopeRepository (create, check_access)
- [x] MessageRepository (create, list_by_dialog, find_by_id_and_dialog)
- [x] Unit тесты (tests/domain_test.rs - 6 тестов)
- [x] Рефакторинг main.rs на использование repositories

### Этап 3: Management API ✅
- [x] POST /dialogs (create)
- [x] GET /dialogs/{id} (get with participants & scopes)
- [x] DELETE /dialogs/{id}
- [x] POST /dialogs/{id}/participants
- [x] DELETE /dialogs/{id}/participants/{uid}
- [x] PUT /dialogs/{id}/access-scopes
- [x] Admin auth middleware (ADMIN_API_TOKEN env var)
- [x] Integration тесты (10 tests, require running server)

### Этап 4: Chat API ✅
- [x] GET /dialogs?type=participating
- [x] GET /dialogs?type=available
- [x] GET /dialogs/by-object/{type}/{id}
- [x] POST /dialogs/{id}/join
- [x] POST /dialogs/{id}/leave
- [x] ScopeConfig extractor (X-Scope-Config header, base64 JSON)
- [x] UserId extractor (?user_id= query param)
- [x] OptionalScopeConfig for optional scope
- [x] Integration тесты (12 tests, require running server)

### Этап 5: Outgoing Webhooks ✅
- [x] Webhook sender service (async with mpsc channel)
- [x] HMAC-SHA256 signing (X-Webhook-Signature header)
- [x] Retry logic (exponential backoff, max 3 retries)
- [x] Events: message.new, participant.joined, participant.left
- [x] Environment config: WEBHOOK_URL, WEBHOOK_SECRET
- [x] Unit tests for signature verification

### Этап 6: Vue SDK Updates ✅
- [x] Новые типы (Dialog, DialogListItem, DialogParticipant, ScopeConfig, etc.)
- [x] API client updates (scope header, join/leave, getDialogByObject)
- [x] useChat composable updates (two dialog lists, join/leave, inline mode)
- [x] MTChat component: full mode (tabs: My Chats / Available)
- [x] MTChat component: inline mode (objectType + objectId props)
- [x] Two lists UI (participatingDialogs / availableDialogs)
- [x] Join/Leave UI with buttons
- [x] WebSocket client updated for new auth

### Этап 7: Example App (Dev Playground) ✅
- [x] **7.1 Admin Config Panel** — предварительный конфиг (данные в localStorage)
  - [x] Генерация тенантов с валидными UUID (TenantsPanel.vue)
  - [x] Генерация пользователей с валидными UUID (UsersPanel.vue)
  - [x] Назначение пользователям департаментов (scope_level1)
  - [x] Назначение пользователям пермишнов (scope_level2)
  - [x] Список созданных сущностей с возможностью редактирования/удаления
- [x] **7.2 Chat Management** — создание чатов для тестирования
  - [x] Создание чатов с привязкой к object_id/object_type (DialogsPanel.vue)
  - [x] Выбор непосредственных участников (dialog_participants)
  - [x] Настройка access scopes для потенциальных участников
  - [x] Список созданных чатов
- [x] **7.3 User Login & Mode Switch** — вход под пользователем
  - [x] Выбор пользователя из списка для "входа" (UserSelector.vue)
  - [x] **Full Mode** — полноразмерный чат-интерфейс (ChatPage.vue):
    - Список чатов слева (tabs: "Участвую" / "Доступные")
    - Окно сообщений справа
    - Панель участников
  - [x] **Inline Mode** — контекстный чат по объекту (InlinePage.vue):
    - Эмуляция страницы с карточкой объекта (TMSDataTable.vue)
    - Встроенная панель чата справа (TMSChatPanel.vue)
    - Автоматическая загрузка чата по object_type + object_id
- [x] **7.4 Layout & Navigation**
  - [x] TMS-стиль layout с сайдбаром (TMSLayout.vue)
  - [x] Переключатель между Admin Panel и User View
  - [x] Переключатель Full/Inline mode в User View

## Тестирование

### Migration Tests (tests/migrations_test.rs)

Запуск:
```bash
# Создать тестовую БД
createdb mtchat_test

# Запустить тесты миграций
TEST_DATABASE_URL="postgres://postgres:postgres@localhost:5432/mtchat_test" \
  cargo test --test migrations_test
```

Покрытие:
- ✅ Schema tests (new columns, constraints)
- ✅ Unique constraint on (object_id, object_type)
- ✅ Scope matching with array overlap (&&)
- ✅ Available dialogs excludes participants
- ✅ Message reply_to_id
- ✅ Cascade delete
- ✅ GIN indexes exist

### Unit Tests
- Access check logic
- Scope matching
- Webhook signing

### Integration Tests
- Dialog CRUD
- Participant management
- Join/Leave flow
- Message access control

### E2E Tests
- Full user flow: create → join → send → receive
- Inline mode
- Webhook delivery

## Риски и mitigation

| Риск | Вероятность | Mitigation |
|------|-------------|------------|
| Scope matching performance | Medium | GIN индексы, кеширование |
| Webhook reliability | Medium | Retry queue, dead letter |
| Breaking changes | High | Версионирование API, миграция данных |
