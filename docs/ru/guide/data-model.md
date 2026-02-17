# Модель данных

## Основные сущности

### Диалог (Dialog)

Диалог -- это разговор, привязанный к бизнес-объекту в вашей системе.

```
┌─────────────────────────────────────────────┐
│                   Dialog                     │
├─────────────────────────────────────────────┤
│  id              UUID (v7, time-ordered)     │
│  object_id       UUID         (обязательно)  │
│  object_type     STRING       (обязательно)  │
│  title           STRING                      │
│  object_url      STRING       (опционально)  │
│  created_by      UUID                        │
│  created_at      TIMESTAMP                   │
└─────────────────────────────────────────────┘
```

- **object_id** + **object_type** связывают диалог с сущностью вашей системы (заказ, тендер, рейс)
- Можно создать несколько диалогов для одного объекта
- **object_url** -- опциональная ссылка на объект в вашем приложении

### Участник (Participant)

Прямой участник диалога. Получает уведомления и видит диалог в списке "Участвую".

```
┌─────────────────────────────────────────────┐
│               Participant                    │
├─────────────────────────────────────────────┤
│  dialog_id             UUID                  │
│  user_id               UUID                  │
│  display_name          STRING                │
│  company               STRING                │
│  email                 STRING (опционально)   │
│  phone                 STRING (опционально)   │
│  joined_at             TIMESTAMP              │
│  joined_as             "creator" | "member"   │
│  notifications_enabled BOOLEAN                │
│  last_read_msg_id      UUID (nullable)        │
│  unread_count          INTEGER                │
│  is_archived           BOOLEAN                │
│  is_pinned             BOOLEAN                │
└─────────────────────────────────────────────┘
```

- **user_id** -- внешний идентификатор из вашей системы (не управляется MTChat)
- **display_name** и **company** задаются при присоединении
- **unread_count** отслеживается для каждого участника
- **is_archived** и **is_pinned** -- состояния для каждого пользователя отдельно

### Scope доступа (Access Scope)

Определяет правила для потенциальных участников -- пользователей, которые могут увидеть и присоединиться к диалогу.

```
┌─────────────────────────────────────────────┐
│              Access Scope                    │
├─────────────────────────────────────────────┤
│  dialog_id       UUID                        │
│  tenant_uid      STRING                      │
│  scope_level1    STRING[]    (департаменты)  │
│  scope_level2    STRING[]    (права доступа) │
└─────────────────────────────────────────────┘
```

Диалог может иметь несколько scope доступа. Подробности в разделе [Scope-сопоставление](scope-matching.md).

### Сообщение (Message)

```
┌─────────────────────────────────────────────┐
│                  Message                     │
├─────────────────────────────────────────────┤
│  id              UUID (v7, time-ordered)     │
│  dialog_id       UUID                        │
│  sender_id       UUID (nullable для system)  │
│  message_type    "user" | "system"           │
│  content         STRING (HTML)               │
│  reply_to_id     UUID (nullable)             │
│  is_edited       BOOLEAN                     │
│  is_deleted      BOOLEAN                     │
│  created_at      TIMESTAMP                   │
│  updated_at      TIMESTAMP                   │
└─────────────────────────────────────────────┘
```

- Пользовательские сообщения содержат санитизированный HTML (разрешенные теги: `p`, `br`, `strong`, `em`, `u`, `s`, `a`, `ul`, `ol`, `li`, `blockquote`, `code`, `pre`, `span`)
- Системные сообщения (присоединение, выход, создание) имеют `message_type = "system"` и `sender_id = NULL`
- Системные сообщения хранят JSON-контент для i18n-рендеринга на фронтенде
- Отредактированные сообщения имеют `is_edited = true`; удаленные -- `is_deleted = true`

### Вложение (Attachment)

```
┌─────────────────────────────────────────────┐
│                Attachment                    │
├─────────────────────────────────────────────┤
│  id              UUID (v7)                   │
│  message_id      UUID                        │
│  s3_key          STRING                      │
│  filename        STRING                      │
│  content_type    STRING (MIME)                │
│  size            BIGINT (байты)              │
│  created_at      TIMESTAMP                   │
└─────────────────────────────────────────────┘
```

- Файлы хранятся в S3/MinIO и доступны через presigned URLs
- Максимальный размер: 100 МБ
- Максимум 10 вложений на сообщение

## Связи между сущностями

```
Dialog ──┬── Participants (прямые участники)
         ├── Access Scopes (кто может присоединиться)
         └── Messages ── Attachments
                      └── Edit History
```

## Таблицы БД

| Таблица | Описание |
|---------|----------|
| `dialogs` | Диалоги, привязанные к бизнес-объектам |
| `dialog_participants` | Прямые участники с per-user состоянием |
| `dialog_access_scopes` | Scope-правила для потенциальных участников |
| `messages` | Сообщения с поддержкой ответов |
| `attachments` | Файловые вложения к сообщениям |
| `message_edit_history` | История редактирования сообщений |

Все таблицы используют UUIDv7 для первичных ключей (упорядочены по времени для лучшей производительности индексов). Миграции выполняются автоматически при запуске сервера.
