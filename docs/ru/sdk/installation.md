# Установка SDK

## Требования

- **Vue.js** 3.4 или новее
- **Node.js** 18 или новее
- Работающий бэкенд MTChat

## Установка

=== "npm"

    ```bash
    npm install @mtchat/vue
    ```

=== "yarn"

    ```bash
    yarn add @mtchat/vue
    ```

=== "pnpm"

    ```bash
    pnpm add @mtchat/vue
    ```

### Зависимости

SDK требует Vue 3.4+ как peer-зависимость:

```bash
npm install vue@^3.4
```

## Базовая настройка

```vue
<script setup lang="ts">
import { MTChat } from '@mtchat/vue'
import '@mtchat/vue/style.css'
import type { MTChatConfig } from '@mtchat/vue'

const config: MTChatConfig = {
  baseUrl: 'https://chat.example.com',
  userId: currentUser.id,
  scopeConfig: {
    tenant_uid: currentUser.tenantId,
    scope_level1: currentUser.departments,
    scope_level2: currentUser.permissions,
  },
  userProfile: {
    displayName: currentUser.name,
    company: currentUser.company,
  },
}
</script>

<template>
  <MTChat :config="config" />
</template>
```

!!! note "Импорт стилей"
    Импорт `@mtchat/vue/style.css` обязателен. Без него UI не отобразится корректно.

## Интеграция с PrimeVue

Если ваше приложение использует [PrimeVue](https://primevue.org/), установите пакет-компаньон:

```bash
npm install @mtchat/vue-primevue
```

Подробнее: [Интеграция PrimeVue](primevue.md).

## Размер бандла

| Импорт | Размер (gzip) |
|--------|---------------|
| Полный компонент (`MTChat`) | ~80 КБ |
| Только composables (`useChat`) | ~15 КБ |
| Только типы | 0 КБ |

## TypeScript

SDK поставляется с полными TypeScript-декларациями:

```typescript
import type {
  MTChatConfig,
  Message,
  DialogListItem,
  WsEvent,
  UseChatReturn,
} from '@mtchat/vue'
```

## Дальнейшие шаги

- [Конфигурация](configuration.md) -- настройка SDK
- [Полный режим](full-mode.md) -- интерфейс со списком чатов
- [Встроенный режим](inline-mode.md) -- встраивание чата на страницу объекта
