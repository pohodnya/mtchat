# MTChat

Встраиваемый чат-сервис для B2B/B2C платформ. Разработан как микросервис для **TRUCKER TMS**, но спроектирован как универсальное open-source решение.

## Концепция

MTChat разделяет **бизнес-логику формирования чатов** и **функционал общения**:

- **Ваше приложение** решает, когда создать чат, к какому объекту его привязать и кого добавить
- **MTChat** обеспечивает хранение, доставку сообщений, уведомления и UI

### Ключевые особенности

- **Чат привязан к объекту** — каждый чат связан с сущностью вашей системы (тендер, заказ, рейс)
- **Прямые участники** — конкретные пользователи, получают уведомления
- **Потенциальные участники** — пользователи по scope-правилам, могут присоединиться
- **Два списка чатов** — "Участвую" и "Доступные"
- **Inline-режим** — встраивание чата в карточку объекта

## Архитектура

```
┌─────────────────────────────────────────────────────────────────┐
│                    Ваше приложение (TMS)                         │
│  ┌─────────────┐                 ┌─────────────────────┐        │
│  │  Frontend   │                 │  Backend            │        │
│  │ ┌─────────┐ │                 │  - Создание чатов   │        │
│  │ │ MTChat  │ │                 │  - Управление       │        │
│  │ │ Vue SDK │ │                 │  - Webhook handler  │        │
│  │ └────┬────┘ │                 └──────────┬──────────┘        │
└────────┼───────┘                            │                   │
         │                                    │
         │ Chat API                           │ Management API
         │ (User Token)                       │ (Admin Token)
         ▼                                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                      MTChat Backend                              │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────┐       │
│  │   Chat API     │  │ Management API │  │  Webhooks    │       │
│  │ (для юзеров)   │  │ (для системы)  │  │  (исходящие) │       │
│  └────────────────┘  └────────────────┘  └──────────────┘       │
│                              │                                   │
│                    PostgreSQL + Redis + MinIO                    │
└─────────────────────────────────────────────────────────────────┘
```

## Модель данных

### Диалог (Chat)

```
Dialog
├── object_id      ← привязка к объекту (обязательно)
├── object_type    "tender", "order", "route"
├── title
├── participants[] ← прямые участники (user_id)
└── access_scopes  ← правила для потенциальных участников
```

### Участники

| Тип | Описание | Уведомления | Список |
|-----|----------|-------------|--------|
| **Прямые** | Конкретные user_id | Да | "Участвую" |
| **Потенциальные** | По scope-правилам | После join | "Доступные" |

### Scope-правила (для потенциальных участников)

```json
{
  "tenant_uid": "tenant-123",
  "scope_level1": ["dept_logistics", "dept_sales"],
  "scope_level2": ["tender:manager", "tender:admin"]
}
```

**Логика**: `tenant AND (ANY из scope1) AND (ANY из scope2)`

Пример:
- `scope_level1` — департаменты (какую часть организации обслуживает сотрудник)
- `scope_level2` — роли/permissions (5-6 значений максимум)

## Структура проекта

```
mtchat/
├── mtchat-rust/       # Backend API (Rust)
├── mtchat-vue/        # Vue.js SDK для встраивания
├── mtchat-example/    # Демо-приложение
├── plans/             # Планы реализации
└── docker-compose.yml
```

## Быстрый старт

```bash
git clone https://github.com/your-org/mtchat.git
cd mtchat
docker-compose up -d

# Приложение: http://localhost
# API: http://localhost:8080
```

## Использование

### 1. Создание чата (Management API)

```bash
POST /api/v1/management/dialogs
Authorization: Bearer {admin_token}
{
  "object_id": "tender-uuid-123",
  "object_type": "tender",
  "title": "Обсуждение тендера #1234",
  "participants": ["user-1", "user-2"],
  "access_scopes": [
    {
      "tenant_uid": "tenant-abc",
      "scope_level1": ["dept_logistics"],
      "scope_level2": ["tender:manager", "tender:admin"]
    }
  ]
}
```

### 2. Встраивание компонента (Vue)

#### Полный режим (список чатов)

```vue
<template>
  <MTChat :config="config" mode="full" />
</template>

<script setup>
import { MTChat } from '@mtchat/vue'
import '@mtchat/vue/style.css'

const config = {
  baseUrl: 'https://chat.example.com',
  token: userToken,
  scopeConfig: {
    tenant_uid: currentUser.tenant_id,
    scope_level1: currentUser.departments,
    scope_level2: currentUser.permissions,
  }
}
</script>
```

#### Inline-режим (чат в карточке объекта)

```vue
<template>
  <div class="tender-card">
    <h1>Тендер #1234</h1>

    <!-- Чат встроен в карточку -->
    <MTChat
      :config="config"
      mode="inline"
      :object-id="tender.id"
      object-type="tender"
    />
  </div>
</template>
```

### 3. Обработка Webhook (ваш backend)

```python
# POST /webhooks/mtchat

def handle_mtchat_webhook(request):
    event = request.json

    if event["event"] == "message.new":
        # Отправить уведомления участникам
        for user_id in event["data"]["participants"]:
            send_notification(
                user_id=user_id,
                title=f"Новое сообщение в {event['data']['object_type']}",
                body=event["data"]["message"]["content"]
            )
```

## Пример: TRUCKER TMS

### Сценарий: "Связаться с организатором тендера"

```
Пользователь                    TMS Backend                MTChat API
     │                               │                          │
     │  1. Клик "Связаться"          │                          │
     ├──────────────────────────────►│                          │
     │                               │                          │
     │                               │  2. POST /management/    │
     │                               │     dialogs              │
     │                               ├─────────────────────────►│
     │                               │                          │
     │                               │  3. { id: "dialog-123" } │
     │                               │◄─────────────────────────┤
     │                               │                          │
     │  4. Redirect /chats?dialog=.. │                          │
     │◄──────────────────────────────┤                          │
     │                               │                          │
     │  5. MTChat загружает чат      │                          │
     ├─────────────────────────────────────────────────────────►│
```

## API Reference

### Management API (Admin Token)

| Метод | Endpoint | Описание |
|-------|----------|----------|
| POST | `/api/v1/management/dialogs` | Создать чат |
| POST | `/api/v1/management/dialogs/{id}/participants` | Добавить участника |
| DELETE | `/api/v1/management/dialogs/{id}/participants/{uid}` | Удалить участника |
| DELETE | `/api/v1/management/dialogs/{id}` | Удалить чат |

### Chat API (User Token)

| Метод | Endpoint | Описание |
|-------|----------|----------|
| GET | `/api/v1/dialogs?type=participating` | Мои чаты |
| GET | `/api/v1/dialogs?type=available` | Доступные чаты |
| GET | `/api/v1/dialogs/by-object/{type}/{id}` | Чат по объекту |
| POST | `/api/v1/dialogs/{id}/join` | Присоединиться |
| POST | `/api/v1/dialogs/{id}/leave` | Покинуть |
| GET | `/api/v1/dialogs/{id}/messages` | Сообщения |
| POST | `/api/v1/dialogs/{id}/messages` | Отправить |

### Webhook Events

| Event | Описание |
|-------|----------|
| `message.new` | Новое сообщение |
| `participant.joined` | Пользователь присоединился |
| `participant.left` | Пользователь покинул |

## Конфигурация

### Переменные окружения

```bash
DATABASE_URL=postgres://user:pass@localhost:5432/mtchat
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-secret-key
ADMIN_JWT_SECRET=your-admin-secret-key
WEBHOOK_URL=https://your-app.com/webhooks/mtchat
WEBHOOK_SECRET=webhook-signing-secret
S3_ENDPOINT=http://localhost:9000
S3_BUCKET=mtchat-uploads
```

## Roadmap

- [x] Базовый чат (сообщения, WebSocket)
- [x] Object-bound dialogs
- [x] Прямые участники
- [x] Потенциальные участники (scopes)
- [x] Join/Leave flow
- [x] Два списка чатов
- [x] Inline mode
- [x] Outgoing webhooks
- [x] Reply (ответ на сообщение)
- [x] Vue SDK (MTChat component)
- [x] Demo App (Dev Playground)
- [ ] Загрузка файлов (см. [план 003](./plans/003-file-attachments.md))
- [ ] @упоминания
- [ ] Read receipts

## Лицензия

MIT License

---

**MTChat** — часть экосистемы TRUCKER TMS
