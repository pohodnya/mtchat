# Быстрый старт

Запустите MTChat за 5 минут.

## Требования

- Docker и Docker Compose
- Node.js 18+ (для Vue SDK)

## 1. Запуск бэкенда

```bash
git clone https://github.com/nicenemo/mtchat.git
cd mtchat
docker-compose up -d
```

Проверьте, что API работает:

```bash
curl http://localhost:8080/health
# {"status":"ok"}
```

## 2. Создание чата через Management API

Используйте Management API из вашего бэкенда для создания диалога:

```bash
curl -X POST http://localhost:8080/api/v1/management/dialogs \
  -H "Authorization: Bearer demo-admin-token" \
  -H "Content-Type: application/json" \
  -d '{
    "object_id": "550e8400-e29b-41d4-a716-446655440000",
    "object_type": "order",
    "title": "Обсуждение заказа #1234",
    "participants": [
      {
        "user_id": "11111111-1111-1111-1111-111111111111",
        "display_name": "Алиса",
        "company": "ООО Логистика",
        "joined_as": "creator"
      }
    ],
    "access_scopes": [
      {
        "tenant_uid": "tenant-uuid",
        "scope_level1": ["logistics"],
        "scope_level2": ["manager", "admin"]
      }
    ]
  }'
```

## 3. Установка Vue SDK

```bash
npm install @mtchat/vue
```

## 4. Добавление MTChat в приложение

### Полный режим (список чатов + область чата)

```vue
<template>
  <MTChat :config="chatConfig" mode="full" theme="light" />
</template>

<script setup lang="ts">
import { MTChat } from '@mtchat/vue'
import '@mtchat/vue/style.css'

const chatConfig = {
  baseUrl: 'http://localhost:8080',
  userId: '11111111-1111-1111-1111-111111111111',
  scopeConfig: {
    tenant_uid: 'tenant-uuid',
    scope_level1: ['logistics'],
    scope_level2: ['manager'],
  },
  userProfile: {
    displayName: 'Алиса',
    company: 'ООО Логистика',
  },
  locale: 'ru',
}
</script>
```

### Inline-режим (один чат на странице объекта)

```vue
<template>
  <div class="order-page">
    <h1>Заказ #1234</h1>

    <MTChat
      :config="chatConfig"
      mode="inline"
      object-id="550e8400-e29b-41d4-a716-446655440000"
      object-type="order"
      theme="light"
    />
  </div>
</template>
```

## 5. Демо-приложение

Откройте [http://localhost](http://localhost) для просмотра полного демо-приложения:

- Админ-панель для управления тенантами, пользователями, объектами и диалогами
- Полный режим: список чатов с вкладками "Мои чаты" и "Доступные"
- Inline-режим: TMS-интерфейс с таблицей данных и встроенным чатом
- Переключение пользователей для тестирования мультипользовательских сценариев

## Дальнейшие шаги

- [Архитектура](../guide/architecture.md) -- как устроен MTChat
- [Management API](../api/management.md) -- создание чатов и управление участниками
- [Chat API](../api/chat.md) -- справочник API для фронтенд SDK
- [Компоненты Vue SDK](../sdk/components.md) -- props, events, slots
- [Темы](../sdk/theming.md) -- настройка внешнего вида
- [Конфигурация](../configuration.md) -- переменные окружения бэкенда
