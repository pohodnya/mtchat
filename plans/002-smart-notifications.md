# Smart Notifications (Delayed Webhooks)

## Проблема

Пользователь сидит в чате и получает сообщение. Он видит его в реальном времени через WebSocket, но всё равно получает push-уведомление через webhook. Это создаёт лишний шум.

## Решение

Отложенная отправка webhook-событий с проверкой статуса прочтения.

```
Сообщение отправлено
        │
        ▼
┌───────────────────┐
│ Добавить в очередь│
│ с задержкой N сек │
└─────────┬─────────┘
          │
          │ (ждём N секунд)
          ▼
┌───────────────────┐
│ Проверить:        │
│ прочитано ли?     │
└─────────┬─────────┘
          │
    ┌─────┴─────┐
    │           │
    ▼           ▼
  Да           Нет
    │           │
    ▼           ▼
 Не слать    Отправить
 webhook     webhook
```

## Конфигурация

```env
# Задержка перед отправкой webhook (0 = отключено, instant)
WEBHOOK_DELAY_SECONDS=5

# Какие события откладывать (comma-separated)
WEBHOOK_DELAYED_EVENTS=message.new
```

## Что считать "прочитанным"

### Вариант 1: Явный mark_read (рекомендуется)
- API: `POST /dialogs/{id}/messages/read`
- Обновляет `last_read_message_id` в `dialog_participants`
- Плюс: точно знаем что пользователь видел
- Минус: требует доработки SDK

### Вариант 2: Активное WebSocket соединение
- Если у пользователя есть активный WS к этому диалогу → считаем прочитанным
- Плюс: не требует изменений в SDK
- Минус: пользователь мог отойти от компьютера

### Вариант 3: Комбинированный
- WS активен И была активность в последние 30 сек → прочитано
- Требует tracking последней активности

## Реализация

### Этап 1: Инфраструктура очереди
- [ ] In-memory delayed queue (tokio DelayQueue)
- [ ] Или Redis-based queue (для персистентности)
- [ ] Конфигурация через env vars

### Этап 2: Mark Read API
- [ ] `POST /api/v1/dialogs/{id}/messages/read` endpoint
- [ ] Body: `{ "message_id": "uuid" }` или `{ "until": "uuid" }`
- [ ] Обновление `last_read_message_id`
- [ ] WebSocket событие `messages.read` для синхронизации

### Этап 3: Интеграция
- [ ] Модифицировать webhook sender для отложенной отправки
- [ ] Проверка read status перед отправкой
- [ ] Метрики: сколько webhooks было suppressed

### Этап 4: SDK
- [ ] Автоматический mark_read при скролле
- [ ] Или при получении сообщения в открытом чате

## Альтернативный подход

Оставить webhooks instant, но добавить в payload информацию:

```json
{
  "type": "message.new",
  "payload": {
    "message": { ... },
    "recipient_status": {
      "user_123": { "online": true, "ws_connected": true },
      "user_456": { "online": false, "ws_connected": false }
    }
  }
}
```

Принимающая сторона сама решает кому слать push.

**Плюсы:** проще реализовать, гибче
**Минусы:** больше данных в webhook, логика на стороне потребителя

## Приоритет

**Низкий** - текущая реализация работает, оптимизация для UX.

Реализовать после:
- Stage 6: Vue SDK
- Stage 7: Example App
- Production testing

## Ссылки

- Основной план: [001-architecture-v3.md](./001-architecture-v3.md)
- Slack approach: webhooks instant, client handles dedup
- Telegram approach: server-side read tracking + delayed notifications
