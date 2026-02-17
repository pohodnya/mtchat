# Management API

Management API используется вашим бэкендом для создания и управления диалогами. Все запросы требуют admin-токен в заголовке `Authorization`.

## Аутентификация

```
Authorization: Bearer <ADMIN_API_TOKEN>
```

Admin-токен настраивается через переменную окружения `ADMIN_API_TOKEN` на сервере MTChat.

---

## Создание диалога

Создаёт новый диалог с участниками и scope-правилами доступа.

```
POST /api/v1/management/dialogs
```

### Тело запроса

```json
{
  "object_id": "550e8400-e29b-41d4-a716-446655440000",
  "object_type": "order",
  "title": "Обсуждение заказа #1234",
  "object_url": "https://app.example.com/orders/1234",
  "participants": [
    {
      "user_id": "11111111-1111-1111-1111-111111111111",
      "display_name": "Алиса",
      "company": "ООО Логистика",
      "email": "alice@logistics.ru",
      "phone": "+79001234567"
    }
  ],
  "access_scopes": [
    {
      "tenant_uid": "22222222-2222-2222-2222-222222222222",
      "scope_level1": ["logistics", "sales"],
      "scope_level2": ["manager", "admin"]
    }
  ]
}
```

| Поле | Тип | Обязательно | Описание |
|------|-----|-------------|----------|
| `object_id` | UUID | Да | ID бизнес-объекта, к которому привязан диалог |
| `object_type` | string | Да | Тип объекта (напр., "order", "tender") |
| `title` | string | Нет | Заголовок диалога (отображается в списке чатов) |
| `object_url` | string | Нет | Ссылка на объект в вашем приложении |
| `participants` | array | Да | Начальные участники (рекомендуется хотя бы один) |
| `participants[].user_id` | UUID | Да | ID пользователя из вашей системы |
| `participants[].display_name` | string | Да | Отображаемое имя в чате |
| `participants[].company` | string | Нет | Название компании |
| `participants[].email` | string | Нет | Контактный email |
| `participants[].phone` | string | Нет | Контактный телефон |
| `access_scopes` | array | Нет | Scope-правила для потенциальных участников |
| `access_scopes[].tenant_uid` | UUID | Да | ID тенанта/организации |
| `access_scopes[].scope_level1` | string[] | Нет | Первый уровень scope (напр., отделы). Пустой = любое значение. |
| `access_scopes[].scope_level2` | string[] | Нет | Второй уровень scope (напр., роли). Пустой = любое значение. |

### Ответ

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

!!! note
    Для одной комбинации `object_id` / `object_type` можно создать несколько диалогов.

---

## Получение диалога

```
GET /api/v1/management/dialogs/{id}
```

Возвращает диалог с участниками и scope-правилами.

---

## Удаление диалога

Удаляет диалог и все его данные (участники, сообщения, вложения, scope-правила).

```
DELETE /api/v1/management/dialogs/{id}
```

---

## Добавление участника

```
POST /api/v1/management/dialogs/{id}/participants
```

### Тело запроса

```json
{
  "user_id": "33333333-3333-3333-3333-333333333333",
  "display_name": "Борис",
  "company": "ООО Партнёр",
  "email": "boris@partner.ru",
  "phone": "+79009876543"
}
```

Участник добавляется с `joined_as = "member"`.

---

## Удаление участника

```
DELETE /api/v1/management/dialogs/{id}/participants/{user_id}
```

---

## Обновление scope-правил

Заменяет все scope-правила диалога.

```
PUT /api/v1/management/dialogs/{id}/access-scopes
```

### Тело запроса

```json
{
  "access_scopes": [
    {
      "tenant_uid": "22222222-2222-2222-2222-222222222222",
      "scope_level1": ["logistics"],
      "scope_level2": ["admin"]
    }
  ]
}
```

**Заменяет** все существующие scope-правила. Для удаления всех правил отправьте пустой массив.

---

## Ошибки

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Dialog not found"
  }
}
```

| HTTP статус | Код | Описание |
|-------------|-----|----------|
| 400 | `BAD_REQUEST` | Невалидное тело запроса |
| 401 | `UNAUTHORIZED` | Отсутствует или невалидный admin-токен |
| 404 | `NOT_FOUND` | Диалог или участник не найден |
| 500 | `INTERNAL_ERROR` | Ошибка сервера |
