# Chat API

Chat API используется Vue SDK (или любым фронтенд-клиентом) для взаимодействия с диалогами и сообщениями в контексте аутентифицированного пользователя.

## Идентификация пользователя

Когда `JWT_AUTH_ENABLED=true`, текущий пользователь извлекается из Bearer-токена по claim, заданному в `JWT_USER_ID_CLAIM`. Когда JWT-аутентификация выключена, API использует query-параметр `user_id`. Scope-конфигурация передаётся через заголовок `X-Scope-Config` (base64-кодированный JSON).

Vue SDK обрабатывает это автоматически на основе `config.token`, `config.userId` и `config.scopeConfig`.

---

## Список диалогов

```
GET /api/v1/dialogs?type=participating&user_id={uuid}
GET /api/v1/dialogs?type=available&user_id={uuid}
```

### Параметры запроса

| Параметр | Тип | По умолчанию | Описание |
|----------|-----|--------------|----------|
| `type` | string | `participating` | `participating` (мои чаты) или `available` (доступные) |
| `user_id` | string | обязателен в legacy-режиме | ID текущего пользователя, когда JWT-аутентификация выключена |
| `search` | string | -- | Поиск по заголовку диалога или компании участника |
| `archived` | boolean | -- | Фильтр архивных диалогов |
| `limit` | integer | 50 | Количество диалогов (макс. 100) |
| `offset` | integer | 0 | Пропустить N диалогов |

### Поля ответа

| Поле | Тип | Описание |
|------|-----|----------|
| `participants_count` | integer | Количество участников |
| `i_am_participant` | boolean | Является ли текущий пользователь участником |
| `can_join` | boolean | Может ли текущий пользователь присоединиться (для списка `available`) |
| `unread_count` | integer | Непрочитанные сообщения для этого пользователя |
| `is_archived` | boolean | Архивирован ли диалог этим пользователем |
| `is_pinned` | boolean | Закреплён ли диалог этим пользователем |
| `notifications_enabled` | boolean | Включены ли уведомления |
| `last_message_at` | datetime | Время последнего сообщения |

---

## Получение диалога

```
GET /api/v1/dialogs/{id}?user_id={uuid}
```

Возвращает метаданные диалога.

```json
{
  "data": {
    "id": "019481a2-...",
    "object_id": "550e8400-...",
    "object_type": "order",
    "title": "Обсуждение заказа #1234",
    "object_url": "https://app.example.com/orders/1234",
    "created_by": "11111111-...",
    "created_at": "2026-02-17T12:00:00Z"
  }
}
```

---

## Получение диалога по объекту

Возвращает последний диалог для указанного бизнес-объекта. Используется SDK в inline-режиме.

```
GET /api/v1/dialogs/by-object/{object_type}/{object_id}?user_id={uuid}
```

---

## Присоединение к диалогу

```
POST /api/v1/dialogs/{id}/join?user_id={uuid}
```

### Тело запроса

```json
{
  "display_name": "Иван Петров",
  "company": "ООО Логистика",
  "email": "ivan@logistics.ru",
  "phone": "+79001234567"
}
```

### Ответ

```json
{
  "status": "joined",
  "dialog_id": "019481a2-..."
}
```

Автоматически создаётся системное сообщение и отправляются WebSocket/webhook-события.

---

## Выход из диалога

```
POST /api/v1/dialogs/{id}/leave?user_id={uuid}
```

---

## Отметка о прочтении

```
POST /api/v1/dialogs/{id}/read?user_id={uuid}
```

```json
{
  "last_read_message_id": "019481b3-..."
}
```

---

## Архивация / Разархивация

Персональное состояние -- не влияет на других участников.

```
POST /api/v1/dialogs/{id}/archive?user_id={uuid}
POST /api/v1/dialogs/{id}/unarchive?user_id={uuid}
```

---

## Закрепление / Открепление

```
POST /api/v1/dialogs/{id}/pin?user_id={uuid}
POST /api/v1/dialogs/{id}/unpin?user_id={uuid}
```

---

## Настройка уведомлений

```
POST /api/v1/dialogs/{id}/notifications?user_id={uuid}
```

```json
{
  "enabled": false
}
```

---

## Список участников

Возвращает участников диалога. Прямые участники видят контактные данные; потенциальные участники с подходящим scope видят список без `email` и `phone`.

```
GET /api/v1/dialogs/{id}/participants?user_id={uuid}
```

```json
{
  "data": [
    {
      "dialog_id": "019481a2-...",
      "user_id": "11111111-...",
      "display_name": "Алиса",
      "company": "ООО Логистика",
      "email": "alice@logistics.ru",
      "phone": "+79001234567",
      "joined_as": "participant",
      "joined_at": "2026-02-17T12:00:00Z",
      "notifications_enabled": true,
      "last_read_message_id": null,
      "unread_count": 0,
      "is_archived": false,
      "is_pinned": false,
      "is_online": true
    }
  ]
}
```

---

## Список сообщений

Требует участия в диалоге.

```
GET /api/v1/dialogs/{dialog_id}/messages?user_id={uuid}
```

### Параметры пагинации

| Параметр | Тип | По умолчанию | Описание |
|----------|-----|--------------|----------|
| `limit` | integer | 50 | Количество сообщений |
| `before` | UUID | -- | Загрузить сообщения до этого ID (прокрутка вверх) |
| `after` | UUID | -- | Загрузить сообщения после этого ID (прокрутка вниз) |
| `around` | UUID | -- | Загрузить сообщения вокруг этого ID (переход к сообщению) |

Ответ включает `has_more_before`, `has_more_after` и `first_unread_message_id`.

```json
{
  "data": {
    "messages": [
      {
        "id": "019481b3-...",
        "dialog_id": "019481a2-...",
        "sender_id": "11111111-...",
        "message_type": "user",
        "content": "<p>Привет!</p>",
        "reply_to_id": null,
        "sent_at": "2026-02-17T12:10:00Z",
        "last_edited_at": null,
        "attachments": []
      }
    ],
    "first_unread_message_id": "019481b5-...",
    "has_more_before": true,
    "has_more_after": false
  }
}
```

---

## Отправка сообщения

```
POST /api/v1/dialogs/{dialog_id}/messages?user_id={uuid}
```

```json
{
  "content": "<p>Привет! Это <strong>форматированное</strong> сообщение.</p>",
  "reply_to": "019481b3-...",
  "attachments": [
    {
      "s3_key": "dialogs/019481a2-.../019481d5-....pdf",
      "filename": "отчёт.pdf",
      "content_type": "application/pdf",
      "size": 245760
    }
  ]
}
```

HTML-контент санитизируется на сервере. Разрешённые теги: `p`, `br`, `strong`, `em`, `u`, `s`, `a`, `ul`, `ol`, `li`, `blockquote`, `code`, `pre`, `span`.

---

## Редактирование сообщения

Только автор может редактировать. Системные сообщения защищены.

```
PUT /api/v1/dialogs/{dialog_id}/messages/{id}?user_id={uuid}
```

```json
{
  "content": "<p>Обновлённый текст сообщения.</p>"
}
```

Устанавливает `last_edited_at`, сохраняет прежний текст в историю правок и отправляет WebSocket-событие `message.edited`.

---

## Удаление сообщения

Удаляет сообщение. Только автор может удалить своё сообщение.

```
DELETE /api/v1/dialogs/{dialog_id}/messages/{id}?user_id={uuid}
```

Удаляет строку сообщения и отправляет WebSocket-событие `message.deleted`.

### Ответ

```
204 No Content
```

---

## Ошибки

```json
{
  "error": {
    "code": "NOT_PARTICIPANT",
    "message": "Not a participant. Join the dialog first."
  }
}
```

### Коды ошибок

| Код | HTTP статус | Описание |
|-----|-------------|----------|
| `DIALOG_NOT_FOUND` | 404 | Диалог не существует |
| `MESSAGE_NOT_FOUND` | 404 | Сообщение не существует |
| `PARTICIPANT_NOT_FOUND` | 404 | Участник не найден в диалоге |
| `ATTACHMENT_NOT_FOUND` | 404 | Вложение не существует |
| `INVALID_INPUT` | 400 | Невалидные данные или превышение лимитов |
| `FILE_TOO_LARGE` | 400 | Файл превышает лимит 100 МБ |
| `UNSUPPORTED_FILE_TYPE` | 400 | MIME-тип файла не разрешён |
| `TOO_MANY_ATTACHMENTS` | 400 | Более 10 вложений на сообщение |
| `NOT_PARTICIPANT` | 403 | Пользователь должен сначала присоединиться |
| `NOT_MESSAGE_AUTHOR` | 403 | Только автор может редактировать/удалять |
| `SCOPE_MISMATCH` | 403 | Scope пользователя не соответствует правилам доступа |
| `INTERNAL_ERROR` | 500 | Ошибка сервера |
