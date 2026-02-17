# WebSocket

MTChat использует WebSocket для коммуникации в реальном времени. Vue SDK управляет соединением автоматически, но можно подключиться и напрямую.

## Подключение

```
WS /api/v1/ws?user_id={uuid}
```

Параметр `user_id` идентифицирует подключённого пользователя. При подключении сервер отправляет событие `connected` и устанавливает онлайн-статус.

## События сервера

### message.new

Новое сообщение в диалоге.

```json
{
  "type": "message.new",
  "id": "019481b3-...",
  "dialog_id": "019481a2-...",
  "sender_id": "11111111-...",
  "content": "<p>Привет!</p>",
  "sent_at": "2026-02-17T12:10:00Z",
  "message_type": "user"
}
```

### message.edited

Сообщение отредактировано.

```json
{
  "type": "message.edited",
  "id": "019481b3-...",
  "dialog_id": "019481a2-...",
  "content": "<p>Обновлённое содержание</p>",
  "last_edited_at": "2026-02-17T12:15:00Z"
}
```

### message.deleted

Сообщение удалено.

```json
{
  "type": "message.deleted",
  "id": "019481b3-...",
  "dialog_id": "019481a2-..."
}
```

### message.read

Обновление отметки о прочтении.

```json
{
  "type": "message.read",
  "dialog_id": "019481a2-...",
  "user_id": "11111111-...",
  "last_read_message_id": "019481b3-..."
}
```

### participant.joined / participant.left

Изменения участников.

```json
{
  "type": "participant.joined",
  "dialog_id": "019481a2-...",
  "user_id": "11111111-..."
}
```

### dialog.archived / dialog.unarchived

Изменения состояния архивации.

```json
{
  "type": "dialog.archived",
  "dialog_id": "019481a2-..."
}
```

### presence.update

Изменение онлайн-статуса пользователя.

```json
{
  "type": "presence.update",
  "user_id": "11111111-...",
  "is_online": true
}
```

Обновления присутствия отправляются только пользователям, имеющим общие диалоги.

## Сообщения клиента

### ping

Heartbeat-сообщение. Отправляйте каждые 30 секунд для поддержания онлайн-статуса (TTL 60 секунд).

```json
{
  "type": "ping"
}
```

### subscribe / unsubscribe

Подписка на события конкретного диалога.

```json
{
  "type": "subscribe",
  "dialog_id": "019481a2-..."
}
```

!!! note
    В текущей версии все подключённые пользователи получают все события. Фильтрация по подпискам запланирована в будущих релизах.
