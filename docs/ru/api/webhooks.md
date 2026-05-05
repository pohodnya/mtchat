# Вебхуки

MTChat отправляет исходящие HTTP-вебхуки на ваш бэкенд при важных событиях. Это позволяет вашему приложению реагировать на события чата (отправка push-уведомлений, обновление бизнес-логики).

## Настройка

```bash
WEBHOOK_URL=https://your-app.com/webhooks/mtchat
WEBHOOK_SECRET=your-signing-secret
```

Если `WEBHOOK_URL` не указан, вебхуки отключены.

## Формат запроса

Все события отправляются как HTTP POST с JSON-телом:

```
POST https://your-app.com/webhooks/mtchat
Content-Type: application/json
X-Webhook-Signature: sha256=<hmac-hex-digest>
X-Webhook-Event: message.new
```

### Заголовки

| Заголовок | Описание |
|-----------|----------|
| `Content-Type` | Всегда `application/json` |
| `X-Webhook-Signature` | HMAC-SHA256 подпись тела запроса |
| `X-Webhook-Event` | Тип события |
| `X-Webhook-Id` | Уникальный ID события (тот же, что и `id` в обёртке) |

### Верификация подписи

Заголовок `X-Webhook-Signature` содержит HMAC-SHA256 hex-дайджест тела запроса, подписанный `WEBHOOK_SECRET`. Проверяйте его для обеспечения аутентичности:

```javascript
const crypto = require('crypto');

function verifyWebhook(body, signature, secret) {
  const expected = crypto
    .createHmac('sha256', secret)
    .update(body)
    .digest('hex');
  return crypto.timingSafeEqual(
    Buffer.from(`sha256=${expected}`),
    Buffer.from(signature)
  );
}
```

!!! warning
    Всегда используйте constant-time сравнение для предотвращения timing-атак.

## Обёртка события

Все события имеют общий формат:

```json
{
  "id": "019481e5-...",
  "type": "message_new",
  "timestamp": "2026-02-17T12:10:00Z",
  "payload": { }
}
```

| Поле | Тип | Описание |
|------|-----|----------|
| `id` | UUID | Уникальный ID события |
| `type` | string | Тип события в snake_case |
| `timestamp` | datetime | Время события |
| `payload` | object | Данные конкретного события |

## Типы событий

### message.new

Новое сообщение отправлено в диалог.

```json
{
  "id": "019481e5-...",
  "type": "message_new",
  "timestamp": "2026-02-17T12:10:00Z",
  "payload": {
    "dialog_id": "019481a2-...",
    "object_id": "550e8400-...",
    "object_type": "order",
    "message": {
      "id": "019481b3-...",
      "sender_id": "11111111-...",
      "content": "<p>Привет!</p>",
      "reply_to": null,
      "created_at": "2026-02-17T12:10:00Z",
      "message_type": "user"
    }
  }
}
```

### participant.joined

Пользователь присоединился к диалогу.

```json
{
  "id": "019481e6-...",
  "type": "participant_joined",
  "timestamp": "2026-02-17T12:05:00Z",
  "payload": {
    "dialog_id": "019481a2-...",
    "object_id": "550e8400-...",
    "object_type": "order",
    "user_id": "33333333-...",
    "joined_as": "joined",
    "joined_at": "2026-02-17T12:05:00Z"
  }
}
```

### participant.left

Пользователь покинул диалог.

```json
{
  "id": "019481e7-...",
  "type": "participant_left",
  "timestamp": "2026-02-17T12:20:00Z",
  "payload": {
    "dialog_id": "019481a2-...",
    "object_id": "550e8400-...",
    "object_type": "order",
    "user_id": "33333333-...",
    "left_at": "2026-02-17T12:20:00Z"
  }
}
```

### notification.pending

Отправляется, когда сообщение не было прочитано получателем после короткой серверной задержки. Ваше приложение должно отправить push-уведомление или email.

```json
{
  "id": "019481e8-...",
  "type": "notification_pending",
  "timestamp": "2026-02-17T12:10:30Z",
  "payload": {
    "dialog_id": "019481a2-...",
    "object_id": "550e8400-...",
    "object_type": "order",
    "recipient_id": "22222222-...",
    "message": {
      "id": "019481b3-...",
      "sender_id": "11111111-...",
      "content": "<p>Привет!</p>",
      "reply_to": null,
      "created_at": "2026-02-17T12:10:00Z",
      "message_type": "user"
    }
  }
}
```

**Умные уведомления:**

- Задачи уведомлений коротко ждут перед проверкой, прочитал ли получатель сообщение
- Если сообщение прочитано до истечения задержки, уведомление не отправляется
- Каждая непрочитанная пара сообщение/получатель может породить webhook `notification.pending`
- Уведомления пропускаются, если пользователь отключил уведомления для этого чата

## Политика повторов

- Макс. попыток: 3
- Начальная задержка: 1 секунда
- Множитель: 2x
- Таймаут запроса: 10 секунд

Ваш эндпоинт должен возвращать статус 2xx для подтверждения получения.
