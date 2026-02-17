# Интеграция PrimeVue

Пакет `@mtchat/vue-primevue` обеспечивает интеграцию с [PrimeVue](https://primevue.org/) 4.x. Он заменяет встроенные UI-примитивы MTChat (кнопки, диалоги, инпуты, табы) на PrimeVue-эквиваленты и маппит ваши токены темы PrimeVue.

## Установка

```bash
npm install @mtchat/vue-primevue
```

### Зависимости

| Пакет | Версия |
|-------|--------|
| `vue` | `^3.4.0` |
| `primevue` | `^4.0.0` |

!!! note
    Не нужно устанавливать `@mtchat/vue` отдельно. PrimeVue-пакет реэкспортирует всё из базового SDK.

## Быстрый старт

Используйте `MTChatPrime` вместо `MTChat`:

```vue
<script setup lang="ts">
import { MTChatPrime, type MTChatConfig } from '@mtchat/vue-primevue'

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
  <MTChatPrime :config="config" theme="light" />
</template>
```

## Маппинг темы

PrimeVue-интеграция автоматически маппит CSS-переменные PrimeVue на переменные MTChat:

| MTChat | PrimeVue |
|--------|----------|
| `--mtchat-primary` | `--p-primary-color` |
| `--mtchat-bg` | `--p-content-background` |
| `--mtchat-text` | `--p-text-color` |
| `--mtchat-border` | `--p-content-border-color` |

## Компоненты

PrimeVue-пакет заменяет следующие внутренние компоненты:

| MTChat компонент | PrimeVue аналог |
|------------------|-----------------|
| `MtButton` | `Button` |
| `MtDialog` | `Dialog` |
| `MtMenu` | `Menu` |
| `MtContextMenu` | `ContextMenu` |
| `MtTooltip` | `Tooltip` директива |
| `MtBadge` | `Badge` |

Это обеспечивает единообразный внешний вид с вашим PrimeVue-приложением.
