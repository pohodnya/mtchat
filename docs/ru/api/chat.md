# Chat API

Chat API используется Vue SDK (или любым фронтенд-клиентом) для взаимодействия с диалогами и сообщениями в контексте аутентифицированного пользователя.

## Идентификация пользователя

Текущий пользователь определяется параметром `user_id` в query string. Scope-конфигурация передаётся через заголовок `X-Scope-Config` (base64-кодированный JSON).

Vue SDK обрабатывает это автоматически на основе `config.userId` и `config.scopeConfig`.

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
| `user_id` | UUID | обязателен | ID текущего пользователя |
| `search` | string | -- | Поиск по заголовку диалога или компании участника |
| `archived` | boolean | -- | Фильтр архивных диалогов |

### Поля ответа

| Поле | Тип | Описание |
|------|-----|----------|
| `participants_count` | integer | Количество участников |
| `unread_count` | integer | Непрочитанные сообщения для этого пользователя |
| `is_archived` | boolean | Архивирован ли диалог этим пользователем |
| `is_pinned` | boolean | Закреплён ли диалог этим пользователем |
| `notifications_enabled` | boolean | Включены ли уведомления |
| `last_message_at` | datetime | Время последнего сообщения |

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
      "s3_key": "dialogs/019481a2-.../pending/019481d5-....pdf",
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

---

## Удаление сообщения

Мягкое удаление (soft-delete). Только автор может удалить.

```
DELETE /api/v1/dialogs/{dialog_id}/messages/{id}?user_id={uuid}
```

---

## Ошибки

| HTTP статус | Код | Описание |
|-------------|-----|----------|
| 400 | `BAD_REQUEST` | Невалидный запрос |
| 403 | `FORBIDDEN` | Пользователь не является участником |
| 404 | `NOT_FOUND` | Диалог или сообщение не найдено |
| 500 | `INTERNAL_ERROR` | Ошибка сервера |
