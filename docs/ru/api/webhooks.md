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
      "created_at": "2026-02-17T12:10:00Z",
      "message_type": "user"
    }
  }
}
```

### participant.joined / participant.left

Пользователь присоединился или покинул диалог.

### notification.pending

Отправляется, когда сообщение не было прочитано получателем после настраиваемой задержки (по умолчанию 30 секунд). Ваше приложение должно отправить push-уведомление или email.

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
      "created_at": "2026-02-17T12:10:00Z",
      "message_type": "user"
    }
  }
}
```

**Умные уведомления:**

- Уведомления задерживаются (по умолчанию 30 сек, настраивается через `NOTIFICATION_DELAY_SECS`)
- Если сообщение прочитано до истечения задержки, уведомление не отправляется
- Несколько сообщений одному получателю дедуплицируются в одно уведомление
- Уведомления пропускаются, если пользователь отключил уведомления для этого чата

## Политика повторов

- Макс. попыток: 3
- Начальная задержка: 1 секунда
- Множитель: 2x
- Таймаут запроса: 10 секунд

Ваш эндпоинт должен возвращать статус 2xx для подтверждения получения.
